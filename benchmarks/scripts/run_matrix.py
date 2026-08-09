#!/usr/bin/env python3
"""Execute the shared Java/Rust benchmark matrix using prebuilt runners."""

from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
MEASURE = ROOT / "benchmarks" / "scripts" / "measure_process.py"
FIXTURE_HASHES: dict[Path, str] = {}


def fixture_sha256(path: Path) -> str:
    """Hash an immutable generated fixture once per orchestration process."""
    resolved = path.resolve()
    if resolved not in FIXTURE_HASHES:
        FIXTURE_HASHES[resolved] = hashlib.sha256(resolved.read_bytes()).hexdigest()
    return FIXTURE_HASHES[resolved]


def runner_command(
    implementation: str,
    arguments: argparse.Namespace,
    scenario: dict[str, Any],
    rows: int,
    workers: int,
    input_path: Path | None,
    output_path: Path | None,
    temperature: str = "cold",
    warmups: int = 0,
    gc_log: Path | None = None,
    temp_dir: Path | None = None,
    internal_map_work_factor: int | None = None,
    internal_map_queue_capacity: int | None = None,
) -> list[str]:
    common = [
        "--spec", str(arguments.spec),
        "--scenario", scenario["id"],
        "--rows", str(rows),
        "--workers", str(workers),
        "--temperature", temperature,
        "--warmups", str(warmups),
    ]
    if input_path:
        common.extend(["--input", str(input_path)])
    if output_path:
        common.extend(["--output", str(output_path)])
    if internal_map_work_factor is not None:
        if internal_map_queue_capacity is None:
            raise ValueError("internal parallel-map requires a queue capacity")
        common.extend(
            [
                "--internal-map-work-factor",
                str(internal_map_work_factor),
                "--internal-map-queue-capacity",
                str(internal_map_queue_capacity),
            ]
        )
    if implementation == "rust":
        return [str(arguments.rust_bin), *common]
    java_options = [
        str(arguments.java_bin),
        f"-Xms{arguments.java_xms}",
        f"-Xmx{arguments.java_xmx}",
        "-XX:+UseG1GC",
        "-Duser.timezone=UTC",
        "-Duser.language=en",
        "-Duser.country=US",
        f"-Deasyexcel.git.sha={arguments.java_git_sha}",
    ]
    if gc_log:
        gc_log.parent.mkdir(parents=True, exist_ok=True)
        java_options.append(f"-Xlog:gc*:file={gc_log}:time,uptime,level,tags")
    if temp_dir:
        java_options.append(f"-Djava.io.tmpdir={temp_dir}")
    return [
        *java_options,
        "-cp", arguments.java_classpath,
        "com.alibaba.easyexcel.test.benchmark.EasyExcelBenchmarkRunner",
        *common,
    ]


