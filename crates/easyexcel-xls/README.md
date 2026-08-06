# easyexcel-xls

[简体中文](README.zh-CN.md)

BIFF8 `.xls` workbook reading and writing.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Detects Compound File Binary containers and maps BIFF8 records to the shared model.
- Reads and writes XLS workbooks through path and stream APIs.

## Architecture

```text
CFB / BIFF8 bytes <-> easyexcel-xls <-> Workbook
```

Main public surface: `read, read_path, write, write_path, looks_like_cfb`.

## Installation and usage

```toml
[dependencies]
easyexcel-xls = "0.1.1"
```

```rust
use easyexcel_xls::{read_path, write_path};
```

## Compatibility and limits

XLS Event Mode, legacy XLS password protection and placeholder filling are not claimed. Prefer `easyexcel::xls` in application code.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-xls)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
