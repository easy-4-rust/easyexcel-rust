#!/usr/bin/env python3
"""Refresh stale SHA-256 hashes in evidence catalog source_files.

Scans all evidence JSON files under parity/ and recalculates SHA-256
for every source_files entry whose path resolves to an existing file.
Updates the JSON files in place and prints a summary.

Usage:
    python3 scripts/refresh_source_hashes.py [--dry-run]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

# Evidence JSON files to scan (relative to repo root)
EVIDENCE_FILES = [
    "parity/public-api-evidence.json",
    "parity/public-api-evidence/excel-writer.json",
    "parity/public-api-evidence/excel-builder.json",
    "parity/public-api-evidence/excel-analyser.json",
    "parity/public-api-evidence/poi-enums.json",
    "parity/public-api-evidence/style-annotations.json",
    "parity/templates/converters.json",
]


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_json(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def refresh_file(path: Path, repo_root: Path, dry_run: bool) -> dict[str, int]:
    """Refresh source_files hashes in a single evidence JSON file.

    Returns counts: {updated, missing, unchanged, errors}.
    """
    catalog = json.loads(path.read_text(encoding="utf-8"))
    evidence = catalog.get("evidence", [])
    updated = 0
    missing = 0
    unchanged = 0
    errors = 0

    for record in evidence:
        source_files = record.get("source_files", [])
        if not isinstance(source_files, list):
            continue
        for entry in source_files:
            if not isinstance(entry, dict):
                errors += 1
                continue
            relative = entry.get("path")
            if not isinstance(relative, str) or not relative:
                errors += 1
                continue
            full_path = (repo_root / relative).resolve()
            try:
                full_path.relative_to(repo_root.resolve())
            except ValueError:
                errors += 1
                continue
            if not full_path.is_file():
                missing += 1
                continue
            actual_hash = file_sha256(full_path)
            expected_hash = entry.get("sha256", "")
            if actual_hash == expected_hash:
                unchanged += 1
            else:
                entry["sha256"] = actual_hash
                updated += 1

    if updated > 0 and not dry_run:
        path.write_text(canonical_json(catalog), encoding="utf-8")

    return {"updated": updated, "missing": missing, "unchanged": unchanged, "errors": errors}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Report what would change without modifying files.",
    )
    args = parser.parse_args()

    repo_root = Path.cwd()
    total_updated = 0
    total_missing = 0
    total_unchanged = 0
    total_errors = 0

    for relative in EVIDENCE_FILES:
        path = (repo_root / relative).resolve()
        if not path.is_file():
            print(f"SKIP (not found): {relative}")
            continue
        counts = refresh_file(path, repo_root, args.dry_run)
        total_updated += counts["updated"]
        total_missing += counts["missing"]
        total_unchanged += counts["unchanged"]
        total_errors += counts["errors"]
        action = "would update" if args.dry_run else "updated"
        if counts["updated"] > 0:
            print(
                f"{relative}: {action} {counts['updated']} hashes, "
                f"{counts['unchanged']} unchanged, {counts['missing']} missing, "
                f"{counts['errors']} errors"
            )
        else:
            print(f"{relative}: no changes")

    print()
    print(f"Summary ({'dry-run' if args.dry_run else 'applied'}):")
    print(f"  Updated:   {total_updated}")
    print(f"  Unchanged: {total_unchanged}")
    print(f"  Missing:   {total_missing}")
    print(f"  Errors:    {total_errors}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
