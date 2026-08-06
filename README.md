# easyexcel-rust

[![Rust](https://img.shields.io/badge/rust-1.88+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![CI](https://github.com/easy-4-rust/easyexcel-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/easy-4-rust/easyexcel-rust/actions/workflows/ci.yml)

**easyexcel-rust** is a native Rust port of Alibaba [EasyExcel](https://github.com/alibaba/easyexcel) 4.0.3.
It delivers the Java EasyExcel programming model in idiomatic Rust: builders,
typed row mapping, event listeners, converters, streaming reads,
constant-memory writes, template filling, and write handlers.

The workspace also exposes reusable format-neutral foundations (`easyexcel-model`,
`easyexcel-formula`, `easyexcel-io`, XLS/XLSX/CSV backends and tabular conversion).
The independent `xls-cli` product owns its library-first command application layer;
this facade no longer depends on the full `xls` fork.

> 📖 [中文 README](README_CN.md) | [Usage Guide](docs/GUIDE.md) | [API reference](docs/API.md) | [Architecture](docs/ARCHITECTURE.md) | [xls-cli integration](docs/xls-cli-integration-plan.md) | [Capability matrix](docs/xls-cli-capability-matrix.md)

---

## Quick Start

```toml
[dependencies]
easyexcel = "0.1"
```

### Read Excel

```rust
use easyexcel::{EasyExcel, ExcelRow, PageReadListener};

#[derive(Debug, ExcelRow)]
struct User {
    #[excel(name = "Name", index = 0)]
    name: String,
    #[excel(name = "Age", index = 1)]
    age: Option<u32>,
}

fn main() -> easyexcel::Result<()> {
    // Event-driven for large files
    let listener = PageReadListener::new(1000, |rows, _ctx| {
        println!("received {} rows", rows.len());
    });
    EasyExcel::read::<User, _>("users.xlsx", listener)
        .sheet("Users")
        .do_read()?;

    // Synchronous for small datasets
    let users: Vec<User> = EasyExcel::read_sync::<User>("users.xlsx")
        .sheet("Users")
        .do_read_sync()?;

    Ok(())
}
```

### Write Excel

```rust
use easyexcel::{EasyExcel, ExcelRow};

#[derive(Debug, ExcelRow)]
#[excel(column_width = 18)]
struct User {
    #[excel(name = "Name", column_width = 30)]
    name: String,
    #[excel(name = "Age")]
    age: u32,
    #[excel(name = "Birthday", format = "yyyy-MM-dd")]
    birthday: chrono::NaiveDate,
}

fn main() -> easyexcel::Result<()> {
    let users = vec![
        User { name: "Alice".into(), age: 28, birthday: chrono::NaiveDate::from_ymd_opt(1996, 5, 20).unwrap() },
        User { name: "Bob".into(), age: 32, birthday: chrono::NaiveDate::from_ymd_opt(1992, 3, 15).unwrap() },
    ];

    EasyExcel::write::<User>("users.xlsx")
        .sheet("Users")
        .do_write(users)?;

    Ok(())
}
```

### Template Fill

```rust
use easyexcel::{EasyExcel, TemplateData, FillWrapper, FillConfig};

// Scalar fill {key}
let data = TemplateData::new()
    .with("name", "Alice")
    .with("date", "2024-01-15");
EasyExcel::fill_template("template.xlsx", "output.xlsx", &data)?;

// List fill {.field}
let list = FillWrapper::new([
    TemplateData::new().with("name", "Alice").with("score", 95),
    TemplateData::new().with("name", "Bob").with("score", 88),
]);
EasyExcel::fill_template_list("template.xlsx", "output.xlsx", &list, FillConfig::default())?;
```

### Foundation Component Facade

Downstream projects only need the `easyexcel` dependency to access CSV, I/O,
and workbook model APIs through stable namespaces:

```rust
use easyexcel::csv::{CsvReadOptions, CsvWriteOptions};
use easyexcel::io::{Format, ResourceLimits};
use easyexcel::model::{Cell, Workbook};
```

These are direct re-exports of the foundation crate types, with no wrapper or
conversion overhead.

### Markdown Projection

Use `easyexcel::markdown`, not an internal engine crate, to convert XLS/XLSX/CSV and GFM tables:

```rust
use easyexcel::markdown::{
    MarkdownConversionMode, MarkdownFormulaPolicy, MarkdownMergePolicy,
};
use easyexcel::EasyExcel;

let report = EasyExcel::export_markdown("report.xlsx", "report.md")
    .all_sheets()
    .mode(MarkdownConversionMode::Auto)
    .formula_policy(MarkdownFormulaPolicy::CachedValue)
    .merge_policy(MarkdownMergePolicy::AnchorWithWarning)
    .do_export()?;

for warning in report.warnings {
    eprintln!("{:?}: {}", warning.code, warning.message);
}

EasyExcel::import_markdown("tables.md", "generated.xlsx")
    .conservative_types()
    .apply_header_style(true)
    .do_import()?;
```

The default `AgentStable` profile emits deterministic UTF-8 GFM tables. XLSX and CSV can use Event Mode; XLS, expression output, and merge policies requiring full workbook metadata use Workbook Mode. Markdown is a semantic projection with a structured loss report, not a lossless round trip.

## Annotation Mapping (Java → Rust)

| Java Annotation | Rust Attribute | Purpose |
|-----------------|---------------|---------|
| `@ExcelProperty` | `#[excel(value/head, name, index, order, converter)]` | Column mapping and multi-level heads |
| `@ExcelIgnore` | `#[excel(ignore)]` | Skip field |
| `@ExcelIgnoreUnannotated` | `#[excel(ignore_unannotated)]` | Skip unannotated |
| `@DateTimeFormat` | `#[excel(date_time_format = "...", use_1904_windowing = true)]` | Date format |
| `@NumberFormat` | `#[excel(number_format = "...", rounding_mode = "HALF_UP")]` | Numeric format |
| `@ColumnWidth` | `#[excel(column_width = N)]` | Column width |
| `@HeadRowHeight` | `#[excel(head_row_height = N)]` | Header row height |
| `@ContentRowHeight` | `#[excel(content_row_height = N)]` | Content row height |
| `@HeadStyle` | `#[excel(head_style(...))]` | Header style |
| `@ContentStyle` | `#[excel(content_style(...))]` | Content style |
| `@HeadFontStyle` | `#[excel(head_font_style(...))]` | Header font |
| `@ContentFontStyle` | `#[excel(content_font_style(...))]` | Content font |
| `@ContentLoopMerge` | `#[excel(content_loop_merge(...))]` | Loop merge |
| `@OnceAbsoluteMerge` | `#[excel(once_absolute_merge(...))]` | Absolute merge |

## Write Handlers

```rust
use easyexcel::{Result, WriteHandler, WriteSheetContext};

struct LoggingHandler;

impl WriteHandler for LoggingHandler {
    fn order(&self) -> i32 { 100 }
    fn after_sheet(&mut self, ctx: &WriteSheetContext) -> Result<()> {
        println!("Sheet '{}' written", ctx.sheet_name());
        Ok(())
    }
}

EasyExcel::write::<User>("output.xlsx")
    .register_write_handler(LoggingHandler)
    .sheet("Sheet1")
    .do_write(data)?;
```

## Crate Map

| Crate | Purpose | Java Mirror |
|-------|---------|-------------|
| `easyexcel` | User-facing facade | `EasyExcel` / `EasyExcelFactory` |
| `easyexcel-derive` | `#[derive(ExcelRow)]` proc-macro | Annotation processing |
| `easyexcel-model` | Format-neutral workbook and cell model | Core data model |
| `easyexcel-io` | Format detection, I/O contracts and resource limits | Read/write infrastructure |
| `easyexcel-csv` | CSV codec, charset and streaming writer | CSV backend |
| `easyexcel-xls` | BIFF8 parsing, encoding, encryption and formula tokens | XLS backend |
| `easyexcel-xlsx` | OOXML streaming, package handling and encryption | XLSX backend |
| `easyexcel-formula` | Formula AST, parser and evaluator | Formula engine |
| `easyexcel-markdown` | GFM parsing, streaming export, projection policy and loss reports | Markdown projection |
| `easyexcel-tabular` | Static HTML, JSON and generic text-format dispatch | Tabular interchange |
| `easyexcel-web` | Framework-neutral streaming import/export, limits, cancellation and error protocol | Web execution kernel |

Foundation crates are internal engine layers. Application code should depend only on `easyexcel`:

```rust
use easyexcel::csv::{CsvCharset, CsvReadOptions, CsvWriteOptions};
use easyexcel::io::{Format, ResourceLimits};
use easyexcel::model::{Cell, Workbook};
use easyexcel::markdown::{MarkdownConversionMode, MarkdownFormulaPolicy};
use easyexcel::xls;
use easyexcel::xlsx;
```

`easyexcel::{csv, io, markdown, model, formula, tabular, xls, xlsx}` directly re-export the public engine types without creating a second model. The facade continues to own `EasyExcel`, builders, listeners, converters, handlers, and `#[derive(ExcelRow)]`.

Markdown is a semantic projection with an explicit loss report, not a lossless
Excel round trip. XLS uses Workbook Mode; XLSX and CSV also support real Event
Mode. Rust users only need the facade:

```rust
let report = EasyExcel::export_markdown("report.xlsx", "report.md")
    .mode(MarkdownConversionMode::Auto)
    .formula_policy(MarkdownFormulaPolicy::CachedValue)
    .do_export()?;

EasyExcel::import_markdown("report.md", "report.xlsx")
    .conservative_types()
    .do_import()?;
```

Web services additionally depend on `easyexcel-web` and use `ExcelImport<T>`, `ExcelRows<T>`, `ExcelExport<T>`, `ExcelWebPolicy`, and an application-level `ExcelWebRuntime`. Framework crates provide only native extractor/responder adapters; shared buffering, limits, cancellation, cleanup, and error mapping stay in the web kernel.

Axum, Actix Web, Hyper, Poem, Rocket, Salvo, and Warp expose equivalent `ExcelRequest<T>` and `ExcelResponse<T>` semantics. Runnable integrations live under `examples/<framework>` and share the conformance suite in `tests/easyexcel-web-conformance`.

## Java Compatibility

easyexcel-rust is a 1:1 mirror of Java EasyExcel 4.0.3:

- **335 Java @Test methods** — all have Rust `#[test]` counterparts
- **88 Golden tests** — byte-level output matches Java
- **152 Parity tests** — end-to-end behavioral equivalence
- **0 FAILEDs** across entire workspace

See [Migration Audit](docs/migration/TEST_AUDIT_REPORT.md).

## License

Apache-2.0
