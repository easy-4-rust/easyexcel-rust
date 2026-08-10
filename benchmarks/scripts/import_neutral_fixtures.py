#!/usr/bin/env python3
"""Import independently generated benchmark inputs into the committed manifest."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import shutil
import tempfile


ROOT = Path(__file__).resolve().parents[2]
FIXTURE_ROOT = ROOT / "benchmarks" / "fixtures"
SPEC = ROOT / "benchmarks" / "spec" / "benchmark-suite-v1.json"


def sha256(path: Path) -> str:
    """返回文件的 SHA-256。"""
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_source(path: Path, file_format: str) -> None:
    """在导入前拒绝空文件、扩展名漂移和明显错误的容器。"""
    if not path.is_file() or path.stat().st_size == 0:
        raise ValueError(f"neutral {file_format} source is missing or empty: {path}")
    if path.suffix.lower() != f".{file_format}":
        raise ValueError(f"neutral {file_format} source has the wrong extension: {path}")
    with path.open("rb") as source:
        prefix = source.read(8)
    if file_format == "xlsx" and prefix[:4] != b"PK\x03\x04":
        raise ValueError(f"neutral XLSX source is not an OOXML ZIP: {path}")
    if file_format == "xls" and prefix != bytes.fromhex("d0cf11e0a1b11ae1"):
        raise ValueError(f"neutral XLS source is not an OLE2 container: {path}")
    if file_format == "csv" and b"\x00" in prefix:
        raise ValueError(f"neutral CSV source appears to be binary: {path}")


def require_independent_generator(name: str) -> None:
    """防止把任一被测 EasyExcel runner 重新标记成 neutral。"""
    normalized = re.sub(r"[^a-z0-9]+", "", name.lower())
    if not normalized or "easyexcel" in normalized:
        raise ValueError(
            "neutral fixtures require an independent generator, not an EasyExcel runner"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--xlsx", type=Path, required=True)
    parser.add_argument("--xls", type=Path, required=True)
    parser.add_argument("--csv", type=Path, required=True)
    parser.add_argument("--rows", type=int, default=1_000_000)
    parser.add_argument("--generator-name", required=True)
    parser.add_argument("--generator-version", required=True)
    parser.add_argument("--generator-command", required=True)
    parser.add_argument(
        "--generator-artifact",
        type=Path,
        required=True,
        help="independent generator executable or source archive to bind by SHA-256",
    )
    parser.add_argument("--reviewer", required=True)
    parser.add_argument("--review-notes", required=True)
    parser.add_argument(
        "--replace",
        action="store_true",
        help="replace an already populated committed fixture set",
    )
    arguments = parser.parse_args()

    if arguments.rows <= 0:
        raise ValueError("--rows must be positive")
    require_independent_generator(arguments.generator_name)
    generator_artifact = arguments.generator_artifact.resolve()
    if not generator_artifact.is_file():
        raise ValueError(f"generator artifact does not exist: {generator_artifact}")
    sources = {
        "xlsx": arguments.xlsx.resolve(),
        "xls": arguments.xls.resolve(),
        "csv": arguments.csv.resolve(),
    }
    if len(set(sources.values())) != len(sources):
        raise ValueError("each neutral format must come from a distinct source path")
    for file_format, source in sources.items():
        validate_source(source, file_format)

    existing_manifest = FIXTURE_ROOT / "manifest.json"
    if existing_manifest.is_file():
        existing = json.loads(existing_manifest.read_text(encoding="utf-8"))
        if existing.get("fixtures") and not arguments.replace:
            raise ValueError("neutral fixture manifest is populated; pass --replace explicitly")

    FIXTURE_ROOT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="easyexcel-neutral-fixtures-") as directory:
        staging = Path(directory)
        entries = []
        for file_format, source in sorted(sources.items()):
            temporary = staging / f"source.{file_format}"
            shutil.copyfile(source, temporary)
            content_sha256 = sha256(temporary)
            identity = f"neutral-{file_format}-{arguments.rows}-{content_sha256[:12]}"
            name = f"{identity}.{file_format}"
            destination = staging / name
            temporary.rename(destination)
            entries.append(
                {
                    "id": identity,
                    "format": file_format,
                    "rows": arguments.rows,
                    "path": name,
                    "sha256": content_sha256,
                }
            )

        manifest = {
            "schema_version": 2,
            "benchmark_spec_sha256": sha256(SPEC),
            "generator": {
                "kind": "independent",
                "name": arguments.generator_name,
                "version": arguments.generator_version,
                "command": arguments.generator_command,
                "artifact_name": generator_artifact.name,
                "artifact_sha256": sha256(generator_artifact),
            },
            "review": {
                "status": "approved",
                "reviewer": arguments.reviewer,
                "reviewed_at": datetime.now(timezone.utc).isoformat(),
                "notes": arguments.review_notes,
            },
            "fixtures": entries,
        }
        staged_manifest = staging / "manifest.json"
        staged_manifest.write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

        for entry in entries:
            destination = FIXTURE_ROOT / entry["path"]
            if destination.exists() and sha256(destination) != entry["sha256"]:
                raise ValueError(f"neutral fixture hash-prefix collision: {destination}")
            if not destination.exists():
                shutil.copyfile(staging / entry["path"], destination)
        shutil.copyfile(staged_manifest, existing_manifest)

    print(existing_manifest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
