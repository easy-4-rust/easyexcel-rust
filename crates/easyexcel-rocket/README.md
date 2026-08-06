# easyexcel-rocket

[简体中文](README.zh-CN.md)

Native Rocket integration for the shared EasyExcel Web runtime.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Exposes Rocket-native data guard and responder types as `ExcelRequest<T>` and `ExcelResponse<T>`.
- Maps shared policy and problem details to Rocket transport primitives.

## Architecture

```text
Rocket request -> easyexcel-rocket -> easyexcel-web -> EasyExcel engines -> Rocket response
```

Main public surface: `ExcelRequest, ExcelResponse, ExcelRocketError, ExcelWebPolicy, ExcelWebRuntime`.

## Installation and usage

```toml
[dependencies]
easyexcel-rocket = "0.1.1"
```

```rust
use easyexcel_rocket::{ExcelRequest, ExcelResponse, ExcelWebPolicy};
```

## Compatibility and limits

Business rules, parsing and resource enforcement stay in `easyexcel-web`; this crate only owns the Rocket transport adapter. See `examples/rocket` in the repository.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-rocket)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
