#!/usr/bin/env python3
"""Aggregate Java/Rust BenchmarkResult JSONL and enforce stable-baseline gates."""

from __future__ import annotations

import argparse
from collections import defaultdict
import hashlib
import json
import math
from pathlib import Path
import random
import re
import statistics
from typing import Any


ROOT = Path(__file__).resolve().parents[2]


def percentile(values: list[float], probability: float) -> float:
    ordered = sorted(values)
    rank = max(0, math.ceil(probability * len(ordered)) - 1)
    return ordered[rank]


def summarize(values: list[float]) -> dict[str, float]:
    median = statistics.median(values)
    deviations = [abs(value - median) for value in values]
    mean = statistics.fmean(values)
    return {
        "median": median,
        "maximum": max(values),
        "mad": statistics.median(deviations),
        "p50": percentile(values, 0.50),
        "p95": percentile(values, 0.95),
        "p99": percentile(values, 0.99),
        "coefficient_of_variation": statistics.pstdev(values) / mean if mean else 0.0,
    }


def load_results(paths: list[Path]) -> list[dict[str, Any]]:
    results = []
    for path in paths:
        for line in path.read_text(encoding="utf-8").splitlines():
            if line.strip():
                results.append(json.loads(line))
    return results


def validate_spec_semantics(spec: dict[str, Any], failures: list[str]) -> None:
    """Reject benchmark labels that claim unavailable BIFF8 streaming semantics."""
    xls_writes = [
        scenario
        for scenario in spec.get("scenarios", [])
        if scenario.get("format") == "xls" and scenario.get("operation") == "write"
    ]
    if len(xls_writes) != 1:
        failures.append(
            f"benchmark spec must contain exactly one XLS write scenario, got {len(xls_writes)}"
        )
        return
    scenario = xls_writes[0]
    expected = {
        "id": "xls-batched-write",
        "mode": "workbook",
        "memory": "batched",
    }
    for field, value in expected.items():
        if scenario.get(field) != value:
            failures.append(
                f"XLS write scenario must declare {field}={value}, got {scenario.get(field)}"
            )
    internal = spec.get("internal_parallel_map")
    if not isinstance(internal, dict):
        failures.append("benchmark spec lacks internal_parallel_map contract")
        return
    internal_scenario = next(
        (
            item
            for item in spec.get("scenarios", [])
            if item.get("id") == internal.get("scenario_id")
        ),
        None,
    )
    if not isinstance(internal_scenario, dict) or any(
        internal_scenario.get(field) != value
        for field, value in {
            "id": "xlsx-event-read",
            "format": "xlsx",
            "operation": "read",
            "mode": "event",
        }.items()
    ):
        failures.append("internal parallel-map must target xlsx-event-read/event mode")
    if internal.get("temperature") != "steady":
        failures.append("internal parallel-map gate must use steady temperature")
    if internal.get("worker_counts") != [1, 2, 4]:
        failures.append("internal parallel-map worker counts must be exactly [1, 2, 4]")
    for field in ("queue_capacity_per_worker", "work_factor", "max_peak_rss_bytes"):
        value = internal.get(field)
        if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            failures.append(f"internal parallel-map {field} must be a positive integer")
    minimum = internal.get("min_median_speedup")
    if (
        not isinstance(minimum, (int, float))
        or isinstance(minimum, bool)
        or not math.isfinite(minimum)
        or minimum < 1.20
    ):
        failures.append("internal parallel-map min_median_speedup must be at least 1.20")
    release = spec.get("profiles", {}).get("release", {})
    if release.get("measurements", 0) < 7 or "steady" not in release.get("temperatures", []):
        failures.append("release profile lacks seven steady internal-map measurements")


def json_type_matches(value: Any, expected: str) -> bool:
    """Match the JSON types used by the v1 result schema without dependencies."""
    return {
        "object": isinstance(value, dict),
        "array": isinstance(value, list),
        "string": isinstance(value, str),
        "integer": isinstance(value, int) and not isinstance(value, bool),
        "number": isinstance(value, (int, float))
        and not isinstance(value, bool)
        and math.isfinite(value),
        "boolean": isinstance(value, bool),
        "null": value is None,
    }.get(expected, False)


def validate_json_schema(value: Any, schema: dict[str, Any], path: str = "$") -> list[str]:
    """Validate the JSON-Schema subset used by BenchmarkResult v1."""
    errors: list[str] = []
    declared_type = schema.get("type")
    if declared_type is not None:
        expected_types = declared_type if isinstance(declared_type, list) else [declared_type]
        if not any(json_type_matches(value, expected) for expected in expected_types):
            return [f"{path}: expected type {expected_types}"]
    if "const" in schema and value != schema["const"]:
        errors.append(f"{path}: expected constant {schema['const']!r}")
    if "enum" in schema and value not in schema["enum"]:
        errors.append(f"{path}: value is outside enum")
    if "minimum" in schema and isinstance(value, (int, float)) and value < schema["minimum"]:
        errors.append(f"{path}: value is below minimum {schema['minimum']}")
    if "pattern" in schema and isinstance(value, str) and re.search(schema["pattern"], value) is None:
        errors.append(f"{path}: string does not match required pattern")
    if isinstance(value, dict):
        required = schema.get("required", [])
        for name in required:
            if name not in value:
                errors.append(f"{path}: missing required property {name}")
        properties = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            for name in value.keys() - properties.keys():
                errors.append(f"{path}: unexpected property {name}")
        for name, child_schema in properties.items():
            if name in value:
                errors.extend(validate_json_schema(value[name], child_schema, f"{path}.{name}"))
    return errors


def group_key(result: dict[str, Any]) -> tuple[str, str, str, str, str | None, int, int]:
    return (
        result["implementation"],
        result["phase"],
        result["temperature"],
        result["scenario_id"],
        result.get("fixture_origin"),
        result["rows"],
        result["worker_count"],
    )


def summarize_present(samples: list[dict[str, Any]], field: str) -> dict[str, float] | None:
    values = [float(sample[field]) for sample in samples if sample.get(field) is not None]
    return summarize(values) if values else None


