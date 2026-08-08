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
        expected_workers = list(range(workers))
        for trial, worker_ids in sorted(trials.items()):
            if sorted(worker_ids) != expected_workers:
                failures.append(
                    f"worker set mismatch: {label}/trial-{trial}: "
                    f"expected {expected_workers}, got {sorted(worker_ids)}"
                )


def validate_result_provenance(
    results: list[dict[str, Any]], spec: dict[str, Any], spec_path: Path, failures: list[str]
) -> None:
    """Ensure every sample belongs to this exact shared contract and a real build."""
    expected_spec_sha = hashlib.sha256(spec_path.read_bytes()).hexdigest()
    for index, result in enumerate(results, start=1):
        environment = result.get("environment") or {}
        if environment.get("spec_sha256") != expected_spec_sha:
            failures.append(f"spec SHA mismatch at sample {index}")
        if environment.get("git_sha") in (None, "", "unknown"):
            failures.append(f"unknown implementation Git SHA at sample {index}")
        runtime = environment.get("runtime", "")
        contract = spec["runtime_contract"]
        implementation = result.get("implementation")
        if implementation == "java" and not runtime.startswith(contract["java_version"]):
            failures.append(f"Java runtime contract mismatch at sample {index}")
        if implementation == "rust" and contract["rust_toolchain"] not in runtime:
            failures.append(f"Rust runtime contract mismatch at sample {index}")
        origin = result.get("fixture_origin")
        operation = result.get("operation")
        if result.get("phase") == "matrix":
            if operation in ("read", "roundtrip") and origin not in ("rust", "java"):
                failures.append(f"missing fixture origin at sample {index}")
            if operation == "write" and origin is not None:
                failures.append(f"unexpected fixture origin at sample {index}")
            if origin is not None and not result.get("input_sha256"):
                failures.append(f"missing input SHA at sample {index}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("results", nargs="+", type=Path)
    parser.add_argument("--spec", required=True, type=Path)
    parser.add_argument("--schema", type=Path)
    parser.add_argument("--profile", required=True, choices=("pr", "nightly", "release"))
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--require-baseline", action="store_true")
    parser.add_argument("--output", type=Path)
    arguments = parser.parse_args()
    spec = json.loads(arguments.spec.read_text(encoding="utf-8"))
    schema_path = arguments.schema or arguments.spec.parent / "benchmark-result-v1.schema.json"
    result_schema = json.loads(schema_path.read_text(encoding="utf-8"))
    gates = spec["gates"]
    input_results = load_results(arguments.results)
    failures: list[str] = []
    results = []
    for index, result in enumerate(input_results, start=1):
        schema_errors = validate_json_schema(result, result_schema)
        if schema_errors:
            failures.extend(f"schema violation at sample {index}: {error}" for error in schema_errors)
        else:
            results.append(result)
    validate_result_provenance(results, spec, arguments.spec, failures)
    if arguments.require_baseline and not arguments.baseline:
        failures.append("stable baseline is required for this benchmark layer")
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
        if stability["coefficient_of_variation"] > gates["max_coefficient_of_variation"]:
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

    if arguments.baseline:
        baseline = json.loads(arguments.baseline.read_text(encoding="utf-8"))["summaries"]
        for label, current in summaries.items():
            previous = baseline.get(label)
            if not previous:
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
