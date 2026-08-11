# easyexcel-derive

[简体中文](README.zh-CN.md)

> **文档说明**：easyexcel-derive 引擎层 crate 文档，面向贡献者与引擎实现者说明模块边界。
>
> **版本**：0.1.3
> **最后更新**：2026-08-11

Procedural macro implementing typed EasyExcel row schemas, conversion and Java annotation metadata.

> Release: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Overview

This crate is independently published to support the EasyExcel-Rust internal dependency graph. Its README documents the engine boundary for contributors and engine implementors; application code should depend on `easyexcel` and use the matching `easyexcel::...` facade path.

## At a glance

```text
Application -> easyexcel:: facade -> easyexcel-derive internal engine -> typed result
```

## Architecture

```mermaid
flowchart LR
    Struct["Rust struct"] --> Parser["syn attribute parser"]
    Parser --> Metadata["Annotation model"]
    Metadata --> Expand["quote code generation"]
    Expand --> Trait["ExcelRow implementation"]
    Trait --> Facade["easyexcel builders"]
```

Dependency direction remains from the facade or format engines toward foundations; this crate never depends back on an application.

## Capabilities and boundaries

### What this crate can do

- Generate static Excel column metadata and bidirectional row conversion from `#[derive(ExcelRow)]`.
- Map fourteen Java annotation families to `#[excel(...)]` attributes.
- Provide Rust extensions beyond Java: formula, image, comment, hyperlink, validation, conditional and filter metadata.
- Reject conflicting forced column indices at compile time.
- Support `ignore_unannotated` for strict mapping (only `ExcelProperty`-equivalent fields are included).
- Generate `schema()` method returning field metadata for runtime introspection.

### What this crate cannot do

- Render file-format output: metadata support and file-format rendering are separate concerns.
- Replace runtime reflection: `#[derive(ExcelRow)]` generates static schema, not dynamic field access.

## Format support matrix

This crate is a proc-macro, not a file format engine. Its output is consumed by all format engines.

| Dimension | Derive output | Consumed by |
|:---|:---|:---|
| Column mapping (`value`, `name`, `index`, `order`) | Static metadata | All format engines |
| Style metadata (`column_width`, `head_row_height`, etc.) | Static metadata | XLS, XLSX |
| Format metadata (`date_time_format`, `number_format`) | Static metadata | XLS, XLSX, CSV |
| Converter (`converter = MyConverter`) | Trait implementation | All format engines |
| Merge metadata (`content_loop_merge`, `once_absolute_merge`) | Static metadata | XLS, XLSX |
| Strict mapping (`ignore_unannotated`) | Schema filtering | All format engines |
| Default values (`default = expression`) | Rust extension | All format engines |
| Formula / image / comment / hyperlink | Rust extension metadata | XLSX (primary) |

## Annotation mapping

| Java annotation | Rust attribute |
|:---|:---|
| `ExcelIgnore` | `ignore` |
| `ExcelIgnoreUnannotated` | `ignore_unannotated` |
| `ExcelProperty` | `property`, `value/head`, `name`, `index`, `order`, `converter` |
| `DateTimeFormat` | `date_time_format`, `use_1904_windowing` |
| `NumberFormat` | `number_format`, `rounding_mode` |
| `ColumnWidth` | `column_width` |
| `ContentFontStyle` / `HeadFontStyle` | `content_font_style(...)` / `head_font_style(...)` |
| `ContentStyle` / `HeadStyle` | `content_style(...)` / `head_style(...)` |
| `ContentLoopMerge` | `content_loop_merge(...)` |
| `ContentRowHeight` / `HeadRowHeight` | `content_row_height` / `head_row_height` |
| `OnceAbsoluteMerge` | `once_absolute_merge(...)` |

Multi-level `ExcelProperty.value()` maps to `value = ["Level 1", "Level 2"]`. The `default = expression` attribute is an explicitly documented Rust extension.

## Public API

| API | Purpose |
|:---|:---|
| `#[derive(ExcelRow)]` | Generate row schema and conversion implementation. |
| `#[excel(name/index/order/...)]` | Column mapping metadata. |
| Style attributes | Header/content font, style, width, height and merge metadata. |
| Format attributes | Date/time, number format and rounding mode. |

The current `src/lib.rs` re-exports and their implementations are authoritative. This README does not present private implementation objects as stable API.

## Installation

```toml
[dependencies]
easyexcel = "0.1.3"
```

`easyexcel-derive` is a procedural-macro implementation detail. Applications should import `easyexcel::ExcelRow`; direct macro-crate dependencies are not part of the recommended public usage.

## Basic usage

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::ExcelRow;

#[derive(Debug, ExcelRow)]
#[excel(column_width = 18, head_row_height = 24)]
struct OrderRow {
    #[excel(value = ["Order", "ID"], index = 0)]
    id: String,

    #[excel(name = "Amount", number_format = "0.00")]
    amount: f64,
}
Ok(())
}
```

## Advanced usage

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::ExcelRow;

#[derive(ExcelRow)]
#[excel(ignore_unannotated)]
struct StrictRow {
    #[excel(property, name = "Included")]
    included: String,

    // Style-only metadata does not opt this field into strict mapping.
    #[excel(number_format = "0.00")]
    ignored: f64,

    #[excel(ignore, default = String::new())]
    internal: String,
}
Ok(())
}
```

## Errors and capability boundaries

- Users should consume the macro through `easyexcel::ExcelRow`, not add a direct runtime dependency on this proc-macro crate.
- Metadata support and file-format rendering are separate: backend-specific limitations remain authoritative.

Resource limits, format loss and unsupported behavior must surface through typed errors, `Option`, warnings or conversion reports; silent guessing and downgrade are forbidden.

## Relationship to other crates

```mermaid
flowchart LR
    User["Application"] --> Facade["easyexcel"]
    Facade --> This["easyexcel-derive"]
    This --> Foundation["Shared foundation crates"]
```

The diagram shows the public dependency direction, not that this crate depends on every foundation module. `Cargo.toml` is authoritative.

## Evidence map

| Claim | Source of truth |
|:---|:---|
| Package version, MSRV and dependencies | [`Cargo.toml`](Cargo.toml) |
| Public exports | [`src/lib.rs`](src/lib.rs) |
| Implementation behavior | `src/annotation/ and src/expand/` |
| Cross-format boundaries | [Workspace compatibility matrix](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## Related links

- [Repository](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-derive)
- [Compatibility matrix](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [Changelog](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [Chinese README](README.zh-CN.md)

---

**文档版本**：V1.0.0
**最后更新**：2026-08-11
**文档状态**：已评审
