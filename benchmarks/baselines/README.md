# Stable performance baselines

Nightly and release gates consume reviewed reports from this directory. A
baseline is created only from a successful, low-variance run on the same CI
runner class; it must never be synthesized or copied from the historical
numbers in `docs/benchmarks.md`.

Expected names are `nightly-ubuntu-x64.json` and `release-ubuntu-x64.json`.
Until the corresponding file exists, the workflow deliberately reports a
missing-baseline failure while still uploading the candidate result artifact.

The comparator accepts a baseline only from this directory. It must be a
passing report for the same profile and benchmark-spec SHA, contain no gate
failures, and retain non-empty summaries. Release runner attestation includes
the clean Rust source fingerprint, so the reviewed baseline bytes are bound to
the candidate's repository SHA instead of being supplied as an arbitrary JSON
file at comparison time.

---

## Stub files

Files containing `"pending_generation": true` are schema-valid placeholders
created before the first real benchmark run on the fixed Linux runner.  They
exist so that CI checks for "baseline file presence" can parse the file, but
they are **not** accepted by the comparator gate -- a real baseline must
replace the stub before any nightly or release gate can pass.

Consumers **must** reject a baseline where `pending_generation` is `true`.

## Baseline generation runbook

This runbook describes how to produce a reviewed stable baseline on the
dedicated Linux benchmark runner.  Every step must succeed on the fixed
runner; local macOS runs are **not** acceptable substitutes.

### 1. Fixed runner environment

| Attribute            | Value                                                  |
|----------------------|--------------------------------------------------------|
| Machine type         | Bare-metal x86_64 (no shared VM, no noisy neighbours) |
| CPU                  | >= 4 physical cores, turbo boost disabled              |
| RAM                  | >= 16 GB                                               |
| Disk                 | NVMe SSD, >= 50 GB free on partition holding `/tmp`    |
| OS                   | Ubuntu 22.04 LTS (or 24.04 LTS) x86_64                |
| Kernel               | Stock distro kernel, no custom `isolcpus`/`cgroup`     |
| Rust toolchain       | `1.97.1` (pinned via `rust-toolchain.toml`)            |
| Java                 | Temurin 17 (same major as `runtime_contract.java_version`) |
| Maven                | 3.9+ (bundled `mvnw` wrapper is sufficient)            |
| Python               | 3.10+ (for benchmark scripts)                          |
| Locale / TZ          | `en_US.UTF-8` / `UTC` (validated by `run_matrix.py`)   |

CPU-frequency governor should be `performance`.  Disable frequency
scaling and ensure no background services consume significant CPU or
I/O during the benchmark window (~50 min for release, ~5 min for nightly).

### 2. Java runtime configuration

The benchmark spec (`benchmark-suite-v1.json`) mandates:

```
java_gc   = G1
java_xms  = 512m
java_xmx  = 4g
timezone  = UTC
locale    = en_US.UTF-8
```

These are passed to `run_matrix.py` via `--java-xms 512m --java-xmx 4g`.
The JVM selects G1 GC by default on Java 17; no extra flags are required.

### 3. Build the benchmark runners

```bash
# --- Rust ---
rustup toolchain install 1.97.1
cargo build --locked --release \
  --manifest-path rust/Cargo.toml \
  -p easyexcel-benchmark-runner

# --- Java ---
cd java
./mvnw -q -pl easyexcel-test -am -DskipTests test-compile
./mvnw -q -pl easyexcel-test dependency:build-classpath \
  -DincludeScope=test -Dmdep.outputFile=target/benchmark-classpath.txt
cd ..

# Compile the Java benchmark runner
module_classes="$(find java -type d -path '*/target/classes' -print | paste -sd: -)"
dependencies="$(cat java/easyexcel-test/target/benchmark-classpath.txt)"
java_test_classes="$PWD/java/easyexcel-test/target/test-classes"
classpath="$java_test_classes:$module_classes:$dependencies"
javac -encoding UTF-8 -cp "$classpath" -d "$java_test_classes" \
  $(find benchmarks/java-runner/src -name '*.java' -print | sort)
```

### 4. Attest release runners (release profile only)

```bash
python3 benchmarks/scripts/prepare_release_artifacts.py \
  --rust-repo . \
  --java-repo ../java \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --output artifacts/release-runner-artifact.json
```

This produces `artifacts/release-runner-artifact.json` containing SHA-256
fingerprints of the exact binaries and source trees used in the run.

### 5. Run the release benchmark matrix

```bash
python3 benchmarks/scripts/run_matrix.py \
  --profile release \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --java-repo ../java \
  --rust-repo . \
  --artifact-manifest artifacts/release-runner-artifact.json \
  --output-dir artifacts/release/matrix
```

Profile parameters (from `benchmark-suite-v1.json`):
- `rows`: [1000000]
- `temperatures`: [cold, steady]
- `warmups`: 3
- `measurements`: 7
- `duration_seconds`: 1800
- Scenarios: 9 total (xlsx-stream-write, xlsx-full-write, xlsx-event-read,
  xlsx-workbook-read, xlsx-roundtrip, xls-batched-write, xls-event-read,
  csv-stream-write, csv-event-read)