def summarize_concurrent_throughput(
    samples: list[dict[str, Any]],
) -> dict[str, float] | None:
    """Aggregate workers by trial before computing concurrency statistics."""
    trials: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for sample in samples:
        if sample.get("trial") is not None:
            trials[sample["trial"]].append(sample)
    rates = []
    for trial_samples in trials.values():
        elapsed_ns = max(sample["wall_time_ns"] for sample in trial_samples)
        total_rows = sum(sample["rows"] for sample in trial_samples)
        rates.append(total_rows / (elapsed_ns / 1_000_000_000))
    return summarize(rates) if rates else None


def trial_throughput_rates(samples: list[dict[str, Any]]) -> list[float]:
    """Return one aggregate throughput value per concurrency trial."""
    trials: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for sample in samples:
        if isinstance(sample.get("trial"), int):
            trials[sample["trial"]].append(sample)
    rates = []
    for trial in sorted(trials):
        trial_samples = trials[trial]
        elapsed_ns = max(sample["wall_time_ns"] for sample in trial_samples)
        total_rows = sum(sample["rows"] for sample in trial_samples)
        rates.append(total_rows / (elapsed_ns / 1_000_000_000))
    return rates


def bootstrap_median_ratio(
    rust_rates: list[float], java_rates: list[float], *, seed: str, iterations: int = 10_000
) -> dict[str, float]:
    """Compute a deterministic independent-bootstrap CI for median Rust/Java throughput."""
    if not rust_rates or not java_rates:
        raise ValueError("both runtimes require at least one trial")
    actual = statistics.median(rust_rates) / statistics.median(java_rates)
    generator = random.Random(int(hashlib.sha256(seed.encode("utf-8")).hexdigest()[:16], 16))
    ratios = []
    for _ in range(iterations):
        rust_median = statistics.median(generator.choices(rust_rates, k=len(rust_rates)))
        java_median = statistics.median(generator.choices(java_rates, k=len(java_rates)))
        ratios.append(rust_median / java_median if java_median else 0.0)
    ordered = sorted(ratios)
    return {
        "median_ratio": actual,
        "confidence_level": 0.95,
        "confidence_lower_bound": ordered[int(0.025 * (len(ordered) - 1))],
        "confidence_upper_bound": ordered[int(0.975 * (len(ordered) - 1))],
        "bootstrap_iterations": iterations,
    }


def expected_matrix_groups(
    spec: dict[str, Any], profile_name: str
) -> dict[tuple[str, str, str, str, str | None, int, int], int]:
    """Build the exact matrix shape required by one benchmark profile."""
    profile = spec["profiles"][profile_name]
    expected: dict[tuple[str, str, str, str, str | None, int, int], int] = {}
    for implementation in ("rust", "java"):
        for temperature in profile["temperatures"]:
            for scenario in spec["scenarios"]:
                workers = [1]
                if (
                    profile_name == "release"
                    and scenario["id"] in spec["concurrency_scenarios"]
                ):
                    workers = spec["concurrency"]
                origins: list[str | None] = [None]
                if scenario["operation"] in ("read", "roundtrip"):
                    origins = ["rust", "java"]
                for origin in origins:
                    for rows in profile["rows"]:
                        for worker_count in workers:
                            key = (
                                implementation,
                                "matrix",
                                temperature,
                                scenario["id"],
                                origin,
                                rows,
                                worker_count,
                            )
                            expected[key] = profile["measurements"] * worker_count
    return expected


def validate_matrix_completeness(
    grouped: dict[tuple[str, str, str, str, str | None, int, int], list[dict[str, Any]]],
    spec: dict[str, Any],
    profile_name: str,
    failures: list[str],
) -> None:
    """Reject missing, extra, duplicated, or malformed matrix samples."""
    expected = expected_matrix_groups(spec, profile_name)
    actual = {key: samples for key, samples in grouped.items() if key[1] == "matrix"}
    for key in sorted(expected.keys() - actual.keys(), key=str):
        failures.append(f"missing benchmark group: {'/'.join(map(str, key))}")
    for key in sorted(actual.keys() - expected.keys(), key=str):
        failures.append(f"unexpected benchmark group: {'/'.join(map(str, key))}")
    for key in sorted(expected.keys() & actual.keys(), key=str):
        samples = actual[key]
        expected_samples = expected[key]
        label = "/".join(map(str, key))
        if len(samples) != expected_samples:
            failures.append(
                f"sample count mismatch: {label}: expected {expected_samples}, got {len(samples)}"
            )
        workers = key[-1]
        trials: dict[int, list[int]] = defaultdict(list)
        for sample in samples:
            trial = sample.get("trial")
            worker_id = sample.get("worker_id")
            if not isinstance(trial, int) or not isinstance(worker_id, int):
                failures.append(f"missing integer trial/worker identity: {label}")
                continue
            trials[trial].append(worker_id)
        expected_measurements = spec["profiles"][profile_name]["measurements"]
        if len(trials) != expected_measurements:
            failures.append(
                f"trial count mismatch: {label}: expected {expected_measurements}, got {len(trials)}"
            )
        expected_trials = set(range(expected_measurements))
        if set(trials) != expected_trials:
            failures.append(
                f"trial id set mismatch: {label}: expected {sorted(expected_trials)}, "
                f"got {sorted(trials)}"
            )
        expected_workers = list(range(workers))
        for trial, worker_ids in sorted(trials.items()):
            if sorted(worker_ids) != expected_workers:
                failures.append(
                    f"worker set mismatch: {label}/trial-{trial}: "
                    f"expected {expected_workers}, got {sorted(worker_ids)}"
                )


