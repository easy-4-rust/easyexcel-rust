#!/usr/bin/env python3
"""Run the deterministic 70/30 read/write release soak outside build processes."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import time

import run_matrix


ROOT = Path(__file__).resolve().parents[2]


def scenario(spec: dict, scenario_id: str) -> dict:
    for value in spec["scenarios"]:
        if value["id"] == scenario_id:
            return value
    raise KeyError(f"missing soak scenario {scenario_id}")


def run_operation(
    implementation: str,
    arguments: argparse.Namespace,
    selected: dict,
    rows: int,
    workers: int,
    trial: int,
    fixture: Path,
    measured: bool,
    warmups: int,
) -> list[dict]:
    is_read = selected["operation"] == "read"
    return run_matrix.run_group(
        implementation,
        arguments,
        selected,
        rows,
        workers,
        trial,
        "rust" if is_read else None,
        fixture if is_read else None,
        measured,
        temperature="steady",
        warmups=warmups,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--spec",
        type=Path,
        default=ROOT / "benchmarks/spec/benchmark-suite-v1.json",
    )
    parser.add_argument("--rust-bin", type=Path, required=True)
    parser.add_argument("--java-bin", type=Path, default=Path("java"))
    parser.add_argument("--java-classpath", required=True)
    parser.add_argument("--java-xms", default="512m")
    parser.add_argument("--java-xmx", default="4g")
    parser.add_argument("--java-repo", type=Path)
    parser.add_argument("--rust-repo", type=Path, default=ROOT)
    parser.add_argument("--artifact-manifest", type=Path)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--duration-seconds", type=int)
    parser.add_argument("--rows", type=int)
    parser.add_argument("--workers", type=int)
    arguments = parser.parse_args()
    arguments.java_git_sha = run_matrix.git_sha(arguments.java_repo)

    spec = json.loads(arguments.spec.read_text(encoding="utf-8"))
    profile = spec["profiles"]["release"]
    soak = spec["mixed_workload"]
    rows = arguments.rows or profile["rows"][-1]
    workers = arguments.workers or soak["workers"]
    duration_seconds = arguments.duration_seconds or profile["duration_seconds"]
    read_scenario = scenario(spec, soak["read_scenario"])
    write_scenario = scenario(spec, soak["write_scenario"])
    cycle = [read_scenario] * soak["read_weight"] + [write_scenario] * soak["write_weight"]

    arguments.output_dir.mkdir(parents=True, exist_ok=True)
    run_matrix.validate_runtime_contract(spec, arguments)
    run_matrix.validate_release_inputs(arguments)
    run_matrix.write_environment_manifest(arguments, spec)
    fixtures = run_matrix.create_fixtures(spec, arguments, rows, soak["format"])
    fixture = fixtures["rust"]
    raw_path = arguments.output_dir / "raw-results.jsonl"
    counts = {"rust": {"read": 0, "write": 0}, "java": {"read": 0, "write": 0}}
    next_trial = {"rust": 0, "java": 0}
    order = run_matrix.execution_order(soak["measurements_per_implementation"])
    phases = []

    with raw_path.open("w", encoding="utf-8") as raw:
        for phase_index, implementation in enumerate(order):
            phase_started = time.monotonic()
            deadline = phase_started + duration_seconds
            trial = next_trial[implementation]
            first_trial = trial
            phase_counts = {"read": 0, "write": 0}
            # Always finish a complete ten-operation cycle so the measured row mix is exactly 70/30.
            while time.monotonic() < deadline:
                for selected in cycle:
                    results = run_operation(
                        implementation,
                        arguments,
                        selected,
                        rows,
                        workers,
                        trial,
                        fixture,
                        measured=True,
                        warmups=profile["warmups"],
                    )
                    counts[implementation][selected["operation"]] += len(results)
                    phase_counts[selected["operation"]] += len(results)
                    for result in results:
                        result["phase"] = "mixed-soak"
                        raw.write(
                            json.dumps(result, ensure_ascii=False, separators=(",", ":"))
                            + "\n"
                        )
                    raw.flush()
                    trial += 1
            next_trial[implementation] = trial
            phases.append(
                {
                    "phase_index": phase_index,
                    "implementation": implementation,
                    "target_duration_seconds": duration_seconds,
                    "elapsed_seconds": time.monotonic() - phase_started,
                    "first_trial": first_trial,
                    "last_trial_exclusive": trial,
                    "operation_counts": phase_counts,
                }
            )

    manifest = {
        "schema_version": 2,
        "profile": "release",
        "spec_sha256": hashlib.sha256(arguments.spec.read_bytes()).hexdigest(),
        "duration_seconds_per_phase": duration_seconds,
        "execution_order": order,
        "workers": workers,
        "rows_per_operation": rows,
        "operation_counts": counts,
        "phases": phases,
        "read_weight": soak["read_weight"],
        "write_weight": soak["write_weight"],
        "raw_results": str(raw_path.resolve()),
        "raw_results_sha256": hashlib.sha256(raw_path.read_bytes()).hexdigest(),
    }
    manifest_path = arguments.output_dir / "soak-manifest.json"
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(manifest_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
