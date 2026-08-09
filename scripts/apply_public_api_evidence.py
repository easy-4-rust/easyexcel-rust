#!/usr/bin/env python3
"""Overlay curated, executable evidence onto deterministic API candidates."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


FIELDS = {
    "compile_probe": "compile_probes",
    "behavior_test": "behavior_tests",
    "java_golden": "java_golden",
}


def append_semantic_note(existing: Any, note: str) -> str:
    """保留候选器的语义边界，并确定性追加证据状态说明。"""
    base = existing.strip() if isinstance(existing, str) else ""
    if note in base:
        return base
    return f"{base}; {note}" if base else note


def strip_evidence_notes(existing: Any) -> str:
    """移除上一次 overlay 追加的状态，保留候选器原始语义说明。"""
    if not isinstance(existing, str):
        return ""
    return "; ".join(
        part.strip()
        for part in existing.split(";")
        if part.strip()
        and part.strip()
        != "curated mapping resolution selected existing deterministic candidates"
        and not part.strip().endswith(
            "verified by curated executable evidence catalog"
        )
    )


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_catalog(
    path: Path, root: Path | None = None, stack: tuple[Path, ...] = ()
) -> dict[str, Any]:
    root = path.parent.resolve() if root is None else root
    resolved = path.resolve()
    try:
        resolved.relative_to(root)
    except ValueError as error:
        raise ValueError(f"evidence catalog escapes root: {path}") from error
    if resolved in stack:
        chain = " -> ".join(str(item) for item in (*stack, resolved))
        raise ValueError(f"cyclic evidence catalog include: {chain}")
    catalog = load(resolved)
    evidence = list(catalog.get("evidence", []))
    mapping_resolutions = list(catalog.get("mapping_resolutions", []))
    includes = catalog.get("include", [])
    if not isinstance(includes, list) or any(not isinstance(item, str) for item in includes):
        raise ValueError(f"invalid evidence catalog include list: {resolved}")
    for relative in includes:
        included = load_catalog(resolved.parent / relative, root, (*stack, resolved))
        evidence.extend(included.get("evidence", []))
        mapping_resolutions.extend(included.get("mapping_resolutions", []))
    if any(
        not isinstance(record, dict)
        or not isinstance(record.get("id"), str)
        or not record["id"]
        for record in evidence
    ):
        raise ValueError(f"invalid evidence record in catalog tree: {resolved}")
    ids = [
        record.get("id")
        for record in evidence
        if isinstance(record, dict) and isinstance(record.get("id"), str)
    ]
    duplicate_ids = sorted(item for item in set(ids) if ids.count(item) > 1)
    if duplicate_ids:
        raise ValueError(f"duplicate evidence ids: {duplicate_ids[:10]}")
    if any(
        not isinstance(record, dict)
        or not isinstance(record.get("java_id"), str)
        or not record["java_id"]
        or not isinstance(record.get("rust_ids"), list)
        or not record["rust_ids"]
        or any(not isinstance(rust_id, str) or not rust_id for rust_id in record["rust_ids"])
        or len(record["rust_ids"]) != len(set(record["rust_ids"]))
        for record in mapping_resolutions
    ):
        raise ValueError(f"invalid mapping resolution in catalog tree: {resolved}")
    resolution_java_ids = [record["java_id"] for record in mapping_resolutions]
    duplicate_resolutions = sorted(
        java_id
        for java_id in set(resolution_java_ids)
        if resolution_java_ids.count(java_id) > 1
    )
    if duplicate_resolutions:
        raise ValueError(
            f"duplicate mapping resolutions: {duplicate_resolutions[:10]}"
        )
    return {
        "schema_version": catalog.get("schema_version", 1),
        "evidence": evidence,
        "mapping_resolutions": mapping_resolutions,
    }


def apply(mapping: dict[str, Any], catalog: dict[str, Any]) -> dict[str, Any]:
    by_java: dict[str, dict[str, list[dict[str, Any]]]] = {}
    for record in catalog.get("evidence", []):
        kind = record.get("kind")
        if kind not in FIELDS:
            continue
        for java_id in record.get("java_ids", []):
            by_java.setdefault(java_id, {}).setdefault(kind, []).append(record)

    resolutions = {
        record["java_id"]: record["rust_ids"]
        for record in catalog.get("mapping_resolutions", [])
    }
    known_java_ids = {
        entry.get("java_id")
        for entry in mapping.get("entries", [])
        if isinstance(entry.get("java_id"), str)
    }
    unknown_resolutions = sorted(set(resolutions) - known_java_ids)
    if unknown_resolutions:
        raise ValueError(
            f"mapping resolutions reference unknown Java ids: {unknown_resolutions[:10]}"
        )

    for entry in mapping.get("entries", []):
        java_id = entry.get("java_id")
        entry["semantic_notes"] = strip_evidence_notes(entry.get("semantic_notes"))
        resolution = resolutions.get(java_id)
        if resolution is not None:
            candidates = set(entry.get("rust_ids", []))
            resolved = set(resolution)
            if entry.get("status") not in {"candidate", "ambiguous", "verified"}:
                raise ValueError(
                    f"mapping resolution targets non-candidate Java API: {java_id}"
                )
            if not resolved.issubset(candidates):
                unexpected = sorted(resolved - candidates)
                raise ValueError(
                    f"mapping resolution introduces non-candidate Rust ids for {java_id}: "
                    f"{unexpected}"
                )
            entry["rust_ids"] = sorted(resolved)
            entry["implementation_carriers"] = sorted(
                {rust_id.split(":", 1)[0] for rust_id in resolved}
            )
            entry["semantic_notes"] = append_semantic_note(
                entry.get("semantic_notes"),
                "curated mapping resolution selected existing deterministic candidates",
            )
        records = by_java.get(entry.get("java_id"), {})
        mapped_rust = set(entry.get("rust_ids", []))
        if entry.get("status") not in {"candidate", "ambiguous", "verified"} or not mapped_rust:
            continue
        # 每次都从当前 catalog 重新计算三类证据，禁止沿用旧 catalog 遗留的
        # verified 状态。needs_implementation 即使误配了证据也不能被提升。
        for field in FIELDS.values():
            entry[field] = []
        if entry.get("status") == "ambiguous" and resolution is None:
            continue
        entry["status"] = "candidate"
        if entry.get("implementation_strategy") == "needs_implementation":
            entry["status"] = "unmapped"
            continue
        selected: dict[str, list[str]] = {}
        for kind, field in FIELDS.items():
            matches = [
                record
                for record in records.get(kind, [])
                if mapped_rust.issubset(set(record.get("rust_ids", [])))
            ]
            selected[field] = sorted(record["id"] for record in matches)
        if all(selected.values()):
            entry.update(selected)
            entry["status"] = "verified"
            strategy = entry.get("implementation_strategy")
            verification_note = (
                f"{strategy} verified by curated executable evidence catalog"
                if isinstance(strategy, str) and strategy
                else "verified by curated executable evidence catalog"
            )
            entry["semantic_notes"] = append_semantic_note(
                entry.get("semantic_notes"), verification_note
            )
    return mapping


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mapping", type=Path, required=True)
    parser.add_argument("--catalog", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    result = apply(load(args.mapping), load_catalog(args.catalog))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
