#!/usr/bin/env python3
"""Compute crates.io publish order for the easyexcel-rust workspace.

The workspace uses `path = "...", version = "..."` dependencies, so cargo
resolves internal dependencies from crates.io when publishing. A crate can
only be published after every internal dependency it references. This script
derives that order from `cargo metadata` instead of maintaining a hand-written
array, so adding or removing a crate never desynchronizes the release
workflow.

Usage:
    python3 scripts/publish-order.py             # print publish order, one crate per line
    python3 scripts/publish-order.py --roots     # print crates with no internal deps
    python3 scripts/publish-order.py --json      # {"order": [...], "roots": [...]}

`roots` are the only crates safe to run `cargo publish --dry-run` on before
anything has been published: dry-run still validates the full package, and a
crate with unpublished internal dependencies cannot complete that validation.
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parents[1]


def metadata() -> dict:
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=WORKSPACE_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(out.stdout)


def internal_deps(pkg: dict, pkg_names: set[str]) -> set[str]:
    """Names of workspace crates this package depends on (path deps inside crates/)."""
    deps = set()
    for dep in pkg.get("dependencies", []):
        dep_path = dep.get("path")
        if dep_path is None:
            continue
        # Only path deps that resolve inside crates/ are workspace internals;
        # external path deps (if any ever appear) must not gate the order.
        if WORKSPACE_ROOT.joinpath("crates", dep["name"]).resolve() == Path(dep_path).resolve():
            deps.add(dep["name"])
    return deps & pkg_names


def compute_order(packages: list[dict]) -> list[str]:
    pkg_names = {p["name"] for p in packages}
    # Published nodes = everything whose manifest lives under crates/ (the
    # facade `easyexcel` has no hyphen suffix, so name prefixes are unusable).
    # xtask / examples / benches are workspace members but never published,
    # so they cannot be gating edges.
    crates_dir = WORKSPACE_ROOT / "crates"
    nodes = {
        p["name"] for p in packages
        if Path(p["manifest_path"]).is_relative_to(crates_dir)
    }
    edges = {name: set() for name in nodes}
    rev = {name: set() for name in nodes}
    for pkg in packages:
        name = pkg["name"]
        if name not in nodes:
            continue
        for dep in internal_deps(pkg, pkg_names):
            if dep in nodes and dep != name:
                edges[name].add(dep)  # name must come after dep
                rev[dep].add(name)
    # Kahn topological sort. Deterministic: sort candidates at each step.
    ready = sorted(n for n in nodes if not edges[n])
    order: list[str] = []
    while ready:
        name = ready.pop(0)
        order.append(name)
        for downstream in sorted(rev[name]):
            edges[downstream].discard(name)
            if not edges[downstream]:
                ready.append(downstream)
    if len(order) != len(nodes):
        leftover = sorted(n for n in nodes if n not in order)
        sys.exit(f"cycle detected among unpublished crates: {leftover}")
    return order


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--roots", action="store_true", help="print crates with no internal deps")
    parser.add_argument("--json", action="store_true", help="machine-readable output")
    args = parser.parse_args()

    packages = metadata()["packages"]
    order = compute_order(packages)
    pkg_names = {p["name"] for p in packages}
    by_name = {p["name"]: p for p in packages}
    roots = [
        name for name in order
        if not (internal_deps(by_name[name], pkg_names) - {name})
    ]

    if args.json:
        print(json.dumps({"order": order, "roots": roots}, indent=2))
        return 0
    if args.roots:
        print("\n".join(roots))
    else:
        print("\n".join(order))
    return 0


if __name__ == "__main__":
    sys.exit(main())
