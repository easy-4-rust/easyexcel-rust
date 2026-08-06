# easyexcel-xlsx

[简体中文](README.zh-CN.md)

OOXML `.xlsx` reading, writing, event streaming, templates, encryption and round-trip support.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Reads/writes workbook packages and exposes event-oriented sheet readers.
- Supports template materialization, encrypted OOXML and preservation-oriented package handling.

## Architecture

```text
ZIP / OOXML bytes <-> easyexcel-xlsx <-> Workbook / event stream
```

Main public surface: `read_path, write_path, XlsxCellEventReader, OoxmlPackage, TemplateFillData`.

## Installation and usage

```toml
[dependencies]
easyexcel-xlsx = "0.1.1"
```

```rust
use easyexcel_xlsx::{XlsxCellEventReader, read_path, write_path};
```

## Compatibility and limits

Unknown OOXML parts are preserved where supported, but macro, chart and every advanced-object edit are not guaranteed lossless. Prefer `easyexcel::xlsx`.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-xlsx)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
