# easyexcel-csv

[简体中文](README.zh-CN.md)

CSV/TSV decoding, encoding, delimiter detection, type inference and streaming row sources.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Reads and writes delimited workbooks with configurable charset and dialect options.
- Provides `CsvRowSource` for incremental row processing.

## Architecture

```text
CSV / TSV bytes -> easyexcel-csv -> Workbook or row stream
```

Main public surface: `CsvReadOptions, CsvWriteOptions, CsvRowSource, CsvRecordReader, CsvRecordWriter`.

## Installation and usage

```toml
[dependencies]
easyexcel-csv = "0.1.1"
```

```rust
use easyexcel_csv::{CsvReadOptions, CsvRowSource, CsvWriteOptions};
```

## Compatibility and limits

CSV has no native formulas, merges, styles or multiple-sheet semantics. Prefer `easyexcel::csv` in application code.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-csv)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
