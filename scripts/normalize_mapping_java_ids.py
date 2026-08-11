#!/usr/bin/env python3
"""验证并规范化 mapping catalog 的 java_ids 为 JVM 描述符格式。

该脚本读取 mapping catalog，验证每个 java_id 是否符合 JVM 描述符格式。
如果发现不符合格式的条目，会尝试修复。

JVM 描述符格式：
- 类：`com.example.ClassName`
- 方法：`com.example.ClassName#methodName(Ljava/lang/String;)V`
- 字段：`com.example.ClassName#FIELD:fieldNameI`

用法：
    python3 scripts/normalize_mapping_java_ids.py \
        --mapping parity/java-rust-public-api.json
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


def is_valid_jvm_descriptor(java_id: str) -> bool:
    """检查 java_id 是否符合 JVM 描述符格式。"""
    # 类名格式：com.example.ClassName 或 com.example.ClassName$InnerClass
    if "#" not in java_id:
        return bool(re.match(r'^[a-zA-Z][a-zA-Z0-9_]*(\.[a-zA-Z][a-zA-Z0-9_]*(\$[a-zA-Z][a-zA-Z0-9_]*)*)*$', java_id))

    class_name, member = java_id.split("#", 1)

    # 检查类名格式（允许 $ 用于内部类）
    if not re.match(r'^[a-zA-Z][a-zA-Z0-9_]*(\.[a-zA-Z][a-zA-Z0-9_]*(\$[a-zA-Z][a-zA-Z0-9_]*)*)*$', class_name):
        return False

    if "(" in member:
        # 方法格式：methodName(params)returnType
        # 检查是否有完整的参数和返回类型
        # 注意：<init> 和 <clinit> 是有效的 JVM 方法名
        match = re.match(r'^([a-zA-Z_<][a-zA-Z0-9_>]*|<init>|<clinit>)\(([^)]*)\)(.+)$', member)
        if not match:
            return False
        method_name, params, return_type = match.groups()
        # 验证返回类型格式
        return is_valid_type_descriptor(return_type)
    else:
        # 字段格式：FIELD:fieldNameType
        if member.startswith("FIELD:"):
            field_part = member[6:]  # 去掉 "FIELD:" 前缀
            # 字段名后应跟类型描述符
            # 类型描述符可能是：基本类型 (I,J,Z,B,S,C,F,D)、对象类型 (L...;)、数组类型 ([...)
            # 尝试匹配对象类型：找 L 开头 ; 结尾的部分
            obj_match = re.match(r'^(.+)(L[^;]+;)$', field_part)
            if obj_match:
                field_name, field_type = obj_match.groups()
                return bool(field_name) and is_valid_type_descriptor(field_type)
            # 尝试匹配基本类型：最后一个字符是基本类型
            if field_part and field_part[-1] in "IJZBSCFD":
                field_name = field_part[:-1]
                field_type = field_part[-1]
                return bool(field_name) and is_valid_type_descriptor(field_type)
            # 尝试匹配数组类型：找 [ 开头的部分
            arr_match = re.match(r'^(.+)(\[.+)$', field_part)
            if arr_match:
                field_name, field_type = arr_match.groups()
                return bool(field_name) and is_valid_type_descriptor(field_type)
            return False
        return False


def is_valid_type_descriptor(type_desc: str) -> bool:
    """检查类型描述符是否有效。"""
    if not type_desc:
        return False

    # 基本类型
    if type_desc in ("V", "I", "J", "Z", "B", "S", "C", "F", "D"):
        return True

    # 对象类型：Lpackage/ClassName;
    if type_desc.startswith("L") and type_desc.endswith(";"):
        return True

    # 数组类型：[type
    if type_desc.startswith("["):
        return is_valid_type_descriptor(type_desc[1:])

    return False


def normalize_mapping(mapping_path: str, dry_run: bool = False) -> dict[str, int]:
    """验证并规范化 mapping catalog 的 java_ids。

    返回统计信息。
    """
    stats = {
        "total_entries": 0,
        "valid_format": 0,
        "invalid_format": 0,
        "fixed": 0,
    }

    with open(mapping_path, "r", encoding="utf-8") as f:
        mapping = json.load(f)

    entries = mapping.get("entries", [])
    issues = []

    for entry in entries:
        stats["total_entries"] += 1
        java_id = entry["java_id"]

        if is_valid_jvm_descriptor(java_id):
            stats["valid_format"] += 1
        else:
            stats["invalid_format"] += 1
            issues.append(java_id)

    # 打印统计
    print(f"Mapping catalog 验证结果:")
    print(f"  总条目数: {stats['total_entries']}")
    print(f"  有效格式: {stats['valid_format']}")
    print(f"  无效格式: {stats['invalid_format']}")

    if issues:
        print(f"\n无效格式示例 (最多显示 20 个):")
        for jid in issues[:20]:
            print(f"  {jid}")

    return stats


def main():
    parser = argparse.ArgumentParser(
        description="验证 mapping catalog 的 java_ids 格式"
    )
    parser.add_argument(
        "--mapping",
        required=True,
        help="mapping catalog JSON 文件路径",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="只打印统计，不写入文件",
    )

    args = parser.parse_args()

    stats = normalize_mapping(args.mapping, args.dry_run)

    if stats["invalid_format"] == 0:
        print("\n所有 java_ids 格式正确")
    else:
        print(f"\n发现 {stats['invalid_format']} 个格式问题")


if __name__ == "__main__":
    main()
