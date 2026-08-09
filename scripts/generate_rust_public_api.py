#!/usr/bin/env python3
"""Generate deterministic cargo-public-api snapshots for all published workspace crates."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


EXPECTED_CARGO_PUBLIC_API_VERSION = "cargo-public-api 0.52.0"


def output(command: list[str], cwd: Path) -> str:
    process = subprocess.run(command, cwd=cwd, text=True, capture_output=True)
    if process.returncode != 0:
        raise RuntimeError(
            f"command failed ({process.returncode}): {' '.join(command)}\n{process.stderr.strip()}"
        )
    return process.stdout.strip()


def git_dirty(repo: Path) -> bool:
    return bool(output(["git", "status", "--porcelain", "--untracked-files=normal"], repo))


def published_workspace_packages(repo: Path) -> list[str]:
    """发现 workspace 中可发布 crate，避免只快照 facade 而漏掉重导出来源。"""
    metadata = json.loads(output(["cargo", "metadata", "--no-deps", "--format-version", "1"], repo))
    workspace_members = set(metadata["workspace_members"])
    return sorted(
        package["name"]
        for package in metadata["packages"]
        if package["id"] in workspace_members and package.get("publish", ["crates-io"]) != []
    )


def api_kind(signature: str) -> str:
    tokens = signature.split()
    if tokens[:2] == ["pub", "mod"]:
        return "module"
    if tokens[:2] == ["pub", "use"]:
        return "reexport"
    if tokens[:2] == ["pub", "variant"]:
        return "variant"
    if tokens[:2] == ["pub", "field"]:
        return "field"
    # cargo-public-api 当前对 enum variant 输出为
    # `pub crate::path::Enum::Variant`（可能带 tuple/struct payload），没有
    # 固定 `variant` 关键字。末段 PascalCase 可与小写 struct field 区分。
    if re.match(
        r"^pub\s+(?:[A-Za-z_][A-Za-z0-9_]*::)+[A-Z][A-Za-z0-9_]*(?:\(|\s*\{|$)",
        signature,
    ):
        return "variant"
    if " fn " in f" {signature} ":
        return "function"
    if tokens and tokens[0] == "impl":
        return "impl"
    for kind in ("struct", "enum", "trait", "type", "const", "static", "union"):
        if kind in tokens[:3]:
            return kind
    return "other"


def run_public_api(repo: Path, package: str, all_features: bool) -> dict[str, Any]:
    command = ["cargo", "public-api", "-p", package, "-sss", "--color", "never"]
    if all_features:
        command.append("--all-features")
    raw = output(command, repo)
    signatures = sorted({line.strip() for line in raw.splitlines() if line.strip()})
    items = [
        {
            "id": f"{package}:{hashlib.sha256(signature.encode('utf-8')).hexdigest()[:20]}",
            "kind": api_kind(signature),
            "signature": signature,
        }
        for signature in signatures
    ]
    return {"mode": "all_features" if all_features else "default", "count": len(items), "items": items}


def build_manifest(
    repo: Path, packages: list[str], extractor_version: str
) -> dict[str, Any]:
    package_items = []
    for package in sorted(set(packages)):
        package_items.append(
            {
                "name": package,
                "snapshots": [run_public_api(repo, package, False), run_public_api(repo, package, True)],
            }
        )
    return {
        "schema_version": 1,
        "artifact": "easyexcel-rust-public-api",
        "rust_repo": {
            "name": repo.resolve().name,
            "git_sha": output(["git", "rev-parse", "HEAD"], repo),
            "dirty": git_dirty(repo),
        },
        "extractor": {
            "command": "cargo public-api -sss --color never",
            "version": extractor_version,
        },
        "summary": {
            "packages": len(package_items),
            "default_items": sum(item["snapshots"][0]["count"] for item in package_items),
            "all_feature_items": sum(item["snapshots"][1]["count"] for item in package_items),
        },
        "packages": package_items,
    }


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust-root", type=Path, required=True)
    parser.add_argument("--package", action="append", default=[])
    parser.add_argument(
        "--allow-partial",
        action="store_true",
        help="允许仅生成 --package 指定 crate；权威 Java parity 快照不得使用",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    parser.add_argument(
        "--expected-version",
        default=EXPECTED_CARGO_PUBLIC_API_VERSION,
        help="权威快照使用的 cargo-public-api 完整版本字符串",
    )
    args = parser.parse_args()
    if git_dirty(args.rust_root) and not args.allow_partial:
        parser.error("authoritative Rust public API extraction requires a clean worktree")
    extractor_version = output(["cargo", "public-api", "--version"], args.rust_root)
    if extractor_version != args.expected_version:
        parser.error(
            "cargo-public-api version mismatch: "
            f"expected {args.expected_version!r}, got {extractor_version!r}"
        )
    published_packages = published_workspace_packages(args.rust_root)
    packages = sorted(set(args.package)) if args.package else published_packages
    unknown_packages = sorted(set(packages) - set(published_packages))
    if unknown_packages:
        parser.error(
            "--package contains non-published or non-workspace crates: "
            + ", ".join(unknown_packages)
        )
    missing_packages = sorted(set(published_packages) - set(packages))
    if missing_packages and not args.allow_partial:
        parser.error(
            "authoritative Rust public API snapshot must include every published workspace crate; "
            "missing: " + ", ".join(missing_packages)
        )
    manifest = build_manifest(args.rust_root, packages, extractor_version)
    manifest["scope"] = {
        "authoritative": not missing_packages,
        "published_workspace_packages": published_packages,
        "included_packages": packages,
        "missing_packages": missing_packages,
    }
    rendered = canonical_json(manifest)
    if args.check:
        if not args.output.is_file() or args.output.read_text(encoding="utf-8") != rendered:
            print(f"stale Rust public API snapshot: {args.output}", file=sys.stderr)
            return 1
        print(f"Rust public API snapshot is current: {manifest['summary']}")
        return 0
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(rendered, encoding="utf-8")
    print(f"wrote {args.output}: {manifest['summary']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
