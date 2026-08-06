//! Core row-dispatch loop shared by the XLSX, XLS, and CSV read engines.

use crate::core::{CellValue, ExcelError, ExcelRow, ReadListener, Result};
#[cfg(test)]
use crate::read::cell_conversion::from_data;
use crate::read::cell_conversion::from_model_cell;
use crate::read::read_helpers::analysis_context;
#[cfg(test)]
use crate::read::read_helpers::to_column_index;
use crate::read::read_options::ReadOptions;
use crate::read::row_consumer::{ReadFlow, RowConsumer, SourceRowMetadata, TypedRowConsumer};
use crate::read::sheet_selector::SheetSelector;
use crate::read::xlsx_rows::XlsxDisplayCellReader;
#[cfg(test)]
use calamine::{Data, Range, Reader, Xlsx};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub(crate) fn select_sheet_names(
    names: Vec<String>,
    selector: &SheetSelector,
    auto_trim: bool,
) -> Result<Vec<(usize, String)>> {
    let selection = match selector {
        SheetSelector::First => easyexcel_io::SheetSelection::First,
        SheetSelector::Index(index) => easyexcel_io::SheetSelection::Index(*index),
        SheetSelector::Name(name) => easyexcel_io::SheetSelection::Name(name),
        SheetSelector::All => easyexcel_io::SheetSelection::All,
    };
    easyexcel_io::select_sheet_names(names, selection, auto_trim).map_err(ExcelError::from)
}

#[cfg(test)]
pub(crate) fn selected_sheet_names<RS: std::io::Read + std::io::Seek>(
    workbook: &Xlsx<RS>,
    selector: &SheetSelector,
    auto_trim: bool,
) -> Result<Vec<(usize, String)>> {
    select_sheet_names(workbook.sheet_names(), selector, auto_trim)
}

