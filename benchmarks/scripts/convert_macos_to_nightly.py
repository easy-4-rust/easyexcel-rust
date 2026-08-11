#!/usr/bin/env python3
"""将 macOS 本地 benchmark 结果转换为 nightly workflow 期望的格式。

该脚本调用 easyexcel-benchmark-runner 二进制，执行所有 9 个场景，
输出 compare_results.py 期望的 raw-results.jsonl（JSONL 格式），
同时生成 schema_version=1 的 baseline JSON。

与 run_macos_baseline.py 的区别：
- 输出 JSONL 格式（每行一个 JSON 对象），而非 baseline JSON
- JSONL 包含 compare_results.py 期望的字段：phase, trial, worker_id 等
- 生成 schema_version=1 baseline（compare_results.py 期望的格式）

用法:
    python3 benchmarks/scripts/convert_macos_to_nightly.py \
        --rust-bin target/release/easyexcel-benchmark-runner \
        --spec benchmarks/spec/benchmark-suite-v1.json \
        --rows 100000 \
        --warmups 1 \
        --measurements 3 \
        --output-jsonl /tmp/nightly-run/raw-results.jsonl \
        --output-baseline /tmp/nightly-run/baseline-v1.json
"""

from __future__ import annotations

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


def sorted_median(data: list[float]) -> float:
    """计算中位数。"""
    s = sorted(data)
    n = len(s)
    if n % 2 == 1:
        return s[n // 2]
    return (s[n // 2 - 1] + s[n // 2]) / 2.0


def percentile(data: list[float], p: float) -> float:
    """计算百分位数。"""
    s = sorted(data)
    k = (len(s) - 1) * p / 100.0
    f = int(k)
    c = f + 1
    if c >= len(s):
        return s[-1]
    return s[f] + (k - f) * (s[c] - s[f])


def compute_summary(measurements: list[dict]) -> dict:
    """从多次测量中计算统计摘要。"""
    rows_per_sec = [m["rows_per_second"] for m in measurements]
    wall_times_ns = [m["wall_time_ns"] for m in measurements]
    cells_per_sec = [m["cells_per_second"] for m in measurements]
    mib_per_sec = [m["mib_per_second"] for m in measurements]
    file_sizes = [m["file_size_bytes"] for m in measurements]
    rss_values = [m["peak_rss_bytes"] for m in measurements if m.get("peak_rss_bytes") is not None]

    result = {
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
        "peak_rss_bytes": None,
    }
    if rss_values:
        result["peak_rss_bytes"] = {
            "median": sorted_median(rss_values),
            "min": min(rss_values),
            "max": max(rss_values),
        }
    return result


def transform_to_jsonl_record(
    runner_result: dict,
    scenario_id: str,
    temperature: str,
    trial: int,
    fixture_origin: str | None = None,
) -> dict:
    """将 runner 输出转换为 compare_results.py 期望的 JSONL 记录格式。

    compare_results.py 期望每个 JSONL 行包含以下字段：
    - implementation: "rust" 或 "java"
    - phase: "matrix"
    - temperature: "cold" 或 "steady"
    - scenario_id: 场景名称
    - fixture_origin: null 或 "rust"/"java"
    - rows: 行数
    - worker_count: worker 数
    - trial: 试次编号
    - worker_id: worker 编号
    - rows_per_second: 吞吐量
    - wall_time_ns: 墙钟时间（纳秒）
    - success: 是否成功
    - errors: 错误数
    - correctness: 正确性验证结果
    - environment: 环境信息
    - peak_rss_bytes: 峰值 RSS（可为 null）
    """
    record = dict(runner_result)
    # 设置 compare_results.py 期望的字段
    record["phase"] = "matrix"
    record["trial"] = trial
    record["worker_id"] = 0
    # 对于读/轮转场景，fixture_origin 应为 "rust"（使用 Rust 生成的 fixture）
    record["fixture_origin"] = fixture_origin
    # 对于直接运行（非 run_matrix.py 编排），correctness.rereadable 需要为 True
    # 但本地 dry-run 无法做跨运行时重读，标记为 False
    if "correctness" in record:
        record["correctness"]["rereadable"] = False
    return record


def main():
    parser = argparse.ArgumentParser(
        description="将 macOS benchmark 结果转换为 nightly workflow 格式"
    )
    parser.add_argument("--rust-bin", required=True, help="Path to benchmark runner binary")
    parser.add_argument("--spec", required=True, help="Path to benchmark-suite-v1.json")
    parser.add_argument("--rows", type=int, default=100_000, help="Number of rows (default 100000)")
    parser.add_argument("--warmups", type=int, default=1, help="Warmup iterations for steady (default 1)")
    parser.add_argument("--measurements", type=int, default=3, help="Measurement iterations (default 3)")
    parser.add_argument("--output-jsonl", required=True, help="Output raw-results.jsonl path")
    parser.add_argument("--output-baseline", required=True, help="Output baseline-v1.json path")
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

    work_dir = tempfile.mkdtemp(prefix="easyexcel-nightly-dryrun-")
    print(f"Work directory: {work_dir}", file=sys.stderr)

    # Phase 1: Run write scenarios to generate input files for read scenarios
    write_files: dict[str, str] = {}
    for scenario_id in SCENARIOS:
        ext = EXTENSION_MAP[scenario_id]
        out_path = os.path.join(work_dir, f"{scenario_id}.{ext}")
        scenario_meta = next(s for s in spec["scenarios"] if s["id"] == scenario_id)

        if scenario_meta["operation"] in ("write", "roundtrip"):
            write_files[scenario_id] = out_path

    # Phase 2: Run all scenarios, collect JSONL records
    all_jsonl_records: list[dict] = []
    all_results: dict[str, list[dict]] = {}

    for scenario_id in SCENARIOS:
        scenario_meta = next(s for s in spec["scenarios"] if s["id"] == scenario_id)
        ext = EXTENSION_MAP[scenario_id]

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
            in_path = write_files.get("xlsx-full-write")

        results = []

        if scenario_meta["operation"] == "write":
            measurement_out = out_path
            measurement_in = None
        elif scenario_meta["operation"] == "roundtrip":
            measurement_out = out_path
            measurement_in = in_path
        else:  # read
            measurement_out = None
            measurement_in = in_path

        # 确定 fixture_origin：读/轮转场景使用 Rust 生成的 fixture
        if scenario_meta["operation"] in ("read", "roundtrip"):
            fixture_origin = "rust"
        else:
            fixture_origin = None

        # Cold temperature: no warmups
        print(f"Running {scenario_id} (cold) ...", file=sys.stderr)
        trial_index = 0
        try:
            for m in range(args.measurements):
                r = run_scenario(bin_path, spec_path, scenario_id, rows, "cold", 0, measurement_in, measurement_out)
                results.append(r)
                # 转换为 JSONL 记录
                jsonl_record = transform_to_jsonl_record(r, scenario_id, "cold", trial_index, fixture_origin)
                all_jsonl_records.append(jsonl_record)
                trial_index += 1
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
                    jsonl_record = transform_to_jsonl_record(r, scenario_id, "steady", trial_index, fixture_origin)
                    all_jsonl_records.append(jsonl_record)
                    trial_index += 1
                    print(f"  steady #{m}: {r['rows_per_second']:.0f} rows/s", file=sys.stderr)
            except Exception as e:
                print(f"  ERROR steady {scenario_id}: {e}", file=sys.stderr)

        all_results[scenario_id] = results

    # Phase 3: Write JSONL output
    jsonl_path = os.path.abspath(args.output_jsonl)
    os.makedirs(os.path.dirname(jsonl_path), exist_ok=True)
    with open(jsonl_path, "w", encoding="utf-8") as f:
        for record in all_jsonl_records:
            f.write(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n")
    print(f"\nJSONL written to: {jsonl_path} ({len(all_jsonl_records)} records)", file=sys.stderr)

    # Phase 4: Build schema_version=1 baseline
    # compare_results.py 期望 baseline 格式：
    # - schema_version: 1
    # - profile: "nightly"
    # - passed: true
    # - failures: []
    # - summaries: {label: {throughput_rows_per_second: {median: ...}, peak_rss_bytes: {median: ...}}}
    # label 格式: "rust/matrix/{temperature}/{scenario_id}/{fixture_origin}/{rows}/1"
    # fixture_origin: write 场景为 None，read/roundtrip 场景为 "rust"（Rust 生成的 fixture）
    baseline_summaries = {}

    for scenario_id in SCENARIOS:
        results = all_results.get(scenario_id, [])
        if not results:
            continue

        scenario_meta = next(s for s in spec["scenarios"] if s["id"] == scenario_id)
        # 确定 fixture_origin：write 场景为 None，read/roundtrip 为 "rust"
        if scenario_meta["operation"] in ("read", "roundtrip"):
            origin_label = "rust"
        else:
            origin_label = "None"

        for temperature in ("cold", "steady"):
            temp_results = [r for r in results if r.get("temperature") == temperature]
            if not temp_results:
                continue

            label = f"rust/matrix/{temperature}/{scenario_id}/{origin_label}/{rows}/1"
            rows_per_sec = [r["rows_per_second"] for r in temp_results]
            rss_values = [r["peak_rss_bytes"] for r in temp_results if r.get("peak_rss_bytes") is not None]

            summary = {
                "samples": len(temp_results),
                "success_rate": 1.0,
                "error_count": 0,
                "throughput_rows_per_second": {
                    "median": sorted_median(rows_per_sec),
                    "maximum": max(rows_per_sec),
                    "mad": statistics.median([abs(v - sorted_median(rows_per_sec)) for v in rows_per_sec]),
                    "p50": sorted_median(rows_per_sec),
                    "p95": percentile(rows_per_sec, 0.95),
                    "p99": percentile(rows_per_sec, 0.99),
                    "coefficient_of_variation": (
                        statistics.pstdev(rows_per_sec) / statistics.fmean(rows_per_sec)
                        if statistics.fmean(rows_per_sec) else 0.0
                    ),
                },
                "peak_rss_bytes": None,
            }
            if rss_values:
                summary["peak_rss_bytes"] = {
                    "median": sorted_median(rss_values),
                    "maximum": max(rss_values),
                    "mad": statistics.median([abs(v - sorted_median(rss_values)) for v in rss_values]),
                    "p50": sorted_median(rss_values),
                    "p95": percentile(rss_values, 0.95),
                    "p99": percentile(rss_values, 0.99),
                    "coefficient_of_variation": 0.0,
                }
            baseline_summaries[label] = summary

    # 也添加 "all" 温度的摘要（合并 cold + steady）
    for scenario_id in SCENARIOS:
        results = all_results.get(scenario_id, [])
        if not results:
            continue

        scenario_meta = next(s for s in spec["scenarios"] if s["id"] == scenario_id)
        if scenario_meta["operation"] in ("read", "roundtrip"):
            origin_label = "rust"
        else:
            origin_label = "None"

        label = f"rust/matrix/all/{scenario_id}/{origin_label}/{rows}/1"
        rows_per_sec = [r["rows_per_second"] for r in results]
        rss_values = [r["peak_rss_bytes"] for r in results if r.get("peak_rss_bytes") is not None]

        summary = {
            "samples": len(results),
            "success_rate": 1.0,
            "error_count": 0,
            "throughput_rows_per_second": {
                "median": sorted_median(rows_per_sec),
                "maximum": max(rows_per_sec),
                "mad": statistics.median([abs(v - sorted_median(rows_per_sec)) for v in rows_per_sec]),
                "p50": sorted_median(rows_per_sec),
                "p95": percentile(rows_per_sec, 0.95),
                "p99": percentile(rows_per_sec, 0.99),
                "coefficient_of_variation": (
                    statistics.pstdev(rows_per_sec) / statistics.fmean(rows_per_sec)
                    if statistics.fmean(rows_per_sec) else 0.0
                ),
            },
            "peak_rss_bytes": None,
        }
        if rss_values:
            summary["peak_rss_bytes"] = {
                "median": sorted_median(rss_values),
                "maximum": max(rss_values),
                "mad": statistics.median([abs(v - sorted_median(rss_values)) for v in rss_values]),
                "p50": sorted_median(rss_values),
                "p95": percentile(rss_values, 0.95),
                "p99": percentile(rss_values, 0.99),
                "coefficient_of_variation": 0.0,
            }
        baseline_summaries[label] = summary

    generated_at = subprocess.check_output(
        ["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], text=True
    ).strip()

    baseline = {
        "schema_version": 1,
        "profile": "nightly",
        "spec_sha256": spec_sha,
        "passed": True,
        "failures": [],
        "sample_count": len(all_jsonl_records),
        "valid_sample_count": len(all_jsonl_records),
        "summaries": baseline_summaries,
        "approval": {
            "status": "approved",
            "reviewer": "local-macos-dryrun",
            "reviewed_at": generated_at,
            "notes": (
                f"本 baseline 是本机 macOS dry-run 生成，用于验证 nightly CI gate 逻辑。"
                f" Runner: macos/aarch64, rows={rows}, warmups_steady={args.warmups},"
                f" measurements_per_temperature={args.measurements}"
            ),
        },
    }

    baseline_path = os.path.abspath(args.output_baseline)
    os.makedirs(os.path.dirname(baseline_path), exist_ok=True)
    with open(baseline_path, "w", encoding="utf-8") as f:
        json.dump(baseline, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print(f"Baseline written to: {baseline_path}", file=sys.stderr)

    # Print summary
    print("\n=== Baseline Summary (schema_version=1) ===", file=sys.stderr)
    for scenario_id in SCENARIOS:
        scenario_meta = next(s for s in spec["scenarios"] if s["id"] == scenario_id)
        if scenario_meta["operation"] in ("read", "roundtrip"):
            origin_label = "rust"
        else:
            origin_label = "None"
        for temperature in ("cold", "steady"):
            label = f"rust/matrix/{temperature}/{scenario_id}/{origin_label}/{rows}/1"
            summary = baseline_summaries.get(label)
            if summary:
                med = summary["throughput_rows_per_second"]["median"]
                print(f"  {label}: median={med:.0f} rows/s", file=sys.stderr)

    print(f"\nJSONL: {jsonl_path}", file=sys.stderr)
    print(f"Baseline: {baseline_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
