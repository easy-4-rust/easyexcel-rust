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

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn select_sheet_names(
    names: Vec<String>,
    selector: &SheetSelector,
    auto_trim: bool,
) -> Result<Vec<(usize, String)>> {
    easyexcel_io::select_sheet_names(names, selector.as_engine_selection(), auto_trim)
        .map_err(ExcelError::from)
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
                    candidate, name, auto_trim,
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
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn read_sheet(
    reader: &mut XlsxDisplayCellReader<'_>,
    sheet_no: usize,
    sheet_name: &str,
    last_explicit_row: Option<u32>,
    extras: &[crate::core::CellExtra],
    options: &ReadOptions,
    consumer: &mut dyn RowConsumer,
) -> Result<ReadFlow> {
    let dispatch_plan = crate::read::read_dispatch_plan::ReadDispatchPlan::compile(consumer);
    let fast = dispatch_plan.typed_scalar_fast_path() && extras.is_empty();
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
                let stop = if fast {
                    dispatch_row_fast(
                        consumer,
                        sheet_no,
                        sheet_name,
                        current,
                        std::mem::take(&mut current_cells),
                        options,
                        &mut headers,
                    )?
                } else {
                    dispatch_row(
                        consumer,
                        sheet_no,
                        sheet_name,
                        current,
                        std::mem::take(&mut current_cells),
                        SourceRowMetadata {
                            formulas: Some(std::mem::take(&mut current_formulas)),
                            display_values: Some(std::mem::take(&mut current_display_values)),
                            decimal_values: Some(std::mem::take(&mut current_decimal_values)),
                            present_columns: Some(std::mem::take(&mut current_present_columns)),
                        },
                        options,
                        &mut headers,
                    )?
                };
                if stop == ReadFlow::Stop {
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
                fast,
            )? == ReadFlow::Stop
            {
                return Ok(ReadFlow::Stop);
            }
            current_index = Some(row);
        }
        if dispatch_plan.retain_display_values() {
            if let Some(value) = cell.display_value {
                current_display_values.insert(column, value);
            }
        }
        if dispatch_plan.retain_decimal_values() {
            if let Some(value) = cell.decimal_value {
                current_decimal_values.insert(column, value);
            }
        }
        if current_cells.len() <= column {
            current_cells.resize(column + 1, CellValue::Empty);
        }
        if dispatch_plan.retain_present_columns() {
            current_present_columns.insert(column);
        }
        current_cells[column] = cell.value;
        if dispatch_plan.retain_formulas() {
            if let Some(formula) = cell.formula {
                current_formulas.insert(column, formula);
            }
        }
    }

    if let Some(row) = current_index {
        let stop = if fast {
            dispatch_row_fast(
                consumer,
                sheet_no,
                sheet_name,
                row,
                current_cells,
                options,
                &mut headers,
            )?
        } else {
            dispatch_row(
                consumer,
                sheet_no,
                sheet_name,
                row,
                current_cells,
                SourceRowMetadata {
                    formulas: Some(current_formulas),
                    display_values: Some(current_display_values),
                    decimal_values: Some(current_decimal_values),
                    present_columns: Some(current_present_columns),
                },
                options,
                &mut headers,
            )?
        };
        if stop == ReadFlow::Stop {
            return Ok(ReadFlow::Stop);
        }
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
            fast,
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
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn process_missing_rows(
    start_row: u32,
    end_row: u32,
    sheet_no: usize,
    sheet_name: &str,
    options: &ReadOptions,
    headers: &mut Arc<HashMap<String, usize>>,
    consumer: &mut dyn RowConsumer,
    fast: bool,
) -> Result<ReadFlow> {
    for row_index in start_row..end_row {
        let stop = if fast {
            dispatch_row_fast(
                consumer,
                sheet_no,
                sheet_name,
                row_index,
                Vec::new(),
                options,
                headers,
            )?
        } else {
            dispatch_row(
                consumer,
                sheet_no,
                sheet_name,
                row_index,
                Vec::new(),
                SourceRowMetadata::default(),
                options,
                headers,
            )?
        };
        if stop == ReadFlow::Stop {
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
                display_values: Some(display_values),
                present_columns: Some(present_columns),
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

/// Dispatch one neutral engine worksheet through the `EasyExcel` listener lifecycle.
///
/// BIFF/CFB parsing belongs to `easyexcel-xls`; this adapter only preserves the
/// facade's row metadata, formula and listener semantics.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
#[allow(clippy::too_many_arguments)]
pub(crate) fn read_model_sheet(
    sheet: &easyexcel_model::Sheet,
    sheet_no: usize,
    sheet_name: &str,
    options: &ReadOptions,
    sheet_displays: &HashMap<(u32, usize), String>,
    rich_text_cells: &HashMap<(usize, u32, usize), crate::core::RichTextStringData>,
    extras: &[&crate::core::CellExtra],
    consumer: &mut dyn RowConsumer,
) -> Result<ReadFlow> {
    let dispatch_plan = crate::read::read_dispatch_plan::ReadDispatchPlan::compile(consumer);
    let mut stored_rows = sheet.stored_rows().peekable();
    if stored_rows.peek().is_none() {
        if dispatch_extras(consumer, sheet_no, sheet_name, options, extras)? == ReadFlow::Stop {
            return Ok(ReadFlow::Stop);
        }
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
            let (mut value, formula) = from_model_cell(cell);
            if let Some(rich_text) = rich_text_cells.get(&(sheet_no, row_index, column)) {
                value = CellValue::RichText(rich_text.clone());
            }
            if dispatch_plan.retain_present_columns() && !value.is_empty() {
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
                formulas: Some(formulas),
                display_values: Some(display_values),
                present_columns: Some(present_columns),
                ..SourceRowMetadata::default()
            },
            options,
            &mut headers,
        )? == ReadFlow::Stop
        {
            return Ok(ReadFlow::Stop);
        }
    }
    if dispatch_extras(consumer, sheet_no, sheet_name, options, extras)? == ReadFlow::Stop {
        return Ok(ReadFlow::Stop);
    }
    consumer.after(&analysis_context(sheet_name, sheet_no, final_row, options))?;
    Ok(ReadFlow::Continue)
}

fn dispatch_extras(
    consumer: &mut dyn RowConsumer,
    sheet_no: usize,
    sheet_name: &str,
    options: &ReadOptions,
    extras: &[&crate::core::CellExtra],
) -> Result<ReadFlow> {
    for extra in extras {
        let context = analysis_context(sheet_name, sheet_no, extra.first_row_index(), options);
        if consumer.extra(extra, &context)? == ReadFlow::Stop {
            return Ok(ReadFlow::Stop);
        }
    }
    Ok(ReadFlow::Continue)
}

#[allow(clippy::too_many_arguments)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
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
    // 只有动态 RowData 需要区分“源中缺失”和“显式空单元格”。强类型 schema
    // 已固定列位置，不为 CSV 的每一行构造 0..len 的 HashSet；与 XLSX
    // ReadDispatchPlan 的 capability 判定保持一致。
    let present_columns = if T::schema().is_empty() {
        Some((0..cells.len()).collect())
    } else {
        None
    };
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
/// 对应 Java：无直接对应对象；Rust 架构扩展。
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
    if !easyexcel_io::row_is_selected(
        row_index,
        options.head_row_number,
        options.start_row,
        options.end_row,
    ) {
        return Ok(ReadFlow::Continue);
    }
    consumer.process(
        sheet_no, sheet_name, row_index, cells, metadata, options, headers,
    )
}

/// 轻量快路径分派：跳过 `SourceRowMetadata` 装配，直接调用 `process_fast`。
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_row_fast(
    consumer: &mut dyn RowConsumer,
    sheet_no: usize,
    sheet_name: &str,
    row_index: u32,
    cells: Vec<CellValue>,
    options: &ReadOptions,
    headers: &mut Arc<HashMap<String, usize>>,
) -> Result<ReadFlow> {
    if !easyexcel_io::row_is_selected(
        row_index,
        options.head_row_number,
        options.start_row,
        options.end_row,
    ) {
        return Ok(ReadFlow::Continue);
    }
    consumer.process_fast(sheet_no, sheet_name, row_index, cells, options, headers)
}
