# easyexcel-axum

[简体中文](README.zh-CN.md)

Native Axum integration for the shared EasyExcel Web runtime.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Exposes Axum-native extractor and responder types as `ExcelRequest<T>` and `ExcelResponse<T>`.
- Maps shared policy and problem details to Axum transport primitives.

## Architecture

```text
Axum request -> easyexcel-axum -> easyexcel-web -> EasyExcel engines -> Axum response
```

Main public surface: `ExcelRequest, ExcelResponse, ExcelRejection, ExcelWebPolicy, ExcelWebRuntime`.

## Installation and usage

```toml
[dependencies]
easyexcel-axum = "0.1.1"
```

```rust
use easyexcel_axum::{ExcelRequest, ExcelResponse, ExcelWebPolicy};
```

## Compatibility and limits

Business rules, parsing and resource enforcement stay in `easyexcel-web`; this crate only owns the Axum transport adapter. See `examples/axum` in the repository.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-axum)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
