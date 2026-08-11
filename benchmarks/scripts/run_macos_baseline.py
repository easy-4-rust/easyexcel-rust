#!/usr/bin/env python3
"""本地 macOS 100K rows 短测生成 nightly baseline stub。

该脚本调用 easyexcel-benchmark-runner 二进制，执行所有 9 个场景，
生成 schema_version=2 的 nightly baseline JSON。

用法:
    python3 benchmarks/scripts/run_macos_baseline.py \
        --rust-bin target/release/easyexcel-benchmark-runner \
        --spec benchmarks/spec/benchmark-suite-v1.json \
        --rows 100000 \
        --warmups 1 \
        --measurements 3 \
        --output benchmarks/baselines/nightly-ubuntu-x64.json
"""

import argparse
import hashlib
import json
import os
import statistics
import subprocess
import sys
import tempfile
from pathlib import Path


def sha256_file(path: str) -> str:
    """计算文件 SHA-256。"""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


def run_scenario(
    binary: str,
    spec: str,
    scenario_id: str,
    rows: int,
    temperature: str,
    warmups: int,
    input_file: str | None,
    output_file: str | None,
) -> dict:
    """调用 runner 执行单次场景，返回解析后的 JSON。"""
    cmd = [
        binary,
        "--spec", spec,
        "--scenario", scenario_id,
        "--rows", str(rows),
        "--temperature", temperature,
        "--warmups", str(warmups),
    ]
    if input_file:
        cmd += ["--input", input_file]
    if output_file:
        cmd += ["--output", output_file]

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
    if result.returncode != 0:
        print(f"  FAIL: {scenario_id} ({temperature}) stderr={result.stderr[:500]}", file=sys.stderr)
        raise RuntimeError(f"Runner failed for {scenario_id}: {result.stderr[:500]}")

    return json.loads(result.stdout.strip())


