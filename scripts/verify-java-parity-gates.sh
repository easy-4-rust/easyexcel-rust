#!/usr/bin/env bash
# Four independent Java 4.0.3 parity gates:
# 1. reproducible javap and cargo-public-api snapshots;
# 2. source inventory + Rust compile presence;
# 3. executable behavior suites;
# 4. Java-produced golden artifacts + fail-closed per-API evidence.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JAVA_REPO="${1:-${EASYEXCEL_JAVA_REPO:-}}"

if [[ -z "$JAVA_REPO" ]]; then
    echo "usage: $0 /path/to/easyexcel-java-repository" >&2
    exit 2
fi

JAVA_TEST_ROOT="$JAVA_REPO/easyexcel-test/src/test/java"
if [[ ! -d "$JAVA_TEST_ROOT" ]]; then
    echo "Java test root does not exist: $JAVA_TEST_ROOT" >&2
    exit 2
fi

if [[ -n "${JAVAP_BIN:-}" ]]; then
    JAVAP="$JAVAP_BIN"
elif [[ -x /usr/libexec/java_home ]]; then
    JAVAP="$(/usr/libexec/java_home -v 17)/bin/javap"
else
    JAVAP="$(command -v javap)"
fi

JAVA_TAG="$(git -C "$JAVA_REPO" describe --tags --exact-match HEAD 2>/dev/null || true)"
if [[ "$JAVA_TAG" != "v4.0.3" ]]; then
    echo "Java public API authority must be the exact v4.0.3 tag, got: ${JAVA_TAG:-<none>}" >&2
    exit 2
fi
if [[ -n "$(git -C "$JAVA_REPO" status --porcelain --untracked-files=normal)" ]]; then
    echo "Java public API authority requires a clean v4.0.3 worktree: $JAVA_REPO" >&2
    exit 2
fi

# javap 必须读取本次从权威源码构建的模块产物，不能回退到 ~/.m2 中来源不明的同版本 JAR。
JAVA_CORE_JAR="$JAVA_REPO/easyexcel-core/target/easyexcel-core-4.0.3.jar"
JAVA_HOME_FOR_GATE="$(cd "$(dirname "$JAVAP")/.." && pwd)"
MAVEN="$JAVA_REPO/mvnw"
if [[ ! -x "$MAVEN" ]]; then
    MAVEN="$(command -v mvn)"
fi
JAVA_HOME="$JAVA_HOME_FOR_GATE" "$MAVEN" \
    -pl easyexcel-core -am -DskipTests package
if [[ ! -f "$JAVA_CORE_JAR" ]]; then
    echo "authoritative EasyExcel 4.0.3 core JAR was not produced: $JAVA_CORE_JAR" >&2
    exit 2
fi

cd "$REPO_ROOT"

echo "[1/4] Java javap and Rust cargo-public-api snapshot gate"
python3 scripts/generate_java_public_api.py \
    --java-root "$JAVA_REPO" \
    --jar "$JAVA_CORE_JAR" \
    --javap "$JAVAP" \
    --output docs/java-public-api-v4.0.3.json \
    --markdown-output docs/java-public-api-v4.0.3.md \
    --check
python3 scripts/generate_rust_public_api.py \
    --rust-root "$REPO_ROOT" \
    --output docs/rust-public-api.json \
    --check

echo "[2/4] Java source inventory and Rust compile-presence gate"
python3 scripts/generate_source_test_parity.py \
    --java-root "$JAVA_TEST_ROOT" \
    --rust-root "$REPO_ROOT" \
    --check
cargo test -p easyexcel-test --no-run

echo "[3/4] Java-mapped behavior gate"
cargo test -p easyexcel-test \
    --test java_parity_tests \
    --test java_full_parity_tests \
    --test temp_1to1_tests \
    --test codegraph_phaseE_metadata_1to1_tests

echo "[4/4] Java-produced golden and public API evidence gate"
./scripts/export-java-golden.sh --check
cargo test -p easyexcel-test --test java_golden_tests
python3 scripts/suggest_public_api_mapping.py \
    --java-api docs/java-public-api-v4.0.3.json \
    --rust-api docs/rust-public-api.json \
    --output target/public-api-candidates.json
python3 scripts/apply_public_api_evidence.py \
    --mapping target/public-api-candidates.json \
    --catalog parity/public-api-evidence.json \
    --output target/java-rust-public-api.json
if ! cmp -s target/java-rust-public-api.json parity/java-rust-public-api.json; then
    echo "public API mapping is stale; regenerate candidates and apply evidence" >&2
    exit 1
fi
python3 scripts/run_public_api_evidence.py \
    --catalog parity/public-api-evidence.json \
    --output target/public-api-evidence-results.json \
    --repo-root "$REPO_ROOT"
python3 scripts/verify_public_api_parity.py \
    --java-api docs/java-public-api-v4.0.3.json \
    --rust-api docs/rust-public-api.json \
    --mapping target/java-rust-public-api.json \
    --evidence-catalog parity/public-api-evidence.json \
    --evidence-results target/public-api-evidence-results.json \
    --repo-root "$REPO_ROOT" \
    --report docs/public-api-parity-report.json
