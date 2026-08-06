# easyexcel-poem

[简体中文](README.zh-CN.md)

Native Poem integration for the shared EasyExcel Web runtime.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Exposes Poem-native extractor and responder types as `ExcelRequest<T>` and `ExcelResponse<T>`.
- Maps shared policy and problem details to Poem transport primitives.

## Architecture

```text
Poem request -> easyexcel-poem -> easyexcel-web -> EasyExcel engines -> Poem response
```

Main public surface: `ExcelRequest, ExcelResponse, ExcelPoemError, ExcelWebPolicy, ExcelWebRuntime`.

## Installation and usage

```toml
[dependencies]
easyexcel-poem = "0.1.1"
```

```rust
use easyexcel_poem::{ExcelRequest, ExcelResponse, ExcelWebPolicy};
```

## Compatibility and limits

Business rules, parsing and resource enforcement stay in `easyexcel-web`; this crate only owns the Poem transport adapter. See `examples/poem` in the repository.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-poem)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