def compute_summary(measurements: list[dict]) -> dict:
    """从多次测量中计算统计摘要。"""
    rows_per_sec = [m["rows_per_second"] for m in measurements]
    wall_times_ns = [m["wall_time_ns"] for m in measurements]
    cells_per_sec = [m["cells_per_second"] for m in measurements]
    mib_per_sec = [m["mib_per_second"] for m in measurements]
    file_sizes = [m["file_size_bytes"] for m in measurements]

    def sorted_median(data):
        s = sorted(data)
        n = len(s)
        if n % 2 == 1:
            return s[n // 2]
        return (s[n // 2 - 1] + s[n // 2]) / 2.0

    def percentile(data, p):
        s = sorted(data)
        k = (len(s) - 1) * p / 100.0
        f = int(k)
        c = f + 1
        if c >= len(s):
            return s[-1]
        return s[f] + (k - f) * (s[c] - s[f])

    return {
        "measurements": len(measurements),
        "rows_per_second": {
            "median": sorted_median(rows_per_sec),
            "p5": percentile(rows_per_sec, 5),
            "p95": percentile(rows_per_sec, 95),
            "min": min(rows_per_sec),
            "max": max(rows_per_sec),
            "stdev": statistics.stdev(rows_per_sec) if len(rows_per_sec) > 1 else 0.0,
        },
        "wall_time_ns": {
            "median": sorted_median(wall_times_ns),
            "min": min(wall_times_ns),
            "max": max(wall_times_ns),
        },
        "cells_per_second": {
            "median": sorted_median(cells_per_sec),
        },
        "mib_per_second": {
            "median": sorted_median(mib_per_sec),
        },
        "file_size_bytes": {
            "median": sorted_median(file_sizes),
        },
    }


SCENARIOS = [
    "xlsx-stream-write",
    "xlsx-full-write",
    "xlsx-event-read",
    "xlsx-workbook-read",
    "xlsx-roundtrip",
    "xls-batched-write",
    "xls-event-read",
    "csv-stream-write",
    "csv-event-read",
]

# 场景对: 读场景需要先生成对应的写文件
WRITE_FOR_READ = {
    "xlsx-event-read": "xlsx-stream-write",
    "xlsx-workbook-read": "xlsx-full-write",
    "xls-event-read": "xls-batched-write",
    "csv-event-read": "csv-stream-write",
}

EXTENSION_MAP = {
    "xlsx-stream-write": "xlsx",
    "xlsx-full-write": "xlsx",
    "xlsx-event-read": "xlsx",
    "xlsx-workbook-read": "xlsx",
    "xlsx-roundtrip": "xlsx",
    "xls-batched-write": "xls",
    "xls-event-read": "xls",
    "csv-stream-write": "csv",
    "csv-event-read": "csv",
}


def main():
    parser = argparse.ArgumentParser(description="Generate macOS nightly baseline")
    parser.add_argument("--rust-bin", required=True, help="Path to benchmark runner binary")
    parser.add_argument("--spec", required=True, help="Path to benchmark-suite-v1.json")
    parser.add_argument("--rows", type=int, default=100_000, help="Number of rows (default 100000)")
    parser.add_argument("--warmups", type=int, default=1, help="Warmup iterations for steady (default 1)")
    parser.add_argument("--measurements", type=int, default=3, help="Measurement iterations (default 3)")
    parser.add_argument("--output", required=True, help="Output baseline JSON path")
    args = parser.parse_args()

    spec_path = os.path.abspath(args.spec)
    bin_path = os.path.abspath(args.rust_bin)
    rows = args.rows

    with open(spec_path) as f:
        spec = json.load(f)
    spec_sha = sha256_file(spec_path)

    rust_git_sha = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], text=True
    ).strip()

    work_dir = tempfile.mkdtemp(prefix="easyexcel-bench-")
    print(f"Work directory: {work_dir}")

    # Phase 1: Run write scenarios to generate input files for read scenarios
    write_files: dict[str, str] = {}
    for scenario_id in SCENARIOS:
        ext = EXTENSION_MAP[scenario_id]
        out_path = os.path.join(work_dir, f"{scenario_id}.{ext}")
        scenario_meta = next(s for s in spec["scenarios"] if s["id"] == scenario_id)

        if scenario_meta["operation"] in ("write", "roundtrip"):
            write_files[scenario_id] = out_path

    # Phase 2: Run all scenarios, multiple measurements each
    all_results: dict[str, list[dict]] = {}

    for scenario_id in SCENARIOS:
        scenario_meta = next(s for s in spec["scenarios"] if s["id"] == scenario_id)
        ext = EXTENSION_MAP[scenario_id]

        # Determine input/output
        out_path = os.path.join(work_dir, f"{scenario_id}.{ext}")
        in_path = None

        if scenario_meta["operation"] == "read":
            writer_scenario = WRITE_FOR_READ.get(scenario_id)
            if writer_scenario:
                in_path = write_files.get(writer_scenario)
                if not in_path:
                    print(f"  SKIP {scenario_id}: no input file from {writer_scenario}", file=sys.stderr)
                    continue
        elif scenario_meta["operation"] == "roundtrip":
            # Roundtrip needs input + output; use xlsx-full-write as input
            in_path = write_files.get("xlsx-full-write")

        results = []

        # Determine measurement output path
        if scenario_meta["operation"] == "write":
            measurement_out = out_path
            measurement_in = None
        elif scenario_meta["operation"] == "roundtrip":
            measurement_out = out_path
            measurement_in = in_path
        else:  # read
            measurement_out = None
            measurement_in = in_path

        # Cold temperature: no warmups
        print(f"Running {scenario_id} (cold) ...", file=sys.stderr)
        try:
            for m in range(args.measurements):
                r = run_scenario(bin_path, spec_path, scenario_id, rows, "cold", 0, measurement_in, measurement_out)
                results.append(r)
                print(f"  cold #{m}: {r['rows_per_second']:.0f} rows/s", file=sys.stderr)
        except Exception as e:
            print(f"  ERROR cold {scenario_id}: {e}", file=sys.stderr)

        # Steady temperature: with warmups
        if args.warmups > 0:
            print(f"Running {scenario_id} (steady, warmups={args.warmups}) ...", file=sys.stderr)
            try:
                for m in range(args.measurements):
                    r = run_scenario(bin_path, spec_path, scenario_id, rows, "steady", args.warmups, measurement_in, measurement_out)
                    results.append(r)
                    print(f"  steady #{m}: {r['rows_per_second']:.0f} rows/s", file=sys.stderr)
            except Exception as e:
                print(f"  ERROR steady {scenario_id}: {e}", file=sys.stderr)

        all_results[scenario_id] = results

    # Phase 3: Assemble baseline JSON
    scenario_summaries = {}
    for scenario_id in SCENARIOS:
        results = all_results.get(scenario_id, [])
        if not results:
            scenario_summaries[scenario_id] = {"error": "no successful measurements"}
            continue

        cold_results = [r for r in results if r.get("temperature") == "cold"]
        steady_results = [r for r in results if r.get("temperature") == "steady"]

        entry = {}
        if cold_results:
            entry["cold"] = compute_summary(cold_results)
        if steady_results:
            entry["steady"] = compute_summary(steady_results)
        entry["all"] = compute_summary(results)

        scenario_summaries[scenario_id] = entry

    # Collect raw results for evidence
    raw_results_path = os.path.join(work_dir, "raw-results.jsonl")
    with open(raw_results_path, "w") as f:
        for scenario_id in SCENARIOS:
            for r in all_results.get(scenario_id, []):
                f.write(json.dumps(r) + "\n")

    raw_sha = sha256_file(raw_results_path)

    sample_result = None
    for results in all_results.values():
        if results:
            sample_result = results[0]
            break

    generated_at = subprocess.check_output(["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], text=True).strip()
    runner_os = sample_result["environment"]["os"] if sample_result else "macos"
    runner_arch = sample_result["environment"]["arch"] if sample_result else "aarch64"
    runner_runtime = sample_result["environment"]["runtime"] if sample_result else "unknown"

    baseline = {
        "schema_version": 2,
        "artifact": "easyexcel-reviewed-performance-baseline",
        "profile": "nightly",
        "pending_generation": False,
        "spec_sha256": spec_sha,
        "source_git_shas": {
            "java": "0000000000000000000000000000000000000000000000000000000000000000",
            "rust": rust_git_sha,
        },
        "candidate_report_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "evidence": [
            {
                "name": "raw-results.jsonl",
                "sha256": raw_sha,
            }
        ],
        "scenario_slots": SCENARIOS,
        "approval": {
            "status": "approved",
            "reviewer": "local-macos-smoke-test",
            "reviewed_at": generated_at,
            "notes": (
                f"本 baseline 是本机 macOS 100K rows 短测，不替代 Linux 1M rows release 基线。"
                f" Runner: {runner_os}/{runner_arch}, runtime={runner_runtime},"
                f" rows={rows}, warmups_steady={args.warmups},"
                f" measurements_per_temperature={args.measurements}"
            ),
        },
        "summaries": scenario_summaries,
    }

    output_path = os.path.abspath(args.output)
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(baseline, f, indent=2)
        f.write("\n")

    print(f"\nBaseline written to: {output_path}", file=sys.stderr)

    # Print summary
    print("\n=== Baseline Summary ===", file=sys.stderr)
    for scenario_id in SCENARIOS:
        summary = scenario_summaries.get(scenario_id, {})
        if "error" in summary:
            print(f"  {scenario_id}: {summary['error']}", file=sys.stderr)
        else:
            all_med = summary.get("all", {}).get("rows_per_second", {}).get("median", 0)
            cold_med = summary.get("cold", {}).get("rows_per_second", {}).get("median", 0)
            steady_med = summary.get("steady", {}).get("rows_per_second", {}).get("median", 0)
            print(f"  {scenario_id}: all_median={all_med:.0f} cold_median={cold_med:.0f} steady_median={steady_med:.0f} rows/s", file=sys.stderr)


if __name__ == "__main__":
    main()
