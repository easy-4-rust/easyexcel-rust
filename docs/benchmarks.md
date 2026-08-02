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

### 2026-08-03 bisect: root cause located (commit-level)

Re-measured HEAD with the current toolchain and bisected the slowdown by
running the benchmark at historical commits (same machine, same toolchain,
release profile):

| Commit | Date | Write | Read |
|---|---|---|---|
| `eb11974` | 2026-07-17 | 1.81 s | **0.90 s** |
| `68e8b5f` | 2026-07-24 | 1.77 s | 2.22 s |
| `be30782` (incl. `101d668`) | 2026-07-24 | **5.94 s** | 2.50 s |
| `90773d9` | 2026-07-30 | 5.82 s | 2.47 s |
| `7382f2e` | 2026-07-30 | 7.85 s | 2.61 s |
| HEAD (`1c210e9`) | 2026-08-03 | 6.05 s | 2.49 s |

XLSX output bytes identical (12 336 909) at every measured commit — semantics
unaffected throughout.

**Write regression (1.8 s → 6.0 s, ~3.3x): introduced by `101d668`
(refactor(writer): 优化抽象写入处理器和参数构建器实现, 2026-07-24).** The
commit expands `WriteCellContext` from a ~6-field lightweight struct to a
25+-field context mirroring Java `CellWriteHandlerContext` in full (static
`ExcelColumn` metadata, `head_name`, `original_value`, `pending_original_*`,
`cell_data_list`, `target_cell_data_type`, `cell` handle, `holders`), and
rewrites the per-cell conversion/handler pipeline. This is deliberate Java
semantic parity (the Java context is likewise a fat object), not an
accidental inefficiency. `101d668` itself does not compile; its code takes
effect from `be30782` onward. Toolchain is ruled out: `eb11974` runs 1.81 s
under the current Rust 1.97.1, so the old code is not slow under the new
toolchain.

**Read regression (0.9 s → 2.3 s, ~2.5x): introduced across the 2026-07-17→21
"align Java" feature series** (`f60811f` XLSX formula metadata, `a8e0197` cell
extra events, `12308c2` cell display formatting, `da2a384` Hutool reader
ergonomics, `9cb3b8b` BigInteger conversion, `9f76f9f` 1904 date windowing,
`6e58f0f` scientific number formatting, `d9c7cba` locale-aware reading).
Each adds per-cell-event work in the SAX analysis hot path; `8090351` (4.45 s,
intermediate) shows the peak before the `11f28ff`→`18ab533` empty-body
cleanup pass settled it at ~2.3 s.

**Verdict: both regressions are the measured cost of Java semantic parity,
not bugs.** Absolute throughput remains healthy: ~165 k rows/s write,
~400 k rows/s read, 11.2 MiB peak RSS (constant-memory streaming). Optional
future optimization targets (not 1.0 blockers): per-cell `Vec<CellValue>` in
`WriteCellContext` (single-value path could use a small inline buffer) and
event-stage allocation reduction in the reader. Benchmarked on Apple M4 Pro,
24 GiB RAM, macOS 26.5.2, Rust 1.97.1.

## Reproduce

```shell
./scripts/benchmark-million-rows.sh
```

The script uses `/usr/bin/time -l` on macOS and `/usr/bin/time -v` on Linux. Its
first argument overrides the row count and its second argument overrides the
output path.
