# cross_runtime Gate Execution Runbook

This runbook covers the end-to-end procedure for executing the `cross_runtime`
gate on a fixed Linux runner with Java v4.0.3.  It supersedes no existing
file; the baseline-generation runbook in `benchmarks/baselines/README.md`
covers the broader release flow.  This document focuses specifically on
the cross_runtime gate prerequisites, build steps, expected outputs, and
failure triage.

> **Important**: Local macOS runs are NOT acceptable substitutes for the
> release gate.  The cross_runtime gate is only meaningful on the fixed
> Linux runner with matched Rust and Java toolchains.

---

## 1. Prerequisites

### 1.1 Fixed Linux Runner

| Attribute          | Requirement                                           |
|--------------------|-------------------------------------------------------|
| Architecture       | x86_64 bare-metal (no shared VM, no noisy neighbours) |
| CPU                | >= 4 physical cores, turbo boost disabled             |
| RAM                | >= 16 GB                                              |
| Disk               | NVMe SSD, >= 50 GB free on partition holding `/tmp`   |
| OS                 | Ubuntu 22.04 LTS (or 24.04 LTS) x86_64               |
| Kernel             | Stock distro kernel, no custom `isolcpus`/`cgroup`    |
| CPU governor        | `performance` (disable frequency scaling)             |
| Locale / TZ        | `en_US.UTF-8` / `UTC`                                 |

No background services should consume significant CPU or I/O during the
benchmark window (~50 minutes for release profile).

### 1.2 Rust Toolchain

```bash
rustup toolchain install 1.97.1
```

The spec (`benchmark-suite-v1.json`) pins `rust_toolchain: "1.97.1"`.
The release artifact attestation records the exact `rustc` binary SHA-256.

### 1.3 Java v4.0.3 Environment

- **JDK**: Temurin 17 (same major as `runtime_contract.java_version`)
- **Maven**: 3.9+ (bundled `mvnw` wrapper is sufficient)
- **Source**: Java easyexcel repository checked out at tag `v4.0.3`

```bash
cd /path/to/java
git checkout v4.0.3
git status   # must be clean
```

### 1.4 Python 3.10+

Required for `run_matrix.py`, `compare_results.py`, and
`prepare_release_artifacts.py`.

---

## 2. Build the Benchmark Runners

### 2.1 Rust Runner

```bash
cd /path/to/easyexcel-rust

cargo build --locked --release \
  --manifest-path benchmarks/rust-runner/Cargo.toml \
  -p easyexcel-benchmark-runner
```

Output: `target/release/easyexcel-benchmark-runner`

### 2.2 Java Runner (compile into test-classes)

```bash
cd /path/to/java

# Step 1: compile all modules + test sources
./mvnw -q -pl easyexcel-test -am -DskipTests test-compile

# Step 2: produce classpath file
./mvnw -q -pl easyexcel-test dependency:build-classpath \
  -DincludeScope=test -Dmdep.outputFile=target/benchmark-classpath.txt

# Step 3: build classpath variable
module_classes="$(find . -type d -path '*/target/classes' -print | paste -sd: -)"
dependencies="$(cat easyexcel-test/target/benchmark-classpath.txt)"
java_test_classes="$PWD/easyexcel-test/target/test-classes"
classpath="$java_test_classes:$module_classes:$dependencies"

# Step 4: compile the Java benchmark runner
javac -encoding UTF-8 -cp "$classpath" -d "$java_test_classes" \
  $(find benchmarks/java-runner/src -name '*.java' -print | sort)
```

The compiled runner class must be at:

```
<java-repo>/easyexcel-test/target/test-classes/com/alibaba/easyexcel/test/benchmark/EasyExcelBenchmarkRunner.class
```

This path is validated by `run_matrix.py:validate_release_inputs` (line 529-533).

---

## 3. Attest Release Artifacts (release profile only)

```bash
cd /path/to/easyexcel-rust

python3 benchmarks/scripts/prepare_release_artifacts.py \
  --rust-repo . \
  --java-repo /path/to/java \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --output artifacts/release-runner-artifact.json
```

The artifact manifest (`schema_version: 2`) records SHA-256 fingerprints of:

- Rust binary and source tree
- `rustc` compiler binary and version
- Java classpath entries (each JAR + test-classes directory)
- Java binary and source tree
- Git SHAs for both repositories

`validate_release_inputs` (line 511-583) rejects any mismatch between the
attestation and the actual environment.

---

## 4. Run the Release Benchmark Matrix

```bash
cd /path/to/easyexcel-rust

python3 benchmarks/scripts/run_matrix.py \
  --profile release \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --java-repo /path/to/java \
  --rust-repo . \
  --artifact-manifest artifacts/release-runner-artifact.json \
  --output-dir artifacts/release/matrix
```