def validate_internal_parallel_map_completeness(
    grouped: dict[tuple[str, str, str, str, str | None, int, int], list[dict[str, Any]]],
    spec: dict[str, Any],
    profile_name: str,
    failures: list[str],
) -> None:
    """Reject incomplete or process-concurrency-shaped internal mapper samples."""
    actual = {
        key: samples
        for key, samples in grouped.items()
        if key[1] == "internal-parallel-map"
    }
    if profile_name != "release":
        for key in sorted(actual, key=str):
            failures.append(f"unexpected internal parallel-map group: {'/'.join(map(str, key))}")
        return
    contract = spec.get("internal_parallel_map")
    if not isinstance(contract, dict):
        return
    required_fields = {
        "temperature",
        "scenario_id",
        "worker_counts",
        "queue_capacity_per_worker",
        "work_factor",
    }
    if not required_fields.issubset(contract):
        return
    rows = spec["profiles"]["release"]["rows"][-1]
    measurements = spec["profiles"]["release"]["measurements"]
    expected = {
        (
            "rust",
            "internal-parallel-map",
            contract["temperature"],
            contract["scenario_id"],
            origin,
            rows,
            workers,
        )
        for origin in ("rust", "java")
        for workers in contract["worker_counts"]
    }
    for key in sorted(expected - actual.keys(), key=str):
        failures.append(f"missing internal parallel-map group: {'/'.join(map(str, key))}")
    for key in sorted(actual.keys() - expected, key=str):
        failures.append(f"unexpected internal parallel-map group: {'/'.join(map(str, key))}")
    expected_trials = set(range(measurements))
    checksums: set[str] = set()
    input_hashes: dict[str, set[str]] = defaultdict(set)
    for key in sorted(expected & actual.keys(), key=str):
        samples = actual[key]
        label = "/".join(map(str, key))
        if len(samples) != measurements:
            failures.append(
                f"internal parallel-map sample count mismatch: {label}: "
                f"expected {measurements}, got {len(samples)}"
            )
        trials = [sample.get("trial") for sample in samples]
        if set(trials) != expected_trials or len(trials) != len(set(trials)):
            failures.append(
                f"internal parallel-map trial set mismatch: {label}: got {sorted(trials, key=str)}"
            )
        if any(sample.get("worker_id") != 0 for sample in samples):
            failures.append(
                f"internal parallel-map must contain one runner per trial: {label}"
            )
        workers = key[-1]
        expected_queue_capacity = contract["queue_capacity_per_worker"] * workers
        if any(
            sample.get("internal_map_work_factor") != contract["work_factor"]
            or sample.get("internal_map_queue_capacity") != expected_queue_capacity
            for sample in samples
        ):
            failures.append(f"internal parallel-map workload contract mismatch: {label}")
        input_hashes[str(key[4])].update(
            sample["input_sha256"] for sample in samples
        )
        checksums.update(sample["correctness"]["checksum"] for sample in samples)
    if actual and len(checksums) != 1:
        failures.append("internal parallel-map checksum differs across workers or fixture origins")
    for origin, hashes in sorted(input_hashes.items()):
        if len(hashes) != 1:
            failures.append(
                f"internal parallel-map input SHA drifted across {origin} samples"
            )


def validate_result_provenance(
    results: list[dict[str, Any]],
    spec: dict[str, Any],
    spec_path: Path,
    failures: list[str],
    expected_git_shas: dict[str, str | None] | None = None,
) -> None:
    """Ensure every sample belongs to this exact shared contract and a real build."""
    expected_spec_sha = hashlib.sha256(spec_path.read_bytes()).hexdigest()
    expected_git_shas = expected_git_shas or {}
    observed_git_shas: dict[str, set[str]] = defaultdict(set)
    for index, result in enumerate(results, start=1):
        environment = result.get("environment") or {}
        if environment.get("spec_sha256") != expected_spec_sha:
            failures.append(f"spec SHA mismatch at sample {index}")
        git_sha = environment.get("git_sha")
        if git_sha in (None, "", "unknown"):
            failures.append(f"unknown implementation Git SHA at sample {index}")
        elif isinstance(git_sha, str):
            implementation = result.get("implementation")
            observed_git_shas[str(implementation)].add(git_sha)
            expected = expected_git_shas.get(str(implementation))
            if expected is not None and git_sha != expected:
                failures.append(
                    f"{implementation} Git SHA mismatch at sample {index}: "
                    f"expected {expected}, got {git_sha}"
                )
        runtime = environment.get("runtime", "")
        contract = spec["runtime_contract"]
        implementation = result.get("implementation")
        if implementation == "java" and not runtime.startswith(contract["java_version"]):
            failures.append(f"Java runtime contract mismatch at sample {index}")
        if implementation == "rust" and contract["rust_toolchain"] not in runtime:
            failures.append(f"Rust runtime contract mismatch at sample {index}")
        origin = result.get("fixture_origin")
        operation = result.get("operation")
        if result.get("phase") in ("matrix", "internal-parallel-map"):
            if operation in ("read", "roundtrip") and origin not in ("rust", "java"):
                failures.append(f"missing fixture origin at sample {index}")
            if operation == "write" and origin is not None:
                failures.append(f"unexpected fixture origin at sample {index}")
            if origin is not None and not result.get("input_sha256"):
                failures.append(f"missing input SHA at sample {index}")
        if result.get("phase") == "internal-parallel-map":
            if result.get("implementation") != "rust" or operation != "read":
                failures.append(f"invalid internal parallel-map sample at index {index}")
        elif result.get("internal_map_work_factor") is not None or result.get(
            "internal_map_queue_capacity"
        ) is not None:
            failures.append(f"non-internal sample declares mapper parameters at index {index}")
    for implementation, shas in sorted(observed_git_shas.items()):
        if len(shas) != 1:
            failures.append(
                f"mixed {implementation} Git SHAs in benchmark results: {sorted(shas)}"
            )


