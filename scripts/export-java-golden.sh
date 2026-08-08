#!/usr/bin/env bash
# Export Java EasyExcel golden JSON expectations into tests/easyexcel-test/tests/golden/.
#
# Uses scripts/java-golden-export (Maven) against Alibaba EasyExcel 4.0.3.
# - Reads checked-in fixtures under tests/easyexcel-test/tests/fixtures
# - Writes SimpleData xlsx/csv artifacts under tests/golden/artifacts/
# - Emits *.expected.json (STRING-mode display cells) for Rust对照
#
# Dependencies:
#   - JDK 8+ (JAVA_HOME or Homebrew OpenJDK; EASYEXCEL_JAVA_HOME overrides)
#   - Apache Maven 3.6+ (`mvn` on PATH)
#   - Network once to resolve Maven deps (com.alibaba:easyexcel:4.0.3)
#
# Usage:
#   ./scripts/export-java-golden.sh
#   ./scripts/export-java-golden.sh --check
#   EASYEXCEL_JAVA_HOME=/path/to/jdk ./scripts/export-java-golden.sh
#   FIXTURES_DIR=/path/to/fixtures OUT_DIR=/path/to/golden ./scripts/export-java-golden.sh
#
# After export, commit updated tests/golden/*.expected.json (and artifacts/) so
# `cargo test -p easyexcel-test --test java_golden_tests` passes without a local JDK.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXPORT_DIR="${ROOT}/scripts/java-golden-export"
FIXTURES_DIR="${FIXTURES_DIR:-${ROOT}/tests/easyexcel-test/tests/fixtures}"
COMMITTED_OUT_DIR="${OUT_DIR:-${ROOT}/tests/easyexcel-test/tests/golden}"
CHECK_MODE=false
TEMP_OUT_DIR=""

if [[ "${1:-}" == "--check" ]]; then
  CHECK_MODE=true
elif [[ $# -ne 0 ]]; then
  echo "usage: $0 [--check]" >&2
  exit 2
fi

if [[ "$CHECK_MODE" == true ]]; then
  TEMP_OUT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/easyexcel-java-golden-check.XXXXXX")"
  trap 'rm -rf "${TEMP_OUT_DIR}"' EXIT
  EFFECTIVE_OUT_DIR="$TEMP_OUT_DIR"
else
  EFFECTIVE_OUT_DIR="$COMMITTED_OUT_DIR"
fi

# Prefer Homebrew OpenJDK when plain `java` is missing from PATH (macOS).
if [[ -z "${JAVA_HOME:-}" ]]; then
  if [[ -n "${EASYEXCEL_JAVA_HOME:-}" ]]; then
    export JAVA_HOME="${EASYEXCEL_JAVA_HOME}"
  elif [[ -d /opt/homebrew/Cellar/openjdk ]]; then
    # shellcheck disable=SC2012
    LATEST="$(ls -1d /opt/homebrew/Cellar/openjdk/*/libexec/openjdk.jdk/Contents/Home 2>/dev/null | tail -1 || true)"
    if [[ -n "${LATEST}" ]]; then
      export JAVA_HOME="${LATEST}"
    fi
  elif command -v /usr/libexec/java_home >/dev/null 2>&1; then
    export JAVA_HOME="$(/usr/libexec/java_home 2>/dev/null || true)"
  fi
fi
if [[ -n "${JAVA_HOME:-}" ]]; then
  export PATH="${JAVA_HOME}/bin:${PATH}"
fi

if ! command -v mvn >/dev/null 2>&1; then
  echo "error: mvn not found; install Maven (https://maven.apache.org/) to export Java goldens" >&2
  exit 1
fi
if ! command -v java >/dev/null 2>&1; then
  echo "error: java not found; set JAVA_HOME or EASYEXCEL_JAVA_HOME (JDK 8+)" >&2
  exit 1
fi

if [[ ! -d "${FIXTURES_DIR}" ]]; then
  echo "error: fixtures dir missing: ${FIXTURES_DIR}" >&2
  exit 1
fi

mkdir -p "${EFFECTIVE_OUT_DIR}"

echo "==> Java golden export"
echo "    fixtures: ${FIXTURES_DIR}"
echo "    out:      ${EFFECTIVE_OUT_DIR}"
echo "    java:     $(java -version 2>&1 | head -1)"
echo "    mvn:      $(mvn -version 2>&1 | head -1)"

(
  cd "${EXPORT_DIR}"
  mvn -q -DskipTests package exec:java \
    -Dexec.mainClass=com.alibaba.easyexcel.golden.JavaGoldenExporter \
    -Dexec.args="${FIXTURES_DIR} ${EFFECTIVE_OUT_DIR}"
)

if [[ "$CHECK_MODE" == true ]]; then
  if [[ ! -d "$COMMITTED_OUT_DIR" ]]; then
    echo "error: committed golden dir missing: ${COMMITTED_OUT_DIR}" >&2
    exit 1
  fi
  while IFS= read -r generated; do
    relative="${generated#${EFFECTIVE_OUT_DIR}/}"
    committed="${COMMITTED_OUT_DIR}/${relative}"
    if [[ ! -f "$committed" ]]; then
      echo "error: generated golden is not committed: ${relative}" >&2
      exit 1
    fi
    # POI embeds volatile ZIP/CFB metadata in binary artifacts. Their observable
    # content is checked by java_golden_tests against the freshly compared JSON.
    if [[ "$relative" == *.expected.json || "$relative" == *.contract.json ]] && ! cmp -s "$generated" "$committed"; then
      echo "error: stale Java golden: ${relative}" >&2
      exit 1
    fi
  done < <(find "$EFFECTIVE_OUT_DIR" -type f -print | LC_ALL=C sort)
  while IFS= read -r committed; do
    relative="${committed#${COMMITTED_OUT_DIR}/}"
    generated="${EFFECTIVE_OUT_DIR}/${relative}"
    if [[ ! -f "$generated" ]]; then
      echo "error: committed golden is no longer generated: ${relative}" >&2
      exit 1
    fi
  done < <(find "$COMMITTED_OUT_DIR" -type f \( -name '*.expected.json' -o -name '*.contract.json' \) -print | LC_ALL=C sort)
  echo "==> Java golden freshness check passed"
  exit 0
fi

echo "==> Done. Golden JSON:"
ls -1 "${EFFECTIVE_OUT_DIR}"/*.expected.json 2>/dev/null || true
if [[ -d "${EFFECTIVE_OUT_DIR}/artifacts" ]]; then
  echo "==> Artifacts (Java-written):"
  ls -1 "${EFFECTIVE_OUT_DIR}/artifacts"/ 2>/dev/null || true
fi