### What the release profile runs

From `benchmark-suite-v1.json`:

| Parameter          | Value                              |
|--------------------|------------------------------------|
| rows               | [1000000]                          |
| temperatures       | [cold, steady]                     |
| warmups            | 3 (steady only)                    |
| measurements       | 7                                  |
| duration_seconds   | 1800                               |
| concurrency        | [1, 2, 4, 8, 16]                   |
| concurrency_scenarios | xlsx-stream-write, xlsx-event-read |

For each scenario in `concurrency_scenarios`, the matrix iterates over all
5 concurrency levels, both temperatures, and alternating Rust/Java trials.
The execution order is `rust, java, java, rust, ...` (7 measurements per
implementation, interleaved).

### Output

```
artifacts/release/matrix/raw-results.jsonl
artifacts/release/matrix/environment-manifest.json
artifacts/release/matrix/benchmark-suite-v1.json
```

Each line in `raw-results.jsonl` is one sample with fields including
`implementation`, `scenario_id`, `rows`, `worker_count`, `wall_time_ns`,
`trial`, `phase`, `temperature`, `fixture_origin`, and `success`.

---

## 5. Run the Four-Phase Soak (release only)

```bash
python3 benchmarks/scripts/run_soak.py \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --java-repo /path/to/java \
  --rust-repo . \
  --artifact-manifest artifacts/release-runner-artifact.json \
  --output-dir artifacts/release/soak
```

Output: `artifacts/release/soak/raw-results.jsonl` and `soak-manifest.json`.

---

## 6. Compare and Validate (cross_runtime gate)

```bash
python3 benchmarks/scripts/compare_results.py \
  --profile release \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --expected-java-git-sha "$(git -C /path/to/java rev-parse HEAD)" \
  --expected-rust-git-sha "$(git rev-parse HEAD)" \
  --soak-manifest artifacts/release/soak/soak-manifest.json \
  --baseline-candidate \
  --output artifacts/release/report.json \
  artifacts/release/matrix/raw-results.jsonl \
  artifacts/release/soak/raw-results.jsonl
```

### 6.1 How cross_runtime ratios are computed

`compare_results.py` (lines 1080-1125) computes cross_runtime ratios as
follows:

1. **Dimension grouping**: For each unique combination of `(phase, temperature,
   scenario_id, fixture_origin, rows, workers)`, gather all Rust and Java
   samples.

2. **Per-trial aggregation**: `trial_throughput_rates()` groups samples by
   trial number, computes `total_rows / max(wall_time_ns)` per trial, and
   returns one throughput value per trial (7 values for release).

3. **Bootstrap median ratio**: `bootstrap_median_ratio()` runs 10,000
   deterministic bootstrap iterations.  For each iteration, it resamples
   (with replacement) the Rust trial rates and Java trial rates, computes
   `median(rust) / median(java)`, and collects the ratio.  The output is:
   ```json
   {
     "median_ratio": <actual median Rust/Java ratio>,
     "confidence_level": 0.95,
     "confidence_lower_bound": <2.5th percentile of bootstrap ratios>,
     "confidence_upper_bound": <97.5th percentile>,
     "bootstrap_iterations": 10000
   }
   ```

4. **Gate enforcement**: For the `release` profile, the gate checks:
   - For scenarios in `cross_runtime.scenarios` (`xlsx-stream-write`,
     `xlsx-event-read`) at worker counts in `cross_runtime.worker_counts`
     (`[1, 2, 4, 8, 16]`):
     - Low concurrency (workers 1, 2, 4): `median_ratio >= 1.00` AND
       `confidence_lower_bound >= 0.95`
     - High concurrency (workers 8, 16): `median_ratio >= 0.90`

### 6.2 Expected output structure

The report JSON (`artifacts/release/report.json`) contains:

