# easyexcel-markdown

[简体中文](README.zh-CN.md)

Policy-driven Markdown projection for workbooks and streaming rows.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Parses GFM tables into `TabularDocument`.
- Exports workbook or row streams with formula, merge, type-inference and loss-report policies.

## Architecture

```text
Workbook / RowSource <-> easyexcel-markdown <-> GFM tables + report
```

Main public surface: `MarkdownExportOptions, MarkdownImportOptions, MarkdownWriter, MarkdownWorkbookWriter`.

## Installation and usage

```toml
[dependencies]
easyexcel-markdown = "0.1.1"
```

```rust
use easyexcel_markdown::{MarkdownExportOptions, MarkdownProfile, MarkdownWriter};
```

## Compatibility and limits

Markdown is a semantic projection, not a lossless Excel round-trip format. Application code should use `easyexcel::markdown`.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-markdown)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
