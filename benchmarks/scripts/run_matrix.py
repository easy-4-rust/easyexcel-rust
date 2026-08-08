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
    scenario_id = f"{file_format}-stream-write"
    for scenario in spec["scenarios"]:
        if scenario["id"] == scenario_id:
            return scenario
    raise KeyError(f"missing fixture writer scenario {scenario_id}")


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
            "path": str(path),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
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
    result["input_sha256"] = (
        hashlib.sha256(fixture.read_bytes()).hexdigest() if fixture else None
    )
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
    order = []
    while counts["rust"] < measurements or counts["java"] < measurements:
        for implementation in ("rust", "java", "java", "rust"):
            if counts[implementation] < measurements:
                order.append(implementation)
                counts[implementation] += 1
    return order


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


def write_environment_manifest(arguments: argparse.Namespace, spec: dict[str, Any]) -> None:
    disk = shutil.disk_usage(arguments.output_dir)
    rust_dirty, rust_source_sha256 = repository_fingerprint(arguments.rust_repo)
    java_dirty, java_source_sha256 = repository_fingerprint(arguments.java_repo)
    java_version = subprocess.run(
        [str(arguments.java_bin), "-version"], check=False, capture_output=True, text=True
    )
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
        "java_version": (java_version.stderr or java_version.stdout).strip(),
        "java_git_sha": arguments.java_git_sha,
        "rust_git_sha": git_sha(arguments.rust_repo),
        "java_worktree_dirty": java_dirty,
        "rust_worktree_dirty": rust_dirty,
        "java_source_sha256": java_source_sha256,
        "rust_source_sha256": rust_source_sha256,
        "spec_sha256": file_sha256(arguments.spec),
        "rust_binary_sha256": file_sha256(arguments.rust_bin),
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
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--scenario", action="append")
    arguments = parser.parse_args()
    arguments.java_git_sha = git_sha(arguments.java_repo)
    spec = json.loads(arguments.spec.read_text(encoding="utf-8"))
    validate_runtime_contract(spec, arguments)
    profile = spec["profiles"][arguments.profile]
    scenarios = [
        scenario for scenario in spec["scenarios"]
        if not arguments.scenario or scenario["id"] in arguments.scenario
    ]
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
                    fixture_cache.setdefault(
                        file_format,
                        create_fixtures(spec, arguments, rows, file_format),
                    )
                    origins = list(fixture_cache[file_format].items())
                for workers in workers_matrix:
                    for origin, fixture in origins:
                        for temperature in profile["temperatures"]:
                            warmups = profile["warmups"] if temperature == "steady" else 0
                            for trial, implementation in enumerate(execution_order(profile["measurements"])):
                                for result in run_group(
                                    implementation, arguments, scenario, rows, workers,
                                    trial, origin, fixture, measured=True,
                                    temperature=temperature, warmups=warmups,
                                ):
                                    raw.write(json.dumps(result, ensure_ascii=False, separators=(",", ":")) + "\n")
                                    raw.flush()
    print(raw_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
