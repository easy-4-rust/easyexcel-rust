#!/usr/bin/env python3
"""Approve one fully passing benchmark candidate as a reviewed stable baseline."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[2]
BASELINE_ROOT = ROOT / "benchmarks" / "baselines"


def sha256(path: Path) -> str:
    """返回文件 SHA-256。"""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate-report", type=Path, required=True)
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--result", type=Path, action="append", required=True)
    parser.add_argument("--soak-manifest", type=Path)
    parser.add_argument("--reviewer", required=True)
    parser.add_argument("--review-notes", required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()

    report_path = arguments.candidate_report.resolve()
    if not report_path.is_file():
        raise ValueError(f"candidate report does not exist: {report_path}")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if report.get("schema_version") != 1 or report.get("baseline_candidate") is not True:
        raise ValueError("report was not emitted in --baseline-candidate mode")
    if report.get("profile") not in ("nightly", "release"):
        raise ValueError("only nightly/release candidates can become stable baselines")
    if report.get("passed") is not True or report.get("failures") != []:
        raise ValueError("baseline candidate did not pass every non-regression gate")
    spec_sha = report.get("spec_sha256")
    if not isinstance(spec_sha, str) or re.fullmatch(r"[0-9a-f]{64}", spec_sha) is None:
        raise ValueError("candidate report lacks a valid benchmark spec SHA")
    summaries = report.get("summaries")
    if not isinstance(summaries, dict) or not summaries:
        raise ValueError("candidate report contains no benchmark summaries")
    source_git_shas = report.get("source_git_shas")
    if not isinstance(source_git_shas, dict) or any(
        not isinstance(source_git_shas.get(implementation), str)
        or re.fullmatch(r"[0-9a-f]{40,64}", source_git_shas[implementation]) is None
        for implementation in ("java", "rust")
    ):
        raise ValueError("candidate report lacks unique Java/Rust source Git SHAs")

    expected_evidence = report.get("evidence")
    if not isinstance(expected_evidence, list) or not expected_evidence:
        raise ValueError("candidate report contains no source evidence")
    for item in expected_evidence:
        if (
            not isinstance(item, dict)
            or not isinstance(item.get("name"), str)
            or not item["name"]
            or not isinstance(item.get("sha256"), str)
            or re.fullmatch(r"[0-9a-f]{64}", item["sha256"]) is None
        ):
            raise ValueError("candidate report contains malformed evidence")

    spec = arguments.spec.resolve()
    results = [path.resolve() for path in arguments.result]
    if not spec.is_file() or any(not path.is_file() for path in results):
        raise ValueError("approval requires the exact spec and every candidate result JSONL")
    expected_git_shas = report.get("expected_git_shas")
    if not isinstance(expected_git_shas, dict):
        raise ValueError("candidate report lacks expected Git SHA bindings")
    compare = ROOT / "benchmarks" / "scripts" / "compare_results.py"
    with tempfile.TemporaryDirectory(prefix="easyexcel-baseline-approval-") as directory:
        reproduced_path = Path(directory) / "report.json"
        command = [
            sys.executable,
            str(compare),
            "--profile",
            report["profile"],
            "--spec",
            str(spec),
            "--baseline-candidate",
            "--output",
            str(reproduced_path),
        ]
        java_sha = expected_git_shas.get("java")
        rust_sha = expected_git_shas.get("rust")
        if report["profile"] == "release":
            if not isinstance(java_sha, str) or not isinstance(rust_sha, str):
                raise ValueError("release candidate lacks expected Java/Rust Git SHAs")
            if arguments.soak_manifest is None:
                raise ValueError("release approval requires --soak-manifest")
            command.extend(
                [
                    "--expected-java-git-sha",
                    java_sha,
                    "--expected-rust-git-sha",
                    rust_sha,
                    "--soak-manifest",
                    str(arguments.soak_manifest.resolve()),
                ]
            )
        elif arguments.soak_manifest is not None:
            raise ValueError("nightly approval must not supply a release soak manifest")
        command.extend(str(path) for path in results)
        completed = subprocess.run(command, check=False, capture_output=True, text=True)
        if completed.returncode != 0:
            raise ValueError(
                "candidate evidence no longer passes comparator:\n"
                f"stdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
            )
        reproduced = json.loads(reproduced_path.read_text(encoding="utf-8"))
    if reproduced != report:
        raise ValueError("candidate report differs from the report reproduced from raw evidence")

    output = arguments.output.resolve()
    if output.parent != BASELINE_ROOT.resolve():
        raise ValueError(f"baseline output must be directly under {BASELINE_ROOT}")
    expected_name = f"{report['profile']}-ubuntu-x64.json"
    if output.name != expected_name:
        raise ValueError(f"baseline output must be named {expected_name}")
    if output.exists():
        raise ValueError("baseline already exists; review replacement as a separate source change")

    baseline = {
        "schema_version": 2,
        "artifact": "easyexcel-reviewed-performance-baseline",
        "profile": report["profile"],
        "spec_sha256": spec_sha,
        "source_git_shas": source_git_shas,
        "candidate_report_sha256": sha256(report_path),
        "evidence": expected_evidence,
        "approval": {
            "status": "approved",
            "reviewer": arguments.reviewer,
            "reviewed_at": datetime.now(timezone.utc).isoformat(),
            "notes": arguments.review_notes,
        },
        "summaries": summaries,
    }
    output.write_text(
        json.dumps(baseline, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
