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

### 2026-08-03 Java comparison (same machine)

Same-day, same-machine (Apple M4 Pro, 24 GiB RAM, macOS 26.5.2, arm64)
head-to-head against the Java original, to ground the migration in a real
cross-implementation baseline.

Java side: EasyExcel 4.0.3 built from source (alibaba/easyexcel repo HEAD
`3afdea9d`), Amazon Corretto 17.0.20 LTS, JVM args `-Xmx4g` only (default G1
GC). Harness: `MillionRowBenchmark` (plain class with `main`, not a JUnit
test) added at `easyexcel-test/src/test/java/com/alibaba/easyexcel/test/
benchmark/MillionRowBenchmark.java` in the Java repo. It mirrors the
`million_rows` example semantics: typed write, streaming read through a
`ReadListener` that counts and releases each row, and a hard row-count check
(expected 1,000,000, read 1,000,000 — passed in every run).

Row model and fairness caveats (read before comparing):

- The Java row model follows the Rust `DemoRow` from
  `easyexcel/examples/generate_compat_fixtures.rs`: ID (Long) / Name
  (`row-{i}`) / Date (`java.util.Date`, written by the default
  `DateNumberConverter` as an Excel serial-number cell) / Score (`i*0.5`) —
  **four cells per row**. The Rust numbers above were measured with the
  `million_rows` example's two-cell `BenchmarkRow` (ID u32 / Value String),
  so XLSX byte size is not an apples-to-apples comparison and the Java run
  does twice the per-row cell work.
- Write was measured in both forms: **stream** (10,000-row chunks through
  repeated `ExcelWriter.write` — the Java equivalent of
  `constant_memory(true)`) and **one-shot** (a single `doWrite(List)` holding
  all 1,000,000 rows in memory, the memory-heavy default usage pattern).
  Read is listener-based streaming in both cases.
- JIT: cold. Every run is a fresh JVM with no warm-up; the write phase also
  absorbs JIT compilation of the write path. Stream mode ran twice and was
  consistent.

| Metric | Rust 1.97.1 (HEAD, 2026-08-03) | Java stream | Java one-shot |
|---|---|---|---|
| Data rows (+ header) | 1,000,000 + 1 | 1,000,000 + 1 | 1,000,000 + 1 |
| Cells per row | 2 (ID/Value) | 4 (ID/Name/Date/Score) | 4 (ID/Name/Date/Score) |
| XLSX bytes | 12,336,909 | 25,658,931 | 25,658,931 |
| Write time | 6.05 s | 4.10 s (repeat 4.16 s) | 4.14 s |
| Read time | 2.49 s | 2.36 s (repeat 2.35 s) | 2.40 s |
| Max RSS (`/usr/bin/time -l`) | 11,370,496 B (~10.8 MiB) | 498,909,184 B (~476 MiB) | 1,195,474,944 B (~1.14 GiB) |
| Peak memory footprint | 3,637,680 B | 388,090,496 B | 1,085,082,816 B |

Throughput on this machine: Java stream write ~244 k rows/s (976 k cells/s)
vs Rust ~165 k rows/s (330 k cells/s); Java read ~424 k rows/s (1.69 M
cells/s) vs Rust ~401 k rows/s (803 k cells/s).

Verdict: same order of magnitude — Java (cold JIT) writes ~1.5x faster than
current Rust HEAD and reads about equal; per cell, Java is ~2-3x faster in
both directions. The headline difference is memory: the JVM streaming run
peaks at ~476 MiB RSS (POI SXSSF keeps the shared-strings table in memory
plus JIT/GC infrastructure) and the one-shot run at ~1.14 GiB (the 1M-row
list itself), versus ~10.8 MiB for Rust's constant-memory streaming — Rust
uses ~44x less memory than the Java stream form and ~105x less than the Java
one-shot form. For reference, the 2026-07-17 Rust baseline (write 2.93 s /
read 0.65 s) beat Java on both axes; the current Rust regression is the
measured cost of Java semantic parity documented above.

Reproduce:

```shell
cd /Users/wandl/workspaces/workspace-github/easyexcel        # Java repo
JAVA_HOME=/Users/wandl/Library/Java/JavaVirtualMachines/corretto-17.0.20/Contents/Home
mvn -q install -pl easyexcel-core -am
mvn -q test-compile -pl easyexcel-test -Dmaven.test.skip=false
mvn -q dependency:build-classpath -pl easyexcel-test -Dmdep.outputFile=/tmp/eex-cp.txt
/usr/bin/time -l java -Xmx4g \
  -cp "easyexcel-test/target/test-classes:easyexcel-test/target/classes:$(cat /tmp/eex-cp.txt)" \
  com.alibaba.easyexcel.test.benchmark.MillionRowBenchmark 1000000 /tmp/out.xlsx stream
```

### 2026-08-03 memory-mode probe: memory-for-speed is not available

The public `constant_memory` switch (`WriteOptions.constant_memory`,
`WriteSheetBuilder::constant_memory`) is the "memory for speed" dial. A
1M-row probe comparing `MODE=full` (constant_memory off, full RAM) with the
default constant-memory streaming (same machine, same day, Rust 1.97.1):

| | constant memory | full memory |
|---|---|---|
| Write | 6.05 s | 7.74 s (slower) |
| Read | 2.49 s | 5.05 s (2x slower) |
| XLSX bytes | 12,336,909 | 14,578,335 (larger) |
| Peak RSS | 10.8 MiB | 1.06 GiB (100x) |

Structural cause: full-memory mode emits a shared-strings table
(sharedStrings.xml ≈ 25 MB raw for 1M unique `row-N` strings) and
cell `<v>index</v>` references; constant-memory mode emits inline
strings (`<is><t>row-N</t></is>`) with no SST. For workloads with unique
strings the SST is pure overhead (larger file, slower read via SST lookup,
no space win); for highly repetitive strings SST would win, but the
constant-memory path is already the fastest of the two on this benchmark.
Conclusion: the memory dial cannot buy speed in the current backend —
constant-memory streaming is the optimal point on both axes. The path to
faster writes is the hot-path regression recovery (101d668 semantic-parity
cost), not a memory-mode switch. Reproduce: `MODE=full ./scripts/benchmark-million-rows.sh`.
