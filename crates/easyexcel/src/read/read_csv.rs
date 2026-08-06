//! CSV sheet read with the same typed listener lifecycle as XLSX.

use std::path::Path;

use crate::core::{CellValue, ExcelError, ExcelRow, ReadListener, Result};
use crate::read::read_helpers::{analysis_context, reject_extra_read};
use crate::read::read_options::ReadOptions;
use crate::read::row_consumer::ReadFlow;
use crate::read::row_processing::process_row;
use crate::read::sheet_selector::SheetSelector;
use std::collections::HashMap;
use std::sync::Arc;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Reads a CSV file through the same typed listener lifecycle as XLSX.
///
/// CSV exposes one logical sheet. Indexes other than zero return `SheetNotFound`.
///
/// # Errors
///
/// Returns an I/O, CSV-format, sheet-selection, conversion, or listener error.
pub fn read_csv<T, L>(path: &Path, options: &ReadOptions, listener: &mut L) -> Result<()>
where
    T: ExcelRow,
    L: ReadListener<T>,
{
    reject_extra_read(options, "CSV")?;
    let sheet_name = csv_sheet_name(&options.sheet)?;
    let mut reader = easyexcel_csv::CsvRecordReader::from_path(path, &options.charset)
        .map_err(ExcelError::from)?;
    read_csv_records::<T, L>(&mut reader.records(), 0, &sheet_name, options, listener)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn read_csv_records<T, L>(
    records: &mut dyn Iterator<Item = easyexcel_io::Result<Vec<String>>>,
    start_row: usize,
    sheet_name: &str,
    options: &ReadOptions,
    listener: &mut L,
) -> Result<()>
where
    T: ExcelRow,
    L: ReadListener<T>,
{
    let mut headers = Arc::new(HashMap::new());
    let mut final_row = 0_u32;
    for (offset, record) in records.enumerate() {
        let row_index = start_row.saturating_add(offset);
        let row_index = csv_row_index(row_index)?;
        final_row = row_index;
        let cells = record
            .map_err(ExcelError::from)?
            .into_iter()
            .map(CellValue::String)
            .collect();
        if process_row::<T>(
            0,
            sheet_name,
            row_index,
            cells,
            options,
            &mut headers,
            listener,
        )? == ReadFlow::Stop
        {
            return Ok(());
        }
    }
    listener.do_after_all_analysed(&analysis_context(sheet_name, 0, final_row, options))
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn csv_row_index(row_index: usize) -> Result<u32> {
    easyexcel_csv::checked_row_index(row_index).map_err(ExcelError::from)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn csv_sheet_name(selector: &SheetSelector) -> Result<String> {
    match selector {
        SheetSelector::First | SheetSelector::Index(0) | SheetSelector::All => {
            Ok("Sheet1".to_owned())
        }
        SheetSelector::Name(name) => Ok(name.clone()),
        SheetSelector::Index(index) => Err(ExcelError::SheetNotFound(index.to_string())),
    }
}