def invoke(
    command: list[str],
    watch_dir: Path | None,
    measured: bool,
    temp_dir: Path | None = None,
) -> dict[str, Any]:
    actual = command
    if measured and watch_dir:
        actual = [sys.executable, str(MEASURE), "--watch-dir", str(watch_dir), "--", *command]
    environment = os.environ.copy()
    environment.update({"TZ": "UTC", "LANG": "en_US.UTF-8", "LC_ALL": "en_US.UTF-8"})
    if temp_dir:
        environment.update(
            {"TMPDIR": str(temp_dir), "TMP": str(temp_dir), "TEMP": str(temp_dir)}
        )
    completed = subprocess.run(
        actual, check=False, capture_output=True, text=True, env=environment
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"benchmark command failed ({completed.returncode}): {' '.join(command)}\n"
            f"stdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    return json.loads(completed.stdout.strip().splitlines()[-1])


def fixture_scenario(spec: dict[str, Any], file_format: str) -> dict[str, Any]:
    fixture_memory = "batched" if file_format == "xls" else "constant"
    matches = [
        scenario
        for scenario in spec["scenarios"]
        if scenario["format"] == file_format
        and scenario["operation"] == "write"
        and scenario["memory"] == fixture_memory
    ]
    if len(matches) != 1:
        raise KeyError(
            f"expected one fixture writer scenario for {file_format}, got {len(matches)}"
        )
    return matches[0]


def create_fixtures(
    spec: dict[str, Any],
    arguments: argparse.Namespace,
    rows: int,
    file_format: str,
) -> dict[str, Path]:
    fixtures: dict[str, Path] = {}
    scenario = fixture_scenario(spec, file_format)
    for implementation in ("rust", "java"):
        path = arguments.output_dir / "fixtures" / str(rows) / f"{implementation}.{file_format}"
        temp_dir = path.parent / "tmp" / implementation
        temp_dir.mkdir(parents=True, exist_ok=True)
        command = runner_command(
            implementation, arguments, scenario, rows, 1, None, path,
            temp_dir=temp_dir,
        )
        result = invoke(command, None, measured=False, temp_dir=temp_dir)
        if not result["success"]:
            raise RuntimeError(f"fixture generation failed: {implementation}/{file_format}/{rows}")
        FIXTURE_HASHES.pop(path.resolve(), None)
        fixtures[implementation] = path
    record_fixture_hashes(arguments, rows, file_format, fixtures)
    return fixtures


def record_fixture_hashes(
    arguments: argparse.Namespace,
    rows: int,
    file_format: str,
    fixtures: dict[str, Path],
) -> None:
    """Freeze generated fixture hashes as part of the reproducibility artifact."""
    manifest_path = arguments.output_dir / "fixtures" / "fixture-manifest.json"
    manifest = {"schema_version": 1, "fixtures": {}}
    if manifest_path.exists():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    for origin, path in fixtures.items():
        key = f"{file_format}/{rows}/{origin}"
        manifest["fixtures"][key] = {
            "format": file_format,
            "rows": rows,
            "origin": origin,
            "path": str(path.resolve()),
            "sha256": fixture_sha256(path),
        }
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def run_worker(
    implementation: str,
    arguments: argparse.Namespace,
    scenario: dict[str, Any],
    rows: int,
    workers: int,
    trial: int,
    worker: int,
    fixture_origin: str | None,
    fixture: Path | None,
    measured: bool,
    temperature: str,
    warmups: int,
) -> dict[str, Any]:
    run_dir = (
        arguments.output_dir / "work" / scenario["id"] / str(rows)
        / temperature / f"trial-{trial}" / (fixture_origin or "generated")
        / f"{implementation}-worker-{worker}"
    )
    temp_dir = run_dir / "tmp"
    temp_dir.mkdir(parents=True, exist_ok=True)
    output = None
    if scenario["operation"] in ("write", "roundtrip"):
        output = run_dir / f"output.{scenario['format']}"
    gc_log = run_dir / "gc.log" if implementation == "java" and measured else None
    command = runner_command(
        implementation, arguments, scenario, rows, workers, fixture, output,
        temperature, warmups, gc_log, temp_dir
    )
    result = invoke(command, temp_dir, measured, temp_dir=temp_dir)
    result["phase"] = "matrix"
    result["fixture_origin"] = fixture_origin
    result["input_sha256"] = fixture_sha256(fixture) if fixture else None
    result["trial"] = trial
    result["worker_id"] = worker
    if gc_log and gc_log.exists():
        pauses = [
            float(value)
            for value in re.findall(r"Pause[^\n]*?([0-9]+(?:\.[0-9]+)?)ms", gc_log.read_text(encoding="utf-8"))
        ]
        result["gc_max_pause_ns"] = int(max(pauses) * 1_000_000) if pauses else 0
    # 编排器在同组所有计时进程结束后再做双向重读，避免校验进程污染并发测量。
    result["_output_path"] = str(output) if output else None
    return result


def verify_written_output(
    arguments: argparse.Namespace,
    scenario: dict[str, Any],
    rows: int,
    output_path: Path,
) -> tuple[int, str]:
    """Use both implementations to re-read one output outside the timed region."""
    read_scenario = {
        "id": f"{scenario['format']}-event-read",
        "format": scenario["format"],
        "operation": "read",
        "mode": "event",
        "memory": "constant",
    }
    validations = []
    for implementation in ("rust", "java"):
        temp_dir = output_path.parent / "verify-tmp" / implementation
        temp_dir.mkdir(parents=True, exist_ok=True)
        command = runner_command(
            implementation,
            arguments,
            read_scenario,
            rows,
            1,
            output_path,
            None,
            temp_dir=temp_dir,
        )
        validations.append(invoke(command, None, measured=False, temp_dir=temp_dir))
    observed = {value["correctness"]["observed_rows"] for value in validations}
    checksums = {value["correctness"]["checksum"] for value in validations}
    if any(not value["success"] for value in validations) or len(observed) != 1 or len(checksums) != 1:
        raise RuntimeError(
            f"cross-runtime reread failed for {output_path}: {validations}"
        )
    return observed.pop(), checksums.pop()


def run_group(
    implementation: str,
    arguments: argparse.Namespace,
    scenario: dict[str, Any],
    rows: int,
    workers: int,
    trial: int,
    fixture_origin: str | None,
    fixture: Path | None,
    measured: bool,
    temperature: str = "cold",
    warmups: int = 0,
) -> list[dict[str, Any]]:
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = [
            executor.submit(
                run_worker,
                implementation,
                arguments,
                scenario,
                rows,
                workers,
                trial,
                worker,
                fixture_origin,
                fixture,
                measured,
                temperature,
                warmups,
            )
            for worker in range(workers)
        ]
        results = [future.result() for future in futures]
    if measured and scenario["operation"] in ("write", "roundtrip"):
        for result in results:
            output_path = Path(result.pop("_output_path"))
            observed_rows, checksum = verify_written_output(
                arguments, scenario, rows, output_path
            )
            result["correctness"]["observed_rows"] = observed_rows
            result["correctness"]["checksum"] = checksum
            result["correctness"]["rereadable"] = True
            result["success"] = observed_rows == rows
            result["errors"] = 0 if result["success"] else 1
            # The measured JSON already retains the exact final byte count and
            # both runtimes have reopened the file successfully. Keeping one
            # multi-megabyte workbook per worker/trial would make the release
            # matrix consume tens of gigabytes and can invalidate a run by
            # exhausting its filesystem. Preserve fixtures, hashes, GC logs,
            # and result records, but discard verified per-sample outputs.
            if result["success"]:
                output_path.unlink()
    else:
        for result in results:
            result.pop("_output_path", None)
    return results


def execution_order(measurements: int) -> list[str]:
    counts = {"rust": 0, "java": 0}
    order: list[str] = []
    while counts["rust"] < measurements or counts["java"] < measurements:
        for implementation in ("rust", "java", "java", "rust"):
            if counts[implementation] < measurements:
                order.append(implementation)
                counts[implementation] += 1
    return order


def execution_order_with_trials(measurements: int) -> list[tuple[str, int]]:
    """Attach a dense implementation-local trial ID to the interleaved order."""
    counts = {"rust": 0, "java": 0}
    result = []
    for implementation in execution_order(measurements):
        result.append((implementation, counts[implementation]))
        counts[implementation] += 1
    return result


def run_internal_parallel_map(
    spec: dict[str, Any],
    arguments: argparse.Namespace,
    rows: int,
    fixtures: dict[str, Path],
) -> list[dict[str, Any]]:
    """Measure one XLSX parser with serial/2/4-worker pure mapping semantics."""
    contract = spec["internal_parallel_map"]
    scenario = next(
        item for item in spec["scenarios"] if item["id"] == contract["scenario_id"]
    )
    measurements = spec["profiles"][arguments.profile]["measurements"]
    temperature = contract["temperature"]
    warmups = spec["profiles"][arguments.profile]["warmups"]
    worker_counts = contract["worker_counts"]
    results: list[dict[str, Any]] = []
    for fixture_origin, fixture in fixtures.items():
        input_sha256 = fixture_sha256(fixture)
        for trial in range(measurements):
            # 交替正反顺序，避免机器温度随 worker 数单调漂移。
            ordered_workers = worker_counts if trial % 2 == 0 else list(reversed(worker_counts))
            for workers in ordered_workers:
                run_dir = (
                    arguments.output_dir
                    / "work"
                    / "internal-parallel-map"
                    / str(rows)
                    / f"trial-{trial}"
                    / fixture_origin
                    / f"workers-{workers}"
                )
                temp_dir = run_dir / "tmp"
                temp_dir.mkdir(parents=True, exist_ok=True)
                queue_capacity = contract["queue_capacity_per_worker"] * workers
                command = runner_command(
                    "rust",
                    arguments,
                    scenario,
                    rows,
                    workers,
                    fixture,
                    None,
                    temperature,
                    warmups,
                    temp_dir=temp_dir,
                    internal_map_work_factor=contract["work_factor"],
                    internal_map_queue_capacity=queue_capacity,
                )
                result = invoke(command, temp_dir, measured=True, temp_dir=temp_dir)
                if result.get("phase") != "internal-parallel-map":
                    raise RuntimeError("Rust runner did not activate internal parallel-map phase")
                result["fixture_origin"] = fixture_origin
                result["input_sha256"] = input_sha256
                result["trial"] = trial
                # 这是一个进程内的 mapper worker 数；每个 trial 只有一个 runner。
                result["worker_id"] = 0
                results.append(result)
    return results


def git_sha(repository: Path | None) -> str:
    if repository is None:
        return "unknown"
    completed = subprocess.run(
        ["git", "-C", str(repository), "rev-parse", "HEAD"],
        check=False,
        capture_output=True,
        text=True,
    )
    return completed.stdout.strip() if completed.returncode == 0 else "unknown"


def repository_fingerprint(repository: Path | None) -> tuple[bool | None, str | None]:
    """Return dirty state and a deterministic hash of tracked/untracked source files."""
    if repository is None:
        return None, None
    status = subprocess.run(
        ["git", "-C", str(repository), "status", "--porcelain=v1"],
        check=False,
        capture_output=True,
        text=True,
    )
    files = subprocess.run(
        ["git", "-C", str(repository), "ls-files", "-co", "--exclude-standard", "-z"],
        check=False,
        capture_output=True,
    )
    if status.returncode != 0 or files.returncode != 0:
        return None, None
    digest = hashlib.sha256()
    for raw_path in sorted(path for path in files.stdout.split(b"\0") if path):
        path = repository / os.fsdecode(raw_path)
        if not path.is_file():
            continue
        digest.update(raw_path)
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return bool(status.stdout), digest.hexdigest()


def file_sha256(path: Path | None) -> str | None:
    if path is None or not path.is_file():
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def path_sha256(path: Path) -> str:
    """Hash a prebuilt classpath entry without trusting timestamps."""
    if path.is_file():
        return hashlib.sha256(path.read_bytes()).hexdigest()
    if not path.is_dir():
        raise RuntimeError(f"classpath entry does not exist: {path}")
    digest = hashlib.sha256()
    for item in sorted(candidate for candidate in path.rglob("*") if candidate.is_file()):
        relative = item.relative_to(path).as_posix().encode("utf-8")
        digest.update(relative)
        digest.update(b"\0")
        digest.update(item.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def resolve_executable(command: Path | str) -> Path:
    """Resolve the exact executable used by a release sample."""
    value = str(command)
    resolved = shutil.which(value)
    if resolved is None:
        candidate = Path(value).expanduser()
        if not candidate.is_file():
            raise RuntimeError(f"runtime executable does not exist: {value}")
        resolved = str(candidate)
    path = Path(resolved).resolve()
    if not path.is_file() or not os.access(path, os.X_OK):
        raise RuntimeError(f"runtime is not executable: {path}")
    return path


def java_runtime(java_bin: Path | str) -> tuple[Path, str, Path]:
    """Return the executable, version line, and Java home for one runtime."""
    executable = resolve_executable(java_bin)
    completed = subprocess.run(
        [str(executable), "-XshowSettings:properties", "-version"],
        check=False,
        capture_output=True,
        text=True,
    )
    output = "\n".join((completed.stdout, completed.stderr))
    home_match = re.search(r"^\s*java\.home\s*=\s*(.+?)\s*$", output, re.MULTILINE)
    version = next(
        (line.strip() for line in output.splitlines() if "version" in line.lower()),
        "",
    )
    if completed.returncode != 0 or home_match is None or not version:
        raise RuntimeError(f"cannot determine Java runtime identity from {executable}")
    return executable, version, Path(home_match.group(1)).resolve()


def validate_release_inputs(arguments: argparse.Namespace) -> None:
    """Reject stale or unpinned runners before any release sample is created."""
    if not arguments.rust_bin.is_file() or not os.access(arguments.rust_bin, os.X_OK):
        raise RuntimeError(f"prebuilt Rust benchmark runner is not executable: {arguments.rust_bin}")
    if arguments.java_repo is None or arguments.rust_repo is None:
        raise RuntimeError("release benchmark requires --java-repo and --rust-repo")
    java_dirty, _ = repository_fingerprint(arguments.java_repo)
    rust_dirty, _ = repository_fingerprint(arguments.rust_repo)
    if java_dirty is not False or rust_dirty is not False:
        raise RuntimeError("release benchmark requires clean Java and Rust worktrees")
    classpath = [Path(item).resolve() for item in arguments.java_classpath.split(os.pathsep) if item]
    expected_test_classes = (
        arguments.java_repo / "easyexcel-test" / "target" / "test-classes"
    ).resolve()
    if not classpath or classpath[0] != expected_test_classes:
        raise RuntimeError(
            "Java classpath must begin with easyexcel-test/target/test-classes from --java-repo"
        )
    runner_class = expected_test_classes / Path(
        "com/alibaba/easyexcel/test/benchmark/EasyExcelBenchmarkRunner.class"
    )
    if not runner_class.is_file():
        raise RuntimeError(f"prebuilt Java benchmark runner is missing: {runner_class}")
    for entry in classpath:
        path_sha256(entry)
    artifact_manifest = getattr(arguments, "artifact_manifest", None)
    if artifact_manifest is None or not artifact_manifest.is_file():
        raise RuntimeError("release benchmark requires --artifact-manifest from prepare_release_artifacts.py")
    attestation = json.loads(artifact_manifest.read_text(encoding="utf-8"))
    if (
        attestation.get("schema_version") != 2
        or attestation.get("artifact") != "easyexcel-release-benchmark-runners"
    ):
        raise RuntimeError("invalid release benchmark artifact manifest")
    rust_attestation = attestation.get("rust", {})
    java_attestation = attestation.get("java", {})
    java_bin, java_version, java_home = java_runtime(arguments.java_bin)
    _, rust_source_sha256 = repository_fingerprint(arguments.rust_repo)
    _, java_source_sha256 = repository_fingerprint(arguments.java_repo)
    expected_rust = {
        "repo": str(arguments.rust_repo.resolve()),
        "git_sha": git_sha(arguments.rust_repo),
        "source_sha256": rust_source_sha256,
        "binary": str(arguments.rust_bin.resolve()),
        "binary_sha256": path_sha256(arguments.rust_bin.resolve()),
    }
    for key, expected in expected_rust.items():
        if rust_attestation.get(key) != expected:
            raise RuntimeError(f"Rust release artifact attestation mismatch for {key}")
    for key in ("rustc", "rustc_sha256", "rustc_version"):
        if not rust_attestation.get(key):
            raise RuntimeError(f"Rust release artifact attestation lacks {key}")
    attested_rustc = Path(rust_attestation["rustc"])
    if path_sha256(attested_rustc) != rust_attestation["rustc_sha256"]:
        raise RuntimeError("attested Rust compiler has changed since runner preparation")
    expected_java_entries = [
        {"path": str(path), "sha256": path_sha256(path)} for path in classpath
    ]
    expected_java = {
        "repo": str(arguments.java_repo.resolve()),
        "git_sha": git_sha(arguments.java_repo),
        "source_sha256": java_source_sha256,
        "runner_class": str(runner_class),
        "runner_class_sha256": path_sha256(runner_class),
        "java_bin": str(java_bin),
        "java_bin_sha256": path_sha256(java_bin),
        "java_home": str(java_home),
        "java_version": java_version,
        "classpath": expected_java_entries,
    }
    for key, expected in expected_java.items():
        if java_attestation.get(key) != expected:
            raise RuntimeError(f"Java release artifact attestation mismatch for {key}")


def total_memory_bytes() -> int | None:
    if platform.system() == "Linux":
        match = re.search(r"^MemTotal:\s*(\d+)\s+kB", Path("/proc/meminfo").read_text(), re.MULTILINE)
        return int(match.group(1)) * 1024 if match else None
    if platform.system() == "Darwin":
        completed = subprocess.run(
            ["sysctl", "-n", "hw.memsize"], check=False, capture_output=True, text=True
        )
        return int(completed.stdout.strip()) if completed.returncode == 0 else None
    return None


def validate_runtime_contract(spec: dict[str, Any], arguments: argparse.Namespace) -> None:
    """Fail before fixture generation when the pinned runtime contract drifts."""
    contract = spec["runtime_contract"]
    orchestrator_values = {
        "java_gc": "G1",
        "timezone": "UTC",
        "locale": "en_US.UTF-8",
        "temp_directory": "isolated-per-worker",
    }
    for name, actual in orchestrator_values.items():
        if contract[name] != actual:
            raise RuntimeError(
                f"orchestrator {name} mismatch: expected {contract[name]}, got {actual}"
            )
    java_version = subprocess.run(
        [str(arguments.java_bin), "-version"], check=False, capture_output=True, text=True
    )
    version_text = java_version.stderr or java_version.stdout
    version_match = re.search(r'version "(\d+)', version_text)
    if java_version.returncode != 0 or version_match is None:
        raise RuntimeError(f"cannot determine Java version from {arguments.java_bin}")
    if version_match.group(1) != contract["java_version"]:
        raise RuntimeError(
            f"Java version mismatch: expected {contract['java_version']}, "
            f"got {version_match.group(1)}"
        )
    if arguments.java_xms != contract["java_xms"]:
        raise RuntimeError(
            f"Java Xms mismatch: expected {contract['java_xms']}, got {arguments.java_xms}"
        )
    if arguments.java_xmx != contract["java_xmx"]:
        raise RuntimeError(
            f"Java Xmx mismatch: expected {contract['java_xmx']}, got {arguments.java_xmx}"
        )
    locales = subprocess.run(
        ["locale", "-a"], check=False, capture_output=True, text=True
    )
    normalized = {value.strip().lower().replace("-", "") for value in locales.stdout.splitlines()}
    expected_locale = contract["locale"].lower().replace("-", "")
    if locales.returncode != 0 or expected_locale not in normalized:
        raise RuntimeError(f"required locale is unavailable: {contract['locale']}")


def validate_internal_parallel_map_contract(spec: dict[str, Any]) -> None:
    """Reject drift in the Rust-only single-workbook concurrency workload."""
    contract = spec.get("internal_parallel_map")
    if not isinstance(contract, dict):
        raise RuntimeError("benchmark spec lacks internal_parallel_map contract")
    scenario = next(
        (
            item
            for item in spec.get("scenarios", [])
            if item.get("id") == contract.get("scenario_id")
        ),
        None,
    )
    if not isinstance(scenario, dict) or any(
        scenario.get(field) != value
        for field, value in {
            "id": "xlsx-event-read",
            "format": "xlsx",
            "operation": "read",
            "mode": "event",
        }.items()
    ):
        raise RuntimeError("internal parallel-map must target xlsx-event-read/event mode")
    if contract.get("temperature") != "steady":
        raise RuntimeError("internal parallel-map must use steady temperature")
    if contract.get("worker_counts") != [1, 2, 4]:
        raise RuntimeError("internal parallel-map workers must be exactly [1, 2, 4]")
    for field in ("queue_capacity_per_worker", "work_factor", "max_peak_rss_bytes"):
        value = contract.get(field)
        if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            raise RuntimeError(f"internal parallel-map {field} must be positive")
    minimum = contract.get("min_median_speedup")
    if not isinstance(minimum, (int, float)) or isinstance(minimum, bool) or minimum < 1.20:
        raise RuntimeError("internal parallel-map speedup gate must be at least 1.20")


def write_environment_manifest(arguments: argparse.Namespace, spec: dict[str, Any]) -> None:
    disk = shutil.disk_usage(arguments.output_dir)
    rust_dirty, rust_source_sha256 = repository_fingerprint(arguments.rust_repo)
    java_dirty, java_source_sha256 = repository_fingerprint(arguments.java_repo)
    java_bin, java_version, java_home = java_runtime(arguments.java_bin)
    java_classpath = [
        Path(item).resolve()
        for item in arguments.java_classpath.split(os.pathsep)
        if item
    ]
    artifact_manifest = getattr(arguments, "artifact_manifest", None)
    artifact = (
        json.loads(artifact_manifest.read_text(encoding="utf-8"))
        if artifact_manifest is not None and artifact_manifest.is_file()
        else {}
    )
    attested_rust = artifact.get("rust", {})
    manifest = {
        "schema_version": 1,
        "platform": platform.platform(),
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "logical_cpu_count": os.cpu_count(),
        "total_memory_bytes": total_memory_bytes(),
        "disk_total_bytes": disk.total,
        "disk_free_bytes_before": disk.free,
        "java_version": java_version,
        "java_bin": str(java_bin),
        "java_bin_sha256": path_sha256(java_bin),
        "java_home": str(java_home),
        "java_git_sha": arguments.java_git_sha,
        "rust_git_sha": git_sha(arguments.rust_repo),
        "java_worktree_dirty": java_dirty,
        "rust_worktree_dirty": rust_dirty,
        "java_source_sha256": java_source_sha256,
        "rust_source_sha256": rust_source_sha256,
        "spec_sha256": file_sha256(arguments.spec),
        "artifact_manifest_sha256": file_sha256(getattr(arguments, "artifact_manifest", None)),
        "rust_binary_path": str(arguments.rust_bin.resolve()),
        "rust_binary_sha256": file_sha256(arguments.rust_bin),
        "rustc": attested_rust.get("rustc"),
        "rustc_sha256": attested_rust.get("rustc_sha256"),
        "rustc_version": attested_rust.get("rustc_version"),
        "java_classpath": [
            {"path": str(path), "sha256": path_sha256(path)} for path in java_classpath
        ],
        "cargo_lock_sha256": file_sha256(
            arguments.rust_repo / "Cargo.lock" if arguments.rust_repo else None
        ),
        "java_root_pom_sha256": file_sha256(
            arguments.java_repo / "pom.xml" if arguments.java_repo else None
        ),
        "java_xms": arguments.java_xms,
        "java_xmx": arguments.java_xmx,
        "locale": spec["runtime_contract"]["locale"],
        "timezone": spec["runtime_contract"]["timezone"],
        "gc": spec["runtime_contract"]["java_gc"],
        "runtime_contract": spec["runtime_contract"],
    }
    path = arguments.output_dir / "environment-manifest.json"
    path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    shutil.copyfile(arguments.spec, arguments.output_dir / "benchmark-suite-v1.json")
    schema = arguments.spec.parent / "benchmark-result-v1.schema.json"
    if schema.is_file():
        shutil.copyfile(schema, arguments.output_dir / "benchmark-result-v1.schema.json")
    if artifact_manifest is not None:
        destination = arguments.output_dir / "artifact-manifest.json"
        if artifact_manifest.resolve() != destination.resolve():
            shutil.copyfile(artifact_manifest, destination)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--spec", type=Path, default=ROOT / "benchmarks/spec/benchmark-suite-v1.json")
    parser.add_argument("--profile", choices=("pr", "nightly", "release"), required=True)
    parser.add_argument("--rust-bin", type=Path, required=True)
    parser.add_argument("--java-bin", type=Path, default=Path("java"))
    parser.add_argument("--java-classpath", required=True)
    parser.add_argument("--java-xms", default="512m")
    parser.add_argument("--java-xmx", default="4g")
    parser.add_argument("--java-repo", type=Path)
    parser.add_argument("--rust-repo", type=Path, default=ROOT)
    parser.add_argument("--artifact-manifest", type=Path)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--scenario", action="append")
    arguments = parser.parse_args()
    arguments.java_git_sha = git_sha(arguments.java_repo)
    spec = json.loads(arguments.spec.read_text(encoding="utf-8"))
    validate_runtime_contract(spec, arguments)
    validate_internal_parallel_map_contract(spec)
    profile = spec["profiles"][arguments.profile]
    if arguments.profile == "release":
        validate_release_inputs(arguments)
    scenarios = [
        scenario for scenario in spec["scenarios"]
        if not arguments.scenario or scenario["id"] in arguments.scenario
    ]
    internal_scenario_id = spec["internal_parallel_map"]["scenario_id"]
    internal_selected = any(
        scenario["id"] == internal_scenario_id for scenario in scenarios
    )
    arguments.output_dir.mkdir(parents=True, exist_ok=True)
    write_environment_manifest(arguments, spec)
    raw_path = arguments.output_dir / "raw-results.jsonl"
    with raw_path.open("w", encoding="utf-8") as raw:
        for rows in profile["rows"]:
            fixture_cache: dict[str, dict[str, Path]] = {}
            for scenario in scenarios:
                workers_matrix = [1]
                if (
                    arguments.profile == "release"
                    and scenario["id"] in spec["concurrency_scenarios"]
                ):
                    workers_matrix = spec["concurrency"]
                origins: list[tuple[str | None, Path | None]] = [(None, None)]
                if scenario["operation"] in ("read", "roundtrip"):
                    file_format = scenario["format"]
                    if file_format not in fixture_cache:
                        fixture_cache[file_format] = create_fixtures(
                            spec, arguments, rows, file_format
                        )
                    origins = list(fixture_cache[file_format].items())
                for workers in workers_matrix:
                    for origin, fixture in origins:
                        for temperature in profile["temperatures"]:
                            warmups = profile["warmups"] if temperature == "steady" else 0
                            for implementation, trial in execution_order_with_trials(
                                profile["measurements"]
                            ):
                                for result in run_group(
                                    implementation, arguments, scenario, rows, workers,
                                    trial, origin, fixture, measured=True,
                                    temperature=temperature, warmups=warmups,
                                ):
                                    raw.write(json.dumps(result, ensure_ascii=False, separators=(",", ":")) + "\n")
                                    raw.flush()
            if arguments.profile == "release" and internal_selected:
                if "xlsx" not in fixture_cache:
                    fixture_cache["xlsx"] = create_fixtures(
                        spec, arguments, rows, "xlsx"
                    )
                fixtures = fixture_cache["xlsx"]
                for result in run_internal_parallel_map(
                    spec, arguments, rows, fixtures
                ):
                    raw.write(
                        json.dumps(result, ensure_ascii=False, separators=(",", ":"))
                        + "\n"
                    )
                    raw.flush()
    print(raw_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
