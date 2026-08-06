# easyexcel-derive

[简体中文](README.zh-CN.md)

Procedural macros for typed EasyExcel row mapping and annotation metadata.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Derives `ExcelRow` schemas and bidirectional row conversion.
- Maps the supported Java EasyExcel annotation semantics to `#[excel(...)]` attributes.

## Architecture

```text
Rust struct + attributes -> easyexcel-derive -> ExcelRow implementation
```

Main public surface: `#[derive(ExcelRow)] and #[excel(...)]`.

## Installation and usage

```toml
[dependencies]
easyexcel-derive = "0.1.1"
```

```rust
use easyexcel::ExcelRow;

#[derive(ExcelRow)]
struct OrderRow {
    #[excel(name = "Order ID", index = 0)]
    id: String,
}
```

## Compatibility and limits

Users should not depend on this proc-macro crate directly; `easyexcel` re-exports `ExcelRow`. Format rendering limits remain backend-specific.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Java annotation mapping

| Java annotation | Rust attribute |
|---|---|
| `ExcelIgnore` | `ignore` |
| `ExcelIgnoreUnannotated` | `ignore_unannotated` |
| `ExcelProperty` | `property`, `value/head`, `name`, `index`, `order`, `converter` |
| `DateTimeFormat` | `date_time_format`, `use_1904_windowing` |
| `NumberFormat` | `number_format`, `rounding_mode` |
| `ColumnWidth` | `column_width` |
| `ContentFontStyle` | `content_font_style(...)` |
| `ContentLoopMerge` | `content_loop_merge(...)` |
| `ContentRowHeight` | `content_row_height` |
| `ContentStyle` | `content_style(...)` |
| `HeadFontStyle` | `head_font_style(...)` |
| `HeadRowHeight` | `head_row_height` |
| `HeadStyle` | `head_style(...)` |
| `OnceAbsoluteMerge` | `once_absolute_merge(...)` |

`value = ["Level 1", "Level 2"]` models a multi-level `ExcelProperty.value()` header. With `ignore_unannotated`, formatting or style attributes alone do not opt a field into mapping. `default = expression`, `image`, `comment`, `hyperlink`, `formula`, `data_validation`, `conditional` and `filter` are documented Rust extensions, not falsely labeled Java annotation members.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-derive)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
