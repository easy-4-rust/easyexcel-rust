#!/usr/bin/env python3
"""Verify fail-closed Java-to-Rust public API evidence mappings."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any


REQUIRED_EVIDENCE = ("rust_ids", "compile_probes", "behavior_tests", "java_golden")
EVIDENCE_KIND = {
    "compile_probes": "compile_probe",
    "behavior_tests": "behavior_test",
    "java_golden": "java_golden",
}
REQUIRED_COMPILE_PROFILES = {"stable-default-features", "stable-all-features"}


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_catalog(path: Path) -> dict[str, Any]:
    catalog = load(path)
    evidence = list(catalog.get("evidence", []))
    for relative in catalog.get("include", []):
        evidence.extend(load_catalog(path.parent / relative).get("evidence", []))
    return {"schema_version": catalog.get("schema_version", 1), "evidence": evidence}


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def java_ids(manifest: dict[str, Any]) -> list[str]:
    return sorted(item["id"] for item in [*manifest["types"], *manifest["members"]])


def rust_ids(manifest: dict[str, Any]) -> set[str]:
    return {
        item["id"]
        for package in manifest["packages"]
        for snapshot in package["snapshots"]
        for item in snapshot["items"]
    }


def skeleton(java_manifest: dict[str, Any], java_sha: str, rust_sha: str) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "authority": "java_easyexcel_4.0.3_javap_public_api",
        "java_manifest_sha256": java_sha,
        "rust_manifest_sha256": rust_sha,
        "entries": [
            {
                "java_id": item_id,
                "status": "unmapped",
                "rust_ids": [],
                "compile_probes": [],
                "behavior_tests": [],
                "java_golden": [],
                "semantic_notes": "",
            }
            for item_id in java_ids(java_manifest)
        ],
    }


def evidence_index(catalog: dict[str, Any] | None) -> tuple[dict[str, dict[str, Any]], list[str]]:
    records = [] if catalog is None else catalog.get("evidence", [])
    result: dict[str, dict[str, Any]] = {}
    errors: list[str] = []
    for record in records:
        evidence_id = record.get("id")
        if not isinstance(evidence_id, str) or not evidence_id:
            errors.append("evidence record lacks a non-empty id")
        elif evidence_id in result:
            errors.append(f"duplicate evidence id: {evidence_id}")
        else:
            result[evidence_id] = record
    return result, errors


def result_index(results: dict[str, Any] | None) -> dict[str, dict[str, Any]]:
    return {
        item["evidence_id"]: item
        for item in ([] if results is None else results.get("results", []))
        if isinstance(item.get("evidence_id"), str)
    }


def validate_source_files(
    evidence_id: str, record: dict[str, Any], repo_root: Path | None
) -> list[str]:
    errors: list[str] = []
    files = record.get("source_files")
    if not isinstance(files, list) or not files:
        return [f"{evidence_id}: evidence lacks source_files"]
    for item in files:
        relative = item.get("path") if isinstance(item, dict) else None
        expected_sha = item.get("sha256") if isinstance(item, dict) else None
        if not isinstance(relative, str) or not relative or not isinstance(expected_sha, str):
            errors.append(f"{evidence_id}: invalid source_files entry")
            continue
        if repo_root is None:
            continue
        path = (repo_root / relative).resolve()
        try:
            path.relative_to(repo_root.resolve())
        except ValueError:
            errors.append(f"{evidence_id}: source path escapes repository: {relative}")
            continue
        if not path.is_file():
            errors.append(f"{evidence_id}: source file does not exist: {relative}")
        elif file_sha256(path) != expected_sha:
            errors.append(f"{evidence_id}: stale source hash: {relative}")
    return errors


def validate(
    java: dict[str, Any],
    rust: dict[str, Any],
    mapping: dict[str, Any],
    evidence_catalog: dict[str, Any] | None = None,
    evidence_results: dict[str, Any] | None = None,
    repo_root: Path | None = None,
) -> dict[str, Any]:
    expected = set(java_ids(java))
    available_rust = rust_ids(rust)
    entries = mapping.get("entries", [])
    entry_ids = [entry.get("java_id") for entry in entries]
    counts: Counter[str] = Counter()
    errors: list[str] = []
    evidence, evidence_errors = evidence_index(evidence_catalog)
    executions = result_index(evidence_results)
    errors.extend(evidence_errors)

    duplicates = sorted(item for item, count in Counter(entry_ids).items() if count > 1)
    unknown = sorted(set(entry_ids) - expected)
    missing = sorted(expected - set(entry_ids))
    if duplicates:
        errors.append(f"duplicate Java mappings: {duplicates[:10]}")
    if unknown:
        errors.append(f"unknown Java ids: {unknown[:10]}")
    if missing:
        errors.append(f"unmapped Java ids: {missing[:10]} (total={len(missing)})")

    for entry in entries:
        status = entry.get("status", "missing_status")
        counts[status] += 1
        java_id = entry.get("java_id", "<missing>")
        if status != "verified":
            errors.append(f"{java_id}: status={status}")
            continue
        for field in REQUIRED_EVIDENCE:
            values = entry.get(field)
            if not isinstance(values, list) or not values:
                errors.append(f"{java_id}: verified entry lacks {field}")
        for rust_id in entry.get("rust_ids", []):
            if rust_id not in available_rust:
                errors.append(f"{java_id}: unknown Rust id {rust_id}")
        mapped_rust = set(entry.get("rust_ids", []))
        for field, expected_kind in EVIDENCE_KIND.items():
            covered_rust: set[str] = set()
            for evidence_id in entry.get(field, []):
                if not isinstance(evidence_id, str):
                    errors.append(f"{java_id}: {field} contains a non-string evidence id")
                    continue
                record = evidence.get(evidence_id)
                if record is None:
                    errors.append(f"{java_id}: unknown evidence id {evidence_id}")
                    continue
                if record.get("kind") != expected_kind:
                    errors.append(
                        f"{java_id}: evidence {evidence_id} has kind={record.get('kind')}, "
                        f"expected={expected_kind}"
                    )
                if java_id not in record.get("java_ids", []):
                    errors.append(f"{java_id}: evidence {evidence_id} is not bound to this Java id")
                covered_rust.update(record.get("rust_ids", []))
                errors.extend(validate_source_files(evidence_id, record, repo_root))
                execution = executions.get(evidence_id)
                if execution is None:
                    errors.append(f"{java_id}: evidence {evidence_id} has no execution result")
                elif execution.get("status") != "passed":
                    errors.append(
                        f"{java_id}: evidence {evidence_id} execution status="
                        f"{execution.get('status', 'missing')}"
                    )
                else:
                    expected_commands = record.get("commands", [])
                    actual_commands = execution.get("commands", [])
                    if [item.get("argv") for item in actual_commands] != expected_commands:
                        errors.append(f"{java_id}: evidence {evidence_id} command attestation mismatch")
                    elif any(
                        item.get("status") != "passed" or item.get("exit_code") != 0
                        for item in actual_commands
                    ):
                        errors.append(f"{java_id}: evidence {evidence_id} has a failed command")
                if expected_kind == "compile_probe" and not REQUIRED_COMPILE_PROFILES.issubset(
                    set(record.get("profiles", []))
                ):
                    errors.append(
                        f"{java_id}: compile evidence {evidence_id} lacks default/all-features profiles"
                    )
            if not mapped_rust.issubset(covered_rust):
                missing_rust = sorted(mapped_rust - covered_rust)
                errors.append(f"{java_id}: {field} does not cover Rust ids {missing_rust}")
    return {
        "java_api_items": len(expected),
        "mapping_entries": len(entries),
        "status": dict(sorted(counts.items())),
        "error_count": len(errors),
        "errors": errors,
    }


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--java-api", type=Path, required=True)
    parser.add_argument("--rust-api", type=Path, required=True)
    parser.add_argument("--mapping", type=Path, required=True)
    parser.add_argument("--evidence-catalog", type=Path)
    parser.add_argument("--evidence-results", type=Path)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path)
    parser.add_argument("--initialize", action="store_true")
    parser.add_argument("--allow-incomplete", action="store_true")
    args = parser.parse_args()
    java = load(args.java_api)
    rust = load(args.rust_api)
    java_sha = file_sha256(args.java_api)
    rust_sha = file_sha256(args.rust_api)
    if args.initialize:
        args.mapping.parent.mkdir(parents=True, exist_ok=True)
        args.mapping.write_text(canonical_json(skeleton(java, java_sha, rust_sha)), encoding="utf-8")
    if not args.mapping.is_file():
        parser.error(f"mapping does not exist: {args.mapping}")
    mapping = load(args.mapping)
    evidence_catalog = load_catalog(args.evidence_catalog) if args.evidence_catalog else None
    evidence_results = load(args.evidence_results) if args.evidence_results else None
    report = validate(
        java,
        rust,
        mapping,
        evidence_catalog=evidence_catalog,
        evidence_results=evidence_results,
        repo_root=args.repo_root.resolve(),
    )
    if args.evidence_catalog and args.evidence_results:
        expected_catalog_sha = hashlib.sha256(
            canonical_json(evidence_catalog).encode("utf-8")
        ).hexdigest()
        if evidence_results.get("catalog_sha256") != expected_catalog_sha:
            report["errors"].append("evidence execution result was produced from a stale catalog")
    report["java_manifest_sha256_matches"] = mapping.get("java_manifest_sha256") == java_sha
    report["rust_manifest_sha256_matches"] = mapping.get("rust_manifest_sha256") == rust_sha
    if not report["java_manifest_sha256_matches"]:
        report["errors"].append("mapping Java snapshot hash is stale")
    if not report["rust_manifest_sha256_matches"]:
        report["errors"].append("mapping Rust snapshot hash is stale")
    report["error_count"] = len(report["errors"])
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(canonical_json(report), encoding="utf-8")
    print(json.dumps({key: value for key, value in report.items() if key != "errors"}, ensure_ascii=False))
    if report["errors"]:
        for error in report["errors"][:25]:
            print(f"- {error}", file=sys.stderr)
        if len(report["errors"]) > 25:
            print(f"- ... {len(report['errors']) - 25} more", file=sys.stderr)
        return 0 if args.allow_incomplete else 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