def validate_release_environment_manifests(
    result_paths: list[Path],
    spec_path: Path,
    failures: list[str],
    expected_git_shas: dict[str, str | None],
) -> None:
    """Bind every release JSONL to the attested prebuilt runners in its directory."""
    expected_spec_sha = hashlib.sha256(spec_path.read_bytes()).hexdigest()
    directories = {path.resolve().parent for path in result_paths}
    for directory in sorted(directories):
        environment_path = directory / "environment-manifest.json"
        artifact_path = directory / "artifact-manifest.json"
        if not environment_path.is_file():
            failures.append(f"release results lack environment manifest: {directory}")
            continue
        if not artifact_path.is_file():
            failures.append(f"release results lack artifact manifest: {directory}")
            continue
        environment = json.loads(environment_path.read_text(encoding="utf-8"))
        artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
        artifact_sha = hashlib.sha256(artifact_path.read_bytes()).hexdigest()
        if environment.get("schema_version") != 1:
            failures.append(f"invalid environment manifest schema: {directory}")
        if environment.get("spec_sha256") != expected_spec_sha:
            failures.append(f"environment manifest spec SHA mismatch: {directory}")
        if environment.get("artifact_manifest_sha256") != artifact_sha:
            failures.append(f"environment/artifact manifest hash mismatch: {directory}")
        if environment.get("rust_worktree_dirty") is not False:
            failures.append(f"release Rust worktree was dirty: {directory}")
        if environment.get("java_worktree_dirty") is not False:
            failures.append(f"release Java worktree was dirty: {directory}")
        if (
            artifact.get("schema_version") != 2
            or artifact.get("artifact") != "easyexcel-release-benchmark-runners"
        ):
            failures.append(f"invalid prebuilt runner attestation: {directory}")
            continue
        rust = artifact.get("rust", {})
        java = artifact.get("java", {})
        if rust.get("git_sha") != expected_git_shas.get("rust"):
            failures.append(f"attested Rust Git SHA mismatch: {directory}")
        if java.get("git_sha") != expected_git_shas.get("java"):
            failures.append(f"attested Java Git SHA mismatch: {directory}")
        if rust.get("source_sha256") != environment.get("rust_source_sha256"):
            failures.append(f"attested Rust source fingerprint mismatch: {directory}")
        if java.get("source_sha256") != environment.get("java_source_sha256"):
            failures.append(f"attested Java source fingerprint mismatch: {directory}")
        if rust.get("binary") != environment.get("rust_binary_path"):
            failures.append(f"attested Rust binary path mismatch: {directory}")
        if rust.get("binary_sha256") != environment.get("rust_binary_sha256"):
            failures.append(f"attested Rust binary SHA mismatch: {directory}")
        for field in ("rustc", "rustc_sha256", "rustc_version"):
            if not rust.get(field):
                failures.append(f"attested Rust compiler lacks {field}: {directory}")
            elif rust.get(field) != environment.get(field):
                failures.append(
                    f"attested Rust compiler {field} mismatch: {directory}"
                )
        if java.get("java_bin") != environment.get("java_bin"):
            failures.append(f"attested Java executable path mismatch: {directory}")
        if java.get("java_bin_sha256") != environment.get("java_bin_sha256"):
            failures.append(f"attested Java executable SHA mismatch: {directory}")
        if java.get("java_home") != environment.get("java_home"):
            failures.append(f"attested Java home mismatch: {directory}")
        if java.get("java_version") != environment.get("java_version"):
            failures.append(f"attested Java version mismatch: {directory}")
        if java.get("classpath") != environment.get("java_classpath"):
            failures.append(f"attested Java classpath mismatch: {directory}")


def validate_fixture_manifests(
    result_paths: list[Path], spec: dict[str, Any], failures: list[str]
) -> None:
    """Bind every timed read to one retained, hashed Java/Rust fixture file."""
    scenario_formats = {
        scenario["id"]: scenario["format"] for scenario in spec.get("scenarios", [])
    }
    file_hashes: dict[Path, str] = {}
    validated_entries: set[tuple[Path, str]] = set()
    for result_path in result_paths:
        manifest_path = result_path.resolve().parent / "fixtures" / "fixture-manifest.json"
        if not manifest_path.is_file():
            failures.append(f"benchmark results lack fixture manifest: {result_path}")
            continue
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            failures.append(f"cannot read fixture manifest {manifest_path}: {error}")
            continue
        fixtures = manifest.get("fixtures")
        if manifest.get("schema_version") != 1 or not isinstance(fixtures, dict):
            failures.append(f"invalid fixture manifest: {manifest_path}")
            continue
        for sample_index, sample in enumerate(load_results([result_path]), start=1):
            if sample.get("operation") not in ("read", "roundtrip"):
                continue
            origin = sample.get("fixture_origin")
            scenario_id = sample.get("scenario_id")
            file_format = scenario_formats.get(scenario_id)
            rows = sample.get("rows")
            if origin not in ("rust", "java") or file_format is None or not isinstance(rows, int):
                failures.append(
                    f"cannot resolve fixture for sample {sample_index}: {result_path}"
                )
                continue
            key = f"{file_format}/{rows}/{origin}"
            fixture = fixtures.get(key)
            if not isinstance(fixture, dict):
                failures.append(f"fixture manifest lacks {key}: {manifest_path}")
                continue
            fixture_path_value = fixture.get("path")
            fixture_path = (
                Path(fixture_path_value)
                if isinstance(fixture_path_value, str) and fixture_path_value
                else None
            )
            if fixture_path is None or not fixture_path.is_absolute() or not fixture_path.is_file():
                failures.append(f"fixture file is missing or non-absolute: {key}")
                continue
            expected_path = (
                manifest_path.parent / str(rows) / f"{origin}.{file_format}"
            ).resolve()
            if fixture_path != expected_path:
                failures.append(f"fixture path escapes its canonical result location: {key}")
                continue
            if fixture_path not in file_hashes:
                file_hashes[fixture_path] = hashlib.sha256(
                    fixture_path.read_bytes()
                ).hexdigest()
            actual_sha = file_hashes[fixture_path]
            if sample.get("input_sha256") != actual_sha:
                failures.append(
                    f"sample input SHA does not match retained fixture: {key}/{sample_index}"
                )
            entry_key = (manifest_path, key)
            if entry_key not in validated_entries:
                expected_entry = {
                    "format": file_format,
                    "rows": rows,
                    "origin": origin,
                    "path": str(fixture_path),
                    "sha256": actual_sha,
                }
                for field, expected in expected_entry.items():
                    if fixture.get(field) != expected:
                        failures.append(f"fixture manifest {field} mismatch: {key}")
                validated_entries.add(entry_key)


