//! Streaming XLSX worksheet reader: emit one row at a time without ever
//! materializing the whole workbook in memory.
//!
//! The shared-string table and the (small) style table are still loaded up
//! front — they are referenced by index from anywhere in the sheet, so there is
//! no way around holding them — but the worksheet body is parsed as a pull
//! stream straight off the zip entry, so a multi-million-row sheet costs only
//! one row of memory at a time. Formula cells yield their cached value; there is
//! no recalculation on the streaming path.

use std::io::{BufReader, Read, Seek};

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::core::error::{Error, Result};

use super::reader::{build_cell, normalize_part_path, parse_cell_ref, parse_rels, parse_workbook};
use super::shared_strings::parse_shared_strings;
use super::styles::parse_styles;
use super::xmlutil::{attr, local_name, local_name_end};

pub use crate::core::stream::{RowSink, StreamCell, StreamInfo};

/// Read a single zip entry fully into memory (used for the small index parts:
/// workbook.xml, rels, sharedStrings, styles).
fn read_entry<R: Read + Seek>(archive: &mut zip::ZipArchive<R>, name: &str) -> Option<Vec<u8>> {
    let mut f = archive.by_name(name).ok()?;
    let mut data = Vec::with_capacity(f.size() as usize);
    f.read_to_end(&mut data).ok()?;
    Some(data)
}

