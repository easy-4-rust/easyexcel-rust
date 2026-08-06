#!/usr/bin/env sh
set -eu

rows="${1:-1000000}"
output="${2:-target/benchmark/million-rows.xlsx}"
cargo_command="${CARGO:-cargo}"
"$cargo_command" build --release -p easyexcel --example million_rows
binary="target/release/examples/million_rows"

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
