//! CSV sheet read with the same typed listener lifecycle as XLSX.

use std::fs::File;
use std::path::Path;

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::core::{CellValue, ExcelError, ExcelRow, ReadListener, Result};
use crate::reader::read_helpers::{analysis_context, format_error, reject_extra_read};
use crate::reader::read_options::ReadOptions;
use crate::reader::row_consumer::ReadFlow;
use crate::reader::row_processing::process_row;
use crate::reader::sheet_selector::SheetSelector;
use std::sync::Arc;
use std::collections::HashMap;

/// Reads a CSV file through the same typed listener lifecycle as XLSX.
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
    let encoding = csv_encoding(&options.charset)?;
    let input = DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding))
        .strip_bom(true)
        .build(File::open(path)?);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(input);
    read_csv_records::<T, L>(&mut reader.records(), 0, &sheet_name, options, listener)
}

fn csv_encoding(charset: &crate::core::CsvCharset) -> Result<&'static Encoding> {
    Encoding::for_label(charset.name().as_bytes()).ok_or_else(|| {
        ExcelError::Unsupported(format!("unsupported CSV charset: {}", charset.name()))
    })
}

pub(crate) fn read_csv_records<T, L>(
    records: &mut dyn Iterator<Item = csv::Result<csv::StringRecord>>,
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
            .map_err(format_error)?
            .iter()
            .map(|value| CellValue::String(value.to_owned()))
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

pub(crate) fn csv_row_index(row_index: usize) -> Result<u32> {
    u32::try_from(row_index).map_err(|_| ExcelError::Format("CSV row index exceeds u32".to_owned()))
}

pub(crate) fn csv_sheet_name(selector: &SheetSelector) -> Result<String> {
    match selector {
        SheetSelector::First | SheetSelector::Index(0) | SheetSelector::All => {
            Ok("Sheet1".to_owned())
        }
        SheetSelector::Name(name) => Ok(name.clone()),
        SheetSelector::Index(index) => Err(ExcelError::SheetNotFound(index.to_string())),
    }
}
