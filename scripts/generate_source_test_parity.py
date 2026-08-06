#!/usr/bin/env python3
"""Generate an explicit Java @Test to Rust #[test] parity manifest.

The generator accepts only source-level evidence written next to a Rust test:
`com.alibaba.easyexcel.test...Class#method`, or an unqualified `Class#method`
when that Java class name is unique in the source inventory.  It deliberately
does not use fuzzy names or a passing Cargo test count as parity evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


JAVA_PACKAGE_RE = re.compile(r"^package\s+([\w.]+);", re.MULTILINE)
JAVA_CLASS_RE = re.compile(r"\b(?:public\s+)?class\s+(\w+)")
JAVA_TEST_RE = re.compile(r"^\s*@Test(?:\s|\(|$)")
JAVA_METHOD_RE = re.compile(
    r"(?:(?:public|protected|private|static|final|synchronized)\s+)*"
    r"(?:void|[\w<>?,.\[\] ]+)\s+(\w+)\s*\("
)
RUST_FUNCTION_RE = re.compile(r"\bfn\s+(\w+)\s*\(")
JAVA_REFERENCE_RE = re.compile(
    r"(?:(com\.alibaba\.easyexcel\.test\.[A-Za-z0-9_.]+(?:Test|Write))|"
    r"([A-Za-z0-9_]+(?:Test|Write)))#([A-Za-z0-9_]+)"
)


def git_root(path: Path) -> Path:
    output = subprocess.check_output(
        ["git", "-C", str(path), "rev-parse", "--show-toplevel"], text=True
    )
    return Path(output.strip()).resolve()


def git_sha(path: Path) -> str:
    return subprocess.check_output(
        ["git", "-C", str(path), "rev-parse", "HEAD"], text=True
    ).strip()


def relative(path: Path, root: Path) -> str:
    return path.resolve().relative_to(root.resolve()).as_posix()


def rust_function_end(lines: list[str], start: int) -> int:
    """Return the exclusive end line of a Rust function for evidence scanning."""
    balance = 0
    found_body = False
    for index in range(start, len(lines)):
        line = re.sub(r'r#+".*?"#+', '""', lines[index])
        line = re.sub(r'"(?:\\.|[^"\\])*"', '""', line)
        line = re.sub(r"'(?:\\.|[^'\\])'", "''", line)
        line = line.split("//", 1)[0]
        opens = line.count("{")
        closes = line.count("}")
        if opens:
            found_body = True
        balance += opens - closes
        if found_body and balance == 0:
            return index + 1
    return len(lines)


def java_inventory(java_root: Path, java_repo: Path) -> list[dict[str, Any]]:
    inventory: list[dict[str, Any]] = []
    for path in sorted(java_root.rglob("*.java")):
        source = path.read_text(encoding="utf-8")
        package = JAVA_PACKAGE_RE.search(source)
        java_class = JAVA_CLASS_RE.search(source)
        if package is None or java_class is None:
            continue
        fqcn = f"{package.group(1)}.{java_class.group(1)}"
        lines = source.splitlines()
        for annotation_index, line in enumerate(lines):
            if JAVA_TEST_RE.match(line) is None:
                continue
            method = None
            method_line = None
            for candidate_index in range(annotation_index + 1, min(len(lines), annotation_index + 12)):
                match = JAVA_METHOD_RE.search(lines[candidate_index])
                if match is not None:
                    method = match.group(1)
                    method_line = candidate_index + 1
                    break
            if method is None or method_line is None:
                raise RuntimeError(f"cannot parse @Test after {path}:{annotation_index + 1}")
            inventory.append(
                {
                    "id": f"{fqcn}#{method}",
                    "class": fqcn,
                    "method": method,
                    "source": {
                        "path": relative(path, java_repo),
                        "annotation_line": annotation_index + 1,
                        "method_line": method_line,
                    },
                }
            )
    duplicate_ids = [item for item, count in Counter(x["id"] for x in inventory).items() if count > 1]
    if duplicate_ids:
        raise RuntimeError(f"duplicate Java test ids: {duplicate_ids}")
    return inventory


def rust_evidence(
    rust_tests: Path,
    rust_repo: Path,
    unique_classes: dict[str, str],
) -> dict[str, list[dict[str, Any]]]:
    evidence: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for path in sorted(rust_tests.rglob("*.rs")):
        lines = path.read_text(encoding="utf-8").splitlines()
        for function_index, line in enumerate(lines):
            function = RUST_FUNCTION_RE.search(line)
            if function is None:
                continue

            prelude: list[str] = []
            cursor = function_index - 1
            while cursor >= 0 and function_index - cursor <= 30:
                stripped = lines[cursor].strip()
                if not stripped or stripped.startswith(("///", "//!", "//", "#[", "#![")):
                    prelude.append(lines[cursor])
                    cursor -= 1
                    continue
                break
            prelude.reverse()
            prelude_text = "\n".join(prelude)
            if "#[test]" not in prelude_text:
                continue

            function_end = rust_function_end(lines, function_index)
            function_text = "\n".join(lines[function_index:function_end])
            ignored = "#[ignore" in prelude_text
            limitation_pattern = re.compile(r"`?PARITY_PARTIAL`?:\s*(.+)")
            limitations = sorted(
                {match.group(1).strip() for match in limitation_pattern.finditer(prelude_text + function_text)}
            )
            coverage = "ignored" if ignored else ("partial" if limitations else "mapped")

            for reference in JAVA_REFERENCE_RE.finditer(prelude_text):
                fqcn = reference.group(1)
                short_class = reference.group(2)
                reference_kind = "fully_qualified"
                if fqcn is None:
                    fqcn = unique_classes.get(short_class or "")
                    reference_kind = "unique_class_name"
                if fqcn is None:
                    continue
                test_id = f"{fqcn}#{reference.group(3)}"
                item = {
                    "path": relative(path, rust_repo),
                    "line": function_index + 1,
                    "function": function.group(1),
                    "reference_kind": reference_kind,
                    "coverage": coverage,
                    "verification": "not_run",
                }
                if limitations:
                    item["limitations"] = limitations
                if item not in evidence[test_id]:
                    evidence[test_id].append(item)
    return evidence


def fixture_set(label: str, root: Path, repo: Path) -> dict[str, Any]:
    files: list[dict[str, Any]] = []
    tree = hashlib.sha256()
    if root.exists():
        for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file()):
            content = path.read_bytes()
            digest = hashlib.sha256(content).hexdigest()
            repo_path = relative(path, repo)
            files.append({"path": repo_path, "bytes": len(content), "sha256": digest})
            tree.update(repo_path.encode("utf-8"))
            tree.update(b"\0")
            tree.update(digest.encode("ascii"))
            tree.update(b"\n")
    return {"label": label, "root": relative(root, repo), "file_count": len(files), "tree_sha256": tree.hexdigest(), "files": files}


def build_manifest(java_root: Path, rust_repo: Path) -> dict[str, Any]:
    java_repo = git_root(java_root)
    rust_repo = git_root(rust_repo)
    rust_tests = rust_repo / "tests/easyexcel-test/tests"
    inventory = java_inventory(java_root, java_repo)

    fqcn_by_short: dict[str, list[str]] = defaultdict(list)
    for item in inventory:
        short = item["class"].rsplit(".", 1)[-1]
        if item["class"] not in fqcn_by_short[short]:
            fqcn_by_short[short].append(item["class"])
    unique_classes = {short: values[0] for short, values in fqcn_by_short.items() if len(values) == 1}
    evidence = rust_evidence(rust_tests, rust_repo, unique_classes)

    counts: Counter[str] = Counter()
    tests: list[dict[str, Any]] = []
    for java_test in inventory:
        rust = evidence.get(java_test["id"], [])
        coverages = {item["coverage"] for item in rust}
        if "mapped" in coverages:
            status = "mapped_unverified"
        elif "partial" in coverages:
            status = "partial_unverified"
        elif "ignored" in coverages:
            status = "ignored"
        else:
            status = "gap"
        counts[status] += 1
        tests.append({**java_test, "status": status, "rust_evidence": rust})

    fixture_sets = [
        fixture_set("java-test-resources", java_repo / "easyexcel-test/src/test/resources", java_repo),
        fixture_set("rust-java-fixtures", rust_tests / "fixtures", rust_repo),
        fixture_set("rust-golden-artifacts", rust_tests / "golden/artifacts", rust_repo),
    ]
    return {
        "schema_version": 1,
        "evidence_level": "static_mapping_only",
        "verification_note": "No Java or Rust test execution is represented by this manifest.",
        "java": {"git_sha": git_sha(java_repo), "test_root": relative(java_root, java_repo)},
        "rust": {"git_sha": git_sha(rust_repo), "test_root": relative(rust_tests, rust_repo)},
        "summary": {"java_tests": len(inventory), **dict(sorted(counts.items()))},
        "fixture_sets": fixture_sets,
        "tests": tests,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--java-root", type=Path, required=True)
    parser.add_argument("--rust-root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, default=Path("docs/source-test-parity.json"))
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    manifest = build_manifest(args.java_root.resolve(), args.rust_root.resolve())
    rendered = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    output = args.output if args.output.is_absolute() else args.rust_root / args.output
    if args.check:
        if not output.exists() or output.read_text(encoding="utf-8") != rendered:
            print(f"stale parity manifest: {output}", file=sys.stderr)
            return 1
        print(json.dumps(manifest["summary"], ensure_ascii=False, sort_keys=True))
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(rendered, encoding="utf-8")
    print(json.dumps(manifest["summary"], ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