- Concurrency matrix [1, 2, 4, 8, 16] for xlsx-stream-write and xlsx-event-read

For the nightly profile, use `--profile nightly` and omit
`--artifact-manifest`.  Nightly uses `rows: [100000]` and
`duration_seconds: 0`.

### 6. Run the four-phase soak (release only)

```bash
python3 benchmarks/scripts/run_soak.py \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --java-repo ../java \
  --rust-repo . \
  --artifact-manifest artifacts/release-runner-artifact.json \
  --output-dir artifacts/release/soak
```

### 7. Compare and validate

```bash
python3 benchmarks/scripts/compare_results.py \
  --profile release \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --expected-java-git-sha "$(git -C ../java rev-parse HEAD)" \
  --expected-rust-git-sha "$(git rev-parse HEAD)" \
  --soak-manifest artifacts/release/soak/soak-manifest.json \
  --baseline-candidate \
  --output artifacts/release/report.json \
  artifacts/release/matrix/raw-results.jsonl \
  artifacts/release/soak/raw-results.jsonl
```

The report must show `"passed": true` and `"failures": []`.

### 8. Approve and commit the baseline

```bash
python3 benchmarks/scripts/approve_benchmark_baseline.py \
  --candidate-report artifacts/release/report.json \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --result artifacts/release/matrix/raw-results.jsonl \
  --result artifacts/release/soak/raw-results.jsonl \
  --soak-manifest artifacts/release/soak/soak-manifest.json \
  --reviewer "operator-name" \
  --review-notes "First release baseline on fixed Linux runner, 2026-08-10" \
  --output benchmarks/baselines/release-ubuntu-x64.json
```

For nightly (no soak):

```bash
python3 benchmarks/scripts/approve_benchmark_baseline.py \
  --candidate-report artifacts/nightly/report.json \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --result artifacts/nightly/matrix/raw-results.jsonl \
  --reviewer "operator-name" \
  --review-notes "First nightly baseline on fixed Linux runner, 2026-08-10" \
  --output benchmarks/baselines/nightly-ubuntu-x64.json
```

### 9. Expected outputs

| File | Description |
|------|-------------|
| `benchmarks/baselines/release-ubuntu-x64.json` | Reviewed release baseline (replaces stub) |
| `benchmarks/baselines/nightly-ubuntu-x64.json` | Reviewed nightly baseline (replaces stub) |

The approve script validates:
- The candidate report passed all gates (no failures).
- The spec SHA matches.
- The Java and Rust source Git SHAs are present and valid.
- The output path is directly under `benchmarks/baselines/` and named
  `{profile}-ubuntu-x64.json`.
- The baseline does not already exist (stubs must be deleted or overwritten
  by the approve script's output).

### 10. Nightly baseline and regression gate (T5.2)

The nightly baseline is produced by the same runbook with `--profile nightly`.
The CI schedule (`benchmark.yml`, cron `30 18 * * *`) runs the nightly profile
automatically.  When a baseline file exists, the comparator enforces:

- `max_median_throughput_regression`: 10%
- `max_peak_rss_regression`: 15%
- `max_coefficient_of_variation`: 10%

These gates are defined in `benchmark-suite-v1.json` under `gates`.

## Known issues

### 1. schema_version in validate_stable_baseline

`compare_results.py:validate_stable_baseline` (line 633) currently checks
for `schema_version == 1`, but the baseline schema is version 2.  This means
the comparator will reject a valid v2 baseline with "unsupported schema
version".  This must be fixed before the first real baseline is committed:

```python
# compare_results.py line 633 -- change:
if report.get("schema_version") != 1:
# to:
if report.get("schema_version") not in (1, 2):
```

### 2. pending_generation not checked by validate_stable_baseline

`validate_stable_baseline` does not check the `pending_generation` field.
If a stub file (with `pending_generation: true`) passes the schema_version
check, the comparator would attempt to validate it as a real baseline and
likely fail on missing/invalid summaries.  The fix is to add an early
rejection after the schema_version check:

```python
if report.get("pending_generation") is True:
    failures.append(
        f"baseline file is a pending stub (pending_generation=true); "
        f"a real baseline must replace it before the gate can pass: "
        f"{baseline_path}"
    )
    return None
```

### 3. CI baseline-missing gate

The CI workflow (`benchmark.yml`, lines 146-155) checks for baseline file
existence with `-f`.  With stub files present, the CI will find the file
and pass `--baseline` to the comparator.  The comparator will then reject
the stub (due to issues 1 and 2 above).  This is the correct behavior:
the CI fails with a clear message instead of silently skipping the
regression gate.

If you want the CI to distinguish "stub present" from "real baseline
present", the workflow can be updated to check `pending_generation`:

```bash
if [[ -f "$baseline" ]] && ! python3 -c "
import json, sys
d = json.load(open('$baseline'))
sys.exit(0 if d.get('pending_generation') is True else 1)
" 2>/dev/null; then
  # baseline file exists but is a stub -- require a real one
  baseline_args=(--require-baseline)
elif [[ -f "$baseline" ]]; then
  baseline_args=(--baseline "$baseline")
elif [[ "$profile" != "pr" ]]; then
  baseline_args=(--require-baseline)
fi
```
