//! 向已存在工作表追加类型化行。
//!
//! 对应 Java：`com.alibaba.excel.write.executor.ExcelWriteAddExecutor`（追加写入口）。

use crate::core::{ExcelError, ExcelRow, ExcelWriteMetadata, Result, WriteCellData, WriteHandler};
use rust_xlsxwriter::Worksheet;

use crate::write::excel_writer_core::{
    HandlerHolderScope, WriteGlobalFlags, apply_loop_merges, collect_handler_content_row_height,
    collect_handler_head_row_height, convert_row_at, dynamic_columns_for_row,
    effective_loop_merges, format_error, head_rows_for_schema, selected_columns,
    write_data_row_with_handlers, write_dynamic_headers_with_handlers, write_headers_with_handlers,
};
use crate::write::image_layout::ImageLayout;
use crate::write::sheet_style_context::SheetStyleContext;
use crate::write::write_options::WriteOptions;
use crate::write::write_progress::WriteProgress;

/// Appends typed rows onto an existing worksheet.
///
/// Java counterpart: the body of `ExcelWriteAddExecutor.add(Collection<?>)`
/// plus `addOneRowOfDataToExcel` (header / cell / handler orchestration).
/// Kept here so the historical `lib.rs` writer path stays intact; the
/// mirrored executor delegates to this function (只增不减).
///
/// # Errors
///
/// Returns a conversion, handler, or XLSX-format error.
pub fn append_rows_to_worksheet<T, I>(
    worksheet: &mut Worksheet,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    progress: WriteProgress,
    write_head: bool,
    metadata: &ExcelWriteMetadata,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    append_rows_to_worksheet_with_gzip::<T, I>(
        worksheet, options, rows, handlers, progress, write_head, metadata, None,
    )
}

/// Like [`append_rows_to_worksheet`], optionally mirroring data rows into a gzip spill.
///
/// Java mapping: when `compress_temp_files` is on, [`crate::write::gzip_spill::GzipSheetDataWriter`]
/// mirrors POI `GZIPSheetDataWriter` for observability and disk spill.
///
/// # Errors
///
/// Returns sheet/write errors from the worksheet writer and I/O errors from
/// the gzip spill.
#[allow(clippy::too_many_arguments)]
pub fn append_rows_to_worksheet_with_gzip<T, I>(
    worksheet: &mut Worksheet,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    progress: WriteProgress,
    write_head: bool,
    metadata: &ExcelWriteMetadata,
    gzip_spill: Option<&mut crate::write::gzip_spill::GzipSheetDataWriter>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    append_rows_to_worksheet_with_gzip_and_context::<T, I>(
        worksheet, options, rows, handlers, progress, write_head, metadata, gzip_spill, None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_rows_to_worksheet_with_gzip_and_context<T, I>(
    worksheet: &mut Worksheet,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    progress: WriteProgress,
    write_head: bool,
    metadata: &ExcelWriteMetadata,
    mut gzip_spill: Option<&mut crate::write::gzip_spill::GzipSheetDataWriter>,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let WriteProgress {
        next_row: mut row_index,
        next_data_index: mut data_index,
    } = progress;
    let global = WriteGlobalFlags::from(options);
    let columns = selected_columns(T::schema(), options)?;
    let loop_merges = effective_loop_merges(&columns, options, handlers)?;
    let head_rows = head_rows_for_schema(T::schema(), options)?;
    let image_layout = ImageLayout::new(&columns, options, metadata, head_rows, handlers)?;
    if write_head && head_rows > 0 {
        if let Some(head) = &options.dynamic_head {
            write_dynamic_headers_with_handlers(
                worksheet,
                &columns,
                head,
                &options.sheet_name,
                SheetStyleContext::head(&options.head_style, metadata, global),
                handlers,
                &image_layout,
                row_index,
                options.automatic_merge_head,
                holder_scope,
            )?;
        } else {
            write_headers_with_handlers(
                worksheet,
                &columns,
                &options.sheet_name,
                SheetStyleContext::head(&options.head_style, metadata, global),
                handlers,
                &image_layout,
                row_index,
                holder_scope,
            )?;
        }
        // Annotation `@HeadRowHeight` or registered `SimpleRowHeightStyleStrategy`
        let head_height = collect_handler_head_row_height(handlers).or(metadata.head_row_height);
        if let Some(height) = head_height {
            for head_row in row_index..row_index + head_rows {
                worksheet
                    .set_row_height(head_row, height)
                    .map_err(format_error)?;
            }
        }
        row_index += head_rows;
    }
    for row in rows {
        if row.is_absent_row() {
            row_index = row_index
                .checked_add(1)
                .ok_or_else(|| ExcelError::Format("XLSX row overflow".to_owned()))?;
            data_index = data_index.saturating_add(1);
            continue;
        }
        // Annotation `@ContentRowHeight` or registered `SimpleRowHeightStyleStrategy`
        let content_height =
            collect_handler_content_row_height(handlers).or(metadata.content_row_height);
        if let Some(height) = content_height {
            worksheet
                .set_row_height(row_index, height)
                .map_err(format_error)?;
        }
        let (original_cells, cells) = convert_row_at(
            &row,
            &options.converters,
            &options.sheet_name,
            row_index,
            &columns,
        )?;
        if let Some(spill) = gzip_spill.as_mut() {
            let values = cells
                .iter()
                .map(WriteCellData::effective_value)
                .collect::<Vec<_>>();
            spill.write_row(&values)?;
        }
        let dynamic_columns = dynamic_columns_for_row(T::schema().is_empty(), cells.len(), options);
        let row_columns = dynamic_columns.as_deref().unwrap_or(&columns);
        let style = (!options.content_styles.is_empty())
            .then(|| &options.content_styles[data_index % options.content_styles.len()]);
        apply_loop_merges(worksheet, row_index, data_index, &loop_merges)?;
        write_data_row_with_handlers(
            worksheet,
            row_index,
            data_index,
            row_columns,
            &original_cells,
            &cells,
            &options.sheet_name,
            SheetStyleContext::content(style, metadata, global),
            handlers,
            &image_layout,
            holder_scope,
        )?;
        row_index += 1;
        data_index += 1;
    }
    Ok(WriteProgress {
        next_row: row_index,
        next_data_index: data_index,
    })
}
