# Write Demo (CLI)

Generates an XLSX file with sample data and writes it to disk. This example
demonstrates the `easyexcel-rust` write API and mirrors the Java
`easyexcel-demo` write module's main entry point.

## Quick start

```bash
# Write to default path (target/demo-write.xlsx)
cargo run -p easyexcel-demo-write

# Write to a custom path
cargo run -p easyexcel-demo-write -- path/to/output.xlsx
```

## What it does

1. Accepts an optional output path as the first CLI argument. If omitted,
   defaults to `target/demo-write.xlsx`.
2. Generates 5 sample `DemoRow` entries (with sequential names like
   "项目0" .. "项目4", a fixed date, and incremental amounts).
3. Calls `EasyExcel::write::<DemoRow>(&path).sheet("数据").do_write(rows)`
   to produce the spreadsheet.
4. Prints the row count and output path to stdout.

## Data model

| Column | Name | Type | Index |
|---|---|---|---:|
| 0 | 名称 | `String` | 0 |
| 1 | 日期 | `NaiveDateTime` | 1 |
| 2 | 数值 | `f64` | 2 |

## Input / Output

- **Input**: None (data is generated in-memory).
- **Output**: An XLSX file at the specified or default path. The file starts
  with the standard ZIP magic bytes (`PK`), confirming a valid Office Open XML
  archive.

## Correspondence with Java demo

| Java class | Rust equivalent |
|---|---|
| `com.alibaba.easyexcel.demo.write.WriteDemo` | `examples/write/src/main.rs` |
| `EasyExcel.write(...).sheet(...).doWrite(...)` | `EasyExcel::write(...).sheet(...).do_write(...)` |

## Associated test

The integration test `tests/spawn_binary.rs` verifies the demo binary:

1. Spawns `easyexcel-demo-write` as a subprocess with a temporary output path.
2. Asserts exit code 0, stdout contains "已写入 5 行", the output file exists,
   and the first two bytes are `PK` (valid XLSX).

Run it with:

```bash
cargo test -p easyexcel-demo-write
```
