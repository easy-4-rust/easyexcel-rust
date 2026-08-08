//! 向已存在工作表追加类型化行。
//!
//! 对应 Java：`com.alibaba.excel.write.executor.ExcelWriteAddExecutor`（追加写入口）。

use crate::core::{ExcelError, ExcelRow, ExcelWriteMetadata, Result, WriteHandler};
use easyexcel_xlsx::xlsx::generation::{self, Worksheet};

use crate::write::excel_writer_core::{
    HandlerHolderScope, WriteGlobalFlags, apply_loop_merges, collect_handler_content_row_height,
    collect_handler_head_row_height, dynamic_columns_for_row, effective_loop_merges, format_error,
    head_rows_for_schema,
    write_data_row_fast, write_data_row_with_handlers, write_dynamic_headers_with_handlers,
};
use crate::write::image_layout::ImageLayout;
use crate::write::row_scratch::RowScratch;
use crate::write::sheet_style_context::SheetStyleContext;
use crate::write::streaming_schema_plan::StreamingSchemaPlan;
use crate::write::write_options::WriteOptions;
use crate::write::write_progress::WriteProgress;

/// 对应 Java：com.alibaba.excel.write.executor.ExcelWriteAddExecutor。 Appends typed rows onto an existing worksheet.
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

/// 对应 Java：com.alibaba.excel.write.executor.ExcelWriteAddExecutor。 Like [`append_rows_to_worksheet`], optionally mirroring data rows into a gzip spill.
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

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
/// 对应 Java：com.alibaba.excel.write.executor.ExcelWriteAddExecutor。
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
    let schema_plan = StreamingSchemaPlan::compile::<T>(options, handlers)?;
    let columns = schema_plan.columns();
    let loop_merges = effective_loop_merges(columns, options, handlers)?;
    let head_rows = head_rows_for_schema(T::schema(), options)?;
    let image_layout = ImageLayout::new(columns, options, metadata, head_rows, handlers)?;
    if write_head && head_rows > 0 {
        let schema_head;
        let head = if let Some(head) = options.dynamic_head.as_deref() {
            head
        } else {
            schema_head = T::schema()
                .iter()
                .map(crate::metadata::ExcelColumn::head_path)
                .collect::<Vec<_>>();
            &schema_head
        };
        let mut final_head_rows = write_dynamic_headers_with_handlers(
            worksheet,
            columns,
            head,
            &options.sheet_name,
            SheetStyleContext::head(&options.head_style, metadata, global),
            handlers,
            &image_layout,
            row_index,
            options.automatic_merge_head,
            holder_scope,
        )?;
        // Annotation `@HeadRowHeight` or registered `SimpleRowHeightStyleStrategy`
        let head_height = collect_handler_head_row_height(handlers).or(metadata.head_row_height);
        if let Some(height) = head_height {
            for head_row in row_index..row_index + head_rows {
                generation::set_row_height(worksheet, head_row, height).map_err(format_error)?;
            }
            for row in &mut final_head_rows {
                row.row_height = Some(height);
            }
        }
        if let Some(spill) = gzip_spill.as_mut() {
            for row in &final_head_rows {
                spill.write_journal_row(row)?;
            }
        }
        row_index += head_rows;
    }
    let content_height = collect_handler_content_row_height(handlers).or(metadata.content_row_height);
    let capture_journal = gzip_spill.is_some();
    let mut row_scratch = RowScratch::with_capacity(T::schema().len());
    for row in rows {
        if row.is_absent_row() {
            if let Some(spill) = gzip_spill.as_mut() {
                spill.write_journal_row(&crate::write::gzip_spill::JournalRow::empty())?;
            }
            row_index = row_index
                .checked_add(1)
                .ok_or_else(|| ExcelError::Format("XLSX row overflow".to_owned()))?;
            data_index = data_index.saturating_add(1);
            continue;
        }
        // Annotation `@ContentRowHeight` or registered `SimpleRowHeightStyleStrategy`
        if let Some(height) = content_height {
            generation::set_row_height(worksheet, row_index, height).map_err(format_error)?;
        }
        row_scratch.convert(
            &row,
            &options.converters,
            &options.sheet_name,
            row_index,
            columns,
            schema_plan.selected_schema_indexes(),
        )?;
        let original_cells = row_scratch.original_cells();
        let cells = row_scratch.converted_cells();
        let dynamic_columns = dynamic_columns_for_row(T::schema().is_empty(), cells.len(), options);
        let row_columns = dynamic_columns.as_deref().unwrap_or(columns);
        let style = (!options.content_styles.is_empty())
            .then(|| &options.content_styles[data_index % options.content_styles.len()]);
        apply_loop_merges(worksheet, row_index, data_index, &loop_merges)?;
        if !capture_journal && !schema_plan.requires_handler_context() {
            write_data_row_fast(
                worksheet,
                row_index,
                row_columns,
                cells,
                SheetStyleContext::content(style, metadata, global),
                &image_layout,
            )?;
            row_index += 1;
            data_index += 1;
            continue;
        }
        let mut final_row = write_data_row_with_handlers(
            worksheet,
            row_index,
            data_index,
            row_columns,
            original_cells,
            cells,
            &options.sheet_name,
            SheetStyleContext::content(style, metadata, global),
            handlers,
            &image_layout,
            holder_scope,
        )?;
        // journal 必须记录 Handler 执行后的最终物理单元格，晋升时不得重跑回调。
        if final_row.row_height.is_none() {
            final_row.row_height = content_height;
        }
        if let Some(spill) = gzip_spill.as_mut() {
            spill.write_journal_row(&final_row)?;
        }
        row_index += 1;
        data_index += 1;
    }
    Ok(WriteProgress {
        next_row: row_index,
        next_data_index: data_index,
    })
}
