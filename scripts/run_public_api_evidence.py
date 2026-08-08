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


def load_catalog(path: Path) -> dict[str, Any]:
    catalog = load(path)
    evidence = list(catalog.get("evidence", []))
    for relative in catalog.get("include", []):
        evidence.extend(load_catalog(path.parent / relative).get("evidence", []))
    return {"schema_version": catalog.get("schema_version", 1), "evidence": evidence}


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
