#!/usr/bin/env sh
set -eu

rows="${1:-1000000}"
output="${2:-target/benchmark/million-rows.xlsx}"
binary="${EASYEXCEL_MILLION_ROWS_BIN:-target/release/examples/million_rows}"

if [ ! -x "$binary" ]; then
    echo "prebuilt benchmark binary is required: $binary" >&2
    echo "build it before measurement; benchmark scripts never compile timed code" >&2
    exit 2
fi

case "$(uname -s)" in
    Darwin)
        exec /usr/bin/time -l "$binary" "$rows" "$output"
        ;;
    Linux)
        exec /usr/bin/time -v "$binary" "$rows" "$output"
        ;;
    *)
        exec "$binary" "$rows" "$output"
        ;;
esac
