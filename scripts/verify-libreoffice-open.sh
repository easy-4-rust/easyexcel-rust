#!/usr/bin/env sh
# Verification evidence 4: Excel and LibreOffice open all generated fixtures
# without repair warnings (docs/compatibility.md).
#
# Behaviour:
#   - If `soffice` / LibreOffice is not installed, prints setup instructions
#     and exits 0 (verification is skipped and documented, not failed).
#   - If LibreOffice is installed, generates a representative fixture set
#     (simple write, template fill, styles, merged cells, image, legacy XLS,
#     CSV) with the `generate_compat_fixtures` example, then opens every
#     fixture with `soffice --headless --convert-to csv` and fails on a
#     non-zero exit code or a "repair" message in the output.
#   - Password-encrypted fixtures are generated under a `protected/`
#     subdirectory and skipped: LibreOffice headless conversion cannot supply
#     a password non-interactively. They are covered by the crate's
#     round-trip tests instead.
#
# Usage:
#   ./scripts/verify-libreoffice-open.sh            # default fixture dir
#   ./scripts/verify-libreoffice-open.sh <outdir>   # custom fixture dir
#
# Exits 0 on PASS or SKIP, 1 when LibreOffice is installed but a fixture
# fails to open cleanly.

set -eu

fixtures_out="${1:-target/compat-fixtures}"
cargo_command="${CARGO:-cargo}"
timeout_seconds="${VERIFY_LO_TIMEOUT:-60}"

# --- locate LibreOffice ---------------------------------------------------
soffice=""
if command -v soffice >/dev/null 2>&1; then
    soffice="$(command -v soffice)"
elif command -v libreoffice >/dev/null 2>&1; then
    soffice="$(command -v libreoffice)"
elif [ -x "/Applications/LibreOffice.app/Contents/MacOS/soffice" ]; then
    soffice="/Applications/LibreOffice.app/Contents/MacOS/soffice"
fi

if [ -z "$soffice" ]; then
    echo "=== LibreOffice open verification: SKIPPED ==="
    echo "LibreOffice / soffice is not installed on this machine, so"
    echo "verification evidence 4 (Excel/LibreOffice open every generated"
    echo "fixture without repair warnings) cannot run here."
    echo
    echo "Install LibreOffice to run it:"
    echo "  macOS:        brew install --cask libreoffice"
    echo "  Debian/Ubuntu: sudo apt install libreoffice"
    echo "  Fedora:       sudo dnf install libreoffice"
    echo
    echo "Then re-run:  ./scripts/verify-libreoffice-open.sh"
    echo "Skipping (exit 0) as documented in docs/compatibility.md."
    exit 0
fi

# --- portable timeout (macOS lacks `timeout` by default) ------------------
# Runs "$@" with a wall-clock limit; returns 124 on timeout, otherwise the
# command's exit status.
run_with_timeout() {
    seconds="$1"
    shift
    "$@" >"$stdout_log" 2>"$stderr_log" &
    pid=$!
    waited=0
    while kill -0 "$pid" 2>/dev/null; do
        sleep 1
        waited=$((waited + 1))
        if [ "$waited" -ge "$seconds" ]; then
            kill "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
            return 124
        fi
    done
    wait "$pid"
    return $?
}

workdir="$(mktemp -d "${TMPDIR:-/tmp}/verify-libreoffice.XXXXXX")"
outdir="$workdir/out"
mkdir -p "$outdir"
stdout_log="$workdir/stdout.log"
stderr_log="$workdir/stderr.log"
trap 'rm -rf "$workdir"' EXIT

echo "=== LibreOffice open verification ==="
echo "soffice:       $soffice"
echo "fixtures dir:  $fixtures_out"

# --- generate fixtures -----------------------------------------------------
echo "--- generating fixtures ---"
"$cargo_command" run --release -p easyexcel --example generate_compat_fixtures -- \
    "$fixtures_out"
fixture_count=0
for file in "$fixtures_out"/*.xlsx "$fixtures_out"/*.xls "$fixtures_out"/*.csv; do
    [ -f "$file" ] && fixture_count=$((fixture_count + 1))
done
echo "generated $fixture_count top-level fixtures (password-protected ones live"
echo "under $fixtures_out/protected/ and are skipped by design)"

# --- convert every fixture and check for repair warnings --------------------
echo "--- opening each fixture with soffice ---"
passed=0
failed=0
for file in "$fixtures_out"/*.xlsx "$fixtures_out"/*.xls "$fixtures_out"/*.csv; do
    [ -f "$file" ] || continue
    name="$(basename "$file")"
    rm -f "$stdout_log" "$stderr_log"
    rm -rf "$outdir"/*
    if run_with_timeout "$timeout_seconds" "$soffice" --headless --norestore \
        -env:UserInstallation="file://$workdir/profile" \
        --convert-to csv --outdir "$outdir" "$file"; then
        combined="$(cat "$stdout_log" "$stderr_log")"
        if printf '%s' "$combined" | grep -qi 'repair'; then
            echo "FAIL  $name  (soffice reported a repair warning)"
            printf '%s\n' "$combined" | grep -i 'repair' | sed 's/^/      /' || true
            failed=$((failed + 1))
        else
            echo "PASS  $name"
            passed=$((passed + 1))
        fi
    else
        status=$?
        if [ "$status" -eq 124 ]; then
            echo "FAIL  $name  (soffice timed out after ${timeout_seconds}s)"
        else
            echo "FAIL  $name  (soffice exited with status $status)"
        fi
        sed 's/^/      /' "$stderr_log" | head -5
        failed=$((failed + 1))
    fi
done

echo "--- summary ---"
if [ "$failed" -eq 0 ]; then
    echo "RESULT: PASS — $passed/$passed top-level fixtures opened without repair warnings."
    exit 0
fi
echo "RESULT: FAIL — $passed passed, $failed failed."
exit 1
