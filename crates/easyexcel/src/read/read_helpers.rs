//! Shared read-side helpers: validation, trimming, header mapping, and listener wiring.

use crate::core::{AnalysisContext, CellValue, ErrorAction, ExcelError, ReadListener, Result};
use crate::read::read_options::ReadOptions;
use crate::read::row_consumer::ReadFlow;
use std::collections::HashMap;

pub(crate) fn validate_read_options(options: &ReadOptions) -> Result<()> {
    easyexcel_io::validate_row_range(options.start_row, options.end_row).map_err(ExcelError::from)
}

pub(crate) fn reject_extra_read(options: &ReadOptions, format: &str) -> Result<()> {
    validate_read_options(options)?;
    if options.extra_read.is_empty() {
        Ok(())
    } else {
        Err(ExcelError::Unsupported(format!(
            "{format} extra metadata is not supported"
        )))
    }
}

pub(crate) fn trim_string_cells(cells: &mut [CellValue]) {
    for cell in cells {
        if let CellValue::String(value) = cell {
            let trimmed = easyexcel_utils::string_utils::java_trim(value);
            if trimmed.len() != value.len() {
                *value = trimmed.to_owned();
            }
        }
    }
}

pub(crate) fn is_empty_read_cell(cell: &CellValue) -> bool {
    cell.is_empty() || matches!(cell, CellValue::String(value) if value.is_empty())
}

pub(crate) fn header_map(
    cells: &[CellValue],
    header_aliases: &HashMap<String, String>,
) -> HashMap<String, usize> {
    cells
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let name = value.as_text();
            (!name.is_empty()).then(|| {
                let alias = header_aliases.get(&name).cloned().unwrap_or(name);
                (alias, index)
            })
        })
        .collect()
}

pub(crate) fn analysis_context(
    sheet_name: &str,
    sheet_no: usize,
    row_index: u32,
    options: &ReadOptions,
) -> AnalysisContext {
    AnalysisContext::new(sheet_name, sheet_no, row_index)
        .with_custom_object(options.custom_object.clone())
}

pub(crate) fn listener_result<T>(
    result: Result<()>,
    listener: &mut dyn ReadListener<T>,
    context: &AnalysisContext,
) -> Result<ReadFlow> {
    match result {
        Ok(()) if listener.has_next(context) => Ok(ReadFlow::Continue),
        Ok(()) => Ok(ReadFlow::Stop),
        Err(error) => listener_error(error, listener, context),
    }
}

pub(crate) fn listener_error<T>(
    error: ExcelError,
    listener: &mut dyn ReadListener<T>,
    context: &AnalysisContext,
) -> Result<ReadFlow> {
    match listener.on_exception(&error, context) {
        ErrorAction::Continue | ErrorAction::SkipRow => Ok(ReadFlow::Continue),
        ErrorAction::Stop => Err(error),
    }
}

#[cfg(test)]
pub(crate) fn to_column_index(column: u32) -> Result<usize> {
    easyexcel_utils::int_utils::checked_u16(column)
        .map(usize::from)
        .ok_or_else(|| ExcelError::Format("column index exceeds spreadsheet limit".to_owned()))
}
