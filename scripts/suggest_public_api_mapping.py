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
}


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snake_case(name: str) -> str:
    value = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", value)
    return value.replace("$", "_").lower()


def method_names(java_name: str) -> set[str]:
    names = {snake_case(java_name)}
    for prefix in ("get", "set", "is", "has", "with"):
        if java_name.startswith(prefix) and len(java_name) > len(prefix):
            tail = java_name[len(prefix) :]
            names.add(snake_case(tail))
            names.add(f"{prefix}_{snake_case(tail)}")
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
) -> tuple[dict[str, list[dict[str, str]]], dict[str, list[dict[str, str]]]]:
    types: dict[str, list[dict[str, str]]] = defaultdict(list)
    members: dict[str, list[dict[str, str]]] = defaultdict(list)
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
        elif item["kind"] == "function":
            match = re.search(r"::([A-Z][A-Za-z0-9_]*)(?:<[^>]*>)?::[a-z_]", signature)
            if match:
                members[match.group(1)].append(item)
    return dict(types), dict(members)


def owner_simple(item: dict[str, Any]) -> str:
    return item["owner"].rsplit(".", 1)[-1].replace("$", "")


def rust_owner(item: dict[str, Any]) -> str:
    return OWNER_ALIASES.get(owner_simple(item), owner_simple(item))


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
    owner = MEMBER_OWNER_ALIASES.get(owner_simple(java), rust_owner(java))
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
    names = (
        explicit_names
        if explicit_names is not None
        else {"new"}
        if java_name == "<init>"
        else method_names(java_name)
    )
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
            if any(f"::{name}:" in signature or f"::{name} " in signature for name in names):
                result.append(item)
        elif item["kind"] == "function" and any(
            re.search(rf"::{re.escape(name)}(?:<|\()", signature) for name in names
        ):
            result.append(item)
    return prefer_primary(result, owner)


def suggest(java_manifest: dict[str, Any], rust_manifest: dict[str, Any]) -> list[dict[str, Any]]:
    rust = rust_items(rust_manifest)
    type_index, member_index = rust_indexes(rust)
    java_items = [*java_manifest["types"], *java_manifest["members"]]
    java_member_owners = {item["owner"] for item in java_manifest["members"]}
    entries = []
    for item in sorted(java_items, key=lambda value: value["id"]):
        if item["kind"] == "type":
            candidates = type_candidates(item, type_index.get(rust_owner(item), []))
            if not candidates:
                candidates = marker_interface_candidates(
                    item,
                    member_index.get(rust_owner(item), []),
                    java_member_owners,
                )
        else:
            member_owner = MEMBER_OWNER_ALIASES.get(owner_simple(item), rust_owner(item))
            candidates = member_candidates(item, member_index.get(member_owner, []))
            if not candidates and item["kind"] == "constructor" and owner_simple(item).startswith(
                "Abstract"
            ):
                # Java javap 会列出 abstract class 的 public 构造器，但抽象类本身
                # 不可实例化；Rust trait 类型就是对应的构造边界。
                candidates = type_candidates(item, type_index.get(rust_owner(item), []))
        status = "unmapped" if not candidates else "candidate" if len(candidates) == 1 else "ambiguous"
        entries.append(
            {
                "java_id": item["id"],
                "status": status,
                "rust_ids": candidates,
                "compile_probes": [],
                "behavior_tests": [],
                "java_golden": [],
                "semantic_notes": "deterministic name-based candidate; evidence not yet verified"
                if candidates
                else "",
            }
        )
    return entries


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--java-api", required=True, type=Path)
    parser.add_argument("--rust-api", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    java = load(args.java_api)
    rust = load(args.rust_api)
    mapping = {
        "schema_version": 1,
        "authority": "java_easyexcel_4.0.3_javap_public_api",
        "java_manifest_sha256": sha256(args.java_api),
        "rust_manifest_sha256": sha256(args.rust_api),
        "entries": suggest(java, rust),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(mapping, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
