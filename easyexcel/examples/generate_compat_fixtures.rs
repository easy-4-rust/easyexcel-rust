//! Generates a representative fixture set for external-application
//! compatibility checks (compatibility.md verification evidence 4: Excel and
//! `LibreOffice` open every generated fixture without repair warnings).
//!
//! Fixtures cover the main write surfaces:
//!
//! - `01-simple.xlsx` — simple typed XLSX write (strings, dates, numbers)
//! - `02-template-fill.xlsx` — OOXML template `{placeholder}` fill
//! - `03-styled.xlsx` — header/content styles and column widths
//! - `04-merged.xlsx` — merged cell ranges
//! - `protected/05-encrypted.xlsx` — ECMA-376 Agile password-encrypted XLSX
//!   (kept in a `protected/` subdirectory because `LibreOffice` headless cannot
//!   open a password-protected file non-interactively)
//! - `06-image.xlsx` — embedded JPEG image
//! - `07-legacy.xls` — Minimal BIFF8 write
//! - `08-data.csv` — CSV write
//! - `09-freeze.xls` — BIFF8 with frozen header row (`freeze_head` → PANE)
//!
//! Run from the repository root:
//!
//! ```shell
//! cargo run --release -p easyexcel --example generate_compat_fixtures -- target/compat-fixtures
//! ```

use std::path::PathBuf;

use chrono::NaiveDate;
use easyexcel::{
    CellStyle, CellValue, ConvertContext, ConverterRegistry, EasyExcel, ExcelColumn, ExcelRow,
    ExcelWriteMetadata, FromExcelCell, IntoExcelCell, MergeRange, ReadConverterContext, Result,
    RowData, TemplateData, WriteCellData,
};

#[derive(Debug, Clone, ExcelRow)]
struct DemoRow {
    #[excel(name = "ID", index = 0)]
    id: u32,
    #[excel(name = "Name", index = 1)]
    name: String,
    #[excel(name = "Date", index = 2)]
    date: NaiveDate,
    #[excel(name = "Score", index = 3)]
    score: f64,
}

fn demo_rows(count: u32) -> Vec<DemoRow> {
    (0..count)
        .map(|i| DemoRow {
            id: i,
            name: format!("row-{i}"),
            date: NaiveDate::from_ymd_opt(2024, 1, (i % 28) + 1).unwrap(),
            score: f64::from(i) * 0.5,
        })
        .collect()
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn missing_fixture(name: &str) -> easyexcel::ExcelError {
    easyexcel::ExcelError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("required fixture missing: {name}"),
    ))
}

/// 内嵌图片写入行（局部使用，提到模块级以避免在语句后声明条目）。
#[derive(Debug, Clone, ExcelRow)]
struct ImageRow {
    #[excel(name = "Image")]
    image: WriteCellData,
}

fn main() -> easyexcel::Result<()> {
    let out = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("target/compat-fixtures"), PathBuf::from);
    std::fs::create_dir_all(&out)?;

    // 1. Simple typed XLSX write.
    let simple = out.join("01-simple.xlsx");
    EasyExcel::write::<DemoRow>(&simple)
        .sheet("Data")
        .do_write(demo_rows(50))?;
    println!("{}", simple.display());

    // 2. Template fill (Java `simple.xlsx` placeholder template). The
    //    fixtures live in the easyexcel-test crate since the test-suite
    //    migration; CARGO_MANIFEST_DIR here is the easyexcel crate.
    let fixtures_root = manifest_dir()
        .parent()
        .expect("easyexcel crate has a workspace parent")
        .join("easyexcel-test/tests/fixtures");
    let template = fixtures_root.join("demo/fill/simple.xlsx");
    if !template.exists() {
        return Err(missing_fixture(&template.display().to_string()));
    }
    let filled = out.join("02-template-fill.xlsx");
    EasyExcel::fill_template(
        &template,
        &filled,
        &TemplateData::new().with("name", "张三").with("number", 5.2),
    )?;
    println!("{}", filled.display());

    // 3. Header/content styles and column widths.
    let styled = out.join("03-styled.xlsx");
    EasyExcel::write::<DemoRow>(&styled)
        .head_style(CellStyle::new().bold(true).background_color(0x00DD_DDDD))
        .content_style(CellStyle::new().background_color(0x00F5_F5F5))
        .column_width(0, 12)
        .column_width(1, 30)
        .do_write(demo_rows(20))?;
    println!("{}", styled.display());

    // 4. Merged cell ranges (header span plus a data-row span).
    let merged = out.join("04-merged.xlsx");
    EasyExcel::write::<DemoRow>(&merged)
        .merge_cells(MergeRange::new(0, 0, 0, 3))
        .merge_cells(MergeRange::new(2, 2, 0, 1))
        .do_write(demo_rows(10))?;
    println!("{}", merged.display());

    // 5. Password-encrypted XLSX, isolated under `protected/` so the open
    //    verifier skips it (headless conversion cannot supply a password).
    let protected = out.join("protected");
    std::fs::create_dir_all(&protected)?;
    let encrypted = protected.join("05-encrypted.xlsx");
    EasyExcel::write::<DemoRow>(&encrypted)
        .password("verify-password")
        .do_write(demo_rows(5))?;
    println!("{}", encrypted.display());

    // 6. Embedded image.
    let img = fixtures_root.join("converter/img.jpg");
    if !img.exists() {
        return Err(missing_fixture(&img.display().to_string()));
    }
    let bytes = std::fs::read(&img)?;
    let image = out.join("06-image.xlsx");
    EasyExcel::write::<ImageRow>(&image).do_write(vec![ImageRow {
        image: WriteCellData::from_image(bytes),
    }])?;
    println!("{}", image.display());

    // 7. Legacy XLS (Minimal BIFF8).
    let xls = out.join("07-legacy.xls");
    EasyExcel::write::<DemoRow>(&xls).do_write(demo_rows(10))?;
    println!("{}", xls.display());

    // 9. BIFF8 with frozen header row (freeze_head → PANE + WINDOW2 fFrozen).
    let frozen_xls = out.join("09-freeze.xls");
    EasyExcel::write::<DemoRow>(&frozen_xls)
        .freeze_head(true)
        .do_write(demo_rows(10))?;
    println!("{}", frozen_xls.display());

    // 8. CSV write.
    let csv = out.join("08-data.csv");
    EasyExcel::write::<DemoRow>(&csv).do_write(demo_rows(10))?;
    println!("{}", csv.display());

    Ok(())
}
