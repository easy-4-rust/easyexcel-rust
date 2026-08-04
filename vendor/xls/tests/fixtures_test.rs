//! Integration tests over the handcrafted fixtures in `tests/fixtures/`.

use std::path::PathBuf;

use xls::core::value::CellValue;
use xls::core::{CellError, open_path};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(rel)
}

#[test]
fn csv_basic_quoted_comma() {
    let wb = open_path(&fixture("csv/basic.csv")).unwrap();
    let s = &wb.sheets[0];
    assert_eq!(s.value(0, 0), CellValue::Text("name".into()));
    assert_eq!(s.value(1, 0), CellValue::Text("Alice".into()));
    assert_eq!(s.value(1, 1), CellValue::Number(30.0));
    // quoted field containing a comma stays one field
    assert_eq!(s.value(2, 2), CellValue::Text("Portland, OR".into()));
}

#[test]
fn csv_bom_is_stripped() {
    let wb = open_path(&fixture("csv/bom_utf8.csv")).unwrap();
    // header "id" must not carry the BOM bytes
    assert_eq!(wb.sheets[0].value(0, 0), CellValue::Text("id".into()));
    assert_eq!(wb.sheets[0].value(1, 1), CellValue::Number(100.0));
}

#[test]
fn csv_semicolon_autodetect() {
    let wb = open_path(&fixture("csv/semicolon.csv")).unwrap();
    assert_eq!(wb.sheets[0].value(1, 1), CellValue::Number(2.0));
}

#[test]
fn csv_quoted_newlines_and_escapes() {
    let wb = open_path(&fixture("csv/quoted_newlines.csv")).unwrap();
    let s = &wb.sheets[0];
    assert_eq!(s.value(0, 1), CellValue::Text("multi\nline".into()));
    assert_eq!(
        s.value(1, 1),
        CellValue::Text("has \"quotes\" inside".into())
    );
}

#[test]
fn csv_crlf_and_cr() {
    let crlf = open_path(&fixture("csv/crlf.csv")).unwrap();
    assert_eq!(crlf.sheets[0].value(2, 2), CellValue::Number(6.0));
    let cr = open_path(&fixture("csv/cr_only.csv")).unwrap();
    assert_eq!(cr.sheets[0].value(1, 0), CellValue::Number(1.0));
}

#[test]
fn csv_leading_zero_stays_text() {
    let wb = open_path(&fixture("csv/edge_types.csv")).unwrap();
    // "007" must not be coerced to the number 7
    assert_eq!(wb.sheets[0].value(2, 0), CellValue::Text("007".into()));
    assert_eq!(
        wb.sheets[0].value(1, 0),
        CellValue::Text("Widget, Deluxe".into())
    );
}

#[test]
fn xlsx_known_values() {
    let wb = open_path(&fixture("xlsx/sample.xlsx")).unwrap();
    assert_eq!(wb.sheets.len(), 2);
    let s = &wb.sheets[0];
    assert_eq!(s.name, "Data");
    assert_eq!(s.value(0, 0), CellValue::Number(42.0));
    assert_eq!(s.value(0, 1), CellValue::Text("hello".into()));
    assert_eq!(s.value(1, 0), CellValue::Bool(true));
    assert_eq!(s.value(1, 1), CellValue::Error(CellError::Div0));
    // formula cached value
    assert_eq!(s.value(2, 2), CellValue::Number(30.0));
    // merged region preserved
    assert!(s.merged.iter().any(|m| m.to_a1() == "A5:C5"));
    // date format applied on display
    assert_eq!(wb.display_cell(0, 0, 2), "2023-03-15");
    // second sheet
    assert_eq!(wb.sheets[1].value(4, 0), CellValue::Number(5.0));
}

#[test]
fn xls_known_values() {
    let wb = open_path(&fixture("xls/sample.xls")).unwrap();
    assert_eq!(wb.sheets.len(), 2);
    let s = &wb.sheets[0];
    assert_eq!(s.name, "Data");
    assert_eq!(s.value(0, 0), CellValue::Number(42.0));
    assert_eq!(s.value(0, 1), CellValue::Text("hello".into()));
    assert_eq!(s.value(1, 0), CellValue::Bool(true));
    assert_eq!(s.value(1, 1), CellValue::Error(CellError::Div0));
    assert_eq!(s.value(2, 2), CellValue::Number(30.0));
    assert!(s.merged.iter().any(|m| m.to_a1() == "A5:C5"));
    assert_eq!(wb.sheets[1].value(4, 0), CellValue::Number(5.0));
}

// ── Tables & named ranges round-trip (public API, via real files) ──────────

use xls::core::addr::{CellAddress, CellRange};
use xls::core::model::{Cell, Table, Workbook};
use xls::core::stream::{RowSink, StreamCell, StreamInfo};
use xls::core::{Result, save_path};

fn tmp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("xls_it_{}_{}.xlsx", std::process::id(), name))
}

#[test]
fn table_roundtrips_through_a_file() {
    let mut wb = Workbook::new();
    {
        let s = wb.sheet_mut(0).unwrap();
        s.set_a1("A1", Cell::Text("Name".into()));
        s.set_a1("B1", Cell::Text("Amount".into()));
        s.set_a1("A2", Cell::Text("foo".into()));
        s.set_a1("B2", Cell::Number(10.0));
        s.set_a1("A3", Cell::Text("bar".into()));
        s.set_a1("B3", Cell::Number(20.0));
        s.tables.push(Table {
            name: "Sales".into(),
            display_name: "Sales".into(),
            range: CellRange::new(CellAddress::new(0, 0), CellAddress::new(2, 1)),
            columns: vec!["Name".into(), "Amount".into()],
            header_rows: 1,
            totals_rows: 0,
            id: 0,
            raw_xml: Vec::new(),
        });
    }
    let path = tmp_path("table");
    save_path(&wb, &path).unwrap();
    let wb2 = open_path(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(wb2.sheets[0].tables.len(), 1);
    let (sidx, t) = wb2.table_by_name("Sales").unwrap();
    assert_eq!(sidx, 0);
    assert_eq!(t.range.to_a1(), "A1:B3");
    assert_eq!(t.columns, vec!["Name", "Amount"]);
    // Structured reference resolves to the column's data body.
    let (_, col) = wb2.resolve_structured("Sales[Amount]").unwrap();
    assert_eq!(col.to_a1(), "B2:B3");
}

#[test]
fn streaming_matches_full_read() {
    let mut wb = Workbook::new();
    {
        let s = wb.sheet_mut(0).unwrap();
        for r in 0..50u32 {
            s.set(r, 0, Cell::Number(r as f64));
            s.set(r, 1, Cell::Text(format!("row{r}")));
        }
    }
    let path = tmp_path("stream");
    save_path(&wb, &path).unwrap();

    #[derive(Default)]
    struct Rows(Vec<(u32, Vec<(u32, CellValue)>)>);
    impl RowSink for Rows {
        fn begin(&mut self, _i: &StreamInfo) -> Result<()> {
            Ok(())
        }
        fn row(&mut self, row: u32, cells: &[StreamCell]) -> Result<()> {
            self.0.push((
                row,
                cells.iter().map(|c| (c.col, c.value.clone())).collect(),
            ));
            Ok(())
        }
    }
    let mut rows = Rows::default();
    xls::core::stream::stream_path(&path, None, &mut rows).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(rows.0.len(), 50);
    assert_eq!(rows.0[0].1[0], (0, CellValue::Number(0.0)));
    assert_eq!(rows.0[49].1[1], (1, CellValue::Text("row49".into())));
}