#[cfg(test)]
pub(crate) fn select_xls_sheets(
    sheets: Vec<(String, Range<Data>)>,
    selector: &SheetSelector,
    auto_trim: bool,
) -> Result<Vec<(usize, String, Range<Data>)>> {
    match selector {
        SheetSelector::First => sheets
            .into_iter()
            .next()
            .map(|(name, range)| vec![(0, name, range)])
            .ok_or_else(|| ExcelError::SheetNotFound("0".to_owned())),
        SheetSelector::Index(index) => sheets
            .into_iter()
            .nth(*index)
            .map(|(name, range)| vec![(*index, name, range)])
            .ok_or_else(|| ExcelError::SheetNotFound(index.to_string())),
        SheetSelector::Name(name) => sheets
            .into_iter()
            .enumerate()
            .find(|(_, (candidate, _))| {
                easyexcel_utils::string_utils::equals_with_optional_java_trim(
                    candidate,
                    name,
                    auto_trim,
                )
            })
            .map(|(index, (candidate, range))| vec![(index, candidate, range)])
            .ok_or_else(|| ExcelError::SheetNotFound(name.clone())),
        SheetSelector::All => Ok(sheets
            .into_iter()
            .enumerate()
            .map(|(index, (name, range))| (index, name, range))
            .collect()),
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn read_sheet(
    reader: &mut XlsxDisplayCellReader<'_>,
    sheet_no: usize,
    sheet_name: &str,
    last_explicit_row: Option<u32>,
    extras: &[crate::core::CellExtra],
    options: &ReadOptions,
    consumer: &mut dyn RowConsumer,
) -> Result<ReadFlow> {
    let mut current_index = None;
    let mut current_cells = Vec::new();
    let mut current_formulas = HashMap::new();
    let mut current_display_values = HashMap::new();
    let mut current_decimal_values = HashMap::new();
    let mut current_present_columns = HashSet::new();
    let mut headers = Arc::new(HashMap::new());
    let mut next_row_index = 0;

    while let Some(cell) = reader.next_cell()? {
        let (row, column) = cell.position;
        if current_index != Some(row) {
            if let Some(current) = current_index {
                if dispatch_row(
                    consumer,
                    sheet_no,
                    sheet_name,
                    current,
                    std::mem::take(&mut current_cells),
                    SourceRowMetadata {
                        formulas: std::mem::take(&mut current_formulas),
                        display_values: std::mem::take(&mut current_display_values),
                        decimal_values: std::mem::take(&mut current_decimal_values),
                        present_columns: std::mem::take(&mut current_present_columns),
                    },
                    options,
                    &mut headers,
                )? == ReadFlow::Stop
                {
                    return Ok(ReadFlow::Stop);
                }
                next_row_index = current.saturating_add(1);
            }
            if process_missing_rows(
                next_row_index,
                row,
                sheet_no,
                sheet_name,
                options,
                &mut headers,
                consumer,
            )? == ReadFlow::Stop
            {
                return Ok(ReadFlow::Stop);
            }
            current_index = Some(row);
        }
        if let Some(value) = cell.display_value {
            current_display_values.insert(column, value);
        }
        if let Some(value) = cell.decimal_value {
            current_decimal_values.insert(column, value);
        }
        if current_cells.len() <= column {
            current_cells.resize(column + 1, CellValue::Empty);
        }
        current_present_columns.insert(column);
        current_cells[column] = cell.value;
        if let Some(formula) = cell.formula {
            current_formulas.insert(column, formula);
        }
    }

    if let Some(row) = current_index
        && dispatch_row(
            consumer,
            sheet_no,
            sheet_name,
            row,
            current_cells,
            SourceRowMetadata {
                formulas: current_formulas,
                display_values: current_display_values,
                decimal_values: current_decimal_values,
                present_columns: current_present_columns,
            },
            options,
            &mut headers,
        )? == ReadFlow::Stop
    {
        return Ok(ReadFlow::Stop);
    }

    if let Some(last_row) = last_explicit_row {
        let first_trailing_row = current_index.map_or(0, |row| row.saturating_add(1));
        if process_missing_rows(
            first_trailing_row,
            last_row.saturating_add(1),
            sheet_no,
            sheet_name,
            options,
            &mut headers,
            consumer,
        )? == ReadFlow::Stop
        {
            return Ok(ReadFlow::Stop);
        }
    }

    let final_row = last_explicit_row.or(current_index).unwrap_or_default();
    let context = analysis_context(sheet_name, sheet_no, final_row, options);
    for extra in extras {
        if consumer.extra(extra, &context)? == ReadFlow::Stop {
            return Ok(ReadFlow::Stop);
        }
    }
    consumer.after(&context)?;
    Ok(ReadFlow::Continue)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn process_missing_rows(
    start_row: u32,
    end_row: u32,
    sheet_no: usize,
    sheet_name: &str,
    options: &ReadOptions,
    headers: &mut Arc<HashMap<String, usize>>,
    consumer: &mut dyn RowConsumer,
) -> Result<ReadFlow> {
    for row_index in start_row..end_row {
        if dispatch_row(
            consumer,
            sheet_no,
            sheet_name,
            row_index,
            Vec::new(),
            SourceRowMetadata::default(),
            options,
            headers,
        )? == ReadFlow::Stop
        {
            return Ok(ReadFlow::Stop);
        }
    }
    Ok(ReadFlow::Continue)
}

#[cfg(test)]
pub(crate) fn read_range(
    range: &Range<Data>,
    sheet_no: usize,
    sheet_name: &str,
    options: &ReadOptions,
    sheet_displays: &HashMap<(u32, usize), String>,
    consumer: &mut dyn RowConsumer,
) -> Result<ReadFlow> {
    let mut headers = Arc::new(HashMap::new());
    let Some((start_row, start_column)) = range.start() else {
        consumer.after(&analysis_context(sheet_name, sheet_no, 0, options))?;
        return Ok(ReadFlow::Continue);
    };
    let start_column = to_column_index(start_column)?;
    let mut row_index = start_row;
    let mut final_row = start_row;
    for row in range.rows() {
        final_row = row_index;
        let mut cells = vec![CellValue::Empty; start_column];
        cells.extend(
            row.iter()
                .map(|value| from_data(value, options.use_1904_windowing)),
        );
        let present_columns = row
            .iter()
            .enumerate()
            .filter_map(|(offset, value)| {
                (!matches!(value, Data::Empty)).then_some(start_column + offset)
            })
            .collect::<HashSet<_>>();
        let mut display_values = HashMap::new();
        for &column in &present_columns {
            if let Some(display) = sheet_displays.get(&(row_index, column)) {
                display_values.insert(column, display.clone());
            }
        }
        if dispatch_row(
            consumer,
            sheet_no,
            sheet_name,
            row_index,
            cells,
            SourceRowMetadata {
                display_values,
                present_columns,
                ..SourceRowMetadata::default()
            },
            options,
            &mut headers,
        )? == ReadFlow::Stop
        {
            return Ok(ReadFlow::Stop);
        }
        row_index = row_index.saturating_add(1);
    }
    consumer.after(&analysis_context(sheet_name, sheet_no, final_row, options))?;
    Ok(ReadFlow::Continue)
}

/// Dispatch one neutral engine worksheet through the EasyExcel listener lifecycle.
///
/// BIFF/CFB parsing belongs to `easyexcel-xls`; this adapter only preserves the
/// facade's row metadata, formula and listener semantics.
pub(crate) fn read_model_sheet(
    sheet: &easyexcel_model::Sheet,
    sheet_no: usize,
    sheet_name: &str,
    options: &ReadOptions,
    sheet_displays: &HashMap<(u32, usize), String>,
    consumer: &mut dyn RowConsumer,
) -> Result<ReadFlow> {
    let mut stored_rows = sheet.stored_rows().peekable();
    if stored_rows.peek().is_none() {
        consumer.after(&analysis_context(sheet_name, sheet_no, 0, options))?;
        return Ok(ReadFlow::Continue);
    }
    let mut headers = Arc::new(HashMap::new());
    let mut final_row = 0;
    for stored_row in stored_rows {
        let row_index = stored_row.index();
        final_row = row_index;
        let width = usize::try_from(stored_row.physical_width())
            .map_err(|_| ExcelError::Format("XLS column width exceeds usize".to_owned()))?;
        let mut cells = vec![CellValue::Empty; width];
        let mut formulas = HashMap::new();
        let mut present_columns = HashSet::new();
        let mut display_values = HashMap::new();

        for (column, cell) in stored_row.cells() {
            let column = usize::try_from(column)
                .map_err(|_| ExcelError::Format("XLS column index exceeds usize".to_owned()))?;
            let (value, formula) = from_model_cell(cell);
            if !value.is_empty() {
                present_columns.insert(column);
            }
            cells[column] = value;
            if let Some(formula) = formula {
                formulas.insert(column, formula);
            }
            if let Some(display) = sheet_displays.get(&(row_index, column)) {
                display_values.insert(column, display.clone());
            }
        }

        if dispatch_row(
            consumer,
            sheet_no,
            sheet_name,
            row_index,
            cells,
            SourceRowMetadata {
                formulas,
                display_values,
                present_columns,
                ..SourceRowMetadata::default()
            },
            options,
            &mut headers,
        )? == ReadFlow::Stop
        {
            return Ok(ReadFlow::Stop);
        }
    }
    consumer.after(&analysis_context(sheet_name, sheet_no, final_row, options))?;
    Ok(ReadFlow::Continue)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn process_row<T>(
    sheet_no: usize,
    sheet_name: &str,
    row_index: u32,
    cells: Vec<CellValue>,
    options: &ReadOptions,
    headers: &mut Arc<HashMap<String, usize>>,
    listener: &mut dyn ReadListener<T>,
) -> Result<ReadFlow>
where
    T: ExcelRow,
{
    let present_columns = (0..cells.len()).collect();
    let mut consumer = TypedRowConsumer::<T> { listener };
    dispatch_row(
        &mut consumer,
        sheet_no,
        sheet_name,
        row_index,
        cells,
        SourceRowMetadata {
            present_columns,
            ..SourceRowMetadata::default()
        },
        options,
        headers,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_row(
    consumer: &mut dyn RowConsumer,
    sheet_no: usize,
    sheet_name: &str,
    row_index: u32,
    cells: Vec<CellValue>,
    metadata: SourceRowMetadata,
    options: &ReadOptions,
    headers: &mut Arc<HashMap<String, usize>>,
) -> Result<ReadFlow> {
    if row_index >= options.head_row_number
        && (options.start_row.is_some_and(|start| row_index < start)
            || options.end_row.is_some_and(|end| row_index > end))
    {
        return Ok(ReadFlow::Continue);
    }
    consumer.process(
        sheet_no, sheet_name, row_index, cells, metadata, options, headers,
    )
}
