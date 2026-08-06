# easyexcel-format

[简体中文](README.zh-CN.md)

Spreadsheet number, date and display formatting algorithms with Java-compatible behavior.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Resolves built-in and custom number formats.
- Formats decimal, integer, floating-point and date values deterministically.

## Architecture

```text
raw cell value + format code -> easyexcel-format -> display text
```

Main public surface: `ExcelLocale, NumberRoundingMode, builtin_format_code, format_with_code`.

## Installation and usage

```toml
[dependencies]
easyexcel-format = "0.1.1"
```

```rust
use easyexcel_format::{NumberRoundingMode, builtin_format_code, format_with_code};
```

## Compatibility and limits

This crate formats values; it does not read or write spreadsheet containers.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-format)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
