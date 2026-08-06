# easyexcel-actix

[简体中文](README.zh-CN.md)

Native Actix Web integration for the shared EasyExcel Web runtime.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Exposes Actix Web-native extractor and responder types as `ExcelRequest<T>` and `ExcelResponse<T>`.
- Maps shared policy and problem details to Actix Web transport primitives.

## Architecture

```text
Actix Web request -> easyexcel-actix -> easyexcel-web -> EasyExcel engines -> Actix Web response
```

Main public surface: `ExcelRequest, ExcelResponse, ExcelActixError, ExcelWebPolicy, ExcelWebRuntime`.

## Installation and usage

```toml
[dependencies]
easyexcel-actix = "0.1.1"
```

```rust
use easyexcel_actix::{ExcelRequest, ExcelResponse, ExcelWebPolicy};
```

## Compatibility and limits

Business rules, parsing and resource enforcement stay in `easyexcel-web`; this crate only owns the Actix Web transport adapter. See `examples/actix` in the repository.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-actix)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