/// Stream the rows of one sheet from a seekable XLSX reader into `sink`.
///
/// `sheet` selects by name (case-insensitive); `None` uses the first sheet.
pub fn stream<R: Read + Seek, S: RowSink>(
    reader: R,
    sheet: Option<&str>,
    sink: &mut S,
) -> Result<()> {
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| Error::Zip(e.to_string()))?;

    let wb_xml = read_entry(&mut archive, "xl/workbook.xml")
        .ok_or_else(|| Error::Xlsx("missing xl/workbook.xml".into()))?;
    let info = parse_workbook(&wb_xml)?;

    let rels = read_entry(&mut archive, "xl/_rels/workbook.xml.rels")
        .map(|b| parse_rels(&b))
        .transpose()?
        .unwrap_or_default();

    let shared = read_entry(&mut archive, "xl/sharedStrings.xml")
        .map(|b| parse_shared_strings(&b))
        .transpose()?
        .unwrap_or_default();

    // xf index -> number-format code (empty = General).
    let number_formats: Vec<String> = read_entry(&mut archive, "xl/styles.xml")
        .map(|b| parse_styles(&b))
        .transpose()?
        .unwrap_or_default()
        .into_iter()
        .map(|st| st.number_format)
        .collect();

    // Pick the requested sheet (or the first).
    let sref = match sheet {
        Some(name) => info
            .sheets
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| Error::Other(format!("sheet not found: {name}")))?,
        None => info
            .sheets
            .first()
            .ok_or_else(|| Error::Xlsx("workbook has no sheets".into()))?,
    };

    let path = rels
        .get(&sref.rid)
        .map(|t| normalize_part_path(t))
        .ok_or_else(|| Error::Xlsx(format!("cannot resolve worksheet for sheet {}", sref.name)))?;

    sink.begin(&StreamInfo {
        sheet_name: sref.name.clone(),
        date_system: info.date_system,
    })?;

    // Stream the worksheet body straight off the zip entry.
    let entry = archive
        .by_name(&path)
        .map_err(|_| Error::Xlsx(format!("missing worksheet part {path}")))?;
    let mut xml = Reader::from_reader(BufReader::new(entry));
    xml.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut cur_row: u32 = 0;
    let mut row_cells: Vec<StreamCell> = Vec::new();

    // Per-cell scratch.
    let mut cell_rc: Option<(u32, u32)> = None;
    let mut cell_type = String::new();
    let mut cell_xf: Option<usize> = None;
    let mut in_v = false;
    let mut in_f = false;
    let mut in_is_t = false;
    let mut v_text = String::new();
    let mut f_text = String::new();
    let mut is_text = String::new();
    let mut f_is_shared_member = false;

    loop {
        match xml.read_event_into(&mut buf).map_err(Error::from)? {
            Event::Eof => break,
            Event::Start(e) => match local_name(&e).as_str() {
                "row" => {
                    cur_row = attr(&e, "r")
                        .and_then(|s| s.parse::<u32>().ok())
                        .map(|r| r.saturating_sub(1))
                        .unwrap_or(cur_row);
                    row_cells.clear();
                }
                "c" => {
                    cell_rc = parse_cell_ref(&e, cur_row);
                    cell_type = attr(&e, "t").unwrap_or_default();
                    cell_xf = attr(&e, "s").and_then(|s| s.parse::<usize>().ok());
                    v_text.clear();
                    f_text.clear();
                    is_text.clear();
                    f_is_shared_member = false;
                }
                "v" => in_v = true,
                "f" => in_f = true,
                "t" => in_is_t = true,
                _ => {}
            },
            Event::Empty(e) if local_name(&e) == "f" => {
                f_is_shared_member = true;
            }
            Event::Text(t) => {
                if in_v {
                    v_text.push_str(&t.unescape().unwrap_or_default());
                } else if in_f {
                    f_text.push_str(&t.unescape().unwrap_or_default());
                } else if in_is_t {
                    is_text.push_str(&t.unescape().unwrap_or_default());
                }
            }
            Event::End(e) => match local_name_end(&e).as_str() {
                "v" => in_v = false,
                "f" => in_f = false,
                "t" => in_is_t = false,
                "c" => {
                    if let Some((_, col)) = cell_rc.take() {
                        let has_formula = !f_text.is_empty() || f_is_shared_member;
                        let cell = build_cell(
                            &cell_type,
                            &v_text,
                            &f_text,
                            &is_text,
                            has_formula,
                            &shared,
                        );
                        let value = cell.value();
                        if !matches!(value, crate::core::value::CellValue::Empty) {
                            let number_format = cell_xf
                                .and_then(|xf| number_formats.get(xf))
                                .cloned()
                                .unwrap_or_default();
                            row_cells.push(StreamCell {
                                col,
                                value,
                                number_format,
                            });
                        }
                    }
                }
                "row" if !row_cells.is_empty() => {
                    sink.row(cur_row, &row_cells)?;
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    sink.end()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{Cell, Workbook};
    use crate::core::value::CellValue;
    use std::io::Cursor;

    #[derive(Default)]
    struct Collector {
        rows: Vec<(u32, Vec<(u32, CellValue)>)>,
        sheet: String,
    }
    impl RowSink for Collector {
        fn begin(&mut self, info: &StreamInfo) -> Result<()> {
            self.sheet = info.sheet_name.clone();
            Ok(())
        }
        fn row(&mut self, row: u32, cells: &[StreamCell]) -> Result<()> {
            self.rows.push((
                row,
                cells.iter().map(|c| (c.col, c.value.clone())).collect(),
            ));
            Ok(())
        }
    }

    #[test]
    fn streams_rows_matching_model() {
        let mut wb = Workbook::new();
        {
            let s = wb.sheet_mut(0).unwrap();
            s.set_a1("A1", Cell::Text("name".into()));
            s.set_a1("B1", Cell::Text("amt".into()));
            s.set_a1("A2", Cell::Text("foo".into()));
            s.set_a1("B2", Cell::Number(12.5));
            // A sparse row: only column B populated.
            s.set_a1("B3", Cell::Number(99.0));
        }
        let mut bytes = Vec::new();
        super::super::write(&wb, Cursor::new(&mut bytes)).unwrap();

        let mut c = Collector::default();
        stream(Cursor::new(bytes), None, &mut c).unwrap();

        assert_eq!(c.sheet, "Sheet1");
        assert_eq!(c.rows.len(), 3);
        assert_eq!(c.rows[0].0, 0);
        assert_eq!(c.rows[0].1[0], (0, CellValue::Text("name".into())));
        assert_eq!(c.rows[1].1[1], (1, CellValue::Number(12.5)));
        // Sparse row keeps the real column index (B = 1), not 0.
        assert_eq!(c.rows[2].0, 2);
        assert_eq!(c.rows[2].1, vec![(1, CellValue::Number(99.0))]);
    }

    #[test]
    fn carries_number_format_for_display() {
        use crate::core::styles::CellStyle;
        let mut wb = Workbook::new();
        let idx = wb.styles.intern(CellStyle {
            number_format: "0.00".into(),
            ..Default::default()
        });
        {
            let s = wb.sheet_mut(0).unwrap();
            s.set_a1("A1", Cell::Number(3.5));
            s.set_style(0, 0, idx);
        }
        let mut bytes = Vec::new();
        super::super::write(&wb, Cursor::new(&mut bytes)).unwrap();

        struct Fmt(String);
        impl RowSink for Fmt {
            fn row(&mut self, _r: u32, cells: &[StreamCell]) -> Result<()> {
                self.0 = cells[0].display(crate::core::dates::DateSystem::Date1900);
                Ok(())
            }
        }
        let mut f = Fmt(String::new());
        stream(Cursor::new(bytes), None, &mut f).unwrap();
        assert_eq!(f.0, "3.50");
    }
}
