# easyexcel-salvo

[简体中文](README.zh-CN.md)

Native Salvo integration for the shared EasyExcel Web runtime.

> Release line: 0.1.1 · Rust 1.89+ · Edition 2024 · Apache-2.0

## Responsibilities

- Exposes Salvo-native extractor and writer types as `ExcelRequest<T>` and `ExcelResponse<T>`.
- Maps shared policy and problem details to Salvo transport primitives.

## Architecture

```text
Salvo request -> easyexcel-salvo -> easyexcel-web -> EasyExcel engines -> Salvo response
```

Main public surface: `ExcelRequest, ExcelResponse, ExcelSalvoError, ExcelWebPolicy, ExcelWebRuntime`.

## Installation and usage

```toml
[dependencies]
easyexcel-salvo = "0.1.1"
```

```rust
use easyexcel_salvo::{ExcelRequest, ExcelResponse, ExcelWebPolicy};
```

## Compatibility and limits

Business rules, parsing and resource enforcement stay in `easyexcel-web`; this crate only owns the Salvo transport adapter. See `examples/salvo` in the repository.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-salvo)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
