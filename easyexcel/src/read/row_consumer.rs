//! Internal row-event consumer abstraction shared by the XLSX, XLS, and CSV engines.

use crate::core::{
    AnalysisContext, CellExtra, CellValue, ExcelRow, FormulaData, ReadListener, Result, RowData,
};
use crate::read::read_helpers::{
    analysis_context, header_map, is_empty_read_cell, listener_error, listener_result,
    trim_string_cells,
};
use crate::read::read_options::ReadOptions;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Controls whether the read loop continues or stops after a row event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadFlow {
    Continue,
    Stop,
}

/// Per-row metadata collected while materializing cells before dispatch.
#[derive(Default)]
pub(crate) struct SourceRowMetadata {
    pub(crate) formulas: HashMap<usize, FormulaData>,
    pub(crate) display_values: HashMap<usize, String>,
    pub(crate) decimal_values: HashMap<usize, bigdecimal::BigDecimal>,
    pub(crate) present_columns: HashSet<usize>,
}

pub(crate) trait RowConsumer {
    #[allow(clippy::too_many_arguments)]
    fn process(
        &mut self,
        sheet_no: usize,
        sheet_name: &str,
        row_index: u32,
        cells: Vec<CellValue>,
        metadata: SourceRowMetadata,
        options: &ReadOptions,
        headers: &mut Arc<HashMap<String, usize>>,
    ) -> Result<ReadFlow>;

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<ReadFlow>;

    fn after(&mut self, context: &AnalysisContext) -> Result<()>;
}

pub(crate) struct TypedRowConsumer<'a, T> {
    pub(crate) listener: &'a mut dyn ReadListener<T>,
}

impl<T: ExcelRow> RowConsumer for TypedRowConsumer<'_, T> {
    fn process(
        &mut self,
        sheet_no: usize,
        sheet_name: &str,
        row_index: u32,
        cells: Vec<CellValue>,
        metadata: SourceRowMetadata,
        options: &ReadOptions,
        headers: &mut Arc<HashMap<String, usize>>,
    ) -> Result<ReadFlow> {
        process_row_with_metadata::<T>(
            sheet_no,
            sheet_name,
            row_index,
            cells,
            metadata,
            options,
            headers,
            self.listener,
        )
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<ReadFlow> {
        let result = self.listener.extra(extra, context);
        listener_result(result, self.listener, context)
    }

    fn after(&mut self, context: &AnalysisContext) -> Result<()> {
        self.listener.do_after_all_analysed(context)
    }
}

#[allow(clippy::too_many_arguments)]
fn process_row_with_metadata<T>(
    sheet_no: usize,
    sheet_name: &str,
    row_index: u32,
    mut cells: Vec<CellValue>,
    metadata: SourceRowMetadata,
    options: &ReadOptions,
    headers: &mut Arc<HashMap<String, usize>>,
    listener: &mut dyn ReadListener<T>,
) -> Result<ReadFlow>
where
    T: ExcelRow,
{
    let SourceRowMetadata {
        formulas,
        display_values,
        decimal_values,
        present_columns,
    } = metadata;
    if options.auto_trim {
        trim_string_cells(&mut cells);
    }
    let context = analysis_context(sheet_name, sheet_no, row_index, options);
    if row_index < options.head_row_number {
        let current_headers = Arc::new(header_map(&cells, &options.header_aliases));
        if row_index + 1 == options.head_row_number {
            *headers = Arc::clone(&current_headers);
        }
        let result = listener.invoke_head(&current_headers, &context);
        return listener_result(result, listener, &context);
    }
    if options.ignore_empty_row && cells.iter().all(is_empty_read_cell) {
        return Ok(ReadFlow::Continue);
    }

    let row = RowData::new(sheet_name, row_index, cells, Arc::clone(headers))
        .with_formulas(formulas)
        .with_display_values(display_values)
        .with_decimal_values(decimal_values)
        .with_present_columns(present_columns)
        .with_read_default_return(options.read_default_return)
        .with_use_1904_windowing(options.use_1904_windowing);
    match T::from_row_with_converters(&row, &options.converters) {
        Ok(data) => {
            let result = listener.invoke(data, &context);
            listener_result(result, listener, &context)
        }
        Err(error) => listener_error(error, listener, &context),
    }
}
