#!/usr/bin/env python3
"""将 evidence catalog 的 java_ids 关联到 mapping catalog 的 entries。

读取所有 evidence catalogs（根 catalog + 子 catalogs），对每个 evidence 的 java_ids 数组，
在 mapping catalog 找匹配 java_id 的 entry，把 evidence id 加入 entry 的对应字段。

匹配策略（严格 java_id 精确匹配优先，再用前缀匹配处理简化格式）：
1. 精确匹配：evidence java_id == mapping java_id
2. 方法前缀匹配：evidence 用简化格式如 `Class#method()`，mapping 用 JVM 描述符如 `Class#method()ReturnType`
3. 字段前缀匹配：evidence 用 `Class#FIELD_NAME`，mapping 用 `Class#FIELD:FIELD_NAMETYPE`

用法：
    python3 scripts/link_evidence_to_mapping.py \
        --evidence-dir parity/ \
        --mapping parity/java-rust-public-api.json \
        --output parity/java-rust-public-api.json
"""

import argparse
import json
import os
import sys
from collections import defaultdict


def load_evidence_catalogs(evidence_dir: str) -> list[dict]:
    """加载所有 evidence catalog 文件（根目录 + 子目录）。"""
    catalogs = []

    # 根 evidence catalog
    root_path = os.path.join(evidence_dir, "public-api-evidence.json")
    if os.path.exists(root_path):
        with open(root_path, "r", encoding="utf-8") as f:
            data = json.load(f)
            catalogs.append({"path": root_path, "evidence": data["evidence"]})
            print(f"  加载根 catalog: {root_path} ({len(data['evidence'])} evidence)")

    # 子 evidence catalogs
    sub_dir = os.path.join(evidence_dir, "public-api-evidence")
    if os.path.isdir(sub_dir):
        for fname in sorted(os.listdir(sub_dir)):
            if not fname.endswith(".json"):
                continue
            fpath = os.path.join(sub_dir, fname)
            with open(fpath, "r", encoding="utf-8") as f:
                data = json.load(f)
                catalogs.append({"path": fpath, "evidence": data["evidence"]})
                print(f"  加载子 catalog: {fname} ({len(data['evidence'])} evidence)")

    return catalogs


def build_mapping_index(entries: list[dict]) -> dict[str, list[int]]:
    """构建 mapping java_id 到 entry 索引的映射。

    返回 {java_id: [entry_index, ...]} 用于精确匹配。
    同时构建前缀索引用于模糊匹配。
    """
    exact_index: dict[str, list[int]] = defaultdict(list)

    for i, entry in enumerate(entries):
        java_id = entry["java_id"]
        exact_index[java_id].append(i)

    return exact_index


def match_evidence_to_entries(
    evidence_java_id: str,
    exact_index: dict[str, list[int]],
    entries: list[dict],
) -> list[int]:
    """将单个 evidence java_id 匹配到 mapping entries。

    匹配策略：
    1. 精确匹配
    2. 方法前缀匹配（简化格式 vs JVM 描述符）
    3. 字段前缀匹配
    """
    # 1. 精确匹配
    if evidence_java_id in exact_index:
        return exact_index[evidence_java_id]

    # 2. 如果没有 # 分隔符，无法做前缀匹配
    if "#" not in evidence_java_id:
        return []

    class_name, member = evidence_java_id.split("#", 1)

    # 3. 判断是方法还是字段
    if "(" in member:
        # 方法匹配：evidence 用 `method()` 或 `method(ArgType)`，mapping 用 `method(...)ReturnType`
        method_name = member.split("(")[0]
        prefix = f"{class_name}#{method_name}("
        matches = []
        for i, entry in enumerate(entries):
            if entry["java_id"].startswith(prefix):
                matches.append(i)
        return matches
    else:
        # 字段匹配：evidence 用 `FIELD_NAME`，mapping 用 `FIELD:FIELD_NAMETYPE`
        field_prefix = f"{class_name}#FIELD:{member}"
        matches = []
        for i, entry in enumerate(entries):
            if entry["java_id"].startswith(field_prefix):
                matches.append(i)
        return matches


def kind_to_field(kind: str) -> str:
    """将 evidence kind 映射到 mapping entry 的字段名。"""
    if kind == "compile_probe":
        return "compile_probes"
    elif kind == "behavior_test":
        return "behavior_tests"
    elif kind == "java_golden":
        return "java_golden"
    else:
        raise ValueError(f"未知 evidence kind: {kind}")


