# easyexcel-io

[简体中文](README.zh-CN.md)

Shared spreadsheet I/O contracts, format detection, streaming rows, limits and typed errors.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Defines `RowSource`/`RowSink`, stream metadata and read/write modes.
- Centralizes format detection, sheet selection and resource-limit enforcement contracts.

## Architecture

```text
bytes / paths -> easyexcel-io contracts -> format engines
```

Main public surface: `Format, ResourceLimits, RowSource, RowSink, StreamCell, ReadMode, WriteMode`.

## Installation and usage

```toml
[dependencies]
easyexcel-io = "0.1.1"
```

```rust
use easyexcel_io::{Format, ResourceLimits, RowSink, RowSource};
```

## Compatibility and limits

Concrete XLS, XLSX and CSV codecs live in their format crates. Prefer `easyexcel::io` in application code.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-io)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
