#!/usr/bin/env python3
"""Verify fail-closed Java-to-Rust public API evidence mappings."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


REQUIRED_EVIDENCE = ("rust_ids", "compile_probes", "behavior_tests", "java_golden")
EVIDENCE_KIND = {
    "compile_probes": "compile_probe",
    "behavior_tests": "behavior_test",
    "java_golden": "java_golden",
}
REQUIRED_COMPILE_PROFILES = {"stable-default-features", "stable-all-features"}
IMPLEMENTATION_STRATEGIES = {
    "existing_implementation",
    "idiomatic_alternative",
    "needs_implementation",
}
EXPECTED_SNAPSHOT_MODES = {"default", "all_features"}
REQUIRED_MAPPED_RUST_MODES = {"default", "all_features"}
EXPECTED_JAVA_API_ITEMS = 3236
EXPECTED_CARGO_PUBLIC_API_VERSION = "cargo-public-api 0.52.0"


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
    return {
        "schema_version": catalog.get("schema_version", 1),
        "evidence": evidence,
        "mapping_resolutions": mapping_resolutions,
    }


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def java_ids(manifest: dict[str, Any]) -> list[str]:
    return sorted(
        item_id
        for item in [*manifest.get("types", []), *manifest.get("members", [])]
        if isinstance((item_id := item.get("id")), str) and item_id
    )


def rust_ids(manifest: dict[str, Any]) -> set[str]:
    return {
        item_id
        for package in manifest.get("packages", [])
        for snapshot in package.get("snapshots", [])
        for item in snapshot.get("items", [])
        if isinstance((item_id := item.get("id")), str) and item_id
    }


def rust_item_index(manifest: dict[str, Any]) -> dict[str, dict[str, str]]:
    """按稳定 Rust ID 合并 default/all-features 中的同一公开项。"""
    return {
        item["id"]: {"kind": item["kind"], "signature": item["signature"]}
        for package in manifest.get("packages", [])
        for snapshot in package.get("snapshots", [])
        for item in snapshot.get("items", [])
        if isinstance(item.get("id"), str)
        and isinstance(item.get("kind"), str)
        and isinstance(item.get("signature"), str)
    }


def rust_item_modes(manifest: dict[str, Any]) -> dict[str, set[str]]:
    result: dict[str, set[str]] = defaultdict(set)
    for package in manifest.get("packages", []):
        for snapshot in package.get("snapshots", []):
            mode = snapshot.get("mode")
            if not isinstance(mode, str):
                continue
            for item in snapshot.get("items", []):
                rust_id = item.get("id")
                if isinstance(rust_id, str):
                    result[rust_id].add(mode)
    return result


def skeleton(java_manifest: dict[str, Any], java_sha: str, rust_sha: str) -> dict[str, Any]:
    return {
        "schema_version": 2,
        "authority": "java_easyexcel_4.0.3_javap_public_api",
        "java_manifest_sha256": java_sha,
        "rust_manifest_sha256": rust_sha,
        "rust_extensions": [],
        "entries": [
            {
                "java_id": item_id,
                "status": "unmapped",
                "implementation_strategy": "needs_implementation",
                "implementation_carriers": [],
                "capability_carriers": [],
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
    result: dict[str, dict[str, Any]] = {}
    errors: list[str] = []
    if catalog is None:
        return result, errors
    if not isinstance(catalog, dict):
        return result, ["evidence catalog must be an object"]
    records = catalog.get("evidence", [])
    if not isinstance(records, list):
        return result, ["evidence catalog records must be a list"]
    for record in records:
        if not isinstance(record, dict):
            errors.append("evidence catalog contains a non-object record")
            continue
        evidence_id = record.get("id")
        if not isinstance(evidence_id, str) or not evidence_id:
            errors.append("evidence record lacks a non-empty id")
        elif evidence_id in result:
            errors.append(f"duplicate evidence id: {evidence_id}")
        else:
            result[evidence_id] = record
    return result, errors


def resolution_index(catalog: dict[str, Any] | None) -> tuple[dict[str, set[str]], list[str]]:
    """校验证据目录中的显式候选消歧，并按 Java ID 建索引。"""
    result: dict[str, set[str]] = {}
    errors: list[str] = []
    if catalog is None:
        return result, errors
    records = catalog.get("mapping_resolutions", [])
    if not isinstance(records, list):
        return result, ["mapping_resolutions must be a list"]
    for record in records:
        if not isinstance(record, dict):
            errors.append("mapping_resolutions contains a non-object record")
            continue
        java_id = record.get("java_id")
        rust_ids = record.get("rust_ids")
        if not isinstance(java_id, str) or not java_id:
            errors.append("mapping resolution lacks a non-empty Java id")
            continue
        if java_id in result:
            errors.append(f"duplicate mapping resolution: {java_id}")
            continue
        if (
            not isinstance(rust_ids, list)
            or not rust_ids
            or any(not isinstance(rust_id, str) or not rust_id for rust_id in rust_ids)
            or len(rust_ids) != len(set(rust_ids))
        ):
            errors.append(f"{java_id}: invalid mapping resolution Rust ids")
            continue
        result[java_id] = set(rust_ids)
    return result, errors


def result_index(results: dict[str, Any] | None) -> dict[str, dict[str, Any]]:
    if not isinstance(results, dict):
        return {}
    records = results.get("results", [])
    if not isinstance(records, list):
        return {}
    return {
        item["evidence_id"]: item
        for item in records
        if isinstance(item, dict) and isinstance(item.get("evidence_id"), str)
    }


def validate_java_manifest(java: dict[str, Any]) -> list[str]:
    """校验 javap 清单自身，防止残缺清单让映射门禁错误通过。"""
    errors: list[str] = []
    if java.get("schema_version") != 1:
        errors.append("unsupported Java public API manifest schema")
    if java.get("artifact") != "easyexcel-java-public-api":
        errors.append("unexpected Java public API artifact")
    if java.get("easyexcel_version") != "4.0.3":
        errors.append("Java public API manifest is not EasyExcel 4.0.3")
    java_repo = java.get("java_repo", {})
    if java_repo.get("exact_tag") != "v4.0.3":
        errors.append("Java public API manifest is not bound to the exact v4.0.3 tag")
    if java_repo.get("dirty") is not False:
        errors.append("Java public API manifest was extracted from a dirty worktree")
    if not isinstance(java_repo.get("git_sha"), str) or not java_repo["git_sha"]:
        errors.append("Java public API manifest lacks its source Git SHA")
    jars = java.get("jars")
    if not isinstance(jars, list) or not jars:
        errors.append("Java public API manifest contains no JAR provenance")
    elif any(
        not isinstance(jar, dict)
        or not isinstance(jar.get("sha256"), str)
        or not re.fullmatch(r"[0-9a-f]{64}", jar["sha256"])
        or not isinstance(jar.get("bytes"), int)
        or jar["bytes"] <= 0
        for jar in jars
    ):
        errors.append("Java public API manifest contains invalid JAR provenance")
    items = [*java.get("types", []), *java.get("members", [])]
    ids = [item.get("id") for item in items]
    if any(not isinstance(item_id, str) or not item_id for item_id in ids):
        errors.append("Java public API manifest contains an invalid id")
    if any(
        not isinstance(item.get("flags"), list)
        or not isinstance(item.get("synthetic"), bool)
        or not isinstance(item.get("bridge"), bool)
        for item in items
    ):
        errors.append("Java public API manifest lacks javap synthetic/bridge classification")
    duplicates = sorted(
        str(item_id) for item_id, count in Counter(ids).items() if count > 1
    )
    if duplicates:
        errors.append(f"duplicate Java public API ids: {duplicates[:10]}")
    summary = java.get("summary", {})
    if summary.get("public_api_items") != len(items):
        errors.append("Java public API summary count does not match manifest items")
    if len(items) != EXPECTED_JAVA_API_ITEMS:
        errors.append(
            f"EasyExcel 4.0.3 Java public API must contain {EXPECTED_JAVA_API_ITEMS} items"
        )
    return errors


def validate_rust_manifest(rust: dict[str, Any]) -> list[str]:
    """校验所有发布 crate 的 default/all-features 快照结构。"""
    errors: list[str] = []
    if rust.get("schema_version") != 1:
        errors.append("unsupported Rust public API manifest schema")
    if rust.get("artifact") != "easyexcel-rust-public-api":
        errors.append("unexpected Rust public API artifact")
    rust_repo = rust.get("rust_repo", {})
    if rust_repo.get("dirty") is not False:
        errors.append("Rust public API manifest was extracted from a dirty worktree")
    if not isinstance(rust_repo.get("git_sha"), str) or not rust_repo["git_sha"]:
        errors.append("Rust public API manifest lacks its source Git SHA")
    extractor = rust.get("extractor", {})
    if extractor.get("version") != EXPECTED_CARGO_PUBLIC_API_VERSION:
        errors.append("Rust public API snapshot used an unpinned cargo-public-api version")
    packages = rust.get("packages", [])
    package_names = [package.get("name") for package in packages]
    duplicates = sorted(
        str(name) for name, count in Counter(package_names).items() if count > 1
    )
    if duplicates:
        errors.append(f"duplicate Rust public API packages: {duplicates[:10]}")
    for package in packages:
        name = package.get("name", "<missing>")
        snapshots = package.get("snapshots", [])
        modes = [snapshot.get("mode") for snapshot in snapshots]
        if len(snapshots) != len(EXPECTED_SNAPSHOT_MODES) or set(modes) != EXPECTED_SNAPSHOT_MODES:
            errors.append(f"{name}: Rust public API snapshot must contain default/all_features")
        if len(modes) != len(set(modes)):
            errors.append(f"{name}: duplicate Rust public API snapshot mode")
        for snapshot in snapshots:
            items = snapshot.get("items", [])
            if snapshot.get("count") != len(items):
                errors.append(f"{name}/{snapshot.get('mode')}: snapshot count mismatch")
            item_ids = [item.get("id") for item in items]
            if any(not isinstance(item_id, str) or not item_id for item_id in item_ids):
                errors.append(f"{name}/{snapshot.get('mode')}: invalid Rust public API id")
            if len(item_ids) != len(set(item_ids)):
                errors.append(f"{name}/{snapshot.get('mode')}: duplicate Rust public API id")
    summary = rust.get("summary", {})
    if summary.get("packages") != len(packages):
        errors.append("Rust public API summary package count mismatch")
    default_items = sum(
        snapshot.get("count", 0)
        for package in packages
        for snapshot in package.get("snapshots", [])
        if snapshot.get("mode") == "default"
    )
    all_feature_items = sum(
        snapshot.get("count", 0)
        for package in packages
        for snapshot in package.get("snapshots", [])
        if snapshot.get("mode") == "all_features"
    )
    if summary.get("default_items") != default_items:
        errors.append("Rust public API default-items summary mismatch")
    if summary.get("all_feature_items") != all_feature_items:
        errors.append("Rust public API all-feature-items summary mismatch")
    return errors


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
    available_rust_items = rust_item_index(rust)
    available_rust_modes = rust_item_modes(rust)
    counts: Counter[str] = Counter()
    strategy_counts: Counter[str] = Counter()
    errors: list[str] = []
    raw_entries = mapping.get("entries", [])
    if not isinstance(raw_entries, list):
        errors.append("mapping entries must be a list")
        raw_entries = []
    if any(not isinstance(entry, dict) for entry in raw_entries):
        errors.append("mapping entries contains a non-object entry")
    entries = [entry for entry in raw_entries if isinstance(entry, dict)]
    entry_ids = [entry.get("java_id") for entry in entries]
    evidence, evidence_errors = evidence_index(evidence_catalog)
    resolutions, resolution_errors = resolution_index(evidence_catalog)
    executions = result_index(evidence_results)
    published_packages: set[str] = set()
    errors.extend(evidence_errors)
    errors.extend(resolution_errors)
    evidence_structure_valid = not evidence_errors and not resolution_errors
    java_manifest_errors = validate_java_manifest(java)
    rust_manifest_errors = validate_rust_manifest(rust)
    errors.extend(java_manifest_errors)
    errors.extend(rust_manifest_errors)
    manifest_structure_valid = not java_manifest_errors and not rust_manifest_errors

    mapping_authority_valid = (
        mapping.get("authority") == "java_easyexcel_4.0.3_javap_public_api"
        and mapping.get("schema_version") == 2
    )
    if mapping.get("authority") != "java_easyexcel_4.0.3_javap_public_api":
        errors.append("mapping authority is not the EasyExcel 4.0.3 javap public API")
    if mapping.get("schema_version") != 2:
        errors.append("public API mapping must use schema_version=2")

    if evidence_results is None:
        execution_records = []
        evidence_structure_valid = False
    elif isinstance(evidence_results, dict):
        execution_records = evidence_results.get("results", [])
    else:
        errors.append("evidence execution artifact must be an object")
        execution_records = []
        evidence_structure_valid = False
    if not isinstance(execution_records, list):
        errors.append("evidence execution results must be a list")
        execution_records = []
        evidence_structure_valid = False
    if any(not isinstance(record, dict) for record in execution_records):
        errors.append("evidence execution results contains a non-object entry")
        evidence_structure_valid = False
    execution_records = [
        record for record in execution_records if isinstance(record, dict)
    ]
    execution_ids = [record.get("evidence_id") for record in execution_records]
    duplicate_execution_ids = sorted(
        str(evidence_id)
        for evidence_id, count in Counter(execution_ids).items()
        if count > 1
    )
    if duplicate_execution_ids:
        errors.append(f"duplicate evidence execution results: {duplicate_execution_ids[:10]}")
        evidence_structure_valid = False
    catalog_ids = set(evidence)
    valid_execution_ids = {
        evidence_id for evidence_id in execution_ids if isinstance(evidence_id, str)
    }
    missing_executions = sorted(catalog_ids - valid_execution_ids)
    unknown_executions = sorted(valid_execution_ids - catalog_ids)
    if missing_executions:
        errors.append(f"evidence records lack execution results: {missing_executions[:10]}")
        evidence_structure_valid = False
    if unknown_executions:
        errors.append(f"execution results reference unknown evidence: {unknown_executions[:10]}")
        evidence_structure_valid = False

    valid_evidence_kinds = set(EVIDENCE_KIND.values())
    evidence_record_error_start = len(errors)
    for evidence_id, record in evidence.items():
        kind = record.get("kind")
        if kind not in valid_evidence_kinds:
            errors.append(f"{evidence_id}: invalid evidence kind={kind}")
        bound_java = record.get("java_ids")
        if not isinstance(bound_java, list) or not bound_java or any(
            not isinstance(java_id, str) or not java_id for java_id in bound_java
        ):
            errors.append(f"{evidence_id}: invalid or empty java_ids")
            bound_java = []
        unknown_java = sorted(set(bound_java) - expected)
        if unknown_java:
            errors.append(f"{evidence_id}: unknown Java ids={unknown_java[:10]}")
        bound_rust = record.get("rust_ids")
        if not isinstance(bound_rust, list) or not bound_rust or any(
            not isinstance(rust_id, str) or not rust_id for rust_id in bound_rust
        ):
            errors.append(f"{evidence_id}: invalid or empty rust_ids")
            bound_rust = []
        unknown_rust = sorted(set(bound_rust) - available_rust)
        if unknown_rust:
            errors.append(f"{evidence_id}: unknown Rust ids={unknown_rust[:10]}")
        commands = record.get("commands")
        if not isinstance(commands, list) or not commands or any(
            not isinstance(command, list)
            or not command
            or any(not isinstance(argument, str) or not argument for argument in command)
            for command in commands
        ):
            errors.append(f"{evidence_id}: invalid or empty commands")
        errors.extend(validate_source_files(evidence_id, record, repo_root))
        if kind == "compile_probe":
            profiles = record.get("profiles")
            if (
                not isinstance(profiles, list)
                or any(not isinstance(profile, str) for profile in profiles)
                or not REQUIRED_COMPILE_PROFILES.issubset(set(profiles))
            ):
                errors.append(
                    f"{evidence_id}: compile evidence lacks default/all-features profiles"
                )
    if len(errors) != evidence_record_error_start:
        evidence_structure_valid = False

    scope = rust.get("scope")
    snapshot_scope_valid = True
    if not isinstance(scope, dict) or scope.get("authoritative") is not True:
        errors.append("Rust public API snapshot is partial or lacks authoritative workspace scope")
        snapshot_scope_valid = False
    else:
        included_values = scope.get("included_packages", [])
        published_values = scope.get("published_workspace_packages", [])
        package_values = rust.get("packages", [])
        if (
            not isinstance(included_values, list)
            or any(not isinstance(package, str) or not package for package in included_values)
            or not isinstance(published_values, list)
            or any(not isinstance(package, str) or not package for package in published_values)
            or not isinstance(package_values, list)
            or any(not isinstance(package, dict) for package in package_values)
        ):
            errors.append("Rust public API snapshot has an invalid authoritative scope")
            snapshot_scope_valid = False
            included_values = []
            published_values = []
            package_values = []
        included_packages = set(included_values)
        published_packages = set(published_values)
        manifest_packages = {
            package.get("name") for package in package_values if isinstance(package, dict)
        }
        if included_packages != published_packages:
            errors.append("Rust public API snapshot omits published workspace crates")
            snapshot_scope_valid = False
        if manifest_packages != included_packages:
            errors.append("Rust public API package list does not match its authoritative scope")
            snapshot_scope_valid = False
    manifest_structure_valid = manifest_structure_valid and snapshot_scope_valid

    duplicates = sorted(
        str(item) for item, count in Counter(entry_ids).items() if count > 1
    )
    valid_entry_ids = {item for item in entry_ids if isinstance(item, str)}
    unknown = sorted(valid_entry_ids - expected)
    missing = sorted(expected - valid_entry_ids)
    if any(not isinstance(item, str) or not item for item in entry_ids):
        errors.append("mapping contains an invalid Java id")
    if duplicates:
        errors.append(f"duplicate Java mappings: {duplicates[:10]}")
    if unknown:
        errors.append(f"unknown Java ids: {unknown[:10]}")
    if missing:
        errors.append(f"unmapped Java ids: {missing[:10]} (total={len(missing)})")

    resolution_binding_error_start = len(errors)
    entries_by_java = {
        entry.get("java_id"): entry
        for entry in entries
        if isinstance(entry.get("java_id"), str)
    }
    for java_id, resolved_rust_ids in sorted(resolutions.items()):
        if java_id not in expected:
            errors.append(f"mapping resolution references unknown Java id: {java_id}")
            continue
        entry = entries_by_java.get(java_id)
        if entry is None or Counter(entry_ids)[java_id] != 1:
            errors.append(f"mapping resolution lacks one unique mapping entry: {java_id}")
            continue
        mapped_values = entry.get("rust_ids")
        mapped_rust_ids = set(mapped_values) if isinstance(mapped_values, list) else set()
        if mapped_rust_ids != resolved_rust_ids:
            errors.append(
                f"mapping resolution does not match applied Rust ids for {java_id}"
            )
        if entry.get("status") == "ambiguous":
            errors.append(f"mapping resolution was not applied for {java_id}")
    if len(errors) != resolution_binding_error_start:
        evidence_structure_valid = False

    entry_id_counts = Counter(entry_ids)
    verified_java_ids: set[str] = set()
    for entry in entries:
        entry_error_start = len(errors)
        status = entry.get("status", "missing_status")
        counts[status] += 1
        java_id = entry.get("java_id", "<missing>")
        strategy = entry.get("implementation_strategy")
        if status not in {"unmapped", "candidate", "ambiguous", "verified"}:
            errors.append(f"{java_id}: invalid status={status}")
        mapped_rust_values = entry.get("rust_ids")
        if not isinstance(mapped_rust_values, list) or any(
            not isinstance(rust_id, str) or not rust_id for rust_id in mapped_rust_values
        ):
            errors.append(f"{java_id}: invalid rust_ids")
            mapped_rust_values = []
        if len(mapped_rust_values) != len(set(mapped_rust_values)):
            errors.append(f"{java_id}: duplicate rust_ids")
        mapped_rust = set(mapped_rust_values)
        for rust_id in sorted(mapped_rust):
            if rust_id not in available_rust:
                errors.append(f"{java_id}: unknown Rust id {rust_id}")
            elif not REQUIRED_MAPPED_RUST_MODES.issubset(
                available_rust_modes.get(rust_id, set())
            ):
                errors.append(
                    f"{java_id}: Rust id is not public in default/all-features "
                    f"modes={sorted(available_rust_modes.get(rust_id, set()))}"
                )
        if mapping.get("schema_version", 1) >= 2:
            if strategy not in IMPLEMENTATION_STRATEGIES:
                errors.append(f"{java_id}: invalid implementation_strategy={strategy}")
            else:
                strategy_counts[strategy] += 1
                if status == "verified" and strategy == "needs_implementation":
                    errors.append(f"{java_id}: verified entry still needs implementation")
                if status == "unmapped" and strategy != "needs_implementation":
                    errors.append(
                        f"{java_id}: unmapped entry has implemented strategy={strategy}"
                    )
            carriers = entry.get("implementation_carriers")
            if not isinstance(carriers, list) or any(
                not isinstance(carrier, str) or not carrier for carrier in carriers
            ):
                errors.append(f"{java_id}: invalid implementation_carriers")
            elif strategy == "needs_implementation" and carriers:
                errors.append(f"{java_id}: missing implementation declares carriers={carriers}")
            elif strategy != "needs_implementation" and not carriers:
                errors.append(f"{java_id}: implemented mapping lacks carrier crates")
            elif isinstance(carriers, list):
                duplicate_carriers = sorted(
                    carrier for carrier, count in Counter(carriers).items() if count > 1
                )
                if duplicate_carriers:
                    errors.append(
                        f"{java_id}: duplicate implementation carriers={duplicate_carriers}"
                    )
                unknown_carriers = sorted(set(carriers) - published_packages)
                if unknown_carriers:
                    errors.append(
                        f"{java_id}: carriers are not published workspace crates={unknown_carriers}"
                    )
                rust_id_carriers = {
                    rust_id.split(":", 1)[0]
                    for rust_id in mapped_rust
                    if ":" in rust_id
                }
                missing_carriers = sorted(rust_id_carriers - set(carriers))
                if missing_carriers:
                    errors.append(
                        f"{java_id}: implementation carriers omit mapped Rust crates={missing_carriers}"
                    )
                unexpected_carriers = sorted(set(carriers) - rust_id_carriers)
                if strategy != "needs_implementation" and unexpected_carriers:
                    errors.append(
                        f"{java_id}: implementation carriers are not backed by mapped "
                        f"Rust ids={unexpected_carriers}"
                    )
            capability_carriers = entry.get("capability_carriers")
            if not isinstance(capability_carriers, list) or any(
                not isinstance(carrier, str) or not carrier
                for carrier in capability_carriers
            ):
                errors.append(f"{java_id}: invalid capability_carriers")
            elif strategy == "needs_implementation" and capability_carriers:
                errors.append(
                    f"{java_id}: missing implementation declares capability carriers="
                    f"{capability_carriers}"
                )
            elif isinstance(capability_carriers, list):
                duplicate_capability_carriers = sorted(
                    carrier
                    for carrier, count in Counter(capability_carriers).items()
                    if count > 1
                )
                if duplicate_capability_carriers:
                    errors.append(
                        f"{java_id}: duplicate capability carriers="
                        f"{duplicate_capability_carriers}"
                    )
                unknown_capability_carriers = sorted(
                    set(capability_carriers) - published_packages
                )
                if unknown_capability_carriers:
                    errors.append(
                        f"{java_id}: capability carriers are not published workspace crates="
                        f"{unknown_capability_carriers}"
                    )
                overlap = sorted(set(capability_carriers) & set(carriers or []))
                if overlap:
                    errors.append(
                        f"{java_id}: capability carriers duplicate public implementation "
                        f"carriers={overlap}"
                    )
            notes = entry.get("semantic_notes")
            if strategy in {"idiomatic_alternative", "needs_implementation"} and (
                not isinstance(notes, str) or not notes.strip()
            ):
                errors.append(f"{java_id}: strategy requires semantic_notes")
            if strategy == "needs_implementation" and mapped_rust:
                errors.append(f"{java_id}: missing implementation declares Rust ids")
            if strategy != "needs_implementation" and not mapped_rust:
                errors.append(f"{java_id}: implemented strategy lacks Rust ids")
        if status != "verified":
            errors.append(f"{java_id}: status={status}")
            continue
        for field in REQUIRED_EVIDENCE:
            values = entry.get(field)
            if not isinstance(values, list) or not values:
                errors.append(f"{java_id}: verified entry lacks {field}")
        for field, expected_kind in EVIDENCE_KIND.items():
            covered_rust: set[str] = set()
            evidence_ids = entry.get(field)
            if not isinstance(evidence_ids, list):
                evidence_ids = []
            for evidence_id in evidence_ids:
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
                record_java_ids = record.get("java_ids")
                if not isinstance(record_java_ids, list) or java_id not in record_java_ids:
                    errors.append(f"{java_id}: evidence {evidence_id} is not bound to this Java id")
                evidence_rust_ids = record.get("rust_ids", [])
                if not isinstance(evidence_rust_ids, list) or any(
                    not isinstance(rust_id, str) or not rust_id
                    for rust_id in evidence_rust_ids
                ):
                    errors.append(f"{java_id}: evidence {evidence_id} has invalid rust_ids")
                    evidence_rust_ids = []
                unknown_evidence_rust = sorted(set(evidence_rust_ids) - available_rust)
                if unknown_evidence_rust:
                    errors.append(
                        f"{java_id}: evidence {evidence_id} references unknown Rust ids "
                        f"{unknown_evidence_rust}"
                    )
                covered_rust.update(evidence_rust_ids)
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
                    if not isinstance(expected_commands, list):
                        expected_commands = []
                    if not isinstance(actual_commands, list) or any(
                        not isinstance(item, dict) for item in actual_commands
                    ):
                        errors.append(
                            f"{java_id}: evidence {evidence_id} has invalid command attestation"
                        )
                        actual_commands = []
                    if [item.get("argv") for item in actual_commands] != expected_commands:
                        errors.append(f"{java_id}: evidence {evidence_id} command attestation mismatch")
                    elif any(
                        item.get("status") != "passed" or item.get("exit_code") != 0
                        for item in actual_commands
                    ):
                        errors.append(f"{java_id}: evidence {evidence_id} has a failed command")
                profiles = record.get("profiles", [])
                if not isinstance(profiles, list):
                    profiles = []
                if expected_kind == "compile_probe" and not REQUIRED_COMPILE_PROFILES.issubset(
                    set(profiles)
                ):
                    errors.append(
                        f"{java_id}: compile evidence {evidence_id} lacks default/all-features profiles"
                    )
            if not mapped_rust.issubset(covered_rust):
                missing_rust = sorted(mapped_rust - covered_rust)
                errors.append(f"{java_id}: {field} does not cover Rust ids {missing_rust}")
        if (
            len(errors) == entry_error_start
            and manifest_structure_valid
            and mapping_authority_valid
            and evidence_structure_valid
            and isinstance(java_id, str)
            and java_id in expected
            and entry_id_counts[java_id] == 1
        ):
            verified_java_ids.add(java_id)

    mapped_rust_ids = {
        rust_id
        for entry in entries
        for rust_id in entry.get("rust_ids", [])
        if isinstance(rust_id, str)
    }
    extensions = mapping.get("rust_extensions", [])
    if not isinstance(extensions, list):
        errors.append("mapping rust_extensions must be a list")
        extensions = []
    extension_ids: list[str] = []
    for extension in extensions:
        if not isinstance(extension, dict):
            errors.append("rust_extensions contains a non-object entry")
            continue
        rust_id = extension.get("rust_id")
        if not isinstance(rust_id, str) or not rust_id:
            errors.append("rust extension lacks a non-empty rust_id")
            continue
        extension_ids.append(rust_id)
        if rust_id not in available_rust:
            errors.append(f"unknown Rust extension id: {rust_id}")
        if extension.get("status") != "documented_extension":
            errors.append(f"{rust_id}: Rust extension lacks documented_extension status")
        if extension.get("classification") != "unmapped_rust_public_api":
            errors.append(f"{rust_id}: Rust extension lacks complement classification")
        expected_carrier = rust_id.split(":", 1)[0] if ":" in rust_id else None
        if extension.get("implementation_carriers") != [expected_carrier]:
            errors.append(f"{rust_id}: Rust extension has invalid carrier ownership")
        if extension.get("capability_carriers") != []:
            errors.append(f"{rust_id}: standalone Rust extension declares capability carriers")
        snapshot_item = available_rust_items.get(rust_id)
        if snapshot_item is not None:
            if extension.get("kind") != snapshot_item["kind"]:
                errors.append(f"{rust_id}: Rust extension kind drifted from snapshot")
            if extension.get("signature") != snapshot_item["signature"]:
                errors.append(f"{rust_id}: Rust extension signature drifted from snapshot")
        if extension.get("modes") != sorted(available_rust_modes.get(rust_id, set())):
            errors.append(f"{rust_id}: Rust extension feature modes drifted from snapshot")
        if not isinstance(extension.get("semantic_notes"), str) or not extension["semantic_notes"].strip():
            errors.append(f"{rust_id}: Rust extension lacks semantic_notes")
    duplicate_extensions = sorted(
        rust_id for rust_id, count in Counter(extension_ids).items() if count > 1
    )
    if duplicate_extensions:
        errors.append(f"duplicate Rust extension classifications: {duplicate_extensions[:10]}")
    overlaps = sorted(mapped_rust_ids & set(extension_ids))
    if overlaps:
        errors.append(f"Rust ids classified as both Java mappings and extensions: {overlaps[:10]}")
    orphan_rust = sorted(available_rust - mapped_rust_ids - set(extension_ids))
    if orphan_rust:
        errors.append(
            f"unclassified Rust public API ids: {orphan_rust[:10]} (total={len(orphan_rust)})"
        )
    # 进度数字必须比错误清单更保守：strategy 字符串存在并不代表该 Java ID
    # 已完成所有权分类，更不代表已有可调用实现。只有唯一、权威 Java ID，
    # 当前 default/all-features 均公开的 Rust ID，以及与这些 ID 精确一致的
    # implementation carrier 同时成立时，才计入 classified/coded。
    classified_java_ids: set[str] = set()
    coded_java_ids: set[str] = set()
    classified_strategy_counts: Counter[str] = Counter()
    for entry in entries:
        if not manifest_structure_valid or not mapping_authority_valid:
            continue
        java_id = entry.get("java_id")
        strategy = entry.get("implementation_strategy")
        status = entry.get("status")
        mapped_values = entry.get("rust_ids")
        carriers = entry.get("implementation_carriers")
        capability_carriers = entry.get("capability_carriers")
        notes = entry.get("semantic_notes")
        if (
            not isinstance(java_id, str)
            or java_id not in expected
            or entry_id_counts[java_id] != 1
            or strategy not in IMPLEMENTATION_STRATEGIES
            or not isinstance(mapped_values, list)
            or any(not isinstance(value, str) or not value for value in mapped_values)
            or len(mapped_values) != len(set(mapped_values))
            or not isinstance(carriers, list)
            or any(not isinstance(value, str) or not value for value in carriers)
            or len(carriers) != len(set(carriers))
            or not isinstance(capability_carriers, list)
            or any(
                not isinstance(value, str) or not value
                for value in capability_carriers
            )
            or len(capability_carriers) != len(set(capability_carriers))
            or not set(capability_carriers).issubset(published_packages)
            or set(capability_carriers) & set(carriers)
        ):
            continue
        mapped = set(mapped_values)
        mapped_carriers = {
            rust_id.split(":", 1)[0] for rust_id in mapped if ":" in rust_id
        }
        rust_mapping_is_public = all(
            rust_id in available_rust
            and REQUIRED_MAPPED_RUST_MODES.issubset(
                available_rust_modes.get(rust_id, set())
            )
            for rust_id in mapped
        )
        strategy_notes_are_valid = strategy == "existing_implementation" or (
            isinstance(notes, str) and bool(notes.strip())
        )
        if strategy == "needs_implementation":
            classification_is_valid = (
                status == "unmapped"
                and not mapped
                and not carriers
                and not capability_carriers
                and strategy_notes_are_valid
            )
        else:
            classification_is_valid = (
                status in {"candidate", "ambiguous", "verified"}
                and bool(mapped)
                and rust_mapping_is_public
                and bool(carriers)
                and set(carriers) == mapped_carriers
                and set(carriers).issubset(published_packages)
                and strategy_notes_are_valid
            )
        if classification_is_valid:
            classified_java_ids.add(java_id)
            classified_strategy_counts[strategy] += 1
            if strategy in {"existing_implementation", "idiomatic_alternative"}:
                coded_java_ids.add(java_id)
    classified_java_api_items = len(classified_java_ids)
    coded_java_api_items = len(coded_java_ids)
    return {
        "java_api_items": len(expected),
        "rust_api_items": len(available_rust),
        "classified_rust_extensions": len(set(extension_ids)),
        "unclassified_rust_api_items": len(orphan_rust),
        "mapping_entries": len(entries),
        "manifest_structure_valid": manifest_structure_valid,
        "mapping_structure_valid": mapping_authority_valid,
        "evidence_structure_valid": evidence_structure_valid,
        "progress": {
            "classified_java_api_items": classified_java_api_items,
            "coded_java_api_items": coded_java_api_items,
            "verified_java_api_items": len(verified_java_ids),
            "needs_implementation_java_api_items": classified_strategy_counts[
                "needs_implementation"
            ],
            "total_java_api_items": len(expected),
        },
        "implementation_strategy": dict(sorted(classified_strategy_counts.items())),
        "declared_implementation_strategy": dict(sorted(strategy_counts.items())),
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
    report["verified_progress_authoritative"] = bool(
        args.evidence_catalog and args.evidence_results
    ) and report["evidence_structure_valid"]
    if args.evidence_catalog and args.evidence_results:
        expected_catalog_sha = hashlib.sha256(
            canonical_json(evidence_catalog).encode("utf-8")
        ).hexdigest()
        if (
            not isinstance(evidence_results, dict)
            or evidence_results.get("catalog_sha256") != expected_catalog_sha
        ):
            report["errors"].append("evidence execution result was produced from a stale catalog")
            report["verified_progress_authoritative"] = False
            report["progress"]["verified_java_api_items"] = 0
    report["java_manifest_sha256_matches"] = mapping.get("java_manifest_sha256") == java_sha
    report["rust_manifest_sha256_matches"] = mapping.get("rust_manifest_sha256") == rust_sha
    if not report["java_manifest_sha256_matches"]:
        report["errors"].append("mapping Java snapshot hash is stale")
    if not report["rust_manifest_sha256_matches"]:
        report["errors"].append("mapping Rust snapshot hash is stale")
    report["classification_progress_authoritative"] = (
        report["manifest_structure_valid"]
        and report["mapping_structure_valid"]
        and report["java_manifest_sha256_matches"]
        and report["rust_manifest_sha256_matches"]
    )
    if not report["classification_progress_authoritative"]:
        report["progress"]["classified_java_api_items"] = 0
        report["progress"]["coded_java_api_items"] = 0
        report["progress"]["needs_implementation_java_api_items"] = 0
        report["implementation_strategy"] = {}
        report["verified_progress_authoritative"] = False
        report["progress"]["verified_java_api_items"] = 0
    else:
        report["verified_progress_authoritative"] = (
            report["verified_progress_authoritative"]
            and report["classification_progress_authoritative"]
        )
    if not report["verified_progress_authoritative"]:
        report["progress"]["verified_java_api_items"] = 0
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