def validate_stable_baseline(
    baseline_path: Path,
    profile: str,
    spec_path: Path,
    failures: list[str],
) -> dict[str, Any] | None:
    """Accept only a reviewed, successful report from the repository baseline directory."""
    failure_count = len(failures)
    expected_directory = (ROOT / "benchmarks" / "baselines").resolve()
    resolved = baseline_path.resolve()
    if resolved.parent != expected_directory:
        failures.append(
            f"stable baseline must come from {expected_directory}: {baseline_path}"
        )
        return None
    try:
        report = json.loads(resolved.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        failures.append(f"cannot read stable baseline {baseline_path}: {error}")
        return None
    expected_spec_sha = hashlib.sha256(spec_path.read_bytes()).hexdigest()
    if report.get("schema_version") != 1:
        failures.append("stable baseline has an unsupported schema version")
    if report.get("profile") != profile:
        failures.append("stable baseline profile does not match the candidate profile")
    if report.get("spec_sha256") != expected_spec_sha:
        failures.append("stable baseline spec SHA does not match the candidate contract")
    if report.get("passed") is not True or report.get("failures") != []:
        failures.append("stable baseline was not produced by a passing gate")
    summaries = report.get("summaries")
    if not isinstance(summaries, dict) or not summaries:
        failures.append("stable baseline contains no benchmark summaries")
    else:
        for label, summary in summaries.items():
            if not isinstance(label, str) or not label or not isinstance(summary, dict):
                failures.append("stable baseline contains a malformed benchmark summary")
                continue
            throughput = summary.get("throughput_rows_per_second")
            if (
                not isinstance(throughput, dict)
                or not isinstance(throughput.get("median"), (int, float))
                or isinstance(throughput.get("median"), bool)
                or not math.isfinite(throughput["median"])
                or throughput["median"] <= 0
            ):
                failures.append(f"stable baseline has invalid throughput summary: {label}")
            peak_rss = summary.get("peak_rss_bytes")
            if peak_rss is not None and (
                not isinstance(peak_rss, dict)
                or not isinstance(peak_rss.get("median"), (int, float))
                or isinstance(peak_rss.get("median"), bool)
                or not math.isfinite(peak_rss["median"])
                or peak_rss["median"] < 0
            ):
                failures.append(f"stable baseline has invalid peak RSS summary: {label}")
    return report if len(failures) == failure_count else None


def validate_soak_release(
    results: list[dict[str, Any]],
    spec: dict[str, Any],
    spec_path: Path,
    manifest_path: Path | None,
    result_paths: list[Path],
    failures: list[str],
) -> dict[str, Any] | None:
    """校验 release soak 的顺序、时长、70/30 配比和完整 worker 集合。"""
    if manifest_path is None:
        failures.append("release profile requires --soak-manifest")
        return None
    if not manifest_path.is_file():
        failures.append(f"soak manifest does not exist: {manifest_path}")
        return None
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema_version") != 2:
        failures.append("soak manifest must use schema_version 2")
    soak = spec["mixed_workload"]
    profile = spec["profiles"]["release"]
    gate = spec["gates"]["soak"]
    if gate.get("required_profile") != "release" or manifest.get("profile") != "release":
        failures.append("soak manifest is not bound to the release profile")
    expected_spec_sha = hashlib.sha256(spec_path.read_bytes()).hexdigest()
    if manifest.get("spec_sha256") != expected_spec_sha:
        failures.append("soak manifest spec SHA does not match release contract")
    expected_order = gate["execution_order"]
    if manifest.get("execution_order") != expected_order:
        failures.append("soak execution order does not match release contract")
    if manifest.get("workers") != soak["workers"]:
        failures.append("soak worker count does not match release contract")
    if manifest.get("rows_per_operation") != profile["rows"][-1]:
        failures.append("soak row count does not match release contract")
    if manifest.get("duration_seconds_per_phase") != profile["duration_seconds"]:
        failures.append("soak target duration does not match release contract")
    phases = manifest.get("phases")
    if not isinstance(phases, list) or len(phases) != len(expected_order):
        failures.append("soak manifest does not contain every required phase")
        phases = []
    elif not all(isinstance(phase, dict) for phase in phases):
        failures.append("soak manifest contains a malformed phase")
        phases = []
    phase_intervals: dict[str, list[tuple[int, int, int]]] = defaultdict(list)
    for index, (phase, expected_implementation) in enumerate(zip(phases, expected_order)):
        label = f"soak phase {index}"
        if phase.get("phase_index") != index:
            failures.append(f"{label}: phase index mismatch")
        if phase.get("implementation") != expected_implementation:
            failures.append(f"{label}: implementation order mismatch")
        if phase.get("target_duration_seconds") != profile["duration_seconds"]:
            failures.append(f"{label}: target duration mismatch")
        elapsed_seconds = phase.get("elapsed_seconds")
        if (
            not isinstance(elapsed_seconds, (int, float))
            or isinstance(elapsed_seconds, bool)
            or not math.isfinite(elapsed_seconds)
            or elapsed_seconds < profile["duration_seconds"]
        ):
            failures.append(f"{label}: measured duration is shorter than required")
        counts = phase.get("operation_counts", {})
        if not isinstance(counts, dict):
            failures.append(f"{label}: malformed operation counts")
            counts = {}
        reads = counts.get("read", 0)
        writes = counts.get("write", 0)
        if not isinstance(reads, int) or not isinstance(writes, int) or reads <= 0 or writes <= 0:
            failures.append(f"{label}: missing positive read/write operation counts")
        elif reads * soak["write_weight"] != writes * soak["read_weight"]:
            failures.append(f"{label}: operation mix is not exact 70/30")
        first_trial = phase.get("first_trial")
        last_trial = phase.get("last_trial_exclusive")
        if not isinstance(first_trial, int) or not isinstance(last_trial, int) or last_trial <= first_trial:
            failures.append(f"{label}: empty trial interval")
        else:
            phase_intervals[expected_implementation].append((first_trial, last_trial, index))

    for implementation, intervals in phase_intervals.items():
        previous_end: int | None = None
        for first_trial, last_trial, phase_index in intervals:
            if previous_end is None and first_trial != 0:
                failures.append(f"soak phase {phase_index}: {implementation} trials do not start at zero")
            if previous_end is not None and first_trial != previous_end:
                failures.append(
                    f"soak phase {phase_index}: non-contiguous {implementation} trial interval"
                )
            previous_end = last_trial

    raw_path_value = manifest.get("raw_results")
    raw_path = (
        Path(raw_path_value).resolve()
        if isinstance(raw_path_value, str) and raw_path_value
        else None
    )
    if raw_path is None:
        failures.append("soak manifest lacks raw_results provenance")
    elif raw_path not in {path.resolve() for path in result_paths}:
        failures.append("soak raw_results is not one of the compared result inputs")
    elif manifest.get("raw_results_sha256") != hashlib.sha256(raw_path.read_bytes()).hexdigest():
        failures.append("soak raw_results SHA does not match manifest")
    soak_results = [result for result in results if result.get("phase") == "mixed-soak"]
    if not soak_results:
        failures.append("release profile contains no mixed-soak samples")
        return manifest
    trials: dict[tuple[str, int], list[int]] = defaultdict(list)
    operation_counts: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    phase_operation_counts: dict[int, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for sample in soak_results:
        implementation = sample.get("implementation")
        trial = sample.get("trial")
        worker_id = sample.get("worker_id")
        if not isinstance(implementation, str) or not isinstance(trial, int) or not isinstance(worker_id, int):
            failures.append("mixed-soak sample lacks implementation/trial/worker identity")
            continue
        trials[(implementation, trial)].append(worker_id)
        operation_counts[implementation][sample["operation"]] += 1
        matching_phases = [
            phase_index
            for first_trial, last_trial, phase_index in phase_intervals.get(implementation, [])
            if first_trial <= trial < last_trial
        ]
        if len(matching_phases) != 1:
            failures.append(
                f"mixed-soak sample is not owned by exactly one phase: {implementation}/{trial}"
            )
        else:
            phase_operation_counts[matching_phases[0]][sample["operation"]] += 1
        if sample.get("worker_count") != soak["workers"]:
            failures.append(f"mixed-soak sample has wrong worker count: {implementation}/{trial}")
    expected_workers = list(range(soak["workers"]))
    if gate.get("require_complete_worker_sets"):
        expected_trials = {
            (implementation, trial)
            for implementation, intervals in phase_intervals.items()
            for first_trial, last_trial, _ in intervals
            for trial in range(first_trial, last_trial)
        }
        for missing in sorted(expected_trials - trials.keys()):
            failures.append(f"mixed-soak trial is missing: {missing}")
        for unexpected in sorted(trials.keys() - expected_trials):
            failures.append(f"mixed-soak trial is outside manifest intervals: {unexpected}")
        for key, worker_ids in sorted(trials.items()):
            if sorted(worker_ids) != expected_workers:
                failures.append(
                    f"mixed-soak worker set mismatch: {key}: expected {expected_workers}, "
                    f"got {sorted(worker_ids)}"
                )
    declared_operation_counts = manifest.get("operation_counts", {})
    if not isinstance(declared_operation_counts, dict):
        failures.append("soak manifest has malformed aggregate operation counts")
        declared_operation_counts = {}
    for implementation in ("rust", "java"):
        reads = operation_counts[implementation]["read"]
        writes = operation_counts[implementation]["write"]
        if reads * soak["write_weight"] != writes * soak["read_weight"]:
            failures.append(f"mixed-soak aggregate mix is not exact 70/30: {implementation}")
        declared = declared_operation_counts.get(implementation, {})
        if declared != {"read": reads, "write": writes}:
            failures.append(f"mixed-soak manifest count mismatch: {implementation}")
    for phase_index, phase in enumerate(phases):
        measured = phase_operation_counts[phase_index]
        declared = phase.get("operation_counts", {})
        if declared != {"read": measured["read"], "write": measured["write"]}:
            failures.append(f"mixed-soak phase count mismatch: phase {phase_index}")
    return manifest


def enforce_resource_gates(
    summaries: dict[str, Any], spec: dict[str, Any], failures: list[str]
) -> None:
    """执行 Rust 绝对 RSS 和相对 Java 临时磁盘发布阈值。"""
    gate = spec["gates"].get("resources")
    if not gate:
        failures.append("release spec lacks resource gates")
        return
    for temperature in spec["profiles"]["release"]["temperatures"]:
        for scenario_id in gate["rust_peak_rss_scenarios"]:
            scenario = next(item for item in spec["scenarios"] if item["id"] == scenario_id)
            origins: list[str | None] = [None]
            if scenario["operation"] in ("read", "roundtrip"):
                origins = ["rust", "java"]
            for origin in origins:
                for workers in gate["worker_counts"]:
                    dimensions = ("matrix", temperature, scenario_id, origin, spec["profiles"]["release"]["rows"][-1], workers)
                    rust_label = "/".join(map(str, ("rust", *dimensions)))
                    rust = summaries.get(rust_label)
                    rss = None if rust is None else rust.get("peak_rss_bytes")
                    if rss is None:
                        if gate.get("require_measured_fields"):
                            failures.append(f"missing Rust peak RSS measurement: {rust_label}")
                    elif rss["maximum"] > gate["rust_max_peak_rss_bytes"]:
                        failures.append(f"Rust peak RSS exceeds absolute limit: {rust_label}")

        for scenario_id in gate["temporary_disk_scenarios"]:
            for workers in gate["worker_counts"]:
                dimensions = ("matrix", temperature, scenario_id, None, spec["profiles"]["release"]["rows"][-1], workers)
                rust_label = "/".join(map(str, ("rust", *dimensions)))
                java_label = "/".join(map(str, ("java", *dimensions)))
                rust_temp = summaries.get(rust_label, {}).get("temporary_disk_peak_bytes")
                java_temp = summaries.get(java_label, {}).get("temporary_disk_peak_bytes")
                if rust_temp is None or java_temp is None:
                    if gate.get("require_measured_fields"):
                        failures.append(f"missing temporary disk measurement: {'/'.join(map(str, dimensions))}")
                    continue
                java_peak = java_temp["maximum"]
                rust_peak = rust_temp["maximum"]
                if java_peak <= 0:
                    failures.append(f"Java temporary disk peak is not positive: {java_label}")
                elif rust_peak > java_peak * gate["rust_max_temporary_disk_to_java_ratio"]:
                    failures.append(f"Rust temporary disk ratio exceeds limit: {rust_label}")


def enforce_internal_parallel_map_gate(
    summaries: dict[str, Any], spec: dict[str, Any], failures: list[str]
) -> None:
    """Require 2/4 mapper workers to beat the serial mapper without breaking RSS."""
    contract = spec.get("internal_parallel_map")
    if not isinstance(contract, dict):
        return
    required_fields = {
        "temperature",
        "scenario_id",
        "worker_counts",
        "min_median_speedup",
        "max_peak_rss_bytes",
    }
    if not required_fields.issubset(contract):
        return
    rows = spec["profiles"]["release"]["rows"][-1]
    dimensions = (
        "internal-parallel-map",
        contract["temperature"],
        contract["scenario_id"],
    )
    for origin in ("rust", "java"):
        baseline_label = "/".join(map(str, ("rust", *dimensions, origin, rows, 1)))
        baseline = summaries.get(baseline_label)
        if baseline is None:
            failures.append(f"missing serial mapper baseline: {baseline_label}")
            continue
        baseline_rate = baseline["throughput_rows_per_second"]["median"]
        for workers in contract["worker_counts"]:
            label = "/".join(map(str, ("rust", *dimensions, origin, rows, workers)))
            summary = summaries.get(label)
            if summary is None:
                continue
            rss = summary.get("peak_rss_bytes")
            if rss is None:
                failures.append(f"missing internal parallel-map RSS: {label}")
            elif rss["maximum"] > contract["max_peak_rss_bytes"]:
                failures.append(f"internal parallel-map RSS exceeds limit: {label}")
            if workers == 1:
                continue
            current_rate = summary["throughput_rows_per_second"]["median"]
            speedup = current_rate / baseline_rate if baseline_rate else 0.0
            summary["internal_parallel_map_speedup"] = speedup
            if speedup < contract["min_median_speedup"]:
                failures.append(
                    "internal parallel-map median speedup below "
                    f"{contract['min_median_speedup']:.2f}: {label}"
                )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("results", nargs="+", type=Path)
    parser.add_argument("--spec", required=True, type=Path)
    parser.add_argument("--schema", type=Path)
    parser.add_argument("--profile", required=True, choices=("pr", "nightly", "release"))
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--require-baseline", action="store_true")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--soak-manifest", type=Path)
    parser.add_argument("--expected-java-git-sha")
    parser.add_argument("--expected-rust-git-sha")
    arguments = parser.parse_args()
    spec = json.loads(arguments.spec.read_text(encoding="utf-8"))
    schema_path = arguments.schema or arguments.spec.parent / "benchmark-result-v1.schema.json"
    result_schema = json.loads(schema_path.read_text(encoding="utf-8"))
    gates = spec["gates"]
    input_results = load_results(arguments.results)
    failures: list[str] = []
    validate_spec_semantics(spec, failures)
    results = []
    for index, result in enumerate(input_results, start=1):
        schema_errors = validate_json_schema(result, result_schema)
        if schema_errors:
            failures.extend(f"schema violation at sample {index}: {error}" for error in schema_errors)
        else:
            results.append(result)
    if arguments.profile == "release":
        if not arguments.expected_java_git_sha:
            failures.append("release comparison requires --expected-java-git-sha")
        if not arguments.expected_rust_git_sha:
            failures.append("release comparison requires --expected-rust-git-sha")
        validate_release_environment_manifests(
            arguments.results,
            arguments.spec,
            failures,
            {
                "java": arguments.expected_java_git_sha,
                "rust": arguments.expected_rust_git_sha,
            },
        )
    validate_result_provenance(
        results,
        spec,
        arguments.spec,
        failures,
        {
            "java": arguments.expected_java_git_sha,
            "rust": arguments.expected_rust_git_sha,
        },
    )
    validate_fixture_manifests(arguments.results, spec, failures)
    soak_manifest = None
    if arguments.profile == "release":
        soak_manifest = validate_soak_release(
            results,
            spec,
            arguments.spec,
            arguments.soak_manifest,
            arguments.results,
            failures,
        )
    baseline_report = None
    baseline_required = arguments.require_baseline or arguments.profile == "release"
    if baseline_required and not arguments.baseline:
        failures.append("stable baseline is required for this benchmark layer")
    if arguments.baseline:
        if not arguments.baseline.is_file():
            failures.append(f"stable baseline does not exist: {arguments.baseline}")
        else:
            baseline_report = validate_stable_baseline(
                arguments.baseline,
                arguments.profile,
                arguments.spec,
                failures,
            )
    checksums: dict[tuple[str, str, str, str | None, int, int], set[str]] = defaultdict(set)
    grouped: dict[tuple[str, str, str, str, str | None, int, int], list[dict[str, Any]]] = defaultdict(list)
    for result in results:
        grouped[group_key(result)].append(result)
        checksums[(result["phase"], result["temperature"], result["scenario_id"], result.get("fixture_origin"), result["rows"], result["worker_count"])].add(
            result["correctness"]["checksum"]
        )
        if not result["success"] or result["errors"] != 0:
            failures.append(f"correctness failed: {group_key(result)}")
        if gates["reread_required"] and not result["correctness"]["rereadable"]:
            failures.append(f"reread failed: {group_key(result)}")
    validate_matrix_completeness(grouped, spec, arguments.profile, failures)
    validate_internal_parallel_map_completeness(
        grouped, spec, arguments.profile, failures
    )
    for key, values in checksums.items():
        if len(values) != 1:
            failures.append(f"cross-implementation checksum mismatch: {key}")

    summaries: dict[str, Any] = {}
    for key, samples in grouped.items():
        label = "/".join(map(str, key))
        throughput = summarize([sample["rows_per_second"] for sample in samples])
        latency = summarize([sample["wall_time_ns"] / 1_000_000 for sample in samples])
        rss_values = [sample["peak_rss_bytes"] for sample in samples if sample["peak_rss_bytes"] is not None]
        summaries[label] = {
            "samples": len(samples),
            "success_rate": sum(1 for sample in samples if sample["success"]) / len(samples),
            "error_count": sum(sample["errors"] for sample in samples),
            "throughput_rows_per_second": throughput,
            "throughput_cells_per_second": summarize_present(samples, "cells_per_second"),
            "throughput_mib_per_second": summarize_present(samples, "mib_per_second"),
            "latency_ms": latency,
            "process_wall_time_ns": summarize_present(samples, "process_wall_time_ns"),
            "peak_rss_bytes": summarize(rss_values) if rss_values else None,
            "cpu_user_time_ns": summarize_present(samples, "cpu_user_time_ns"),
            "cpu_system_time_ns": summarize_present(samples, "cpu_system_time_ns"),
            "java_heap_peak_bytes": summarize_present(samples, "java_heap_peak_bytes"),
            "gc_count": summarize_present(samples, "gc_count"),
            "gc_time_ns": summarize_present(samples, "gc_time_ns"),
            "gc_max_pause_ns": summarize_present(samples, "gc_max_pause_ns"),
            "allocator_allocations": summarize_present(samples, "allocator_allocations"),
            "allocator_peak_bytes": summarize_present(samples, "allocator_peak_bytes"),
            "temporary_disk_peak_bytes": summarize_present(samples, "temporary_disk_peak_bytes"),
            "file_size_bytes": summarize_present(samples, "file_size_bytes"),
            "total_written_bytes": summarize_present(samples, "total_written_bytes"),
        }
        summaries[label]["concurrent_rows_per_second"] = summarize_concurrent_throughput(samples)
        stability = summaries[label]["concurrent_rows_per_second"]
        if stability is None:
            failures.append(f"missing concurrency trial data: {label}")
        elif stability["coefficient_of_variation"] > gates["max_coefficient_of_variation"]:
            failures.append(f"unstable throughput environment: {label}")

    # 并发加速比以同实现、同场景、同输入来源、同行数的单 worker 为基线。
    for key in grouped:
        implementation, phase, temperature, scenario_id, fixture_origin, rows, workers = key
        label = "/".join(map(str, key))
        current = summaries[label]["concurrent_rows_per_second"]
        baseline_label = "/".join(
            map(str, (implementation, phase, temperature, scenario_id, fixture_origin, rows, 1))
        )
        baseline_summary = summaries.get(baseline_label)
        if current and baseline_summary and baseline_summary["concurrent_rows_per_second"]:
            base_rate = baseline_summary["concurrent_rows_per_second"]["median"]
            speedup = current["median"] / base_rate if base_rate else 0.0
            summaries[label]["concurrency_speedup"] = speedup
            summaries[label]["concurrency_efficiency"] = speedup / workers
        else:
            summaries[label]["concurrency_speedup"] = None
            summaries[label]["concurrency_efficiency"] = None

    # Java/Rust 比值与 release 发布阈值。原始 trial 先聚合 worker，再做确定性 bootstrap。
    cross_runtime_ratios: dict[str, Any] = {}
    dimensions = {
        (phase, temperature, scenario_id, fixture_origin, rows, workers)
        for _, phase, temperature, scenario_id, fixture_origin, rows, workers in grouped
    }
    for phase, temperature, scenario_id, fixture_origin, rows, workers in sorted(dimensions, key=str):
        rust_label = "/".join(map(str, ("rust", phase, temperature, scenario_id, fixture_origin, rows, workers)))
        java_label = "/".join(map(str, ("java", phase, temperature, scenario_id, fixture_origin, rows, workers)))
        rust = summaries.get(rust_label)
        java = summaries.get(java_label)
        if not rust or not java:
            continue
        ratio_label = "/".join(map(str, (phase, temperature, scenario_id, fixture_origin, rows, workers)))
        ratio = bootstrap_median_ratio(
            trial_throughput_rates(grouped[tuple(["rust", phase, temperature, scenario_id, fixture_origin, rows, workers])]),
            trial_throughput_rates(grouped[tuple(["java", phase, temperature, scenario_id, fixture_origin, rows, workers])]),
            seed=ratio_label,
        )
        ratio["rust_to_java_rows_per_second"] = ratio["median_ratio"]
        ratio["java_to_rust_rows_per_second"] = 1 / ratio["median_ratio"] if ratio["median_ratio"] else None
        cross_runtime_ratios[ratio_label] = ratio

        cross_gate = gates.get("cross_runtime")
        if (
            arguments.profile == "release"
            and cross_gate
            and phase == "matrix"
            and scenario_id in cross_gate["scenarios"]
            and workers in cross_gate["worker_counts"]
        ):
            high_concurrency = workers in cross_gate["high_concurrency_worker_counts"]
            minimum_ratio = (
                cross_gate["min_high_concurrency_median_ratio"]
                if high_concurrency
                else cross_gate["min_median_ratio"]
            )
            if ratio["median_ratio"] < minimum_ratio:
                failures.append(
                    f"Rust/Java median throughput ratio below {minimum_ratio:.2f}: {ratio_label}"
                )
            if not high_concurrency and ratio["confidence_lower_bound"] < cross_gate["min_confidence_lower_bound"]:
                failures.append(
                    "Rust/Java throughput confidence lower bound below "
                    f"{cross_gate['min_confidence_lower_bound']:.2f}: {ratio_label}"
                )

    if arguments.profile == "release":
        enforce_resource_gates(summaries, spec, failures)
        enforce_internal_parallel_map_gate(summaries, spec, failures)

    workload_groups: dict[tuple[str, str, int, int], list[dict[str, Any]]] = defaultdict(list)
    for result in results:
        if result["phase"] == "mixed-soak":
            workload_groups[(
                result["implementation"],
                result["phase"],
                result["rows"],
                result["worker_count"],
            )].append(result)
    workload_summaries: dict[str, Any] = {}
    for key, samples in workload_groups.items():
        trials: dict[int, list[dict[str, Any]]] = defaultdict(list)
        for sample in samples:
            trials[sample["trial"]].append(sample)
        elapsed_ns = sum(
            max(sample["wall_time_ns"] for sample in trial_samples)
            for trial_samples in trials.values()
        )
        total_rows = sum(sample["rows"] for sample in samples)
        read_samples = sum(1 for sample in samples if sample["operation"] == "read")
        total_samples = len(samples)
        workload_summaries["/".join(map(str, key))] = {
            "samples": total_samples,
            "read_operation_ratio": read_samples / total_samples if total_samples else 0.0,
            "write_operation_ratio": 1.0 - (read_samples / total_samples) if total_samples else 0.0,
            "combined_rows_per_second": (
                total_rows / (elapsed_ns / 1_000_000_000) if elapsed_ns else 0.0
            ),
            "success_rate": sum(1 for sample in samples if sample["success"]) / total_samples,
            "error_count": sum(sample["errors"] for sample in samples),
        }

    if baseline_report is not None:
        baseline = baseline_report["summaries"]
        for label, current in summaries.items():
            previous = baseline.get(label)
            if not previous:
                if baseline_required:
                    failures.append(f"stable baseline lacks benchmark summary: {label}")
                continue
            current_rate = current["throughput_rows_per_second"]["median"]
            previous_rate = previous["throughput_rows_per_second"]["median"]
            if current_rate < previous_rate * (1 - gates["max_median_throughput_regression"]):
                failures.append(f"median throughput regression: {label}")
            current_rss = current["peak_rss_bytes"]
            previous_rss = previous["peak_rss_bytes"]
            if current_rss and previous_rss:
                if current_rss["median"] > previous_rss["median"] * (1 + gates["max_peak_rss_regression"]):
                    failures.append(f"peak RSS regression: {label}")

    report = {
        "schema_version": 1,
        "profile": arguments.profile,
        "spec_sha256": hashlib.sha256(arguments.spec.read_bytes()).hexdigest(),
        "sample_count": len(input_results),
        "valid_sample_count": len(results),
        "summaries": summaries,
        "workload_summaries": workload_summaries,
        "cross_runtime_ratios": cross_runtime_ratios,
        "soak_manifest": soak_manifest,
        "failures": failures,
        "passed": not failures,
    }
    encoded = json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True)
    if arguments.output:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
