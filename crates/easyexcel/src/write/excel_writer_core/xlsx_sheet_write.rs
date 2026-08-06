/// 对应 Java：无直接对应对象；Rust 架构扩展。 Sets an OOXML column width that serializes as exact character units.
///
/// Java / POI `Sheet.setColumnWidth(col, chars * 256)` becomes
/// `width="{chars}"` in worksheet XML. `rust_xlsxwriter`'s
/// [`Worksheet::set_column_width`] stores `chars * 7 + 5` pixels and round-trips
/// to `~chars + 0.71`; using `chars * 7` pixels yields exact `width="{chars}"`.
pub(crate) fn set_xlsx_column_width_chars(
    worksheet: &mut Worksheet,
    column: u16,
    chars: u16,
) -> Result<()> {
    generation::set_column_width_chars(worksheet, column, chars).map_err(format_error)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_sheet_to_workbook<T, I>(
    workbook: &mut Workbook,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let mut spill = if options.compress_temp_files {
        Some(crate::write::gzip_spill::GzipSheetDataWriter::create_owned(
            &options.sheet_name,
        )?)
    } else {
        None
    };
    write_sheet_to_workbook_with_gzip::<T, I>(
        workbook,
        options,
        rows,
        handlers,
        spill.as_mut(),
        false,
        holder_scope,
    )
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a worksheet and appends rows, optionally mirroring into a gzip spill.
pub(crate) fn write_sheet_to_workbook_with_gzip<T, I>(
    workbook: &mut Workbook,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    gzip_spill: Option<&mut crate::write::gzip_spill::GzipSheetDataWriter>,
    skip_sheet_create_callbacks: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let mut sheet_creator = XlsxSheetCreator {
        workbook,
        constant_memory: uses_constant_memory_spill(options),
    };
    let worksheet = create_sheet(&mut sheet_creator, &options.sheet_name)?;
    for (column, width) in &options.column_widths {
        set_xlsx_column_width_chars(worksheet, *column, *width)?;
    }
    apply_annotation_column_widths::<T>(worksheet, options)?;
    // Static strategy widths (e.g. SimpleColumnWidth) apply before cells.
    apply_handler_column_widths::<T>(worksheet, options, handlers)?;
    apply_annotation_once_absolute_merge::<T>(worksheet, handlers)?;
    // Java `OnceAbsoluteMergeStrategy.afterSheetCreate` via registerWriteHandler
    apply_handler_once_absolute_merge(worksheet, handlers)?;
    for range in &options.merge_ranges {
        generation::merge_range(
            worksheet,
            range.first_row,
            range.first_column,
            range.last_row,
            range.last_column,
            "",
            &generation::new_format(),
        )
        .map_err(format_error)?;
    }
    let head_rows = head_rows_for_schema(T::schema(), options)?;
    let freeze_panes = options
        .freeze_panes
        .or_else(|| (options.freeze_head && options.need_head).then_some((head_rows, 0)));
    if let Some((row, column)) = freeze_panes {
        generation::freeze_panes(worksheet, row, column).map_err(format_error)?;
    }

    let sheet_context = WriteSheetContext::new(&options.sheet_name);
    let sheet_context =
        holder_scope.map_or(sheet_context.clone(), |scope| scope.sheet(sheet_context));
    if !skip_sheet_create_callbacks {
        before_sheet(handlers, &sheet_context)?;
        after_sheet_create(handlers, &sheet_context)?;
    }

    let progress = append_rows_to_worksheet_with_gzip_and_context::<T, I>(
        worksheet,
        options,
        rows,
        handlers,
        WriteProgress {
            // Java `WriteContextImpl.initHead`: newRowIndex += relativeHeadRowIndex()
            next_row: relative_head_start_row(options),
            next_data_index: 0,
        },
        true,
        T::write_metadata(),
        gzip_spill,
        holder_scope,
    )?;
    after_sheet(handlers, &sheet_context)?;
    // Optional autofit first; byte-length widths reapplied so LongestMatch
    // is not autofit-only (Java setColumnWidth(String.getBytes().length)).
    if options.auto_width || handlers_request_auto_width(handlers) {
        generation::autofit(worksheet);
    }
    // LongestMatch measures in after_cell — re-apply measured widths after write
    // (Java AbstractColumnWidthStyleStrategy.afterCellDispose → setColumnWidth).
    apply_handler_column_widths::<T>(worksheet, options, handlers)?;
    Ok(progress)
}