def link_evidence_to_mapping(
    catalogs: list[dict],
    entries: list[dict],
) -> dict[str, int]:
    """将所有 evidence 关联到 mapping entries。

    返回统计信息。
    """
    exact_index = build_mapping_index(entries)

    stats = {
        "total_evidence": 0,
        "total_java_ids": 0,
        "matched_java_ids": 0,
        "unmatched_java_ids": 0,
        "entries_linked": 0,
        "compile_probes_added": 0,
        "behavior_tests_added": 0,
        "java_golden_added": 0,
    }

    linked_entry_indices = set()
    unmatched_ids = []
    rust_ids_added = 0

    for catalog in catalogs:
        for evidence in catalog["evidence"]:
            stats["total_evidence"] += 1
            evidence_id = evidence["id"]
            kind = evidence["kind"]
            field = kind_to_field(kind)
            evidence_java_ids = evidence["java_ids"]
            evidence_rust_ids = evidence.get("rust_ids", [])

            for pos, java_id in enumerate(evidence_java_ids):
                stats["total_java_ids"] += 1

                matched_indices = match_evidence_to_entries(
                    java_id, exact_index, entries
                )

                if matched_indices:
                    stats["matched_java_ids"] += 1
                    # 获取对应的 rust_id（按位置）
                    corresponding_rust_id = evidence_rust_ids[pos] if pos < len(evidence_rust_ids) else None

                    for idx in matched_indices:
                        entry = entries[idx]
                        # 确保字段存在
                        if field not in entry:
                            entry[field] = []
                        # 避免重复添加 evidence id
                        if evidence_id not in entry[field]:
                            entry[field].append(evidence_id)
                            stats[f"{field}_added"] += 1
                            linked_entry_indices.add(idx)
                        # 添加对应的 rust_id
                        if corresponding_rust_id:
                            existing_rust = entry.get("rust_ids", [])
                            if corresponding_rust_id not in existing_rust:
                                entry.setdefault("rust_ids", []).append(corresponding_rust_id)
                                rust_ids_added += 1
                else:
                    stats["unmatched_java_ids"] += 1
                    unmatched_ids.append(java_id)

    stats["rust_ids_added"] = rust_ids_added

    stats["entries_linked"] = len(linked_entry_indices)

    # 打印未匹配的 java_ids（用于调试）
    if unmatched_ids:
        print(f"\n  未匹配的 java_ids ({len(unmatched_ids)}):")
        for uid in sorted(set(unmatched_ids))[:20]:
            print(f"    {uid}")
        if len(set(unmatched_ids)) > 20:
            print(f"    ... 还有 {len(set(unmatched_ids)) - 20} 个")

    return stats


def main():
    parser = argparse.ArgumentParser(
        description="将 evidence catalog 关联到 mapping catalog"
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
        "--output",
        required=True,
        help="输出 JSON 文件路径（通常与 --mapping 相同）",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="只打印统计，不写入文件",
    )

    args = parser.parse_args()

    # 加载 evidence catalogs
    print("加载 evidence catalogs:")
    catalogs = load_evidence_catalogs(args.evidence_dir)
    print(f"  共加载 {len(catalogs)} 个 catalog\n")

    # 加载 mapping catalog
    print(f"加载 mapping catalog: {args.mapping}")
    with open(args.mapping, "r", encoding="utf-8") as f:
        mapping = json.load(f)
    entries = mapping["entries"]
    print(f"  共 {len(entries)} 个 entries\n")

    # 统计初始状态
    from collections import Counter
    status_before = Counter(e["status"] for e in entries)
    print(f"初始状态: {dict(status_before)}\n")

    # 执行关联
    print("开始关联 evidence 到 mapping entries:")
    stats = link_evidence_to_mapping(catalogs, entries)

    # 打印统计
    print(f"\n关联统计:")
    print(f"  evidence 总数: {stats['total_evidence']}")
    print(f"  java_ids 总数: {stats['total_java_ids']}")
    print(f"  匹配成功: {stats['matched_java_ids']}")
    print(f"  未匹配: {stats['unmatched_java_ids']}")
    print(f"  关联的 entries 数: {stats['entries_linked']}")
    print(f"  compile_probes 添加: {stats['compile_probes_added']}")
    print(f"  behavior_tests 添加: {stats['behavior_tests_added']}")
    print(f"  java_golden 添加: {stats['java_golden_added']}")
    print(f"  rust_ids 添加: {stats['rust_ids_added']}")

    # 写入文件
    if not args.dry_run:
        print(f"\n写入输出文件: {args.output}")
        with open(args.output, "w", encoding="utf-8") as f:
            json.dump(mapping, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print("  写入完成")
    else:
        print("\n[DRY-RUN] 未写入文件")

    return stats


if __name__ == "__main__":
    main()
