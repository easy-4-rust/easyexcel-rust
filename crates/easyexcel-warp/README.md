# easyexcel-warp

[简体中文](README.zh-CN.md)

Native Warp integration for the shared EasyExcel Web runtime.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Exposes Warp-native filter, rejection recovery and reply types as `ExcelRequest<T>` and `ExcelResponse<T>`.
- Maps shared policy and problem details to Warp transport primitives.

## Architecture

```text
Warp request -> easyexcel-warp -> easyexcel-web -> EasyExcel engines -> Warp response
```

Main public surface: `ExcelRequest, ExcelResponse, ExcelWarpRejection, ExcelWebPolicy, ExcelWebRuntime`.

## Installation and usage

```toml
[dependencies]
easyexcel-warp = "0.1.1"
```

```rust
use easyexcel_warp::{ExcelRequest, ExcelResponse, ExcelWebPolicy};
```

## Compatibility and limits

Business rules, parsing and resource enforcement stay in `easyexcel-web`; this crate only owns the Warp transport adapter. See `examples/warp` in the repository.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-warp)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
