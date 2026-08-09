# EasyExcel Java/Rust shared benchmark contract

This directory is the authority for cross-language performance claims. Historical
numbers in `docs/benchmarks.md` are observations only; they are not acceptance
evidence because the former Java and Rust workloads were not equivalent.

## Contract

- `spec/benchmark-suite-v1.json` fixes the row model, scenarios, sizes, batch
  size, concurrency matrix, repetitions, and regression thresholds.
- `spec/benchmark-result-v1.schema.json` is the shared result protocol.
- `rust-runner` executes exactly one read or write phase and prints one JSON
  object to stdout. It must be built before measurement.
- The Java runner is
  `com.alibaba.easyexcel.test.benchmark.EasyExcelBenchmarkRunner` in the pinned
  Java source repository and consumes the same spec.
- `scripts/measure_process.py` directly wraps a prebuilt executable with
  `/usr/bin/time`; Cargo and Maven are never part of the timed process.
- `scripts/compare_results.py` computes median, MAD, p50/p95/p99 and coefficient
  of variation, verifies cross-implementation checksums, and applies each
  implementation's own historical regression gate.
- `scripts/run_soak.py` executes the release-only, deterministic 70% read / 30%
  write workload. Each runtime receives two 30-minute phases in
  `Rust -> Java -> Java -> Rust` order, and every writer uses a unique file.
- `fixtures/fixture-manifest.json` is generated with the SHA-256 of every Java-
  and Rust-produced XLS/XLSX/CSV fixture. Both runtimes read each exact fixture;
  comparisons never use separately generated inputs under the same label. The
  comparator re-hashes every retained fixture once and binds its absolute path,
  manifest entry, origin, row count, and every timed sample's `input_sha256`.

## Required execution rules

1. Pin Java and Rust Git SHAs, JDK, GC, heap, Rust toolchain, release profile,
   operating system, CPU, memory, filesystem, dependency locks, and spec SHA.
   Nightly and release CI require the fixed `easyexcel-benchmark-linux-x64`
   self-hosted runner label; only the PR correctness smoke may use an ephemeral
   hosted runner. `environment-manifest.json` records the observed machine,
   disk, runtime, repository, lockfile, binary, and spec identities.
2. Use UTC and a fixed locale. Java release runs use fixed `-Xms`/`-Xmx` and a
   named GC. BenchmarkSpec v1 pins JDK 17, G1, `-Xms512m -Xmx4g`, Rust 1.97.1,
   UTC, and `en_US.UTF-8`; the orchestrator rejects runtime drift before fixture
   generation. Keep GC logs as raw artifacts so maximum pause can be derived.
3. Run cold-start and steady-state suites separately. `temperature=cold` starts
   without warm-up. `temperature=steady` performs three warm-up operations in
   the same runner process before each measured operation, followed by at least
   seven independent measured processes; starting throwaway warm-up JVMs is not
   accepted as steady state.
   `wall_time_ns` measures only the contracted operation; OS-level
   `process_wall_time_ns`, CPU time, and RSS describe the complete runner
   envelope (including same-process warm-up for steady-state samples), so the
   report never mixes operation latency with process CPU accounting.
   After Java warm-up and before counters are reset, the runner performs an
   explicit collection outside the timed region. This matches Rust's
   deterministic drop boundary and prevents unreachable warm-up workbooks from
   deciding whether a measured iteration happens to trigger G1.
   Each worker receives an isolated temporary directory through `TMPDIR`,
   `TMP`, `TEMP`, and Java `java.io.tmpdir`. Only that directory contributes to
   `temporary_disk_peak_bytes`; final output files and GC logs are excluded.
   This is required for Java's disk-backed shared-strings cache and prevents
   fixture origins or concurrent workers from sharing temporary I/O state.
4. Interleave implementations in `Rust -> Java -> Java -> Rust` order. Never
   publish the best run.
5. Event and Workbook modes, and constant-memory and full-memory modes, are
   separate processes. Every concurrent writer owns a distinct output path.
6. Read both Java- and Rust-produced files with both implementations. A result
   is valid only when row count, canonical SHA-256, and reopen checks all match.
   The XLSX RoundTrip scenario additionally changes the workbook title, saves,
   reopens, verifies the marker, and then validates all data rows.
7. A coefficient of variation above 10% invalidates the environment; it does
   not prove a performance regression or improvement.

## CI layers

| Layer | Workload | Purpose |
|---|---:|---|
| PR | 10K, one measured run | Correctness and runner smoke only |
| Nightly | 100K, 3 warm-ups + 7 runs | Stable Java/Rust comparison and regression detection |
| Release | 1M, Event Read/streaming Write workers 1/2/4/8/16, 30 minutes | Throughput, memory, concurrency, and soak evidence |

Full-memory Write, Workbook Read, RoundTrip, XLS, and CSV remain single-worker
release scenarios. The concurrency matrix targets XLSX Event Read and streaming
Write, while the 16-worker soak exercises their 70/30 mixed workload. This
keeps the concurrency contract focused on the production streaming paths and
avoids multiplying sixteen independent full-memory JVM workloads.

Release additionally contains a Rust-only `internal-parallel-map` phase for the
explicit opt-in `ParallelMapReadListener`. It reads each Java- and Rust-produced
XLSX in one process and compares a serial pure mapper with 2 and 4 mapper
workers. The deterministic mapper workload and bounded queue are fixed by the
shared spec; ordinary XML parsing and downstream Listener callbacks remain
single-threaded and ordered. Both 2- and 4-worker steady-state medians must be
at least 1.20x the serial mapper, every checksum must match, all seven trials
must exist, and peak RSS must remain within 64 MiB. `worker_count` denotes
in-process mapper workers only for this phase; it continues to denote concurrent
runner processes for the normal matrix and soak phases.

