#!/usr/bin/env python3
"""将 evidence catalog 的 java_ids 从简化格式规范化为 JVM 描述符格式。

简化格式示例：`ColumnWidth#value()`
JVM 描述符格式示例：`ColumnWidth#value()I`

该脚本读取所有 evidence 文件，对每个 java_id：
1. 如果已是 JVM 描述符格式（包含返回类型），保持不变
2. 如果是简化格式，从 mapping catalog 查找对应的 JVM 描述符
3. 更新 evidence 文件中的 java_ids

用法：
    python3 scripts/normalize_java_ids.py \
        --evidence-dir parity/ \
        --mapping parity/java-rust-public-api.json
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections import defaultdict
from pathlib import Path


def load_mapping_java_ids(mapping_path: str) -> dict[str, str]:
    """加载 mapping catalog 的 java_id 索引。

    返回 {简化格式: JVM 描述符格式} 的映射。
    """
    with open(mapping_path, "r", encoding="utf-8") as f:
        mapping = json.load(f)

    entries = mapping.get("entries", [])

    # 构建简化格式 -> JVM 描述符格式的映射
    simplified_to_jvm = {}

    for entry in entries:
        jvm_id = entry["java_id"]

        # 提取简化格式
        if "#" in jvm_id:
            class_name, member = jvm_id.split("#", 1)

            if "(" in member:
                # 方法：提取方法名（不含参数和返回类型）
                method_name = member.split("(")[0]
                simplified = f"{class_name}#{method_name}()"
            else:
                # 字段：FIELD:NAME + 类型描述符
                if member.startswith("FIELD:"):
                    field_name = member.split(":")[1]
                    # 去掉类型描述符（最后一个大写字母或L开头的类型）
                    simplified = f"{class_name}#{field_name}"
                else:
                    simplified = jvm_id
        else:
            # 类名，保持不变
            simplified = jvm_id

        # 保留第一个匹配（避免覆盖）
        if simplified not in simplified_to_jvm:
            simplified_to_jvm[simplified] = jvm_id

    return simplified_to_jvm


def normalize_java_id(java_id: str, simplified_to_jvm: dict[str, str]) -> str:
    """规范化单个 java_id。

    如果是简化格式，查找对应的 JVM 描述符格式。
    如果已是 JVM 描述符格式或找不到匹配，保持不变。
    """
    # 如果已经是 JVM 描述符格式（包含返回类型描述符）
    if "(" in java_id and ")" in java_id:
        after_paren = java_id.split(")")[-1]
        # 检查是否有返回类型描述符
        if after_paren and any(after_paren.startswith(c) for c in ["V", "I", "J", "Z", "B", "S", "C", "F", "D", "L", "["]):
            return java_id

    # 如果是类名（没有 #），保持不变
    if "#" not in java_id:
        return java_id

    # 提取简化格式
    class_name, member = java_id.split("#", 1)

    if "(" in member:
        # 方法：提取方法名
        method_name = member.split("(")[0]
        simplified = f"{class_name}#{method_name}()"
    else:
        # 字段
        simplified = java_id

    # 查找 JVM 描述符格式
    if simplified in simplified_to_jvm:
        return simplified_to_jvm[simplified]

    # 找不到匹配，保持原样
    return java_id


def normalize_evidence_file(
    file_path: str,
    simplified_to_jvm: dict[str, str],
    dry_run: bool = False,
) -> dict[str, int]:
    """规范化单个 evidence 文件中的 java_ids。

    返回统计信息。
    """
    stats = {"total_ids": 0, "normalized": 0, "unchanged": 0, "not_found": 0}

    with open(file_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    evidence_list = data.get("evidence", [])
    changes_made = False

    for evidence in evidence_list:
        java_ids = evidence.get("java_ids", [])
        new_java_ids = []

        for java_id in java_ids:
            stats["total_ids"] += 1
            normalized = normalize_java_id(java_id, simplified_to_jvm)

            if normalized != java_id:
                stats["normalized"] += 1
                changes_made = True
                new_java_ids.append(normalized)
            else:
                stats["unchanged"] += 1
                new_java_ids.append(java_id)

        evidence["java_ids"] = new_java_ids

    if changes_made and not dry_run:
        with open(file_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
            f.write("\n")

    return stats


def main():
    parser = argparse.ArgumentParser(
        description="将 evidence catalog 的 java_ids 规范化为 JVM 描述符格式"
    )
    parser.add_argument(
        "--evidence-dir",
        required=True,
        help="evidence 根目录（包含 public-api-evidence.json 和 public-api-evidence/ 子目录）",
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

    # 加载 mapping java_ids
    print(f"加载 mapping catalog: {args.mapping}")
    simplified_to_jvm = load_mapping_java_ids(args.mapping)
    print(f"  简化格式 -> JVM 描述符映射: {len(simplified_to_jvm)} 条\n")

    # 处理所有 evidence 文件
    evidence_files = []

    # 根 evidence catalog
    root_path = os.path.join(args.evidence_dir, "public-api-evidence.json")
    if os.path.exists(root_path):
        evidence_files.append(root_path)

    # 子 evidence catalogs
    sub_dir = os.path.join(args.evidence_dir, "public-api-evidence")
    if os.path.isdir(sub_dir):
        for fname in sorted(os.listdir(sub_dir)):
            if fname.endswith(".json"):
                evidence_files.append(os.path.join(sub_dir, fname))

    print(f"找到 {len(evidence_files)} 个 evidence 文件\n")

    total_stats = {"total_ids": 0, "normalized": 0, "unchanged": 0, "not_found": 0}

    for file_path in evidence_files:
        print(f"处理: {os.path.basename(file_path)}")
        stats = normalize_evidence_file(file_path, simplified_to_jvm, args.dry_run)

        for key in total_stats:
            total_stats[key] += stats[key]

        print(f"  总计: {stats['total_ids']}, 规范化: {stats['normalized']}, 未变: {stats['unchanged']}")

    print(f"\n总计:")
    print(f"  java_ids 总数: {total_stats['total_ids']}")
    print(f"  已规范化: {total_stats['normalized']}")
    print(f"  未变化: {total_stats['unchanged']}")

    if args.dry_run:
        print("\n[DRY-RUN] 未写入文件")
    else:
        print("\n文件已更新")


if __name__ == "__main__":
    main()