```json
{
  "schema_version": 1,
  "profile": "release",
  "sample_count": <total samples>,
  "valid_sample_count": <samples passing validation>,
  "summaries": { ... },
  "cross_runtime_ratios": {
    "matrix/steady/xlsx-event-read/rust/<origin>/<rows>/1": {
      "median_ratio": 1.35,
      "confidence_level": 0.95,
      "confidence_lower_bound": 1.28,
      "confidence_upper_bound": 1.42,
      "bootstrap_iterations": 10000,
      "rust_to_java_rows_per_second": 1.35,
      "java_to_rust_rows_per_second": 0.74
    },
    "matrix/steady/xlsx-event-read/rust/<origin>/<rows>/2": { ... },
    "matrix/steady/xlsx-event-read/rust/<origin>/<rows>/4": { ... },
    "matrix/steady/xlsx-event-read/rust/<origin>/<rows>/8": { ... },
    "matrix/steady/xlsx-event-read/rust/<origin>/<rows>/16": { ... },
    "matrix/steady/xlsx-stream-write/null/<rows>/1": { ... },
    "matrix/steady/xlsx-stream-write/null/<rows>/2": { ... },
    "matrix/steady/xlsx-stream-write/null/<rows>/4": { ... },
    "matrix/steady/xlsx-stream-write/null/<rows>/8": { ... },
    "matrix/steady/xlsx-stream-write/null/<rows>/16": { ... }
  },
  "failures": [],
  "passed": true
}
```

The report must show `"passed": true` and `"failures": []` for the
cross_runtime gate to be considered passing.

---

## 7. Failure Triage

### 7.1 Java classpath errors

**Symptom**: `RuntimeError: Java classpath must begin with
easyexcel-test/target/test-classes from --java-repo`

**Cause**: The `--java-classpath` argument does not start with the
`test-classes` directory from the Java repo.

**Fix**: Rebuild the classpath as shown in Section 2.2, Step 3.  Ensure
the first entry is `$java_test_classes` (the `test-classes` directory).

---

**Symptom**: `RuntimeError: prebuilt Java benchmark runner is missing:
<path>/EasyExcelBenchmarkRunner.class`

**Cause**: The `javac` compilation in Section 2.2, Step 4 either failed
or was not run.

**Fix**: Re-run the `javac` command.  Verify the `.class` file exists:
```bash
ls -la <java-repo>/easyexcel-test/target/test-classes/com/alibaba/easyexcel/test/benchmark/EasyExcelBenchmarkRunner.class
```

---

**Symptom**: `RuntimeError: Java release artifact attestation mismatch for
classpath` (or similar attestation errors)

**Cause**: The Java source tree or classpath entries changed between
artifact attestation and the benchmark run (e.g., `mvn clean` was run
between steps, or the wrong Java repo commit is checked out).

**Fix**: Re-run `prepare_release_artifacts.py` immediately before
`run_matrix.py` with the same environment.

---

### 7.2 Rust toolchain mismatch

**Symptom**: `RuntimeError: attested Rust compiler has changed since
runner preparation`

**Cause**: The `rustc` binary's SHA-256 does not match what was recorded
in the artifact manifest.  This happens when `rustup update` or a
toolchain switch occurs between attestation and the run.

**Fix**: Pin the toolchain and re-attest:
```bash
rustup default 1.97.1
# Rebuild and re-attest
cargo build --locked --release -p easyexcel-benchmark-runner
python3 benchmarks/scripts/prepare_release_artifacts.py ...
```

---

**Symptom**: `RuntimeError: Rust release artifact attestation mismatch for
git_sha` (or `binary_sha256`)

**Cause**: The Rust source tree changed (dirty worktree) or the binary
was rebuilt after attestation.

**Fix**: Ensure the worktree is clean (`git status`) and rebuild/re-attest.

---

### 7.3 Benchmark row count mismatch

**Symptom**: `RuntimeError: cross-runtime reread failed for <path>`

**Cause**: The Java and Rust runners disagree on the number of rows or
checksum of a written output file.  This indicates a correctness bug in
one of the implementations.

**Fix**: Check the `validations` list in the error message.  Compare
`observed_rows` and `checksum` between the two implementations.  If one
reports `success: false`, examine its stderr for the specific error.

---

### 7.4 cross_runtime gate threshold failures

**Symptom**: `Rust/Java median throughput ratio below 1.00: <label>`

**Cause**: The Rust implementation is slower than Java for the given
scenario/worker combination.  This should not happen for low-concurrency
cases (workers 1, 2, 4) given the 307K+ rows/s target.

**Triage**:
1. Check `environment-manifest.json` for unexpected hardware (e.g., shared
   VM, frequency scaling enabled).
2. Check if the runner was noisy (other processes during the run).
3. Examine the raw results for high variance (CV > 10%).
4. If consistently below 1.00, profile the Rust implementation.

---

**Symptom**: `Rust/Java throughput confidence lower bound below 0.95: <label>`

**Cause**: The bootstrap confidence interval is too wide, indicating
high variance in the measurements.

**Triage**:
1. Check if warmups are being applied (3 warmups for steady temperature).
2. Ensure CPU governor is `performance`.
3. Check for thermal throttling (`cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor`).
4. Consider increasing the number of measurements (currently 7).

---

### 7.5 Runner environment contract violations

