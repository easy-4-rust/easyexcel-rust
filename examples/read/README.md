# Read Demo (CLI)

Reads an XLSX or CSV file and prints each row to stdout. This is the simplest
example of the `easyexcel-rust` read API and mirrors the Java
`easyexcel-demo` read module's main entry point.

## Quick start

```bash
# Read an XLSX file (default: target/demo-read.xlsx)
cargo run -p easyexcel-demo-read

# Read a specific file (format is auto-detected from extension)
cargo run -p easyexcel-demo-read -- path/to/data.xlsx
cargo run -p easyexcel-demo-read -- path/to/data.csv
```

## What it does

1. Accepts an optional file path as the first CLI argument. If omitted, defaults
   to `target/demo-read.xlsx`.
2. Calls `EasyExcel::read_sync::<DemoRow>(&path).do_read_sync()` to
   synchronously parse the file into a `Vec<DemoRow>`.
3. Prints the row count and each row's debug representation.

## Data model

The demo uses a simple three-column `DemoRow` struct annotated with
`#[derive(ExcelRow)]`:

| Column | Name | Type | Index |
|---|---|---|---:|
| 0 | 名称 | `String` | 0 |
| 1 | 日期 | `NaiveDateTime` | 1 |
| 2 | 数值 | `f64` | 2 |

You can adapt the struct to match your own spreadsheet schema by changing the
`#[excel(name = ..., index = ...)]` attributes.

## Input / Output

- **Input**: Any XLSX or CSV file whose columns match the `DemoRow` schema.
- **Output**: Row count and per-row debug lines printed to stdout.

## Correspondence with Java demo

| Java class | Rust equivalent |
|---|---|
| `com.alibaba.easyexcel.demo.read.ReadDemo` | `examples/read/src/main.rs` |
| `ReadListener` callback pattern | `EasyExcel::read_sync` + `do_read_sync` (synchronous batch) |

## Associated test

The integration test `tests/spawn_binary.rs` verifies the demo binary is
executable end-to-end:

1. Generates a temporary XLSX with 2 known rows using the library.
2. Spawns `easyexcel-demo-read` as a subprocess with the temp file.
3. Asserts exit code 0 and stdout contains the expected row data.

Run it with:

```bash
cargo test -p easyexcel-demo-read
```
