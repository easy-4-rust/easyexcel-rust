#!/usr/bin/env python3
"""Materialize type-family evidence templates into exact per-Java-ID records.

The checked-in catalog may group a coherent Java owner family so evidence can be
implemented in batches.  The generated catalog remains the gate authority: every
record contains the exact Java IDs, the exact candidate Rust IDs, and current
source hashes.  A family cannot materialize while any selected API still has no
implemented carrier.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


IMPLEMENTED_STRATEGIES = {"existing_implementation", "idiomatic_alternative"}


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def load_template_tree(
    path: Path,
    root: Path | None = None,
    stack: tuple[Path, ...] = (),
) -> tuple[
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[Path],
]:
    """Load exact evidence, family templates, and mapping resolutions safely."""
    root = path.parent.resolve() if root is None else root
    resolved = path.resolve()
    try:
        resolved.relative_to(root)
    except ValueError as error:
        raise ValueError(f"evidence template escapes root: {path}") from error
    if resolved in stack:
        chain = " -> ".join(str(item) for item in (*stack, resolved))
        raise ValueError(f"cyclic evidence template include: {chain}")
    catalog = load(resolved)
    exact = list(catalog.get("evidence", []))
    families = list(catalog.get("family_evidence", []))
    resolutions = list(catalog.get("mapping_resolutions", []))
    sources = [resolved]
    includes = catalog.get("include", [])
    if not isinstance(includes, list) or any(not isinstance(item, str) for item in includes):
        raise ValueError(f"invalid evidence template include list: {resolved}")
    for relative in includes:
        included = load_template_tree(
            resolved.parent / relative,
            root,
            (*stack, resolved),
        )
        exact.extend(included[0])
        families.extend(included[1])
        resolutions.extend(included[2])
        sources.extend(included[3])
    return exact, families, resolutions, sources


def template_tree_sha256(paths: list[Path], root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(set(paths)):
        relative = path.relative_to(root)
        digest.update(relative.as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def java_items(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    items = [*manifest.get("types", []), *manifest.get("members", [])]
    result: dict[str, dict[str, Any]] = {}
    for item in items:
        java_id = item.get("id")
        if not isinstance(java_id, str) or not java_id:
            raise ValueError("Java manifest contains an item without a stable id")
        if java_id in result:
            raise ValueError(f"duplicate Java API id: {java_id}")
        result[java_id] = item
    return result


def matches(item: dict[str, Any], template: dict[str, Any]) -> bool:
    prefixes = template.get("java_owner_prefixes")
    owners = template.get("java_owners")
    names = template.get("java_names")
    kinds = template.get("java_kinds")
    if prefixes is not None and (
        not isinstance(prefixes, list)
        or not prefixes
        or any(not isinstance(value, str) or not value for value in prefixes)
    ):
        raise ValueError("family java_owner_prefixes must be a non-empty string list")
    if owners is not None and (
        not isinstance(owners, list)
        or not owners
        or any(not isinstance(value, str) or not value for value in owners)
    ):
        raise ValueError("family java_owners must be a non-empty string list")
    if names is not None and (
        not isinstance(names, list)
        or not names
        or any(not isinstance(value, str) or not value for value in names)
    ):
        raise ValueError("family java_names must be a non-empty string list")
    if kinds is not None and (
        not isinstance(kinds, list)
        or not kinds
        or any(not isinstance(value, str) or not value for value in kinds)
    ):
        raise ValueError("family java_kinds must be a non-empty string list")
    if prefixes is None and owners is None:
        raise ValueError("family evidence must constrain java owners")
    owner = item.get("owner")
    if not isinstance(owner, str):
        return False
    owner_matches = (prefixes is None or any(owner.startswith(value) for value in prefixes)) and (
        owners is None or owner in owners
    )
    return (
        owner_matches
        and (names is None or item.get("name") in names)
        and (kinds is None or item.get("kind") in kinds)
    )


def source_files(template: dict[str, Any], repo_root: Path) -> list[dict[str, str]]:
    paths = template.get("source_paths", [])
    globs = template.get("source_globs", [])
    if not isinstance(paths, list) or any(not isinstance(value, str) for value in paths):
        raise ValueError("family source_paths must be a string list")
    if not isinstance(globs, list) or any(not isinstance(value, str) for value in globs):
        raise ValueError("family source_globs must be a string list")
    selected: set[Path] = set()
    for value in paths:
        selected.add(repo_root / value)
    for pattern in globs:
        matched = [path for path in repo_root.glob(pattern) if path.is_file()]
        if not matched:
            raise ValueError(f"family source glob matched no files: {pattern}")
        selected.update(matched)
    if not selected:
        raise ValueError("family evidence must bind at least one source file")
    records = []
    for path in sorted(selected):
        resolved = path.resolve()
        try:
            relative = resolved.relative_to(repo_root)
        except ValueError as error:
            raise ValueError(f"family evidence source escapes repository: {path}") from error
        if not resolved.is_file():
            raise ValueError(f"family evidence source does not exist: {relative}")
        records.append({"path": relative.as_posix(), "sha256": sha256_file(resolved)})
    return records


def materialize_family(
    template: dict[str, Any],
    items: dict[str, dict[str, Any]],
    candidates: dict[str, dict[str, Any]],
    repo_root: Path,
) -> dict[str, Any]:
    evidence_id = template.get("id")
    kind = template.get("kind")
    commands = template.get("commands")
    if not isinstance(evidence_id, str) or not evidence_id:
        raise ValueError("family evidence lacks id")
    if kind not in {"compile_probe", "behavior_test", "java_golden"}:
        raise ValueError(f"{evidence_id}: invalid evidence kind={kind}")
    if (
        not isinstance(commands, list)
        or not commands
        or any(
            not isinstance(command, list)
            or not command
            or any(not isinstance(arg, str) or not arg for arg in command)
            for command in commands
        )
    ):
        raise ValueError(f"{evidence_id}: invalid commands")
    selected = sorted(java_id for java_id, item in items.items() if matches(item, template))
    expected_count = template.get("expected_java_api_items")
    if not selected:
        raise ValueError(f"{evidence_id}: selector matched no Java API ids")
    if expected_count is not None and expected_count != len(selected):
        raise ValueError(
            f"{evidence_id}: selector count changed: expected={expected_count}, actual={len(selected)}"
        )
    rust_ids: set[str] = set()
    for java_id in selected:
        candidate = candidates.get(java_id)
        if candidate is None:
            raise ValueError(f"{evidence_id}: Java id has no candidate entry: {java_id}")
        strategy = candidate.get("implementation_strategy")
        mapped = candidate.get("rust_ids")
        if strategy not in IMPLEMENTED_STRATEGIES:
            raise ValueError(
                f"{evidence_id}: Java id is not implemented: {java_id} strategy={strategy}"
            )
        if not isinstance(mapped, list) or not mapped or any(
            not isinstance(rust_id, str) or not rust_id for rust_id in mapped
        ):
            raise ValueError(f"{evidence_id}: Java id lacks exact Rust candidates: {java_id}")
        rust_ids.update(mapped)
    record: dict[str, Any] = {
        "id": evidence_id,
        "kind": kind,
        "java_ids": selected,
        "rust_ids": sorted(rust_ids),
        "commands": commands,
        "source_files": source_files(template, repo_root),
        "materialized_family": {
            "expected_java_api_items": len(selected),
            "selector": {
                key: template[key]
                for key in (
                    "java_owner_prefixes",
                    "java_owners",
                    "java_names",
                    "java_kinds",
                )
                if key in template
            },
        },
    }
    if kind == "compile_probe":
        profiles = template.get("profiles")
        if not isinstance(profiles, list) or not profiles:
            raise ValueError(f"{evidence_id}: compile family lacks profiles")
        record["profiles"] = profiles
    return record


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--template", required=True, type=Path)
    parser.add_argument("--java-api", required=True, type=Path)
    parser.add_argument("--candidate-mapping", required=True, type=Path)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    template_path = args.template.resolve()
    java_path = args.java_api.resolve()
    candidate_path = args.candidate_mapping.resolve()
    repo_root = args.repo_root.resolve()
    exact, families, resolutions, template_sources = load_template_tree(template_path)
    items = java_items(load(java_path))
    candidate_mapping = load(candidate_path)
    candidate_entries = {
        entry.get("java_id"): entry
        for entry in candidate_mapping.get("entries", [])
        if isinstance(entry, dict) and isinstance(entry.get("java_id"), str)
    }
    records = [
        *exact,
        *(
            materialize_family(family, items, candidate_entries, repo_root)
            for family in families
        ),
    ]
    ids = [record.get("id") for record in records]
    duplicates = sorted(value for value in set(ids) if ids.count(value) > 1)
    if duplicates:
        raise ValueError(f"duplicate materialized evidence ids: {duplicates}")
    output = {
        "schema_version": 2,
        "authority": "materialized_exact_public_api_evidence",
        "template_tree_sha256": template_tree_sha256(
            template_sources, template_path.parent
        ),
        "template_sources": [
            {
                "path": path.relative_to(template_path.parent).as_posix(),
                "sha256": sha256_file(path),
            }
            for path in sorted(set(template_sources))
        ],
        "java_manifest_sha256": sha256_file(java_path),
        "candidate_mapping_sha256": sha256_file(candidate_path),
        "evidence": records,
        "mapping_resolutions": resolutions,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(canonical_json(output), encoding="utf-8")
    print(
        json.dumps(
            {
                "exact_records": len(exact),
                "family_records": len(families),
                "materialized_records": len(records),
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
