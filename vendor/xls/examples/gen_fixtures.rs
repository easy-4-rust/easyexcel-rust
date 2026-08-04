//! Generate the committed binary fixtures (`tests/fixtures/{xlsx,xls}/sample.*`)
//! from a single known-value workbook, so the integration tests can read them
//! back and assert exact values. Run with `cargo run --example gen_fixtures`.

use std::path::Path;

use xls::core::model::Cell;
use xls::core::styles::CellStyle;
use xls::core::value::CellValue;
use xls::core::{CellError, Workbook};

fn build() -> Workbook {
    let mut wb = Workbook::new();
    // A date number format for cell C1.
    let date_style = {
        let s = CellStyle {
            number_format: "yyyy-mm-dd".into(),
            ..Default::default()
        };
        wb.styles.intern(s)
    };
    let sheet = wb.sheet_mut(0).unwrap();
    sheet.name = "Data".into();
    sheet.set_a1("A1", Cell::Number(42.0));
    sheet.set_a1("B1", Cell::Text("hello".into()));
    sheet.set_a1("C1", Cell::Number(45000.0)); // a date serial
    sheet.set_style(0, 2, date_style);
    sheet.set_a1("A2", Cell::Bool(true));
    sheet.set_a1("B2", Cell::Error(CellError::Div0));
    sheet.set_a1("A3", Cell::Number(10.0));
    sheet.set_a1("B3", Cell::Number(20.0));
    sheet.set_a1(
        "C3",
        Cell::Formula {
            expr: "=A3+B3".into(),
            cached: CellValue::Number(30.0),
        },
    );
    // a merged region
    sheet
        .merged
        .push(xls::core::CellRange::parse_a1("A5:C5").unwrap());

    // second sheet
    let s2 = wb.add_sheet("Numbers");
    let s2 = wb.sheet_mut(s2).unwrap();
    for i in 0..5u32 {
        s2.set(i, 0, Cell::Number((i + 1) as f64));
    }
    wb
}

fn main() -> anyhow::Result<()> {
    let wb = build();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    std::fs::create_dir_all(root.join("xlsx"))?;
    std::fs::create_dir_all(root.join("xls"))?;
    xls::core::xlsx::write_path(&wb, &root.join("xlsx/sample.xlsx"))?;
    xls::core::xls::write_path(&wb, &root.join("xls/sample.xls"))?;
    println!("wrote tests/fixtures/xlsx/sample.xlsx and tests/fixtures/xls/sample.xls");
    Ok(())
}
