#!/usr/bin/env python3
"""Extract the public Java API of EasyExcel from release JARs using javap."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path
from typing import Any


DECLARATION_RE = re.compile(r"^(?:public|protected)\s+.*\b(class|interface|enum|record)\s+([^\s<{]+)")
DESCRIPTOR_RE = re.compile(r"^\s*descriptor:\s*(\S+)\s*$")
FLAGS_RE = re.compile(r"^\s*flags:\s*\([^)]*\)\s*(.*)$")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def command_output(command: list[str], cwd: Path | None = None) -> str:
    return subprocess.check_output(command, cwd=cwd, text=True, stderr=subprocess.STDOUT).strip()


def git_sha(path: Path) -> str:
    return command_output(["git", "-C", str(path), "rev-parse", "HEAD"])


def git_exact_tag(path: Path) -> str:
    return command_output(["git", "-C", str(path), "describe", "--tags", "--exact-match", "HEAD"])


def git_dirty(path: Path) -> bool:
    return bool(
        command_output(
            ["git", "-C", str(path), "status", "--porcelain", "--untracked-files=normal"]
        )
    )


def class_names(jars: list[Path]) -> list[str]:
    names: set[str] = set()
    for jar in jars:
        with zipfile.ZipFile(jar) as archive:
            for name in archive.namelist():
                if not name.endswith(".class") or name.startswith("META-INF/versions/"):
                    continue
                fqcn = name[:-6].replace("/", ".")
                if fqcn.endswith(("module-info", "package-info")):
                    continue
                names.add(fqcn)
    return sorted(names)


def split_javap_output(output: str) -> list[list[str]]:
    blocks: list[list[str]] = []
    current: list[str] = []
    for line in output.splitlines():
        # verbose javap 在 `Compiled from` 前先输出 Classfile；以 Classfile
        # 分块才能避免把下一类的头部粘到上一类末尾。
        if line.startswith("Classfile ") and current:
            blocks.append(current)
            current = []
        current.append(line)
    if current:
        blocks.append(current)
    return blocks


def member_kind(declaration: str, fqcn: str) -> tuple[str, str]:
    before_args = declaration.split("(", 1)[0].rstrip()
    if "(" not in declaration:
        field_declaration = before_args.split("=", 1)[0].rstrip().rstrip(";")
        name = field_declaration.split()[-1]
        return "field", name
    name = before_args.split()[-1]
    simple_name = fqcn.rsplit(".", 1)[-1]
    if name in {fqcn, simple_name} or name.endswith("." + simple_name):
        return "constructor", "<init>"
    return "method", name


def parse_class(block: list[str]) -> tuple[dict[str, Any], list[dict[str, Any]]] | None:
    type_index = -1
    declaration = ""
    type_kind = ""
    fqcn = ""
    for index, raw in enumerate(block):
        candidate = raw.strip()
        match = DECLARATION_RE.match(candidate)
        if match:
            type_index = index
            declaration = candidate
            type_kind = match.group(1)
            fqcn = match.group(2)
            break
    if type_index < 0:
        return None

    type_item = {
        "id": fqcn,
        "kind": "type",
        "type_kind": type_kind,
        "owner": fqcn,
        "declaration": declaration,
        "flags": [],
        "synthetic": False,
        "bridge": False,
    }
    members: list[dict[str, Any]] = []
    pending: str | None = None
    flags_target: dict[str, Any] | None = type_item
    for raw in block[type_index + 1 :]:
        stripped = raw.strip()
        if not stripped or stripped == "}":
            continue
        flags = FLAGS_RE.match(raw)
        if flags and flags_target is not None:
            values = [value.strip() for value in flags.group(1).split(",") if value.strip()]
            flags_target["flags"] = values
            flags_target["synthetic"] = "ACC_SYNTHETIC" in values
            flags_target["bridge"] = "ACC_BRIDGE" in values
            flags_target = None
            continue
        descriptor = DESCRIPTOR_RE.match(raw)
        if descriptor and pending is not None:
            jvm_descriptor = descriptor.group(1)
            kind, name = member_kind(pending, fqcn)
            separator = "#FIELD:" if kind == "field" else "#"
            member = {
                "id": f"{fqcn}{separator}{name}{jvm_descriptor}",
                "kind": kind,
                "owner": fqcn,
                "name": name,
                "descriptor": jvm_descriptor,
                "declaration": pending,
                "flags": [],
                "synthetic": False,
                "bridge": False,
            }
            members.append(member)
            flags_target = member
            pending = None
            continue
        if stripped.startswith(("public ", "protected ")):
            pending = stripped
            flags_target = None
    return type_item, members


def extract(javap: Path, jars: list[Path], java_repo: Path, batch_size: int) -> dict[str, Any]:
    classes = class_names(jars)
    classpath = ":".join(str(path.resolve()) for path in jars)
    types: list[dict[str, Any]] = []
    members: list[dict[str, Any]] = []
    failures: list[dict[str, str]] = []
    for start in range(0, len(classes), batch_size):
        batch = classes[start : start + batch_size]
        command = [
            str(javap),
            "-public",
            "-s",
            "-constants",
            "-v",
            "-classpath",
            classpath,
            *batch,
        ]
        process = subprocess.run(command, text=True, capture_output=True)
        if process.returncode != 0:
            failures.append({"classes": ",".join(batch), "stderr": process.stderr.strip()})
            continue
        for block in split_javap_output(process.stdout):
            parsed = parse_class(block)
            if parsed is not None:
                type_item, class_members = parsed
                types.append(type_item)
                members.extend(class_members)
    if failures:
        raise RuntimeError(f"javap failed for {len(failures)} batch(es): {failures[0]}")

    types.sort(key=lambda item: item["id"])
    members.sort(key=lambda item: item["id"])
    ids = [item["id"] for item in [*types, *members]]
    if len(ids) != len(set(ids)):
        duplicates = sorted(item for item in set(ids) if ids.count(item) > 1)
        raise RuntimeError(f"duplicate public API ids: {duplicates[:10]}")
    return {
        "schema_version": 1,
        "artifact": "easyexcel-java-public-api",
        "easyexcel_version": "4.0.3",
        "java_repo": {
            "name": java_repo.resolve().name,
            "git_sha": git_sha(java_repo),
            "exact_tag": git_exact_tag(java_repo),
            "dirty": git_dirty(java_repo),
        },
        "extractor": {
            "command": "javap -public -s -constants -v",
            "version": command_output([str(javap), "-version"]),
        },
        "jars": [
            {"file": path.name, "bytes": path.stat().st_size, "sha256": sha256_file(path)}
            for path in jars
        ],
        "summary": {
            "class_files": len(classes),
            "public_types": len(types),
            "public_members": len(members),
            "public_api_items": len(types) + len(members),
        },
        "types": types,
        "members": members,
    }


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def markdown(manifest: dict[str, Any]) -> str:
    """生成人工可审阅但仍保持确定性的 public API 清单。"""
    summary = manifest["summary"]
    lines = [
        "# EasyExcel 4.0.3 Java Public API",
        "",
        f"- Public types: {summary['public_types']}",
        f"- Public members: {summary['public_members']}",
        f"- Total API items: {summary['public_api_items']}",
        "- Authority: JVM descriptors from `javap -public -s -constants -v`",
        "",
        "## Artifacts",
        "",
    ]
    for jar in manifest["jars"]:
        lines.append(f"- `{jar['file']}` — `{jar['sha256']}`")
    lines.extend(["", "## Types", ""])
    for item in manifest["types"]:
        suffix = " synthetic" if item.get("synthetic") else ""
        lines.append(f"- `{item['id']}` ({item['type_kind']}{suffix})")
    lines.extend(["", "## Members", ""])
    for item in manifest["members"]:
        flags = []
        if item.get("bridge"):
            flags.append("bridge")
        if item.get("synthetic"):
            flags.append("synthetic")
        suffix = f" ({', '.join(flags)})" if flags else ""
        lines.append(f"- `{item['id']}`{suffix}")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--java-root", type=Path, required=True)
    parser.add_argument("--jar", type=Path, action="append", required=True)
    parser.add_argument("--javap", type=Path, default=Path("javap"))
    parser.add_argument("--batch-size", type=int, default=64)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--markdown-output", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--expected-tag", default="v4.0.3")
    args = parser.parse_args()
    try:
        actual_tag = git_exact_tag(args.java_root)
    except subprocess.CalledProcessError as error:
        parser.error(f"Java repository HEAD is not on an exact tag: {error.output.strip()}")
    if actual_tag != args.expected_tag:
        parser.error(
            f"Java repository tag mismatch: expected {args.expected_tag!r}, got {actual_tag!r}"
        )
    if git_dirty(args.java_root):
        parser.error("Java public API extraction requires a clean source worktree")
    for jar in args.jar:
        if not jar.is_file():
            parser.error(f"JAR does not exist: {jar}")
    manifest = extract(args.javap, args.jar, args.java_root, args.batch_size)
    rendered = canonical_json(manifest)
    rendered_markdown = markdown(manifest)
    if args.check:
        stale = not args.output.is_file() or args.output.read_text(encoding="utf-8") != rendered
        if args.markdown_output is not None:
            stale = stale or not args.markdown_output.is_file() or args.markdown_output.read_text(
                encoding="utf-8"
            ) != rendered_markdown
        if stale:
            print(f"stale Java public API snapshot: {args.output}", file=sys.stderr)
            return 1
        print(f"Java public API snapshot is current: {manifest['summary']}")
        return 0
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(rendered, encoding="utf-8")
    if args.markdown_output is not None:
        args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_output.write_text(rendered_markdown, encoding="utf-8")
    print(f"wrote {args.output}: {manifest['summary']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