The XLS write scenario is deliberately named `xls-batched-write`: both runners
bound input batches and split at BIFF8's Sheet row limit, while the effective
backend remains workbook-memory. It is declared as `mode=workbook` and
`memory=batched`; it must never be reported as constant-memory or streaming.

Release gates apply both dimensions: each implementation must remain within its
own pinned regression baseline, and Rust must satisfy the cross-runtime median
and confidence-lower-bound thresholds in `benchmark-suite-v1.json`. Checksum,
cross-runtime reopen, RSS, temporary disk, sample completeness, and CV remain
independent fail-closed conditions; a strong result in one dimension cannot
waive another.

## Running the matrix

Build both runners before measurement. The Rust command embeds the exact Git
SHA and compiler version in its result; the Java command uses the already
compiled test classes and dependency classpath.

```shell
EASYEXCEL_GIT_SHA="$(git rev-parse HEAD)" \
EASYEXCEL_RUSTC="$(rustc --version)" \
cargo build --release -p easyexcel-benchmark-runner

python3 benchmarks/scripts/run_matrix.py \
  --profile pr \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-classpath "$JAVA_BENCHMARK_CLASSPATH" \
  --java-repo /path/to/easyexcel \
  --rust-repo /path/to/easyexcel-rust \
  --output-dir benchmarks/results/pr

python3 benchmarks/scripts/compare_results.py \
  --profile pr \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --output benchmarks/results/pr/report.json \
  benchmarks/results/pr/raw-results.jsonl

# Release only: build both clean-SHA runners and attest their exact bytes first.
python3 benchmarks/scripts/prepare_release_artifacts.py \
  --rust-repo /path/to/easyexcel-rust \
  --java-repo /path/to/easyexcel \
  --rust-bin /path/to/easyexcel-rust/target/release/easyexcel-benchmark-runner \
  --java-bin /path/to/pinned-jdk/bin/java \
  --java-classpath "$JAVA_BENCHMARK_CLASSPATH" \
  --output /tmp/easyexcel-release-artifacts.json

# Produce the complete 1M matrix, including workers 1/2/4/8/16.
python3 benchmarks/scripts/run_matrix.py \
  --profile release \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin /path/to/pinned-jdk/bin/java \
  --java-classpath "$JAVA_BENCHMARK_CLASSPATH" \
  --java-repo /path/to/easyexcel \
  --rust-repo /path/to/easyexcel-rust \
  --artifact-manifest /tmp/easyexcel-release-artifacts.json \
  --output-dir benchmarks/results/release

# Release only: complete cycles preserve an exact 70/30 operation ratio.
python3 benchmarks/scripts/run_soak.py \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin /path/to/pinned-jdk/bin/java \
  --java-classpath "$JAVA_BENCHMARK_CLASSPATH" \
  --java-repo /path/to/easyexcel \
  --rust-repo /path/to/easyexcel-rust \
  --artifact-manifest /tmp/easyexcel-release-artifacts.json \
  --output-dir benchmarks/results/release-soak

python3 benchmarks/scripts/compare_results.py \
  --profile release \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --expected-java-git-sha "$JAVA_GIT_SHA" \
  --expected-rust-git-sha "$RUST_GIT_SHA" \
  --baseline benchmarks/baselines/release-ubuntu-x64.json \
  --require-baseline \
  --soak-manifest benchmarks/results/release-soak/soak-manifest.json \
  --output benchmarks/results/release/report.json \
  benchmarks/results/release/raw-results.jsonl \
  benchmarks/results/release-soak/raw-results.jsonl
```

`--profile release` 会无条件要求 `--baseline`；`--require-baseline` 在上述命令中保留是为了让
意图可见，也可用于 nightly 等非 release profile。baseline 必须覆盖候选报告中的每一个矩阵
summary，缺少单个 label 即失败，不能通过省略历史差项绕过吞吐或 RSS 回归比较。

Release comparison is fail-closed: the complete matrix, four soak phases in
`Rust → Java → Java → Rust` order, each phase's measured 30-minute duration,
exact 70/30 operation mix, complete 16-worker trial sets, the complete
serial/2/4-worker internal mapper phase, Rust's 64 MiB
single-worker RSS ceiling, the 15% stable RSS regression ceiling, and Rust
temporary write-disk usage at no more than 25% of Java must all be present.
The schema-v2 runner attestation also proves that `--rust-bin` is the exact
Cargo release target built in the preparation run and binds the Rust compiler,
Java executable/JAVA_HOME/version, runner class, and every classpath entry by
path and SHA-256. Maven compilation is forced onto that same Java home.
Each results directory also carries the artifact attestation and environment
manifest. The comparator binds the clean source fingerprints, Rust binary hash,
Java runner class and full classpath hashes to the expected Git SHAs; a stale
prebuilt runner cannot pass by reporting a runtime-injected SHA.

The Java classpath must begin with the pinned Java repository's
`easyexcel-test/target/test-classes` and include its Maven dependency
classpath. `run_matrix.py` fixes UTC, English locale, G1, and the contracted
`-Xms512m -Xmx4g` bounds. Heap settings can only change together with a
versioned BenchmarkSpec update.