**Symptom**: `RuntimeError: locale must be en_US.UTF-8`

**Fix**: Set locale before running:
```bash
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
```

---

**Symptom**: `RuntimeError: timezone must be UTC`

**Fix**: Set timezone:
```bash
export TZ=UTC
```

---

## 8. Quick Reference: Full Command Sequence

```bash
# --- On the fixed Linux runner ---

# 0. Environment setup
export PATH="$HOME/.cargo/bin:$PATH"
export LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 TZ=UTC

# 1. Build Rust runner
cd /path/to/easyexcel-rust
cargo build --locked --release -p easyexcel-benchmark-runner

# 2. Build Java runner
cd /path/to/java && git checkout v4.0.3
./mvnw -q -pl easyexcel-test -am -DskipTests test-compile
./mvnw -q -pl easyexcel-test dependency:build-classpath \
  -DincludeScope=test -Dmdep.outputFile=target/benchmark-classpath.txt
module_classes="$(find . -type d -path '*/target/classes' -print | paste -sd: -)"
dependencies="$(cat easyexcel-test/target/benchmark-classpath.txt)"
java_test_classes="$PWD/easyexcel-test/target/test-classes"
classpath="$java_test_classes:$module_classes:$dependencies"
javac -encoding UTF-8 -cp "$classpath" -d "$java_test_classes" \
  $(find benchmarks/java-runner/src -name '*.java' -print | sort)

# 3. Attest artifacts
cd /path/to/easyexcel-rust
python3 benchmarks/scripts/prepare_release_artifacts.py \
  --rust-repo . \
  --java-repo /path/to/java \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --output artifacts/release-runner-artifact.json

# 4. Run matrix
python3 benchmarks/scripts/run_matrix.py \
  --profile release \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --java-repo /path/to/java \
  --rust-repo . \
  --artifact-manifest artifacts/release-runner-artifact.json \
  --output-dir artifacts/release/matrix

# 5. Run soak
python3 benchmarks/scripts/run_soak.py \
  --rust-bin target/release/easyexcel-benchmark-runner \
  --java-bin "$JAVA_HOME/bin/java" \
  --java-classpath "$classpath" \
  --java-repo /path/to/java \
  --rust-repo . \
  --artifact-manifest artifacts/release-runner-artifact.json \
  --output-dir artifacts/release/soak

# 6. Compare and validate
python3 benchmarks/scripts/compare_results.py \
  --profile release \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --expected-java-git-sha "$(git -C /path/to/java rev-parse HEAD)" \
  --expected-rust-git-sha "$(git rev-parse HEAD)" \
  --soak-manifest artifacts/release/soak/soak-manifest.json \
  --baseline-candidate \
  --output artifacts/release/report.json \
  artifacts/release/matrix/raw-results.jsonl \
  artifacts/release/soak/raw-results.jsonl

# 7. Verify result
python3 -c "import json; r=json.load(open('artifacts/release/report.json')); print('passed' if r['passed'] else 'FAILED:', r['failures'])"
```

---

## 9. Local macOS Smoke Test (NOT a release gate)

The following is a quick local sanity check only.  It does NOT satisfy
the cross_runtime gate and must NOT be used as release evidence.

```bash
cd /path/to/easyexcel-rust

# Build the million_rows example
cargo build --release -p easyexcel --example million_rows

# Run with 100K rows
./target/release/examples/million_rows 100000
```

### Observed local macOS results (2026-08-11, Apple Silicon)

```
rows=100000
write_seconds=0.263
read_seconds=0.247
xlsx_bytes=1358729
```

Implied throughput:
- **Write**: ~380K rows/s
- **Read**: ~405 rows/s

These numbers are for local smoke-test reference only.  The release
cross_runtime gate requires Linux bare-metal results with the full
Java runner comparison.

---

## 10. Source References

| Component | Location |
|-----------|----------|
| cross_runtime gate definition | `benchmarks/spec/benchmark-suite-v1.json:81-88` |
| cross_runtime ratio + gate logic | `benchmarks/scripts/compare_results.py:1080-1125` |
| bootstrap median ratio | `benchmarks/scripts/compare_results.py:211-231` |
| release input validation | `benchmarks/scripts/run_matrix.py:511-583` |
| runner command construction | `benchmarks/scripts/run_matrix.py:33-93` |
| artifact attestation | `benchmarks/scripts/prepare_release_artifacts.py` |
| baseline generation runbook | `benchmarks/baselines/README.md` |
| ScenarioSpec (Rust) | `benchmarks/rust-runner/src/benchmark_spec/scenario_spec.rs` |
| read operation worker config | `benchmarks/rust-runner/src/operation.rs:92-149` |
