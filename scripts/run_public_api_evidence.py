#!/usr/bin/env python3
"""Execute the commands declared by the per-API evidence catalog."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


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
    resolution_java_ids = [
        record.get("java_id")
        for record in mapping_resolutions
        if isinstance(record, dict)
    ]
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


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def run(catalog_path: Path, repo_root: Path) -> tuple[dict[str, Any], bool]:
    catalog = load_catalog(catalog_path)
    command_results: dict[tuple[str, ...], dict[str, Any]] = {}
    results: list[dict[str, Any]] = []
    passed = True

    for evidence in catalog.get("evidence", []):
        evidence_id = evidence.get("id", "<missing>")
        commands = evidence.get("commands")
        if not isinstance(commands, list) or not commands:
            results.append(
                {
                    "evidence_id": evidence_id,
                    "status": "invalid",
                    "error": "evidence lacks commands",
                }
            )
            passed = False
            continue

        evidence_commands = []
        evidence_passed = True
        for argv in commands:
            if not isinstance(argv, list) or not argv or not all(isinstance(item, str) for item in argv):
                evidence_commands.append({"argv": argv, "exit_code": None, "status": "invalid"})
                evidence_passed = False
                continue
            key = tuple(argv)
            command_result = command_results.get(key)
            if command_result is None:
                completed = subprocess.run(argv, cwd=repo_root, capture_output=True, check=False)
                command_result = {
                    "argv": argv,
                    "exit_code": completed.returncode,
                    "stdout_sha256": sha256_bytes(completed.stdout),
                    "stderr_sha256": sha256_bytes(completed.stderr),
                    "status": "passed" if completed.returncode == 0 else "failed",
                }
                command_results[key] = command_result
                if completed.returncode != 0:
                    sys.stderr.buffer.write(completed.stdout)
                    sys.stderr.buffer.write(completed.stderr)
            evidence_commands.append(command_result)
            evidence_passed = evidence_passed and command_result["status"] == "passed"
        results.append(
            {
                "evidence_id": evidence_id,
                "status": "passed" if evidence_passed else "failed",
                "commands": evidence_commands,
            }
        )
        passed = passed and evidence_passed

    report = {
        "schema_version": 1,
        "catalog_sha256": sha256_bytes(canonical_json(catalog).encode("utf-8")),
        "results": results,
    }
    return report, passed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--catalog", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    args = parser.parse_args()

    report, passed = run(args.catalog.resolve(), args.repo_root.resolve())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(canonical_json(report), encoding="utf-8")
    print(json.dumps({"evidence": len(report["results"]), "passed": passed}))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
