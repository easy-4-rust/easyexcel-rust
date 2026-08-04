//! Memory-bounded streaming row reader for large spreadsheets.
//!
//! The normal [`open_path`](crate::core::open_path) path loads an entire
//! workbook into RAM. For very large `.xlsx`/`.csv` files that is wasteful (or
//! impossible). This module streams one row at a time into a [`RowSink`] without
//! building the workbook, so a linear scan (export, dump, head) costs only a
//! single row of memory.
//!
//! Caveats, by design:
//! - Formula cells yield their **cached** value; there is no recalculation.
//! - The shared-string and style tables are still loaded up front (they are
//!   index-referenced from anywhere in the sheet).
//! - `.xls` (BIFF) is capped at 65 536 rows by the format itself, so it has no
//!   streaming path; callers should fall back to the in-memory reader.

use std::path::Path;

use crate::core::dates::DateSystem;
use crate::core::error::{Error, Result};
use crate::core::value::CellValue;
use crate::core::{Format, numfmt, value};

/// One non-empty cell emitted by the streaming reader.
pub struct StreamCell {
    /// 0-based column index.
    pub col: u32,
    /// The cell's scalar value (formula cells carry their cached value).
    pub value: CellValue,
    /// The cell's number-format code (empty means General), for display.
    pub number_format: String,
}

impl StreamCell {
    /// Render this cell the way `Workbook::display_cell` would: apply the number
    /// format to numbers (dates included), pass everything else through.
    pub fn display(&self, date_system: DateSystem) -> String {
        match &self.value {
            CellValue::Number(n) => {
                if !self.number_format.is_empty()
                    && !self.number_format.eq_ignore_ascii_case("general")
                {
                    numfmt::format_value(*n, &self.number_format, date_system)
                } else {
                    value::format_number_general(*n)
                }
            }
            other => other.to_display_string(),
        }
    }
}

/// Metadata handed to a sink before any rows arrive.
pub struct StreamInfo {
    pub sheet_name: String,
    pub date_system: DateSystem,
}

/// A consumer of streamed rows. `row` is called once per non-empty row, in
/// ascending row order; cells within a row are in ascending column order.
pub trait RowSink {
    /// Called once before the first row.
    fn begin(&mut self, _info: &StreamInfo) -> Result<()> {
        Ok(())
    }
    /// Called for each non-empty row (`row` is the 0-based row index).
    fn row(&mut self, row: u32, cells: &[StreamCell]) -> Result<()>;
    /// Called once after the last row.
    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Whether a path can be streamed (`.xlsx`/`.xlsm` or `.csv`-family).
pub fn is_streamable(path: &Path) -> bool {
    matches!(
        Format::from_path(path),
        Some(Format::Xlsx) | Some(Format::Csv)
    )
}

/// Stream one sheet of a spreadsheet file into `sink`, never loading the whole
/// workbook. `sheet` selects by name (case-insensitive); `None` = first sheet.
///
/// Returns [`Error::Other`] for formats without a streaming path (e.g. `.xls`).
pub fn stream_path<S: RowSink>(path: &Path, sheet: Option<&str>, sink: &mut S) -> Result<()> {
    match Format::from_path(path) {
        Some(Format::Xlsx) => {
            let file = std::fs::File::open(path)?;
            crate::core::xlsx::stream(file, sheet, sink)
        }
        Some(Format::Csv) => {
            let file = std::fs::File::open(path)?;
            stream_csv(file, sink)
        }
        _ => Err(Error::Other(format!(
            "{} cannot be streamed; use the in-memory reader",
            path.display()
        ))),
    }
}

/// Stream a delimited text file row by row.
fn stream_csv<R: std::io::Read, S: RowSink>(mut reader: R, sink: &mut S) -> Result<()> {
    // The `csv` crate streams records; we only buffer one record at a time.
    // Delimiter detection needs a peek, so read a small head sample first.
    let mut head = vec![0u8; 64 * 1024];
    let n = reader.read(&mut head)?;
    head.truncate(n);
    let sample = crate::core::csv::decode_bytes(&head);
    let delimiter = crate::core::csv::detect_delimiter(&sample);

    // Re-chain the consumed head with the rest of the stream.
    let full = std::io::Read::chain(std::io::Cursor::new(head), reader);

    sink.begin(&StreamInfo {
        sheet_name: "Sheet1".to_string(),
        date_system: DateSystem::Date1900,
    })?;

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(full);

    let mut record = csv::StringRecord::new();
    let mut row_idx: u32 = 0;
    let mut cells: Vec<StreamCell> = Vec::new();
    while rdr.read_record(&mut record).map_err(Error::from)? {
        cells.clear();
        for (col, field) in record.iter().enumerate() {
            let cell = crate::core::csv::infer_cell(field);
            let value = cell.value();
            if !matches!(value, CellValue::Empty) {
                cells.push(StreamCell {
                    col: col as u32,
                    value,
                    number_format: String::new(),
                });
            }
        }
        if !cells.is_empty() {
            sink.row(row_idx, &cells)?;
        }
        row_idx += 1;
    }

    sink.end()?;
    Ok(())
}
