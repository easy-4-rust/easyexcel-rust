#!/usr/bin/env python3
"""Aggregate Java/Rust BenchmarkResult JSONL and enforce stable-baseline gates."""

from __future__ import annotations

import argparse
from collections import defaultdict
import json
import math
from pathlib import Path
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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("results", nargs="+", type=Path)
    parser.add_argument("--spec", required=True, type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--require-baseline", action="store_true")
    parser.add_argument("--output", type=Path)
    arguments = parser.parse_args()
    spec = json.loads(arguments.spec.read_text(encoding="utf-8"))
    gates = spec["gates"]
    results = load_results(arguments.results)
    failures: list[str] = []
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
        trials: dict[int, list[dict[str, Any]]] = defaultdict(list)
        for sample in samples:
            if sample.get("trial") is not None:
                trials[sample["trial"]].append(sample)
        concurrent_rates = []
        for trial_samples in trials.values():
            elapsed_ns = max(sample["wall_time_ns"] for sample in trial_samples)
            total_rows = sum(sample["rows"] for sample in trial_samples)
            concurrent_rates.append(total_rows / (elapsed_ns / 1_000_000_000))
        summaries[label]["concurrent_rows_per_second"] = (
            summarize(concurrent_rates) if concurrent_rates else None
        )
        if throughput["coefficient_of_variation"] > gates["max_coefficient_of_variation"]:
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

    # Java/Rust 比值仅展示，不作为任一实现必须获胜的门禁。
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
        rust_rate = rust["concurrent_rows_per_second"]["median"]
        java_rate = java["concurrent_rows_per_second"]["median"]
        ratio_label = "/".join(map(str, (phase, temperature, scenario_id, fixture_origin, rows, workers)))
        cross_runtime_ratios[ratio_label] = {
            "rust_to_java_rows_per_second": rust_rate / java_rate if java_rate else None,
            "java_to_rust_rows_per_second": java_rate / rust_rate if rust_rate else None,
        }

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
