#!/usr/bin/env python3
"""根据已关联的 evidence 将 candidate 升级为 verified。

规则：
- 三类 evidence 齐全（compile_probes + behavior_tests + java_golden）→ 升级 verified
- 部分 evidence → 保留 candidate，添加 partial_evidence 字段标注哪几类有
- 无 evidence → 保持 candidate

已 verified 的 205 个 entry 不破坏。

用法：
    python3 scripts/apply_evidence_to_mapping.py \
        --mapping parity/java-rust-public-api.json \
        --output parity/java-rust-public-api.json
"""

import argparse
import json
from collections import Counter


def apply_evidence_to_mapping(mapping: dict) -> dict:
    """遍历 entries，根据 evidence 字段决定是否升级 status。"""
    entries = mapping["entries"]

    # 三类 evidence 字段名
    evidence_fields = ["compile_probes", "behavior_tests", "java_golden"]

    stats = {
        "total_candidates": 0,
        "upgraded_to_verified": 0,
        "partial_evidence": 0,
        "no_evidence": 0,
        "already_verified_kept": 0,
        "verified_with_new_evidence": 0,
    }

    # 统计已有 verified 的 entries（不破坏）
    for entry in entries:
        if entry["status"] == "verified":
            # 检查是否有新的 evidence 被添加（除了原有的 facade 类）
            has_new = False
            for field in evidence_fields:
                for evid in entry.get(field, []):
                    if not evid.startswith("facade."):
                        has_new = True
                        break
            if has_new:
                stats["verified_with_new_evidence"] += 1
            else:
                stats["already_verified_kept"] += 1
            continue

        # 只处理 candidate
        if entry["status"] != "candidate":
            continue

        stats["total_candidates"] += 1

        # 检查每类 evidence 是否存在
        has = {}
        for field in evidence_fields:
            has[field] = bool(entry.get(field))

        # 三类齐全且有 rust_ids → 升级 verified
        if all(has.values()) and entry.get("rust_ids"):
            entry["status"] = "verified"
            entry["verified_by"] = "evidence-catalog-link"
            entry["semantic_notes"] = add_note(
                entry.get("semantic_notes", ""),
                "verified by linked evidence catalog (compile_probe + behavior_test + java_golden)"
            )
            stats["upgraded_to_verified"] += 1

        # 部分 evidence → 保留 candidate，记录 partial_evidence
        elif any(has.values()):
            present = [f for f, v in has.items() if v]
            entry["partial_evidence"] = present
            entry["semantic_notes"] = add_note(
                entry.get("semantic_notes", ""),
                f"partial evidence: {', '.join(present)}"
            )
            stats["partial_evidence"] += 1

        # 无 evidence → 保持 candidate
        else:
            stats["no_evidence"] += 1

    return mapping, stats


def add_note(existing: str, note: str) -> str:
    """追加语义说明，避免重复。"""
    if not isinstance(existing, str):
        existing = ""
    if note in existing:
        return existing
    return f"{existing}; {note}" if existing else note


def main():
    parser = argparse.ArgumentParser(
        description="根据 evidence 将 candidate 升级为 verified"
    )
    parser.add_argument(
        "--mapping",
        required=True,
        help="mapping catalog JSON 文件路径",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="输出 JSON 文件路径",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="只打印统计，不写入文件",
    )

    args = parser.parse_args()

    # 加载
    print(f"加载 mapping catalog: {args.mapping}")
    with open(args.mapping, "r", encoding="utf-8") as f:
        mapping = json.load(f)

    entries = mapping["entries"]
    status_before = Counter(e["status"] for e in entries)
    print(f"初始状态: {dict(status_before)}\n")

    # 执行升级
    print("开始根据 evidence 升级 entries:")
    mapping, stats = apply_evidence_to_mapping(mapping)

    # 统计最终状态
    status_after = Counter(e["status"] for e in entries)

    print(f"\n升级统计:")
    print(f"  candidate 总数: {stats['total_candidates']}")
    print(f"  升级为 verified: {stats['upgraded_to_verified']}")
    print(f"  部分 evidence (保留 candidate): {stats['partial_evidence']}")
    print(f"  无 evidence (保持 candidate): {stats['no_evidence']}")
    print(f"\n最终状态: {dict(status_before)} -> {dict(status_after)}")
    print(f"  verified 增量: {status_after['verified'] - status_before['verified']}")

    # 写入文件
    if not args.dry_run:
        print(f"\n写入输出文件: {args.output}")
        with open(args.output, "w", encoding="utf-8") as f:
            json.dump(mapping, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print("  写入完成")
    else:
        print("\n[DRY-RUN] 未写入文件")


if __name__ == "__main__":
    main()
