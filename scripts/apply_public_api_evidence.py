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


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_catalog(path: Path) -> dict[str, Any]:
    catalog = load(path)
    evidence = list(catalog.get("evidence", []))
    for relative in catalog.get("include", []):
        evidence.extend(load_catalog(path.parent / relative).get("evidence", []))
    return {"schema_version": catalog.get("schema_version", 1), "evidence": evidence}


def apply(mapping: dict[str, Any], catalog: dict[str, Any]) -> dict[str, Any]:
    by_java: dict[str, dict[str, list[dict[str, Any]]]] = {}
    for record in catalog.get("evidence", []):
        kind = record.get("kind")
        if kind not in FIELDS:
            continue
        for java_id in record.get("java_ids", []):
            by_java.setdefault(java_id, {}).setdefault(kind, []).append(record)

    for entry in mapping.get("entries", []):
        records = by_java.get(entry.get("java_id"), {})
        mapped_rust = set(entry.get("rust_ids", []))
        if entry.get("status") not in {"candidate", "verified"} or not mapped_rust:
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
            entry["semantic_notes"] = "verified by curated executable evidence catalog"
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
