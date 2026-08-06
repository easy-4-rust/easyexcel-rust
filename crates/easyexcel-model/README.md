# easyexcel-model

[简体中文](README.zh-CN.md)

The format-neutral workbook and tabular data model shared by every EasyExcel-Rust engine.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Models workbooks, sheets, cells, styles, merges, names, tables and opaque parts.
- Converts between `Workbook` and `TabularDocument` without claiming lossless formula or style round-trips.

## Architecture

```text
XLS / XLSX / CSV engines -> easyexcel-model -> facade / converters
```

Main public surface: `Workbook, Sheet, Cell, CellValue, CellRange, TabularDocument`.

## Installation and usage

```toml
[dependencies]
easyexcel-model = "0.1.1"
```

```rust
use easyexcel_model::{Cell, CellValue, Workbook};
```

## Compatibility and limits

This crate contains no XLS, XLSX, CSV, ZIP or XML parser. Application code should normally import these types through `easyexcel::model`.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-model)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
