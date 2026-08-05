//! XLSX sheet enumeration and typed row dispatch.

use std::path::Path;

use crate::core::{ExcelRow, ReadListener, Result};
use crate::read::read_options::ReadOptions;
use crate::read::row_consumer::{ReadFlow, RowConsumer, TypedRowConsumer};
use crate::read::row_processing::{read_sheet, select_sheet_names};
use crate::read::xlsx_rows::XlsxRowMetadata;
use crate::read::xlsx_source::{XlsxSource, open_xlsx_source};
use std::collections::HashSet;

/// Reads selected XLSX sheets and dispatches typed row events.
///
/// # Errors
///
/// Returns an I/O, workbook-format, sheet-selection, conversion, or listener error.
pub fn read_xlsx<T, L>(path: &Path, options: &ReadOptions, listener: &mut L) -> Result<()>
where
    T: ExcelRow,
    L: ReadListener<T>,
{
    let source = open_xlsx_source(path, options)?;
    read_xlsx_source::<T, L>(&source, options, listener)
}

/// Discovers worksheet names in workbook order.
///
/// 对应 Java：`XlsxSaxAnalyser` constructor sheet enumeration via `XSSFReader`.
///
/// # Errors
///
/// Returns an I/O or workbook-format error.
pub fn list_xlsx_sheets(path: &Path, options: &ReadOptions) -> Result<Vec<(usize, String)>> {
    let source = open_xlsx_source(path, options)?;
    let reader = source.reader()?;
    let metadata = XlsxRowMetadata::new_with_cache(reader, options)?;
    Ok(metadata.sheet_names().into_iter().enumerate().collect())
}

pub(crate) fn read_xlsx_source<T, L>(
    source: &XlsxSource,
    options: &ReadOptions,
    listener: &mut L,
) -> Result<()>
where
    T: ExcelRow,
    L: ReadListener<T>,
{
    let mut consumer = TypedRowConsumer::<T> { listener };
    read_xlsx_source_with_consumer(source, options, &mut consumer)
}

fn read_xlsx_source_with_consumer(
    source: &XlsxSource,
    options: &ReadOptions,
    consumer: &mut dyn RowConsumer,
) -> Result<()> {
    let mut row_metadata = XlsxRowMetadata::new_with_cache(source.reader()?, options)?;
    let names = select_sheet_names(
        row_metadata.sheet_names(),
        &options.sheet,
        options.auto_trim,
    )?;
    for (sheet_no, sheet_name) in names {
        let (last_explicit_row, extras) = xlsx_sheet_metadata(
            &mut row_metadata,
            &sheet_name,
            options.ignore_empty_row,
            &options.extra_read,
        )?;
        let scientific_enabled = options.scientific_format.is_enabled();
        let mut cell_reader = row_metadata.display_cells(
            &sheet_name,
            options.use_1904_windowing,
            scientific_enabled,
            options.locale.formatter(),
        )?;
        if read_sheet(
            &mut cell_reader,
            sheet_no,
            &sheet_name,
            last_explicit_row,
            &extras,
            options,
            consumer,
        )? == ReadFlow::Stop
        {
            break;
        }
    }
    Ok(())
}

fn xlsx_sheet_metadata(
    metadata: &mut XlsxRowMetadata,
    sheet_name: &str,
    ignore_empty_row: bool,
    enabled_extras: &HashSet<crate::core::CellExtraType>,
) -> Result<(Option<u32>, Vec<crate::core::CellExtra>)> {
    let last_explicit_row = if ignore_empty_row {
        None
    } else {
        metadata.last_explicit_row(sheet_name)?
    };
    let extras = metadata.extras(sheet_name, enabled_extras)?;
    Ok((last_explicit_row, extras))
}
