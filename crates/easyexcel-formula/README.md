# easyexcel-formula

[简体中文](README.zh-CN.md)

Excel-compatible formula parsing, evaluation, recalculation and dynamic-array support.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Parses formulas into an expression model and evaluates them against a workbook context.
- Reports recalculation results while retaining explicit unsupported-function boundaries.

## Architecture

```text
formula text + Workbook -> easyexcel-formula -> value / recalculation report
```

Main public surface: `Engine, Evaluator, Expr, Context, RecalcReport, parse`.

## Installation and usage

```toml
[dependencies]
easyexcel-formula = "0.1.1"
```

```rust
use easyexcel_formula::{Engine, Expr, RecalcReport, parse};
```

## Compatibility and limits

The engine does not promise complete coverage of every Excel function or external-data function.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-formula)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
