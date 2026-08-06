# easyexcel-hyper

[简体中文](README.zh-CN.md)

Native Hyper integration for the shared EasyExcel Web runtime.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Exposes Hyper-native request and streaming-response bridge types as `ExcelRequest<T>` and `ExcelResponse<T>`.
- Maps shared policy and problem details to Hyper transport primitives.

## Architecture

```text
Hyper request -> easyexcel-hyper -> easyexcel-web -> EasyExcel engines -> Hyper response
```

Main public surface: `ExcelRequest, ExcelResponse, ExcelHyperError, ExcelWebPolicy, ExcelWebRuntime`.

## Installation and usage

```toml
[dependencies]
easyexcel-hyper = "0.1.1"
```

```rust
use easyexcel_hyper::{ExcelRequest, ExcelResponse, ExcelWebPolicy};
```

## Compatibility and limits

Business rules, parsing and resource enforcement stay in `easyexcel-web`; this crate only owns the Hyper transport adapter. See `examples/hyper` in the repository.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-hyper)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
