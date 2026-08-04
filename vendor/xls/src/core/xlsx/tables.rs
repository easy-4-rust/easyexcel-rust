//! Parse and emit `xl/tables/tableN.xml` (Excel table / ListObject parts).
//!
//! We model the structurally meaningful bits (name, range, columns, header /
//! totals rows) and keep the original XML verbatim for lossless round-trip of
//! everything else (style, autofilter, calculated columns).

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::core::addr::CellRange;
use crate::core::error::Result;
use crate::core::model::Table;

use super::xmlutil::{attr, local_name, xml_escape};

/// Parse one `<table>` part into a [`Table`], preserving the source XML.
pub fn parse_table(xml: &[u8]) -> Result<Option<Table>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut name = String::new();
    let mut display_name = String::new();
    let mut id: u32 = 0;
    let mut range: Option<CellRange> = None;
    let mut header_rows: u32 = 1;
    let mut totals_rows: u32 = 0;
    let mut columns: Vec<String> = Vec::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => match local_name(&e).as_str() {
                "table" => {
                    name = attr(&e, "name").unwrap_or_default();
                    display_name = attr(&e, "displayName").unwrap_or_else(|| name.clone());
                    id = attr(&e, "id").and_then(|s| s.parse().ok()).unwrap_or(0);
                    range = attr(&e, "ref").and_then(|r| CellRange::parse_a1(&r));
                    if let Some(h) = attr(&e, "headerRowCount").and_then(|s| s.parse().ok()) {
                        header_rows = h;
                    }
                    if let Some(t) = attr(&e, "totalsRowCount").and_then(|s| s.parse().ok()) {
                        totals_rows = t;
                    }
                }
                "tableColumn" => {
                    if let Some(n) = attr(&e, "name") {
                        columns.push(n);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    let Some(range) = range else {
        return Ok(None);
    };
    if name.is_empty() {
        return Ok(None);
    }

    Ok(Some(Table {
        name,
        display_name,
        range,
        columns,
        header_rows,
        totals_rows,
        id,
        raw_xml: xml.to_vec(),
    }))
}

/// Serialize a [`Table`] to table-part XML. Uses the preserved `raw_xml` when
/// present (lossless round-trip); otherwise synthesizes a minimal valid part.
pub fn build_table_xml(table: &Table, id: u32) -> Vec<u8> {
    if !table.raw_xml.is_empty() {
        return table.raw_xml.clone();
    }
    let ref_a1 = table.range.to_a1();
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    s.push_str(&format!(
        r#"<table xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" id="{}" name="{}" displayName="{}" ref="{}" totalsRowShown="{}">"#,
        id,
        xml_escape(&table.name),
        xml_escape(&table.display_name),
        ref_a1,
        if table.totals_rows > 0 { 1 } else { 0 },
    ));
    s.push_str(&format!(r#"<autoFilter ref="{ref_a1}"/>"#));
    s.push_str(&format!(
        r#"<tableColumns count="{}">"#,
        table.columns.len()
    ));
    for (i, col) in table.columns.iter().enumerate() {
        s.push_str(&format!(
            r#"<tableColumn id="{}" name="{}"/>"#,
            i + 1,
            xml_escape(col)
        ));
    }
    s.push_str("</tableColumns>");
    s.push_str(
        r#"<tableStyleInfo name="TableStyleMedium2" showFirstColumn="0" showLastColumn="0" showRowStripes="1" showColumnStripes="0"/>"#,
    );
    s.push_str("</table>");
    s.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_table() {
        let xml = br#"<?xml version="1.0"?>
            <table xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
                   id="1" name="Sales" displayName="Sales" ref="A1:C4" totalsRowShown="0">
                <autoFilter ref="A1:C4"/>
                <tableColumns count="3">
                    <tableColumn id="1" name="Name"/>
                    <tableColumn id="2" name="Amount"/>
                    <tableColumn id="3" name="When"/>
                </tableColumns>
                <tableStyleInfo name="TableStyleMedium2"/>
            </table>"#;
        let t = parse_table(xml).unwrap().unwrap();
        assert_eq!(t.name, "Sales");
        assert_eq!(t.range.to_a1(), "A1:C4");
        assert_eq!(t.columns, vec!["Name", "Amount", "When"]);
        assert_eq!(t.header_rows, 1);
        assert_eq!(t.totals_rows, 0);
        // Body excludes the header row.
        assert_eq!(t.data_range().unwrap().to_a1(), "A2:C4");
        assert_eq!(t.column_range("Amount").unwrap().to_a1(), "B2:B4");
    }

    #[test]
    fn round_trips_through_writer() {
        use crate::core::addr::{CellAddress, CellRange};
        use crate::core::model::{Cell, Table, Workbook};
        use std::io::Cursor;

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
                raw_xml: Vec::new(), // force synthesis
            });
        }

        let mut bytes = Vec::new();
        super::super::write(&wb, Cursor::new(&mut bytes)).unwrap();
        let wb2 = super::super::read(Cursor::new(bytes)).unwrap();

        assert_eq!(wb2.sheets[0].tables.len(), 1);
        let t = &wb2.sheets[0].tables[0];
        assert_eq!(t.name, "Sales");
        assert_eq!(t.range.to_a1(), "A1:B3"); // header row 1 + body rows 2-3
        assert_eq!(t.columns, vec!["Name", "Amount"]);
        // Workbook-level lookup works.
        let (sidx, found) = wb2.table_by_name("sales").unwrap();
        assert_eq!(sidx, 0);
        assert_eq!(found.column_range("Amount").unwrap().to_a1(), "B2:B3");
    }

    #[test]
    fn synthesize_when_no_raw() {
        let mut t = parse_table(
            br#"<table xmlns="x" id="2" name="T" displayName="T" ref="A1:B2"><tableColumns count="2"><tableColumn id="1" name="X"/><tableColumn id="2" name="Y"/></tableColumns></table>"#,
        )
        .unwrap()
        .unwrap();
        t.raw_xml.clear();
        let xml = build_table_xml(&t, 5);
        let reparsed = parse_table(&xml).unwrap().unwrap();
        assert_eq!(reparsed.name, "T");
        assert_eq!(reparsed.range.to_a1(), "A1:B2");
        assert_eq!(reparsed.columns, vec!["X", "Y"]);
    }
}
