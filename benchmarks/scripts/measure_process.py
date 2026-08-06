#!/usr/bin/env python3
"""Run one prebuilt benchmark process and enrich its JSON with OS metrics."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import tempfile
import threading
import time


def directory_size(path: Path) -> int:
    if not path.exists():
        return 0
    if path.is_file():
        return path.stat().st_size
    return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())


def watch_directory(path: Path, stop: threading.Event, peak: list[int]) -> None:
    while not stop.wait(0.05):
        peak[0] = max(peak[0], directory_size(path))
    peak[0] = max(peak[0], directory_size(path))


def parse_time_output(
    text: str,
) -> tuple[int | None, int | None, int | None, int | None]:
    if platform.system() == "Darwin":
        timing = re.search(r"([0-9.]+) real\s+([0-9.]+) user\s+([0-9.]+) sys", text)
        rss = re.search(r"(\d+)\s+maximum resident set size", text)
        user_ns = int(float(timing.group(2)) * 1_000_000_000) if timing else None
        system_ns = int(float(timing.group(3)) * 1_000_000_000) if timing else None
        wall_ns = int(float(timing.group(1)) * 1_000_000_000) if timing else None
        rss_bytes = int(rss.group(1)) if rss else None
        return wall_ns, user_ns, system_ns, rss_bytes
    elapsed = re.search(
        r"^\s*Elapsed \(wall clock\) time.*?:\s*((?:\d+:){1,2}[0-9.]+)\s*$",
        text,
        re.MULTILINE,
    )
    user = re.search(r"User time \(seconds\):\s*([0-9.]+)", text)
    system = re.search(r"System time \(seconds\):\s*([0-9.]+)", text)
    rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", text)
    user_ns = int(float(user.group(1)) * 1_000_000_000) if user else None
    system_ns = int(float(system.group(1)) * 1_000_000_000) if system else None
    rss_bytes = int(rss.group(1)) * 1024 if rss else None
    wall_ns = None
    if elapsed:
        parts = elapsed.group(1).split(":")
        hours = int(parts[0]) if len(parts) == 3 else 0
        minutes = int(parts[-2])
        seconds = float(parts[-1])
        wall_ns = int((hours * 3600 + minutes * 60 + seconds) * 1_000_000_000)
    return wall_ns, user_ns, system_ns, rss_bytes


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    parser.add_argument("--watch-dir", type=Path)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    arguments = parser.parse_args()
    command = arguments.command[1:] if arguments.command[:1] == ["--"] else arguments.command
    if not command:
        parser.error("a command is required after --")

    time_args = ["/usr/bin/time", "-l" if platform.system() == "Darwin" else "-v"]
    with tempfile.NamedTemporaryFile(prefix="easyexcel-time-", delete=False) as timing:
        timing_path = Path(timing.name)
    stop = threading.Event()
    peak = [directory_size(arguments.watch_dir)] if arguments.watch_dir else [0]
    monitor = None
    if arguments.watch_dir:
        monitor = threading.Thread(
            target=watch_directory,
            args=(arguments.watch_dir, stop, peak),
            daemon=True,
        )
        monitor.start()
    try:
        with timing_path.open("w", encoding="utf-8") as timing_output:
            completed = subprocess.run(
                [*time_args, *command],
                check=False,
                stdout=subprocess.PIPE,
                stderr=timing_output,
                text=True,
            )
        stop.set()
        if monitor:
            monitor.join()
        timing_text = timing_path.read_text(encoding="utf-8")
        if completed.returncode != 0:
            raise SystemExit(
                f"benchmark process exited {completed.returncode}\n{timing_text}\n{completed.stdout}"
            )
        result = json.loads(completed.stdout)
        wall_ns, user_ns, system_ns, rss_bytes = parse_time_output(timing_text)
        result["process_wall_time_ns"] = wall_ns
        result["cpu_user_time_ns"] = user_ns
        result["cpu_system_time_ns"] = system_ns
        result["peak_rss_bytes"] = rss_bytes
        result["temporary_disk_peak_bytes"] = peak[0] if arguments.watch_dir else None
        encoded = json.dumps(result, ensure_ascii=False, separators=(",", ":"))
        if arguments.output:
            arguments.output.parent.mkdir(parents=True, exist_ok=True)
            arguments.output.write_text(encoded + "\n", encoding="utf-8")
        print(encoded)
        return 0
    finally:
        stop.set()
        if monitor and monitor.is_alive():
            monitor.join()
        timing_path.unlink(missing_ok=True)


if __name__ == "__main__":
    raise SystemExit(main())
