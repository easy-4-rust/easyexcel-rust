#!/usr/bin/env python3
"""Build and attest the exact Java/Rust runners used by release benchmarks."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

import run_matrix


def run(command: list[str], cwd: Path, environment: dict[str, str] | None = None) -> None:
    completed = subprocess.run(command, cwd=cwd, check=False, env=environment)
    if completed.returncode != 0:
        raise RuntimeError(
            f"release runner build failed ({completed.returncode}): {' '.join(command)}"
        )


def require_clean(repository: Path, label: str) -> str:
    dirty, source_sha256 = run_matrix.repository_fingerprint(repository)
    if dirty is not False or source_sha256 is None:
        raise RuntimeError(f"{label} release runner build requires a clean worktree")
    return source_sha256


def resolve_executable(command: Path | str) -> Path:
    value = str(command)
    resolved = shutil.which(value)
    if resolved is None:
        candidate = Path(value).expanduser()
        if not candidate.is_file():
            raise RuntimeError(f"release runtime executable does not exist: {value}")
        resolved = str(candidate)
    path = Path(resolved).resolve()
    if not path.is_file() or not os.access(path, os.X_OK):
        raise RuntimeError(f"release runtime is not executable: {path}")
    return path


def java_runtime(java_bin: Path) -> tuple[str, Path]:
    completed = subprocess.run(
        [str(java_bin), "-XshowSettings:properties", "-version"],
        check=False,
        capture_output=True,
        text=True,
    )
    output = "\n".join((completed.stdout, completed.stderr))
    home_match = re.search(r"^\s*java\.home\s*=\s*(.+?)\s*$", output, re.MULTILINE)
    if completed.returncode != 0 or home_match is None:
        raise RuntimeError(f"cannot determine Java home/version from {java_bin}")
    version = next(
        (line.strip() for line in output.splitlines() if "version" in line.lower()),
        "",
    )
    if not version:
        raise RuntimeError(f"cannot determine Java version from {java_bin}")
    return version, Path(home_match.group(1)).resolve()


def cargo_release_binary(rust_repo: Path) -> Path:
    metadata = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
        cwd=rust_repo,
        check=False,
        capture_output=True,
        text=True,
    )
    if metadata.returncode != 0:
        raise RuntimeError("cannot resolve Cargo target directory for release runner")
    target_directory = Path(json.loads(metadata.stdout)["target_directory"]).resolve()
    suffix = ".exe" if os.name == "nt" else ""
    return target_directory / "release" / f"easyexcel-benchmark-runner{suffix}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust-repo", type=Path, required=True)
    parser.add_argument("--java-repo", type=Path, required=True)
    parser.add_argument("--rust-bin", type=Path, required=True)
    parser.add_argument("--java-bin", type=Path, default=Path("java"))
    parser.add_argument("--java-classpath", required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()

    rust_repo = arguments.rust_repo.resolve()
    java_repo = arguments.java_repo.resolve()
    rust_source_before = require_clean(rust_repo, "Rust")
    java_source_before = require_clean(java_repo, "Java")

    rustc_bin = resolve_executable("rustc")
    rustc = subprocess.run(
        [str(rustc_bin), "--version"], check=False, capture_output=True, text=True
    )
    if rustc.returncode != 0 or not rustc.stdout.strip().startswith("rustc "):
        raise RuntimeError("cannot determine rustc version for release runner")
    rust_environment = os.environ.copy()
    rust_environment["EASYEXCEL_GIT_SHA"] = run_matrix.git_sha(rust_repo)
    rust_environment["EASYEXCEL_RUSTC"] = rustc.stdout.strip().removeprefix("rustc ")

    java_bin = resolve_executable(arguments.java_bin)
    java_version, java_home = java_runtime(java_bin)
    java_environment = os.environ.copy()
    java_environment["JAVA_HOME"] = str(java_home)

    run(
        ["cargo", "build", "--locked", "--release", "-p", "easyexcel-benchmark-runner"],
        rust_repo,
        rust_environment,
    )
    maven = java_repo / "mvnw"
    maven_command = str(maven) if maven.is_file() else "mvn"
    run(
        [
            maven_command,
            "-pl",
            "easyexcel-test",
            "-am",
            "-DskipTests",
            "test-compile",
        ],
        java_repo,
        java_environment,
    )

    rust_source_after = require_clean(rust_repo, "Rust")
    java_source_after = require_clean(java_repo, "Java")
    if rust_source_before != rust_source_after or java_source_before != java_source_after:
        raise RuntimeError("repository source fingerprint changed while building release runners")

    rust_bin = arguments.rust_bin.resolve()
    expected_rust_bin = cargo_release_binary(rust_repo)
    if rust_bin != expected_rust_bin:
        raise RuntimeError(
            f"--rust-bin must be the Cargo release artifact built in this run: {expected_rust_bin}"
        )
    if not rust_bin.is_file() or not os.access(rust_bin, os.X_OK):
        raise RuntimeError(f"release Rust runner is not executable after build: {rust_bin}")
    classpath = [
        Path(item).resolve()
        for item in arguments.java_classpath.split(os.pathsep)
        if item
    ]
    expected_test_classes = (java_repo / "easyexcel-test/target/test-classes").resolve()
    if not classpath or classpath[0] != expected_test_classes:
        raise RuntimeError(
            "Java classpath must begin with easyexcel-test/target/test-classes from --java-repo"
        )
    runner_class = expected_test_classes / Path(
        "com/alibaba/easyexcel/test/benchmark/EasyExcelBenchmarkRunner.class"
    )
    if not runner_class.is_file():
        raise RuntimeError(f"Java benchmark runner was not built: {runner_class}")

    manifest = {
        "schema_version": 2,
        "artifact": "easyexcel-release-benchmark-runners",
        "rust": {
            "repo": str(rust_repo),
            "git_sha": run_matrix.git_sha(rust_repo),
            "source_sha256": rust_source_after,
            "binary": str(rust_bin),
            "binary_sha256": run_matrix.path_sha256(rust_bin),
            "rustc": str(rustc_bin),
            "rustc_sha256": run_matrix.path_sha256(rustc_bin),
            "rustc_version": rustc.stdout.strip(),
        },
        "java": {
            "repo": str(java_repo),
            "git_sha": run_matrix.git_sha(java_repo),
            "source_sha256": java_source_after,
            "runner_class": str(runner_class),
            "runner_class_sha256": run_matrix.path_sha256(runner_class),
            "java_bin": str(java_bin),
            "java_bin_sha256": run_matrix.path_sha256(java_bin),
            "java_home": str(java_home),
            "java_version": java_version,
            "classpath": [
                {"path": str(path), "sha256": run_matrix.path_sha256(path)}
                for path in classpath
            ],
        },
    }
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(arguments.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
