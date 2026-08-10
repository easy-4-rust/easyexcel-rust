#!/usr/bin/env python3
"""Verify Java wrapper JARs contain only explicitly excluded packaging markers."""

from __future__ import annotations

import argparse
import json
import zipfile
from pathlib import Path


def class_names(jar: Path) -> list[str]:
    """Return deterministic non-metadata class names from a JAR."""
    with zipfile.ZipFile(jar) as archive:
        return sorted(
            name[:-6].replace("/", ".")
            for name in archive.namelist()
            if name.endswith(".class")
            and not name.startswith("META-INF/versions/")
            and not name.endswith(("module-info.class", "package-info.class"))
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--artifact",
        action="append",
        nargs=2,
        metavar=("JAR", "EXPECTED_CLASS"),
        required=True,
        help="wrapper JAR and its only allowed packaging marker class",
    )
    parser.add_argument(
        "--allow-prefix",
        action="append",
        default=[],
        help="relocated dependency class prefix excluded from EasyExcel public API scope",
    )
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    records = []
    for raw_jar, expected_class in args.artifact:
        jar = Path(raw_jar)
        if not jar.is_file():
            parser.error(f"packaging artifact does not exist: {jar}")
        classes = class_names(jar)
        actual = [
            name
            for name in classes
            if not any(name.startswith(prefix) for prefix in args.allow_prefix)
        ]
        if actual != [expected_class]:
            parser.error(
                f"packaging artifact public scope changed for {jar}: "
                f"expected only {expected_class}, found {actual}"
            )
        records.append(
            {
                "artifact": jar.name,
                "classification": "java_packaging_marker_not_in_core_api_denominator",
                "class": expected_class,
                "relocated_dependency_classes": len(classes) - len(actual),
            }
        )

    payload = {
        "schema_version": 1,
        "authority": "easyexcel-v4.0.3-packaging-artifacts",
        "artifacts": sorted(records, key=lambda record: record["artifact"]),
    }
    rendered = json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
