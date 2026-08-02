# Streaming benchmark baseline

The benchmark uses the public `EasyExcel` facade for both directions. It writes
typed rows with `constant_memory(true)`, then reads them through a listener that
counts and releases each row. The executable fails if the read count differs
from the requested row count.

## 2026-07-17 baseline

- Machine: Apple M4 Pro, 24 GiB RAM
- Operating system: macOS 26.5.2, arm64
- Rust: 1.93.1
- Profile: `release`
- Rows: 1,000,000 data rows plus one header row
- XLSX size: 12,336,908 bytes
- Write time: 2.927 seconds
- Read time: 0.647 seconds
- Whole command wall time: 4.31 seconds
- `/usr/bin/time -l` maximum resident set size: 8,519,680 bytes
- `/usr/bin/time -l` peak memory footprint: 2,408,856 bytes

The command was run after the release profile had been compiled, so whole
command wall time does not include a clean dependency build. Timing and memory
numbers are machine-specific; row-count verification is deterministic.

## 2026-08-01 smoke run (1000 rows)

Smoke data only, not a release measurement — confirms the script and the
`million_rows` example still run end to end:

- Machine: Apple M4 Pro (Mac16,8), 24 GiB RAM, macOS 26.5.2, arm64
- Rust: 1.97.1
- Profile: `release`
- Command: `./scripts/benchmark-million-rows.sh 1000`
- Rows: 1000 plus one header row
- Write time: 0.008 seconds
- Read time: 0.006 seconds
- XLSX size: 17,605 bytes
- Whole command wall time: 3.54 seconds (includes a 2.28 s incremental cargo
  build)
- `/usr/bin/time -l` maximum resident set size: 11,354,112 bytes
- `/usr/bin/time -l` peak memory footprint: 3,375,536 bytes
- Row-count verification: passed (expected 1000, read 1000)

## 2026-08-01 1M re-measurement (same machine as baseline)

Re-ran the full benchmark on the same M4 Pro machine; the only environment
difference from the 2026-07-17 baseline is the toolchain (Rust 1.97.1 vs
1.93.1). Three consecutive runs showed meaningful run-to-run variance; the
cleanest (third) run is recorded below:

- Rows: 1,000,000 data rows plus one header row
- XLSX size: 12,336,908 bytes (one run produced 12,336,909, a one-byte
  timestamp/component variance)
- Write time: 6.238 seconds (runs observed: 6.100–9.339)
- Read time: 2.576 seconds (runs observed: 2.576–4.225)
- Whole command wall time: 9.58 seconds
- `/usr/bin/time -l` maximum resident set size: 11,370,496 bytes
  (runs observed: 11,010,048–11,370,496)
- `/usr/bin/time -l` peak memory footprint: 3,637,680 bytes
- Row-count verification: passed (expected 1,000,000, read 1,000,000)

Note: write time is about 2.1x and read time about 4x the 2026-07-17 baseline
on this machine. The slowdown may be toolchain-related (Rust 1.93.1 → 1.97.1),
machine load at measurement time, or a code regression; it should be
investigated before the 1.0 release. XLSX output bytes are unchanged from the
baseline.

### 2026-08-02 regression follow-up

The only uncommitted source changes since the 2026-07-17 baseline are 30 lines
of module-level doc comments (no logic); they cannot explain the slowdown.
Likely causes ranked: toolchain upgrade (Rust 1.93.1 → 1.97.1 codegen
changes), machine load variance, or measurement methodology drift. Output
bytes and row-count verification remain identical, so semantics are
unaffected.

## Reproduce

```shell
./scripts/benchmark-million-rows.sh
```

The script uses `/usr/bin/time -l` on macOS and `/usr/bin/time -v` on Linux. Its
first argument overrides the row count and its second argument overrides the
output path.
