# easyexcel-tabular

[简体中文](README.zh-CN.md)

Safe HTML and JSON table conversion plus generic tabular format dispatch.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Parses and renders static HTML tables and JSON tabular documents.
- Delegates Markdown handling to `easyexcel-markdown` instead of duplicating the codec.

## Architecture

```text
HTML / JSON / Markdown -> dispatcher -> TabularDocument
```

Main public surface: `TabularFormat, TabularDocument, parse_document, render_document, parse_html, parse_json`.

## Installation and usage

```toml
[dependencies]
easyexcel-tabular = "0.1.1"
```

```rust
use easyexcel_tabular::{TabularDocument, TabularFormat, parse_document};
```

## Compatibility and limits

HTML input is treated as static table markup: scripts, network loading and uncontrolled CSS are outside scope. Prefer `easyexcel::tabular`.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-tabular)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
