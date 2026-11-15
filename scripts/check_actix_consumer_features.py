#!/usr/bin/env python3
"""Verify the published easyexcel-actix package does not enable legacy Actix HTTP/2.

The workspace's local [patch.crates-io] is intentionally not visible to this
consumer fixture. The test packages the normalized crate, extracts it outside
the workspace, and resolves it as an ordinary downstream Cargo dependency.
"""

from __future__ import annotations

import json
import re
import subprocess
import tarfile
import tempfile
import tomllib
from pathlib import Path


def run(
    command: list[str], cwd: Path, *, print_stdout: bool = True
) -> subprocess.CompletedProcess[str]:
    print("+", " ".join(command), flush=True)
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    if result.stdout and print_stdout:
        print(result.stdout, end="", flush=True)
    if result.stderr:
        print(result.stderr, end="", flush=True)
    if result.returncode:
        raise subprocess.CalledProcessError(
            result.returncode, command, output=result.stdout, stderr=result.stderr
        )
    return result


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    version = workspace["workspace"]["package"]["version"]
    crate_name = f"easyexcel-actix-{version}"

    run(
        [
            "cargo",
            "package",
            "-p",
            "easyexcel-actix",
            "--allow-dirty",
            "--no-verify",
        ],
        cwd=root,
    )
    metadata = json.loads(
        run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=root,
            print_stdout=False,
        ).stdout
    )
    archive = Path(metadata["target_directory"]) / "package" / f"{crate_name}.crate"
    if not archive.is_file():
        raise RuntimeError(f"cargo package did not create {archive}")

    with tempfile.TemporaryDirectory(prefix="easyexcel-actix-consumer-") as temporary:
        temporary_root = Path(temporary)
        extracted = temporary_root / "package"
        extracted.mkdir()
        with tarfile.open(archive, "r:gz") as package:
            root_resolved = extracted.resolve()
            for member in package.getmembers():
                destination = (extracted / member.name).resolve()
                if not destination.is_relative_to(root_resolved):
                    raise RuntimeError(f"unsafe path in Cargo package: {member.name}")
                package.extract(member, extracted, filter="data")

        package_root = extracted / crate_name
        normalized_manifest = tomllib.loads(
            (package_root / "Cargo.toml").read_text(encoding="utf-8")
        )
        actix = normalized_manifest["dependencies"]["actix-web"]
        if actix.get("default-features") is not False:
            raise AssertionError("published easyexcel-actix must disable Actix default features")
        if "http2" in actix.get("features", []):
            raise AssertionError("published easyexcel-actix must not enable legacy Actix http2")

        consumer = temporary_root / "consumer"
        (consumer / "src").mkdir(parents=True)
        package_path = json.dumps(package_root.as_posix())
        (consumer / "Cargo.toml").write_text(
            "[package]\n"
            'name = "easyexcel-actix-consumer-audit"\n'
            'version = "0.1.0"\n'
            'edition = "2024"\n\n'
            "[dependencies]\n"
            f"easyexcel-actix = {{ path = {package_path} }}\n",
            encoding="utf-8",
        )
        (consumer / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")

        dependency_tree = run(
            [
                "cargo",
                "tree",
                "--manifest-path",
                str(consumer / "Cargo.toml"),
                "--prefix",
                "none",
            ],
            cwd=consumer,
            print_stdout=False,
        ).stdout
        if re.search(r"(?m)^h2 v0\.3\.", dependency_tree):
            raise AssertionError(
                "published easyexcel-actix unexpectedly introduces legacy h2 0.3:\n"
                + dependency_tree
            )

        print(
            f"PASS: {crate_name} downstream dependency graph contains no h2 0.3.x",
            flush=True,
        )


if __name__ == "__main__":
    main()
