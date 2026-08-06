# easyexcel

[简体中文](README.zh-CN.md)

The public EasyExcel-Rust facade with Java EasyExcel-style builders, listeners, converters and handlers.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Orchestrates typed and dynamic XLSX/XLS/CSV reading and writing.
- Re-exports engine APIs through `easyexcel::{model, io, csv, xls, xlsx, formula, markdown, tabular}`.

## Architecture

```text
application -> easyexcel builders -> format engines -> spreadsheet files
```

Main public surface: `EasyExcel, EasyExcelFactory, ExcelRow, ExcelReaderBuilder, ExcelWriterBuilder`.

## Installation and usage

```toml
[dependencies]
easyexcel = "0.1.1"
```

```rust
use easyexcel::{EasyExcel, ExcelRow};

#[derive(ExcelRow)]
struct User {
    #[excel(name = "Name")]
    name: String,
}

let rows = EasyExcel::read_sync::<User>("users.xlsx").do_read_sync()?;
```

## Compatibility and limits

This is the recommended dependency for Rust applications. Advanced-format lossless behavior and unsupported features are documented in the repository compatibility matrix.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
