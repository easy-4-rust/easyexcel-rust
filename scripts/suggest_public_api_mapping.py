#!/usr/bin/env python3
"""为 Java public API 生成确定性的 Rust public API 候选映射。

该脚本只生成候选，不会把任何条目标记为 verified。重载仍按 JVM descriptor
保留为独立条目，歧义候选也不会被静默合并。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import defaultdict
from pathlib import Path
from typing import Any


OWNER_ALIASES = {
    "EasyExcelFactory": "EasyExcel",
    # Rust 将 Java `ExcelWriter` 的配置/上下文/fill 门面保存在
    # `ExcelBuilderImpl`，底层 `ExcelWriter` 仅负责格式写入状态机。
    "ExcelWriter": "ExcelBuilderImpl",
}

MEMBER_OWNER_ALIASES = {
    # Java 抽象基类的默认 support 位于基类本身；Rust 用 supertrait 继承
    # XlsRecordHandler::support，因此方法候选应落到提供实现的 supertrait。
    "AbstractXlsRecordHandler": "XlsRecordHandler",
    # Rust 将 listener 只读快照与可变 Holder 生命周期拆分；Java interface
    # 的生命周期成员由真实 AnalysisContextImpl 承载。
    "AnalysisContext": "AnalysisContextImpl",
}

# 这些 Java package-path 类型在 Rust 中是真实公开 type alias；类型条目应按同名
# `existing_implementation` 映射，仅其关联方法索引需要落到 alias 的底层运行时类型。
TRANSPARENT_MEMBER_OWNER_ALIASES = {
    "CellWriteHandlerContext": "WriteCellContext",
    "RowWriteHandlerContext": "WriteRowContext",
    "SheetWriteHandlerContext": "WriteSheetContext",
    "WorkbookWriteHandlerContext": "WriteWorkbookContext",
}

# Java 类型本身公开，但可调用实现全部是包私有反射辅助；Rust 以编译期
# schema/derive 模块替代，不创建同名无状态对象。
MODULE_ONLY_OWNERS = {
    "BeanMapUtils",
    "BooleanUtils",
    "ClassUtils",
    "ConverterKeyBuild",
    "ConverterUtils",
    "DefaultConverterLoader",
    "FieldUtils",
    "FileTypeUtils",
    "FileUtils",
    "IntUtils",
    "IoUtils",
    "ListUtils",
    "MapUtils",
    "MemberUtils",
    "NumberDataFormatterUtils",
    "NumberUtils",
    "PoiUtils",
    "PositionUtils",
    "SheetUtils",
    "StringUtils",
    "StyleUtil",
    "Validate",
    "WorkBookUtil",
    "WriteHandlerUtils",
}

# 这些 Java 类虽然只有静态 API，但 Rust 已有意保留同名公开类型和关联成员，
# 因而不能被通用“静态工具类 -> module”规则抢先降级为替代实现。
NOMINAL_STATIC_UTILITY_OWNERS = {
    "BuiltinFormats",
    "DateUtils",
    "ExcelXmlConstants",
}

# Java enum getter 返回 POI/Commons 对象；Rust 复用后端中立枚举或字节值，
# 调用形状存在但返回载体按引擎边界归为惯用替代。
BACKEND_NEUTRAL_ENUM_MEMBERS = {
    "ByteOrderMarkEnum": {"getByteOrderMark"},
    "BorderStyleEnum": {"getPoiBorderStyle"},
    "FillPatternTypeEnum": {"getPoiFillPatternType"},
    "HorizontalAlignmentEnum": {"getPoiHorizontalAlignment"},
    "VerticalAlignmentEnum": {"getPoiVerticalAlignmentEnum"},
    "AnchorType": {"getValue"},
    "HyperlinkType": {"getValue"},
}

# Java annotation 由 JVM 元数据和反射实例承载；Rust 的真实等价物是
# `#[derive(ExcelRow)]` / `#[excel(...)]` 生成的静态 schema，加上 facade 中可复用的
# 运行期参数对象。即使类型和成员同名，也不能误报为调用形状完全相同的直接实现。
DECLARATIVE_ANNOTATION_OWNERS = {
    "ExcelIgnore",
    "ExcelIgnoreUnannotated",
    "ExcelProperty",
    "DateTimeFormat",
    "NumberFormat",
    "ColumnWidth",
    "ContentFontStyle",
    "ContentLoopMerge",
    "ContentRowHeight",
    "ContentStyle",
    "HeadFontStyle",
    "HeadRowHeight",
    "HeadStyle",
    "OnceAbsoluteMerge",
}

# Java 公共签名中直接泄露的反射/POI 类型在 Rust 中必须停留在 facade 边界，
# 由 TypeId、稳定类型键或 easyexcel-model 的格式中立枚举/颜色/格式对象承载。
# 这些成员有真实实现，但实现策略属于 idiomatic_alternative。
BACKEND_NEUTRAL_MEMBERS = {
    "ClassUtils": {"CLASS_CONTENT_CACHE", "CONTENT_CACHE", "FIELD_CACHE"},
    "ExcelProperty": {"converter"},
    "ConverterUtils": {
        "defaultClassGeneric",
        "convertToJavaObject",
        "convertToStringMap",
    },
    "FileTypeUtils": {"defaultImageType", "getImageType", "getImageTypeFormat"},
    "FieldUtils": {"nullObjectClass", "getField", "getFieldClass"},
    "PageReadListener": {"BATCH_COUNT"},
    "PoiUtils": {"CUSTOM_HEIGHT", "customHeight"},
    "UrlImageConverter": {"urlConnectTimeout", "urlReadTimeout"},
    "DefaultWriteHandlerLoader": {"DEFAULT_WRITE_HANDLER_LIST"},
    "DateUtils": {"defaultDateFormat", "defaultLocalDateFormat"},
    "EasyExcelConstants": {"EXCEL_MATH_CONTEXT"},
    "WriteCellStyle": {
        "getBorderBottom", "getBorderLeft", "getBorderRight", "getBorderTop",
        "getBottomBorderColor", "getDataFormatData", "getFillBackgroundColor",
        "getFillForegroundColor", "getFillPatternType", "getHorizontalAlignment",
        "getLeftBorderColor", "getRightBorderColor", "getTopBorderColor",
        "getVerticalAlignment", "setBorderBottom", "setBorderLeft", "setBorderRight",
        "setBorderTop", "setBottomBorderColor", "setDataFormatData",
        "setFillBackgroundColor", "setFillForegroundColor", "setFillPatternType",
        "setHorizontalAlignment", "setLeftBorderColor", "setRightBorderColor",
        "setTopBorderColor", "setVerticalAlignment",
    },
}

RAII_ALTERNATIVE_OWNERS = {"EasyExcelTempFileCreationStrategy"}

# Java 运行时反射辅助在 Rust 中由编译期 schema 驱动的 class_utils 承载。
# 这里记录语义所有者映射，不为追求同名而重新引入 member_utils 空壳。
MODULE_OWNER_ALIASES = {
    "MemberUtils": "class_utils",
}

# Java CSV 对象实现 POI 的 Workbook/Sheet/Row/Cell/CellStyle 大接口，其中大量
# 方法只是为了满足 POI 类型约束而存在：空操作、固定 0/false/null，或者明确拒绝
# CSV 无法表达的能力。Rust 不需要把这些 POI 兼容槽位复制成数百个同名空方法；
# 它们由 CSV 类型本身的格式能力边界承载，映射策略应是 idiomatic_alternative。
#
# 下表只列出真正读写 CSV 状态、值、顺序缓存或输出生命周期的成员。类型、构造器
# 仍是 existing_implementation；equals/hashCode 继续走 Rust trait 的通用替代规则。
CSV_STATEFUL_MEMBERS = {
    "CsvRichTextString": {
        "getString",
        "length",
    },
    "CsvCellStyle": {
        "getDataFormat",
        "getDataFormatData",
        "getDataFormatString",
        "getIndex",
        "setDataFormat",
        "setDataFormatData",
        "setIndex",
    },
    "CsvCell": {
        "getBooleanCellValue",
        "getBooleanValue",
        "getCachedFormulaResultType",
        "getCellFormula",
        "getCellStyle",
        "getCellType",
        "getColumnIndex",
        "getCsvRow",
        "getCsvSheet",
        "getCsvWorkbook",
        "getDateCellValue",
        "getDateValue",
        "getErrorCellValue",
        "getFormulaData",
        "getLocalDateTimeCellValue",
        "getNumberValue",
        "getNumericCellType",
        "getNumericCellValue",
        "getRichStringCellValue",
        "getRichTextString",
        "getRow",
        "getRowIndex",
        "getSheet",
        "getStringCellValue",
        "getStringValue",
        "setBooleanValue",
        "setCellErrorValue",
        "setCellStyle",
        "setCellValue",
        "setDateValue",
        "setFormulaData",
        "setNumberValue",
        "setNumericCellType",
        "setRichTextString",
        "setStringValue",
    },
    "CsvRow": {
        "cellIterator",
        "createCell",
        "getCell",
        "getCellList",
        "getCellStyle",
        "getCsvSheet",
        "getCsvWorkbook",
        "getFirstCellNum",
        "getLastCellNum",
        "getPhysicalNumberOfCells",
        "getRowIndex",
        "getRowNum",
        "getRowStyle",
        "getSheet",
        "iterator",
        "removeCell",
        "setCellStyle",
        "setRowIndex",
        "setRowNum",
        "setRowStyle",
    },
    "CsvSheet": {
        "close",
        "createRow",
        "flushData",
        "getCsvFormat",
        "getCsvPrinter",
        "getCsvWorkbook",
        "getFirstRowNum",
        "getLastRowIndex",
        "getLastRowNum",
        "getOut",
        "getPhysicalNumberOfRows",
        "getRow",
        "getRowCache",
        "getRowCacheCount",
        "getWorkbook",
        "iterator",
        "printData",
        "rowIterator",
        "setCsvFormat",
        "setCsvPrinter",
        "setCsvWorkbook",
        "setLastRowIndex",
        "setOut",
        "setRowCache",
        "setRowCacheCount",
    },
    "CsvWorkbook": {
        "createCellStyle",
        "createDataFormat",
        "createSheet",
        "getCellStyleAt",
        "getCharset",
        "getCsvCellStyleList",
        "getCsvDataFormat",
        "getCsvSheet",
        "getLocale",
        "getNumCellStyles",
        "getOut",
        "getSheet",
        "getSheetAt",
        "getUse1904windowing",
        "getUseScientificFormat",
        "getWithBom",
        "setCharset",
        "setCsvCellStyleList",
        "setCsvDataFormat",
        "setCsvSheet",
        "setLocale",
        "setOut",
        "setUse1904windowing",
        "setUseScientificFormat",
        "setWithBom",
        "write",
    },
}

IMPLEMENTATION_STRATEGIES = {
    "existing_implementation",
    "idiomatic_alternative",
    "needs_implementation",
}

OWNER_CAPABILITY_CARRIERS = {
    # 退役 Ehcache 的 API 形状由 facade ReadCache 生命周期承载，真实共享字符串
    # 存储/策略由 easyexcel-cache 的 Memory/File/Moka 组合承载；不是 Ehcache→Moka。
    "Ehcache": ["easyexcel-cache", "easyexcel"],
    # Holder/配置对象属于 Java API 生命周期层；格式引擎只消费解析后的状态，
    # 不应在 xls/xlsx/csv crate 中复制同名 Holder。
    "Holder": ["easyexcel"],
    "ConfigurationHolder": ["easyexcel"],
    "ReadHolder": ["easyexcel"],
    "ReadRowHolder": ["easyexcel"],
    "FieldCache": ["easyexcel", "easyexcel-derive"],
    "FieldWrapper": ["easyexcel", "easyexcel-derive"],
    "CsvWorkbook": ["easyexcel-csv", "easyexcel"],
    "CsvSheet": ["easyexcel-csv", "easyexcel"],
    "CsvCellStyle": ["easyexcel-csv", "easyexcel"],
    "CsvCell": ["easyexcel-csv", "easyexcel"],
    "CsvRow": ["easyexcel-csv", "easyexcel"],
    "CsvRichTextString": ["easyexcel-csv", "easyexcel"],
    "CsvReadWorkbookHolder": ["easyexcel", "easyexcel-csv"],
    "CsvReadSheetHolder": ["easyexcel", "easyexcel-csv"],
    "DataFormatter": ["easyexcel-format", "easyexcel"],
    "ExcelGeneralNumberFormat": ["easyexcel-format", "easyexcel"],
    "NumberDataFormatterUtils": ["easyexcel-format", "easyexcel"],
    "PositionUtils": ["easyexcel-utils", "easyexcel"],
    "ExcelXmlConstants": ["easyexcel-xlsx", "easyexcel"],
    "DateUtils": ["easyexcel-model", "easyexcel"],
    "BuiltinFormats": ["easyexcel-format", "easyexcel"],
    "NumberUtils": ["easyexcel-format", "easyexcel", "easyexcel-model"],
    "MemberUtils": ["easyexcel-derive", "easyexcel"],
    # Holder 是 Java facade 生命周期状态；格式解析和序列化仍由对应引擎执行。
    "AbstractHolder": ["easyexcel"],
    "AbstractReadHolder": ["easyexcel"],
    "ReadWorkbookHolder": [
        "easyexcel",
        "easyexcel-cache",
        "easyexcel-csv",
        "easyexcel-io",
        "easyexcel-model",
        "easyexcel-xls",
        "easyexcel-xlsx",
    ],
    "ReadSheetHolder": [
        "easyexcel",
        "easyexcel-csv",
        "easyexcel-io",
        "easyexcel-model",
        "easyexcel-xls",
        "easyexcel-xlsx",
    ],
    "XlsReadWorkbookHolder": ["easyexcel", "easyexcel-xls"],
    "XlsReadSheetHolder": ["easyexcel", "easyexcel-xls"],
    "XlsxReadWorkbookHolder": ["easyexcel", "easyexcel-xlsx"],
    "XlsxReadSheetHolder": ["easyexcel", "easyexcel-xlsx"],
    "AbstractWriteHolder": ["easyexcel"],
    "WriteWorkbookHolder": [
        "easyexcel",
        "easyexcel-csv",
        "easyexcel-model",
        "easyexcel-xls",
        "easyexcel-xlsx",
    ],
    "WriteSheetHolder": [
        "easyexcel",
        "easyexcel-csv",
        "easyexcel-model",
        "easyexcel-xls",
        "easyexcel-xlsx",
    ],
    "WriteTableHolder": ["easyexcel"],
    # Java 配置模型保留在 facade，规范化后的样式和值由 model/格式引擎承载。
    "WriteCellStyle": ["easyexcel", "easyexcel-model", "easyexcel-xls", "easyexcel-xlsx"],
    "StyleProperty": ["easyexcel", "easyexcel-model"],
    "WriteFont": ["easyexcel", "easyexcel-model", "easyexcel-xls", "easyexcel-xlsx"],
    "FontProperty": ["easyexcel", "easyexcel-model"],
    "WriteCellData": ["easyexcel", "easyexcel-model"],
    "ReadCellData": ["easyexcel", "easyexcel-model"],
    # CommentData 是 Java 体验层的格式中立配置；XLS/XLSX 引擎分别负责编码
    # NOTE/TXO/OBJ/MSODRAWING 与 OOXML note，不能把二进制/XML 逻辑搬回 facade。
    "CommentData": ["easyexcel", "easyexcel-xls", "easyexcel-xlsx"],
    "Head": ["easyexcel", "easyexcel-model"],
    "FillConfig": ["easyexcel", "easyexcel-xls", "easyexcel-xlsx"],
    "ContentStyle": ["easyexcel-derive", "easyexcel", "easyexcel-model"],
    "HeadStyle": ["easyexcel-derive", "easyexcel", "easyexcel-model"],
    "ContentFontStyle": ["easyexcel-derive", "easyexcel", "easyexcel-model"],
    "HeadFontStyle": ["easyexcel-derive", "easyexcel", "easyexcel-model"],
    # POI 命名枚举已经以完整 Java 值域保留在 facade；转换后的中立样式由 model
    # 承载，再交给 XLS/XLSX 引擎编码。它们是已有实现，不应在格式 crate 复制同名枚举。
    "FillPatternTypeEnum": ["easyexcel", "easyexcel-model", "easyexcel-xls", "easyexcel-xlsx"],
    "BorderStyleEnum": ["easyexcel", "easyexcel-model", "easyexcel-xls", "easyexcel-xlsx"],
    "HorizontalAlignmentEnum": ["easyexcel", "easyexcel-model", "easyexcel-xls", "easyexcel-xlsx"],
    "VerticalAlignmentEnum": ["easyexcel", "easyexcel-model", "easyexcel-xls", "easyexcel-xlsx"],
    "CellDataTypeEnum": ["easyexcel", "easyexcel-model"],
    "BooleanEnum": ["easyexcel", "easyexcel-model"],
    "CellWriteHandlerContext": ["easyexcel"],
    "RowWriteHandlerContext": ["easyexcel"],
    "SheetWriteHandlerContext": ["easyexcel"],
    "WorkbookWriteHandlerContext": ["easyexcel"],
    # Java 纯静态工具类由 Rust module/free functions 承载；状态与底层 I/O/模型能力
    # 仍归职责 crate，facade module 只提供 Java 风格入口。
    "WriteHandlerUtils": ["easyexcel"],
    "FileUtils": ["easyexcel-io", "easyexcel"],
    "WorkBookUtil": ["easyexcel", "easyexcel-model", "easyexcel-xls", "easyexcel-xlsx"],
    "StringUtils": ["easyexcel-utils", "easyexcel"],
    "EasyExcelTempFileCreationStrategy": ["easyexcel-io", "easyexcel"],
    "BeanMapUtils": ["easyexcel-derive", "easyexcel"],
    "BooleanUtils": ["easyexcel-utils", "easyexcel"],
    "ClassUtils": ["easyexcel-derive", "easyexcel"],
    "ConverterKeyBuild": ["easyexcel", "easyexcel-model"],
    "ConverterUtils": ["easyexcel-derive", "easyexcel", "easyexcel-model"],
    "DefaultConverterLoader": ["easyexcel", "easyexcel-model"],
    "FieldUtils": ["easyexcel-derive", "easyexcel"],
    "FileTypeUtils": ["easyexcel-io", "easyexcel"],
    "IntUtils": ["easyexcel-utils", "easyexcel"],
    "IoUtils": ["easyexcel-io", "easyexcel"],
    "ListUtils": ["easyexcel-utils", "easyexcel"],
    "MapUtils": ["easyexcel-utils", "easyexcel"],
    "PoiUtils": ["easyexcel", "easyexcel-xls", "easyexcel-xlsx"],
    "SheetUtils": ["easyexcel-utils", "easyexcel"],
    "StyleUtil": ["easyexcel", "easyexcel-model", "easyexcel-xls", "easyexcel-xlsx"],
    "Validate": ["easyexcel"],
}


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snake_case(name: str) -> str:
    value = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", value)
    return value.replace("$", "_").lower()


def digit_separated_snake_case(name: str) -> str:
    """同时识别 `use1904windowing` 与 Rust 常见的 `use_1904_windowing`。"""
    value = snake_case(name)
    return re.sub(r"(?<=[a-z])(?=\d)|(?<=\d)(?=[a-z])", "_", value)


def pascal_case(name: str) -> str:
    """将 Java 常量/成员名转换为 Rust enum variant 惯用名。"""
    return "".join(
        part[:1].upper() + part[1:].lower()
        for part in snake_case(name).split("_")
        if part
    )


def method_names(java_name: str) -> set[str]:
    names = {snake_case(java_name), digit_separated_snake_case(java_name)}
    for prefix in ("get", "set", "is", "has", "with"):
        if java_name.startswith(prefix) and len(java_name) > len(prefix):
            tail = java_name[len(prefix) :]
            tail_names = {snake_case(tail), digit_separated_snake_case(tail)}
            names.update(tail_names)
            names.update(f"{prefix}_{tail_name}" for tail_name in tail_names)
    return names


def rust_items(manifest: dict[str, Any]) -> list[dict[str, str]]:
    return [
        {"id": item["id"], "kind": item["kind"], "signature": item["signature"]}
        for package in manifest["packages"]
        for snapshot in package["snapshots"]
        for item in snapshot["items"]
    ]


def rust_indexes(
    items: list[dict[str, str]],
) -> tuple[
    dict[str, list[dict[str, str]]],
    dict[str, list[dict[str, str]]],
    dict[str, list[dict[str, str]]],
    dict[str, list[dict[str, str]]],
]:
    types: dict[str, list[dict[str, str]]] = defaultdict(list)
    members: dict[str, list[dict[str, str]]] = defaultdict(list)
    modules: dict[str, list[dict[str, str]]] = defaultdict(list)
    module_members: dict[str, list[dict[str, str]]] = defaultdict(list)
    for item in items:
        signature = item["signature"]
        if item["kind"] in {"struct", "enum", "trait", "type", "type_alias"}:
            match = re.search(
                r"\b(?:struct|enum|trait|type)\s+"
                r"(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Za-z_][A-Za-z0-9_]*)",
                signature,
            )
            if match:
                name = match.group(1)
                types[name].append(item)
        elif item["kind"] == "module":
            match = re.search(r"\bmod\s+(?:[A-Za-z_][A-Za-z0-9_]*::)*([a-z_][A-Za-z0-9_]*)", signature)
            if match:
                modules[match.group(1)].append(item)
        elif item["kind"] in {"function", "const", "static", "variant", "field"}:
            match = re.search(
                r"::([A-Z][A-Za-z0-9_]*)(?:<[^>]*>)?::[A-Za-z_]", signature
            )
            if match:
                members[match.group(1)].append(item)
            module_match = re.search(
                r"::([a-z_][A-Za-z0-9_]*)::[A-Za-z_][A-Za-z0-9_]*(?:<|\()",
                signature,
            )
            if module_match:
                module_members[module_match.group(1)].append(item)
    return dict(types), dict(members), dict(modules), dict(module_members)


def owner_simple(item: dict[str, Any]) -> str:
    """返回 Java 最内层类型名，对齐 Rust 每对象独立文件/类型的组织方式。"""
    return item["owner"].rsplit(".", 1)[-1].rsplit("$", 1)[-1]


def rust_owner(item: dict[str, Any]) -> str:
    return OWNER_ALIASES.get(owner_simple(item), owner_simple(item))


def rust_member_owner(item: dict[str, Any]) -> str:
    """返回成员真实实现 owner；透明公开 alias 只在成员查找时解引用。"""
    java_owner = owner_simple(item)
    return TRANSPARENT_MEMBER_OWNER_ALIASES.get(
        java_owner,
        MEMBER_OWNER_ALIASES.get(java_owner, rust_owner(item)),
    )


def rust_module_owner(item: dict[str, Any]) -> str:
    return MODULE_OWNER_ALIASES.get(
        owner_simple(item), snake_case(owner_simple(item))
    )


def is_csv_poi_compatibility_member(item: dict[str, Any]) -> bool:
    """识别 Java 为实现 POI 大接口而暴露、但并非 CSV 真实能力的成员。"""
    owner = owner_simple(item)
    if owner not in CSV_STATEFUL_MEMBERS or item["kind"] in {"type", "constructor"}:
        return False
    if item.get("name") in {"equals", "hashCode"}:
        return False
    return item.get("name") not in CSV_STATEFUL_MEMBERS[owner]


def implementation_strategy(
    item: dict[str, Any],
    candidates: list[str],
    *,
    idiomatic_override: bool = False,
) -> str:
    """区分复用、等价替代与真实缺口，避免把 Java 类型数变成复制任务数。"""
    if not candidates:
        return "needs_implementation"
    java_owner = owner_simple(item)
    if idiomatic_override:
        return "idiomatic_alternative"
    if item.get("name") in {"equals", "hashCode"}:
        return "idiomatic_alternative"
    if java_owner == "Head" and item.get("name") == "<init>":
        return "idiomatic_alternative"
    if java_owner == "Head" and item.get("name") in {"getField", "setField"}:
        return "idiomatic_alternative"
    if java_owner == "ConverterKeyBuild" and item.get("name") == "buildKey":
        return "idiomatic_alternative"
    if java_owner == "ConverterKey" and item.get("name") in {"<init>", "getClazz", "setClazz"}:
        return "idiomatic_alternative"
    if java_owner in DECLARATIVE_ANNOTATION_OWNERS:
        return "idiomatic_alternative"
    if item.get("name") in BACKEND_NEUTRAL_MEMBERS.get(java_owner, set()):
        return "idiomatic_alternative"
    if java_owner in RAII_ALTERNATIVE_OWNERS:
        return "idiomatic_alternative"
    if (
        java_owner in MODULE_ONLY_OWNERS
        or java_owner in MODULE_OWNER_ALIASES
        or rust_owner(item) != java_owner
        or (
            item.get("kind") != "type"
            and MEMBER_OWNER_ALIASES.get(java_owner, java_owner) != java_owner
        )
    ):
        return "idiomatic_alternative"
    return "existing_implementation"


def explicit_alternative_candidates(
    java: dict[str, Any], rust: list[dict[str, str]]
) -> list[str]:
    """处理无法用同名关系表达、但已有明确 Rust 语义载体的 Java API。"""
    java_name = java.get("name", "")
    if owner_simple(java) == "EasyExcelConstants" and java_name == "EXCEL_MATH_CONTEXT":
        matches = [
            item
            for item in rust
            if item["kind"] == "static"
            and "::EXCEL_MATH_CONTEXT" in item["signature"]
        ]
        return prefer_primary(matches, "easy_excel_constants")

    if (
        owner_simple(java) == "BuiltinFormats"
        and java.get("kind") == "field"
        and java_name in {"BUILTIN_FORMATS_MAP_CN", "BUILTIN_FORMATS_MAP_US"}
    ):
        matches = [
            item
            for item in rust
            if item["kind"] == "static"
            and f"::{java_name}" in item["signature"]
        ]
        return prefer_primary(matches, "builtin_formats")

    mutable_global_carriers = {
        ("PageReadListener", "BATCH_COUNT"): ("static", "page_read_listener", "BATCH_COUNT"),
        ("UrlImageConverter", "urlConnectTimeout"): (
            "static", "url_image_converter", "URL_CONNECT_TIMEOUT"
        ),
        ("UrlImageConverter", "urlReadTimeout"): (
            "static", "url_image_converter", "URL_READ_TIMEOUT"
        ),
        ("DefaultWriteHandlerLoader", "DEFAULT_WRITE_HANDLER_LIST"): (
            "function", "DefaultWriteHandlerLoader", "default_write_handler_list"
        ),
        ("DateUtils", "defaultDateFormat"): (
            "static", "date_utils", "DEFAULT_DATE_FORMAT_SETTING"
        ),
        ("DateUtils", "defaultLocalDateFormat"): (
            "static", "date_utils", "DEFAULT_LOCAL_DATE_FORMAT_SETTING"
        ),
    }
    mutable_global = mutable_global_carriers.get((owner_simple(java), java_name))
    if mutable_global is not None:
        target_kind, target_owner, target_name = mutable_global
        matches = [
            item
            for item in rust
            if item["kind"] == target_kind
            and f"::{target_owner}::{target_name}" in item["signature"]
        ]
        return prefer_primary(matches, target_owner)

    if (
        owner_simple(java) == "ClassUtils"
        and java.get("kind") == "field"
        and java_name in {"CLASS_CONTENT_CACHE", "CONTENT_CACHE", "FIELD_CACHE"}
    ):
        matches = [
            item
            for item in rust
            if item["kind"] == "module"
            and re.search(r"::class_utils(?:\b|$)", item["signature"])
        ]
        return prefer_primary(matches, "class_utils")

    if (
        owner_simple(java) == "FieldUtils"
        and java.get("kind") == "field"
        and java_name == "nullObjectClass"
    ):
        matches = [
            item
            for item in rust
            if item["kind"] == "function"
            and "::field_utils::null_object_class(" in item["signature"]
        ]
        return prefer_primary(matches, "field_utils")

    if java_name == "clone":
        target_owner = rust_owner(java)
        matches = [
            item
            for item in rust
            if item["kind"] == "function"
            and f"::{target_owner}::clone_data(" in item["signature"]
        ]
        if matches:
            return prefer_primary(matches, target_owner)

    if java_name in {"equals", "hashCode"}:
        target_owner = rust_owner(java)
        pattern = re.compile(rf"(?:::|\s){re.escape(target_owner)}(?:<|\b)")
        matches = [
            item
            for item in rust
            if item["kind"] in {"struct", "enum", "trait", "type", "type_alias"}
            and pattern.search(item["signature"])
        ]
        return prefer_primary(matches, target_owner)

    if owner_simple(java) == "ExcelContentProperty" and java_name in {
        "getField", "setField", "getConverter", "setConverter"
    }:
        target_name = snake_case(java_name)
        matches = [
            item
            for item in rust
            if item["kind"] == "function"
            and f"::ExcelContentProperty::{target_name}(" in item["signature"]
        ]
        return prefer_primary(matches, "ExcelContentProperty")

    if (
        owner_simple(java) == "FileTypeUtils"
        and java.get("kind") == "field"
        and java_name == "defaultImageType"
    ):
        matches = [
            item
            for item in rust
            if item["kind"] in {"static", "const"}
            and "::file_type_utils::DEFAULT_IMAGE_TYPE" in item["signature"]
        ]
        return prefer_primary(matches, "file_type_utils")

    if owner_simple(java) == "ConverterUtils" and java_name == "defaultClassGeneric":
        matches = [
            item
            for item in rust
            if item["kind"] == "function"
            and "::converter_utils::default_class_generic(" in item["signature"]
        ]
        return prefer_primary(matches, "converter_utils")

    if owner_simple(java) == "PoiUtils" and java_name == "CUSTOM_HEIGHT":
        matches = [
            item
            for item in rust
            if item["kind"] == "function"
            and "::poi_utils::custom_height(" in item["signature"]
        ]
        return prefer_primary(matches, "poi_utils")

    target_owner = rust_owner(java)
    if java_name in BACKEND_NEUTRAL_ENUM_MEMBERS.get(target_owner, set()):
        target_name = snake_case(java_name)
        matches = [
            item
            for item in rust
            if item["kind"] == "function"
            and f"::{target_owner}::{target_name}(" in item["signature"]
        ]
        return prefer_primary(matches, target_owner)

    if is_csv_poi_compatibility_member(java):
        # 绑定到现有 CSV owner，而不是要求 Rust 制造与 POI 同构的空操作方法。
        # 后续行为证据验证的是“CSV 不具备该工作簿能力”的显式边界。
        target_owner = owner_simple(java)
        pattern = re.compile(rf"(?:::|\s){re.escape(target_owner)}(?:<|\b)")
        matches = [
            item
            for item in rust
            if item["kind"] in {"struct", "enum", "trait", "type", "type_alias"}
            and pattern.search(item["signature"])
        ]
        return prefer_primary(matches, target_owner)

    if owner_simple(java) == "StyleProperty" and java_name == "build":
        annotation_owner = (
            "HeadStyle" if "HeadStyle;" in java.get("descriptor", "") else "ContentStyle"
        )
        matches = [
            item
            for item in rust
            if item["kind"] == "function"
            and f"::{annotation_owner}::to_property(" in item["signature"]
        ]
        return prefer_primary(matches, annotation_owner)

    if owner_simple(java) == "FontProperty" and java_name == "build":
        annotation_owner = (
            "HeadFontStyle"
            if "HeadFontStyle;" in java.get("descriptor", "")
            else "ContentFontStyle"
        )
        matches = [
            item
            for item in rust
            if item["kind"] == "function"
            and f"::{annotation_owner}::to_property(" in item["signature"]
        ]
        return prefer_primary(matches, annotation_owner)

    generated_enum_value_of = (
        java_name == "valueOf"
        and java.get("descriptor", "").startswith("(Ljava/lang/String;)")
    )
    if java_name == "values" or generated_enum_value_of:
        target_owner = rust_owner(java)
        target_name = "ALL" if java_name == "values" else "from_str"
        target_kinds = {"const", "static"} if java_name == "values" else {"function", "impl"}
        matches = [
            item
            for item in rust
            if item["kind"] in target_kinds
            and (
                f"::{target_owner}::{target_name}" in item["signature"]
                or (
                    java_name == "valueOf"
                    and "FromStr for" in item["signature"]
                    and re.search(rf"(?:::|\s){re.escape(target_owner)}(?:<|\b)", item["signature"])
                )
            )
        ]
        if matches:
            return prefer_primary(matches, target_owner)

    if owner_simple(java) != "Ehcache":
        return []

    target_owner: str | None = None
    target_name: str | None = None
    target_kind = "type"
    if java["kind"] == "type":
        # Rust 不保留 Ehcache 名义类型。共享字符串的后端选择由引擎策略承载，
        # Java ReadCache 生命周期则由 facade trait 适配；不能把磁盘+活跃缓存
        # 的 Ehcache 错误等同为单一的无淘汰 Moka 内存缓存。
        target_owner = "SharedStringCachePolicy"
    elif java["kind"] == "constructor":
        # Ehcache 的两个构造参数只配置它自己的活跃层容量。Rust 已彻底移除
        # Ehcache/CacheManager；直接磁盘后端由 RAII 临时文件持有，不能把这些
        # 参数伪装成 SimpleReadCacheSelector 的 XML 大小阈值。
        target_kind = "function"
        target_owner, target_name = "FileCache", "new"
    elif java_name in {"init", "put", "get", "destroy"}:
        target_kind = "function"
        target_owner, target_name = "ReadCache", snake_case(java_name)
    elif java_name == "putFinished":
        target_kind = "function"
        target_owner, target_name = "ReadCache", "put_finished"
    elif java["kind"] == "field":
        # 三个 Java 字段只控制 Ehcache 内部批处理/调试日志，不是工作簿可配置状态。
        # 绑定到现有策略载体，不能为了逐字段同名重新引入 Ehcache 常量。
        target_owner = "SharedStringCachePolicy"
    else:
        return []

    if target_kind == "type":
        pattern = re.compile(rf"(?:::|\s){re.escape(target_owner or '')}(?:<|\b)")
        matches = [
            item
            for item in rust
            if item["kind"] in {"struct", "enum", "trait", "type", "type_alias"}
            and pattern.search(item["signature"])
        ]
        return prefer_primary(matches, target_owner or "")

    matches = [
        item
        for item in rust
        if item["kind"] == "function"
        and (target_owner is None or f"::{target_owner}::" in item["signature"])
        and re.search(rf"::{re.escape(target_name or '')}(?:<|\()", item["signature"])
    ]
    return prefer_primary(matches, target_owner or target_name or "")


def semantic_note(
    item: dict[str, Any],
    strategy: str,
    *,
    default_constructor_alternative: bool = False,
) -> str:
    """为自动候选保留可审计的所有权/替代原因。"""
    owner = owner_simple(item)
    if owner == "Ehcache":
        return (
            "Java Ehcache is retired rather than copied: easyexcel-cache SharedStringCachePolicy "
            "selects ownership-managed Memory/File/Moka storage while facade ReadCache and "
            "ReadCacheSelector preserve the observable lifecycle; Ehcache implementation constants "
            "have no standalone Rust API and are carried by the policy type; "
            "evidence not yet verified"
        )
    if owner == "MemberUtils":
        return (
            "Java runtime reflection helper is replaced by compile-time schema/derive "
            "metadata exposed through class_utils; evidence not yet verified"
        )
    if owner == "ExcelContentProperty" and item.get("name") in {
        "getField", "setField", "getConverter", "setConverter"
    }:
        return (
            "Java reflection Field/Converter objects are represented by existing derive/schema "
            "field and converter registration keys on ExcelContentProperty; evidence not yet verified"
        )
    if owner == "Head" and item.get("name") == "<init>":
        return (
            "Java reflection Field construction is represented by the existing Head model plus "
            "a backend-neutral field key; null collections and booleans are normalized at the "
            "constructor boundary; evidence not yet verified"
        )
    if owner == "Head" and item.get("name") in {"getField", "setField"}:
        return (
            "Java reflection Field is represented by the existing backend-neutral field key on "
            "Head while fieldName remains an independent property; evidence not yet verified"
        )
    if owner == "ConverterKeyBuild" and item.get("name") == "buildKey":
        return (
            "Java Class buildKey overload is represented by a descriptor-specific TypeId entry; "
            "the existing generic Rust shortcut remains available; evidence not yet verified"
        )
    if owner == "ConverterKey" and item.get("name") in {"<init>", "getClazz", "setClazz"}:
        return (
            "Java Class identity is represented by TypeId on the existing converter dispatch key; "
            "primitive/boxed normalization is unnecessary in Rust; evidence not yet verified"
        )
    if owner in DECLARATIVE_ANNOTATION_OWNERS:
        return (
            "Java annotation metadata is represented by derive(ExcelRow)/excel attributes and "
            "the existing backend-neutral runtime parameter object; format encoding remains in "
            "the responsible engine crate; evidence not yet verified"
        )
    if owner == "ClassUtils" and item.get("name") in {
        "CLASS_CONTENT_CACHE", "CONTENT_CACHE", "FIELD_CACHE"
    }:
        return (
            "Java mutable reflection caches are replaced by derive-generated static schema and "
            "Rust monomorphization; the existing class_utils module is the lifecycle carrier and "
            "no second global cache is introduced; evidence not yet verified"
        )
    if owner == "FieldUtils" and item.get("name") == "nullObjectClass":
        return (
            "Java mutable Class sentinel is represented by TypeId::of::<NullObject>() through the "
            "existing field_utils carrier; evidence not yet verified"
        )
    if owner in RAII_ALTERNATIVE_OWNERS:
        return (
            "Java deleteOnExit temporary files are represented by the existing easyexcel-io "
            "strategy returning NamedTempFile/TempDir RAII guards; directory recovery remains "
            "engine-owned; evidence not yet verified"
        )
    if item.get("name") in BACKEND_NEUTRAL_MEMBERS.get(owner, set()):
        return (
            "Java member exposes a reflection or POI backend type; Rust uses the existing "
            "TypeId/stable-key or easyexcel-model backend-neutral carrier and converts only at "
            "the XLS/XLSX engine boundary; evidence not yet verified"
        )
    if item.get("name") in BACKEND_NEUTRAL_ENUM_MEMBERS.get(rust_owner(item), set()):
        return (
            "Java enum getter returns a POI or Commons backend object; Rust exposes the existing "
            "backend-neutral enum/byte carrier and converts only at the format-engine boundary; "
            "evidence not yet verified"
        )
    if item.get("name") == "clone":
        return (
            "Java clone() is represented by the existing explicit clone_data method backed by "
            "Rust Clone semantics; evidence not yet verified"
        )
    if is_csv_poi_compatibility_member(item):
        return (
            "Java member is a POI compatibility slot whose CSV implementation is no-op, fixed "
            "value, null, or an unsupported capability; Rust keeps the existing CSV owner and "
            "format capability boundary instead of copying a same-name empty method; behavior "
            "evidence must prove the observable boundary before verification"
        )
    if owner == "StyleProperty" and item.get("name") == "build":
        return (
            "Java overloaded annotation build is represented by the existing HeadStyle or "
            "ContentStyle to_property conversion; XLS/XLSX encoding remains engine-owned; "
            "evidence not yet verified"
        )
    if owner == "FontProperty" and item.get("name") == "build":
        return (
            "Java overloaded annotation build is represented by the existing HeadFontStyle or "
            "ContentFontStyle to_property conversion; font encoding remains engine-owned; "
            "evidence not yet verified"
        )
    if item.get("name") == "values" or (
        item.get("name") == "valueOf"
        and item.get("descriptor", "").startswith("(Ljava/lang/String;)")
    ):
        return (
            "Java compiler-generated enum values/valueOf API is represented by the existing "
            "Rust enum ALL declaration order and strict FromStr contract; enum variants retain "
            "their real implementation owner; evidence not yet verified"
        )
    if owner in TRANSPARENT_MEMBER_OWNER_ALIASES:
        return (
            "Java package-path context is an existing public Rust type alias whose methods are "
            "implemented by the shared runtime context type; evidence not yet verified"
        )
    if item.get("name") == "equals":
        return (
            "Java equals(Object) is represented by Rust PartialEq/Eq semantics on the existing "
            "type; compile and behavior evidence must prove equality before verification"
        )
    if item.get("name") == "hashCode":
        return (
            "Java hashCode() is represented by Rust Hash semantics on the existing type; compile "
            "and behavior evidence must prove hashing before verification"
        )
    if default_constructor_alternative:
        return (
            "Java no-argument construction is represented by the existing Rust new() or Default "
            "contract; compile and behavior evidence must prove every Java default value"
        )
    if strategy == "idiomatic_alternative":
        return "deterministic idiomatic-alternative candidate; evidence not yet verified"
    if strategy == "existing_implementation":
        return "deterministic existing-implementation candidate; evidence not yet verified"
    return "no existing or idiomatic Rust carrier was found"


def implementation_carriers(
    item: dict[str, Any], candidates: list[str]
) -> list[str]:
    """逐 Rust public ID 记录实际公开实现 crate，不按 owner 批量扩张。"""
    if not candidates:
        return []
    return sorted({candidate.split(":", 1)[0] for candidate in candidates})


def capability_carriers(
    item: dict[str, Any], candidates: list[str]
) -> list[str]:
    """记录 owner 的下游能力协作者，不把它们冒充逐成员 public API owner。"""
    if not candidates:
        return []
    public_carriers = set(implementation_carriers(item, candidates))
    return [
        carrier
        for carrier in OWNER_CAPABILITY_CARRIERS.get(owner_simple(item), [])
        if carrier not in public_carriers
    ]


def prefer_primary(items: list[dict[str, str]], owner: str) -> list[str]:
    """保留最佳公开路径；同分项继续按歧义处理，不擅自挑选。"""
    if not items:
        return []

    def score(item: dict[str, str]) -> tuple[int, int]:
        signature = item["signature"]
        root_path = f"easyexcel::{owner}"
        return (
            0 if root_path in signature else 1,
            signature.count("::"),
        )

    best = min(score(item) for item in items)
    return sorted({item["id"] for item in items if score(item) == best})


def type_candidates(java: dict[str, Any], rust: list[dict[str, str]]) -> list[str]:
    name = rust_owner(java)
    pattern = re.compile(rf"(?:::|\s){re.escape(name)}(?:<|\b)")
    matches = [
        item
        for item in rust
        if item["kind"] in {"struct", "enum", "trait", "type", "type_alias"}
        and pattern.search(item["signature"])
    ]
    return prefer_primary(matches, name)


def module_candidates(java: dict[str, Any], rust: list[dict[str, str]]) -> list[str]:
    """将 Java 纯静态工具类映射到 Rust module，而不是制造无状态空 struct。"""
    module_name = rust_module_owner(java)
    matches = [
        item
        for item in rust
        if item["kind"] == "module"
        and re.search(rf"::{re.escape(module_name)}\b", item["signature"])
    ]
    return prefer_primary(matches, module_name)


def marker_interface_candidates(
    java: dict[str, Any],
    rust_members: list[dict[str, str]],
    java_member_owners: set[str],
) -> list[str]:
    """为空 Java marker interface 选择同 owner 的唯一 Rust 查询成员。"""
    if (
        java.get("type_kind") != "interface"
        or java["owner"] in java_member_owners
        or not rust_members
    ):
        return []
    return prefer_primary(rust_members, rust_owner(java))


def easyexcel_factory_names(java: dict[str, Any]) -> set[str] | None:
    if owner_simple(java) != "EasyExcelFactory":
        return None
    java_name = java["name"]
    descriptor = java["id"].split("#", 1)[1]
    parameters = descriptor[descriptor.find("(") + 1 : descriptor.find(")")]
    if java_name == "read":
        if not parameters:
            return {"reader"}
        if parameters.startswith("Ljava/io/InputStream;"):
            return (
                {"reader_from_input_stream"}
                if parameters == "Ljava/io/InputStream;"
                else {"read_from_input_stream"}
            )
        if "ReadListener" in parameters:
            return {"read"}
        return {"reader_from_path"}
    if java_name == "readSheet":
        return {
            "": {"read_sheet"},
            "Ljava/lang/Integer;": {"read_sheet_index"},
            "Ljava/lang/String;": {"read_sheet_name"},
            "Ljava/lang/Integer;Ljava/lang/String;": {"read_sheet_with"},
        }.get(parameters, set())
    if java_name == "write":
        if not parameters:
            return {"writer"}
        if parameters.startswith("Ljava/io/OutputStream;"):
            return {"writer_to_output_stream"}
        return {"write"} if "Ljava/lang/Class;" in parameters else {"writer_to_path"}
    if java_name == "writerSheet":
        return {
            "": {"writer_sheet_builder"},
            "Ljava/lang/Integer;": {"writer_sheet_builder_index"},
            "Ljava/lang/String;": {"writer_sheet_builder_name"},
            "Ljava/lang/Integer;Ljava/lang/String;": {"writer_sheet_builder_with"},
        }.get(parameters, set())
    if java_name == "writerTable":
        return {
            "": {"writer_table_builder_default"},
            "Ljava/lang/Integer;": {"writer_table_builder"},
        }.get(parameters, set())
    return None


def excel_reader_names(java: dict[str, Any]) -> set[str] | None:
    if owner_simple(java) != "ExcelReader":
        return None
    java_name = java["name"]
    descriptor = java["id"].split("#", 1)[1]
    parameters = descriptor[descriptor.find("(") + 1 : descriptor.find(")")]
    if java_name == "read":
        return {"read_deprecated"} if not parameters else {"read"}
    if java_name == "getAnalysisContext":
        return {"get_analysis_context"}
    return None


def excel_writer_names(java: dict[str, Any]) -> set[str] | None:
    if owner_simple(java) != "ExcelWriter":
        return None
    java_name = java["name"]
    descriptor = java["id"].split("#", 1)[1]
    parameters = descriptor[descriptor.find("(") + 1 : descriptor.find(")")]
    if java_name == "<init>":
        return {"from_write_workbook"}
    if java_name == "write":
        supplier = parameters.startswith("Ljava/util/function/Supplier;")
        table = parameters.endswith("Lcom/alibaba/excel/write/metadata/WriteTable;")
        if supplier and table:
            return {"write_with_table_supplier"}
        if supplier:
            return {"write_with_supplier"}
        if table:
            return {"write_with_table"}
        return {"write"}
    if java_name == "fill":
        supplier = parameters.startswith("Ljava/util/function/Supplier;")
        configured = "Lcom/alibaba/excel/write/metadata/fill/FillConfig;" in parameters
        if supplier and configured:
            return {"fill_with_config_supplier"}
        if supplier:
            return {"fill_with_supplier"}
        if configured:
            return {"fill"}
        return {"fill_default"}
    if java_name == "writeContext":
        return {"write_context"}
    return None


def excel_builder_names(java: dict[str, Any]) -> set[str] | None:
    if owner_simple(java) != "ExcelBuilder" or java["name"] != "addContent":
        return None
    descriptor = java["id"].split("#", 1)[1]
    parameters = descriptor[descriptor.find("(") + 1 : descriptor.find(")")]
    return (
        {"add_content_with_table"}
        if parameters.endswith("Lcom/alibaba/excel/write/metadata/WriteTable;")
        else {"add_content"}
    )


def excel_builder_impl_names(java: dict[str, Any]) -> set[str] | None:
    if owner_simple(java) != "ExcelBuilderImpl":
        return None
    if java["name"] == "<init>":
        return {"from_write_workbook"}
    if java["name"] != "addContent":
        return None
    descriptor = java["id"].split("#", 1)[1]
    parameters = descriptor[descriptor.find("(") + 1 : descriptor.find(")")]
    return (
        {"add_content_with_table"}
        if parameters.endswith("Lcom/alibaba/excel/write/metadata/WriteTable;")
        else {"add_content"}
    )


def excel_analyser_impl_names(java: dict[str, Any]) -> set[str] | None:
    if owner_simple(java) == "ExcelAnalyserImpl" and java["name"] == "<init>":
        return {"from_read_workbook"}
    return None


def default_format_read_context_names(java: dict[str, Any]) -> set[str] | None:
    if owner_simple(java) in {
        "DefaultCsvReadContext",
        "DefaultXlsReadContext",
        "DefaultXlsxReadContext",
    } and java["name"] == "<init>":
        return {"from_read_workbook"}
    return None


def number_data_formatter_names(java: dict[str, Any]) -> set[str] | None:
    if owner_simple(java) != "NumberDataFormatterUtils" or java["name"] != "format":
        return None
    return (
        {"format"}
        if "GlobalConfiguration" in java["id"]
        else {"format_with_options"}
    )


def number_utils_names(java: dict[str, Any]) -> set[str] | None:
    """Java NumberUtils 的 parse API 均包含 ExcelContentProperty。"""
    if owner_simple(java) != "NumberUtils":
        return None
    name = java.get("name", "")
    if name in {
        "parseShort",
        "parseLong",
        "parseInteger",
        "parseFloat",
        "parseBigDecimal",
        "parseByte",
        "parseDouble",
    }:
        return {f"{snake_case(name)}_with_property"}
    return None


def write_handler_utils_names(java: dict[str, Any]) -> set[str] | None:
    """区分 Handler 工具的 runOwn 重载与完整 Cell metadata 构造。"""
    if owner_simple(java) != "WriteHandlerUtils":
        return None
    name = java.get("name", "")
    if name == "createCellWriteHandlerContext":
        return {"create_cell_write_handler_context_with_metadata"}
    if name in {
        "beforeWorkbookCreate",
        "afterWorkbookCreate",
        "beforeSheetCreate",
        "afterSheetCreate",
    } and "Z)V" in java.get("descriptor", ""):
        return {f"{snake_case(name)}_with_run_own"}
    return None


def converter_utils_names(java: dict[str, Any]) -> set[str] | None:
    """将 Java 反射转换入口绑定到 registry/schema 驱动的动态路径。"""
    if owner_simple(java) != "ConverterUtils":
        return None
    if java.get("name") == "convertToStringMap":
        return {"convert_read_cells_to_string_map"}
    return None


def read_cell_data_names(java: dict[str, Any]) -> set[str] | None:
    """将 Java ReadCellData 构造/工厂重载绑定到现有后端中立值模型。"""
    if owner_simple(java) != "ReadCellData":
        return None
    name = java.get("name")
    descriptor = java.get("descriptor", "")
    if name == "<init>":
        return {
            "()V": {"empty"},
            "(Lcom/alibaba/excel/enums/CellDataTypeEnum;)V": {"from_type"},
            "(Lcom/alibaba/excel/enums/CellDataTypeEnum;Ljava/lang/String;)V": {"from_type_and_string"},
            "(Ljava/lang/Boolean;)V": {"from_boolean"},
            "(Ljava/lang/Object;)V": {"new_instance"},
            "(Ljava/lang/String;)V": {"from_string"},
            "(Ljava/math/BigDecimal;)V": {"from_number"},
        }.get(descriptor, set())
    if name == "newEmptyInstance":
        return {"empty"} if descriptor.startswith("()") else {"new_empty_instance"}
    if name == "newInstance":
        return {"from_boolean"} if descriptor.startswith("(Ljava/lang/Boolean;)") else {"new_instance"}
    if name == "newInstanceOriginal":
        return {"new_instance_original"}
    if name == "clone":
        return {"clone_data"}
    return None


def analysis_context_names(java: dict[str, Any]) -> set[str] | None:
    """区分 Java AnalysisContext 的同名 getter/setter 重载。"""
    if owner_simple(java) != "AnalysisContext":
        return None
    descriptor = java.get("descriptor", "")
    if java.get("name") == "readRowHolder" and not descriptor.startswith("()"):
        return {"set_read_row_holder"}
    if java.get("name") == "readSheetList" and not descriptor.startswith("()"):
        return {"set_read_sheet_list"}
    return None


def head_names(java: dict[str, Any]) -> set[str] | None:
    """将 Java Head 的反射 Field 构造形状绑定到后端中立字段键构造器。"""
    if owner_simple(java) == "Head" and java.get("name") == "<init>":
        return {"from_java_fields"}
    return None


def converter_key_build_names(java: dict[str, Any]) -> set[str] | None:
    """按 Java buildKey 重载选择 TypeId 后端中立入口。"""
    if owner_simple(java) != "ConverterKeyBuild" or java.get("name") != "buildKey":
        return None
    descriptor = java.get("descriptor", "")
    return (
        {"build_key_for_type_and_cell_data"}
        if "CellDataTypeEnum;" in descriptor
        else {"build_key_for_type"}
    )


def holder_constructor_names(java: dict[str, Any]) -> set[str] | None:
    """按 Java Holder 构造器参数选择真实 Rust 生命周期入口。"""
    if java.get("kind") != "constructor" or ".metadata.holder." not in java["owner"]:
        return None
    owner = owner_simple(java)
    descriptor = java.get("descriptor", "")
    if descriptor == "()V":
        return {
            "ReadWorkbookHolder": {"new"},
            "XlsReadWorkbookHolder": {"new"},
            "ReadSheetHolder": {"default_construction"},
            "XlsReadSheetHolder": {"default_construction"},
            "WriteSheetHolder": {"default_construction"},
        }.get(owner, set())
    if owner in {
        "ReadWorkbookHolder",
        "CsvReadWorkbookHolder",
        "XlsReadWorkbookHolder",
        "XlsxReadWorkbookHolder",
    }:
        return {"from_read_workbook"}
    if owner in {
        "ReadSheetHolder",
        "CsvReadSheetHolder",
        "XlsReadSheetHolder",
        "XlsxReadSheetHolder",
    }:
        return {"from_read_sheet"}
    if owner == "ReadRowHolder":
        return {"new_with_metadata"}
    if owner in {"AbstractReadHolder", "AbstractWriteHolder"}:
        return {"from_parameter"}
    if owner == "WriteWorkbookHolder":
        return {"from_write_workbook"}
    if owner == "WriteSheetHolder":
        return {"from_write_sheet"}
    if owner == "WriteTableHolder":
        return {"from_write_table"}
    return None


def temp_file_strategy_names(java: dict[str, Any]) -> set[str] | None:
    """将 Java File/deleteOnExit 形状绑定到 easyexcel-io 的 RAII 策略。"""
    if owner_simple(java) != "EasyExcelTempFileCreationStrategy":
        return None
    if java.get("name") != "<init>":
        return None
    return {"new"} if java.get("descriptor") == "()V" else {"from_directory"}


def page_read_listener_names(java: dict[str, Any]) -> set[str] | None:
    """将 Java Consumer 构造器绑定到不要求 AnalysisContext/Result 的 Rust 入口。"""
    if owner_simple(java) != "PageReadListener" or java.get("name") != "<init>":
        return None
    if java.get("descriptor") == "(Ljava/util/function/Consumer;)V":
        return {"from_consumer"}
    return {"from_consumer_with_batch_count"}


def excel_writer_signature_matches(java: dict[str, Any], signature: str) -> bool:
    """Disambiguate inherent fluent methods from same-name trait projections."""
    if owner_simple(java) != "ExcelWriter":
        return True
    if java["name"] == "fill":
        return "Result<&mut Self>" in signature
    if java["name"] == "finish":
        return "::finish(&mut self) ->" in signature
    return True


def excel_builder_impl_signature_matches(java: dict[str, Any], signature: str) -> bool:
    """选择 Java 实现类的 void trait 投影，不混入 fluent 门面重载。"""
    if owner_simple(java) != "ExcelBuilderImpl":
        return True
    if java["name"] == "fill":
        return "Result<()>" in signature
    if java["name"] == "finish":
        return "::finish(&mut self, bool) ->" in signature
    return True


def member_candidates(java: dict[str, Any], rust: list[dict[str, str]]) -> list[str]:
    owner = rust_member_owner(java)
    java_name = java["name"]
    explicit_names = easyexcel_factory_names(java)
    if explicit_names is None:
        explicit_names = excel_reader_names(java)
    if explicit_names is None:
        explicit_names = excel_writer_names(java)
    if explicit_names is None:
        explicit_names = excel_builder_names(java)
    if explicit_names is None:
        explicit_names = excel_builder_impl_names(java)
    if explicit_names is None:
        explicit_names = excel_analyser_impl_names(java)
    if explicit_names is None:
        explicit_names = default_format_read_context_names(java)
    if explicit_names is None:
        explicit_names = number_data_formatter_names(java)
    if explicit_names is None:
        explicit_names = number_utils_names(java)
    if explicit_names is None:
        explicit_names = write_handler_utils_names(java)
    if explicit_names is None:
        explicit_names = converter_utils_names(java)
    if explicit_names is None:
        explicit_names = read_cell_data_names(java)
    if explicit_names is None:
        explicit_names = analysis_context_names(java)
    if explicit_names is None:
        explicit_names = head_names(java)
    if explicit_names is None:
        explicit_names = converter_key_build_names(java)
    if explicit_names is None:
        explicit_names = holder_constructor_names(java)
    if explicit_names is None:
        explicit_names = temp_file_strategy_names(java)
    if explicit_names is None:
        explicit_names = page_read_listener_names(java)
    names = (
        explicit_names
        if explicit_names is not None
        else {"new"}
        if java_name == "<init>"
        else method_names(java_name)
    )
    if java_name == "clone":
        names.add("clone_data")
    if java["kind"] == "field":
        names.update({java_name, snake_case(java_name), pascal_case(java_name)})
        if owner == "ImageType" and java_name.startswith("PICTURE_TYPE_"):
            names.add(pascal_case(java_name.removeprefix("PICTURE_TYPE_")))
        if owner == "ByteOrderMarkEnum":
            rust_bom_variant = {
                "UTF_8": "Utf8", "UTF_16BE": "Utf16Be", "UTF_16LE": "Utf16Le",
                "UTF_32BE": "Utf32Be", "UTF_32LE": "Utf32Le",
            }.get(java_name)
            if rust_bom_variant:
                names.add(rust_bom_variant)
    owner_pattern = re.compile(rf"(?:::|\s){re.escape(owner)}(?:<[^>]*>)?::")
    result: list[dict[str, str]] = []
    for item in rust:
        signature = item["signature"]
        if not owner_pattern.search(signature):
            continue
        if not excel_writer_signature_matches(java, signature):
            continue
        if not excel_builder_impl_signature_matches(java, signature):
            continue
        if java["kind"] == "field":
            if any(
                f"::{name}:" in signature
                or f"::{name} " in signature
                or signature.endswith(f"::{name}")
                for name in names
            ):
                result.append(item)
        elif item["kind"] == "function" and any(
            re.search(rf"::{re.escape(name)}(?:<|\()", signature) for name in names
        ):
            result.append(item)
    return prefer_primary(result, owner)


def module_member_candidates(
    java: dict[str, Any], rust: list[dict[str, str]]
) -> list[str]:
    module_name = rust_module_owner(java)
    names = number_data_formatter_names(java)
    if names is None:
        names = number_utils_names(java)
    if names is None:
        names = write_handler_utils_names(java)
    if names is None:
        names = converter_utils_names(java)
    if names is None:
        names = method_names(java["name"])
    result = []
    for item in rust:
        if item["kind"] not in {"function", "const", "static"}:
            continue
        signature = item["signature"]
        if not re.search(rf"::{re.escape(module_name)}::", signature):
            continue
        if java["kind"] == "field":
            field_names = set(names)
            field_names.update(
                {java["name"], snake_case(java["name"]), pascal_case(java["name"])}
            )
            if any(
                f"::{name}:" in signature
                or f"::{name} " in signature
                or signature.endswith(f"::{name}")
                for name in field_names
            ):
                result.append(item)
        elif any(
            re.search(rf"::{re.escape(name)}(?:<|\()", signature) for name in names
        ):
            result.append(item)
    return prefer_primary(result, module_name)


def suggest(java_manifest: dict[str, Any], rust_manifest: dict[str, Any]) -> list[dict[str, Any]]:
    rust = rust_items(rust_manifest)
    type_index, member_index, module_index, module_member_index = rust_indexes(rust)
    java_items = [*java_manifest["types"], *java_manifest["members"]]
    java_member_owners = {item["owner"] for item in java_manifest["members"]}
    members_by_owner: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for member in java_manifest["members"]:
        if member["kind"] != "constructor":
            members_by_owner[member["owner"]].append(member)
    static_utility_owners = {
        owner
        for owner, members in members_by_owner.items()
        if members and all(" static " in member.get("declaration", "") for member in members)
    }
    static_utility_owners.update(
        item["owner"]
        for item in java_manifest["types"]
        if owner_simple(item) in MODULE_ONLY_OWNERS
    )
    static_utility_owners.difference_update(
        item["owner"]
        for item in java_manifest["types"]
        if owner_simple(item) in NOMINAL_STATIC_UTILITY_OWNERS
    )
    entries = []
    for item in sorted(java_items, key=lambda value: value["id"]):
        idiomatic_override = False
        default_constructor_alternative = False
        candidates = explicit_alternative_candidates(item, rust)
        if candidates:
            idiomatic_override = True
        if item["kind"] == "type":
            if candidates:
                pass
            elif item["owner"] in static_utility_owners:
                module_name = rust_module_owner(item)
                candidates = module_candidates(item, module_index.get(module_name, []))
                idiomatic_override = bool(candidates)
            else:
                candidates = []
            if not candidates:
                candidates = type_candidates(item, type_index.get(rust_owner(item), []))
            if not candidates:
                candidates = marker_interface_candidates(
                    item,
                    member_index.get(rust_owner(item), []),
                    java_member_owners,
                )
                idiomatic_override = bool(candidates)
        else:
            member_owner = rust_member_owner(item)
            if candidates:
                pass
            elif item["owner"] in static_utility_owners:
                module_name = rust_module_owner(item)
                if item["kind"] == "constructor":
                    candidates = module_candidates(item, module_index.get(module_name, []))
                else:
                    candidates = module_member_candidates(
                        item, module_member_index.get(module_name, [])
                    )
                idiomatic_override = bool(candidates)
            else:
                candidates = []
            if not candidates:
                candidates = member_candidates(item, member_index.get(member_owner, []))
            if (
                not candidates
                and item["kind"] == "constructor"
                and item.get("descriptor") == "()V"
            ):
                # Rust 的无参构造惯例允许 `Default::default()`；先绑定现有类型，
                # 后续 compile/behavior 证据必须真实调用 Default 并核对 Java 默认值。
                candidates = type_candidates(item, type_index.get(rust_owner(item), []))
                default_constructor_alternative = bool(candidates)
                idiomatic_override = bool(candidates)
            if not candidates and item["kind"] == "constructor" and owner_simple(item).startswith(
                "Abstract"
            ):
                # Java javap 会列出 abstract class 的 public 构造器，但抽象类本身
                # 不可实例化；Rust trait 类型就是对应的构造边界。
                candidates = type_candidates(item, type_index.get(rust_owner(item), []))
                idiomatic_override = bool(candidates)
        status = "unmapped" if not candidates else "candidate" if len(candidates) == 1 else "ambiguous"
        strategy = implementation_strategy(
            item,
            candidates,
            idiomatic_override=idiomatic_override,
        )
        if strategy not in IMPLEMENTATION_STRATEGIES:
            raise AssertionError(f"unknown implementation strategy: {strategy}")
        entries.append(
            {
                "java_id": item["id"],
                "status": status,
                "implementation_strategy": strategy,
                "implementation_carriers": implementation_carriers(item, candidates),
                "capability_carriers": capability_carriers(item, candidates),
                "rust_ids": candidates,
                "compile_probes": [],
                "behavior_tests": [],
                "java_golden": [],
                "semantic_notes": semantic_note(
                    item,
                    strategy,
                    default_constructor_alternative=default_constructor_alternative,
                ),
            }
        )
    return entries


def documented_rust_extensions(
    rust_manifest: dict[str, Any], entries: list[dict[str, Any]]
) -> list[dict[str, Any]]:
    """登记没有被任何 Java 候选占用的 Rust public API 补集。

    该补集不提高 Java verified 数；它只保证全 workspace 的公开 Rust API 没有
    游离在门禁之外。后续某项成为 Java carrier 时，会因 Rust ID 进入 entries 而
    自动从补集中移除。
    """
    mapped = {
        rust_id
        for entry in entries
        for rust_id in entry.get("rust_ids", [])
        if isinstance(rust_id, str)
    }
    unique_items: dict[str, dict[str, str]] = {}
    item_modes: dict[str, set[str]] = defaultdict(set)
    for package in rust_manifest["packages"]:
        for snapshot in package["snapshots"]:
            for item in snapshot["items"]:
                item_modes[item["id"]].add(snapshot["mode"])
    for item in rust_items(rust_manifest):
        unique_items.setdefault(item["id"], item)
    extensions = []
    for rust_id, item in sorted(unique_items.items()):
        if rust_id in mapped:
            continue
        package = rust_id.split(":", 1)[0]
        extensions.append(
            {
                "rust_id": rust_id,
                "status": "documented_extension",
                "classification": "unmapped_rust_public_api",
                "implementation_carriers": [package],
                "capability_carriers": [],
                "kind": item["kind"],
                "modes": sorted(item_modes[rust_id]),
                "signature": item["signature"],
                "semantic_notes": (
                    f"published Rust {item['kind']} owned by {package}; no Java 4.0.3 "
                    "public API candidate uses this exact Rust ID"
                ),
            }
        )
    return extensions


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--java-api", required=True, type=Path)
    parser.add_argument("--rust-api", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    java = load(args.java_api)
    rust = load(args.rust_api)
    entries = suggest(java, rust)
    mapping = {
        "schema_version": 2,
        "authority": "java_easyexcel_4.0.3_javap_public_api",
        "java_manifest_sha256": sha256(args.java_api),
        "rust_manifest_sha256": sha256(args.rust_api),
        "rust_extensions": documented_rust_extensions(rust, entries),
        "entries": entries,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(mapping, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
