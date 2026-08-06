/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_csv_to<T, I>(
    path: &Path,
    output: Box<dyn Write + Send>,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let columns = selected_columns(T::schema(), options)?;
    let first_data_row = head_rows_for_schema(T::schema(), options)?;
    let csv_converters =
        crate::converters::default_converter_loader::load_default_write_converter()
            .merged_with(&options.converters)
            .with_write_target(Some(crate::core::CellDataType::String));
    let mut rows = rows.into_iter().enumerate().map(|(offset, row)| {
        prepare_write_row(
            row,
            &csv_converters,
            &options.sheet_name,
            first_data_row.saturating_add(u32::try_from(offset).unwrap_or(u32::MAX)),
            &columns,
        )
    });
    write_csv_records::<T>(
        path,
        output,
        options,
        &columns,
        T::schema().is_empty(),
        &mut rows,
        handlers,
    )
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_csv_records<T>(
    path: &Path,
    output: Box<dyn Write + Send>,
    options: &WriteOptions,
    columns: &[(usize, usize, &'static ExcelColumn)],
    schema_is_empty: bool,
    rows: &mut dyn Iterator<Item = Result<PreparedWriteRow>>,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    csv_encoding(&options.charset)?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        path,
        i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
        None,
        options,
    )?;
    let sheet_context = holder_scope.sheet(WriteSheetContext::new(&options.sheet_name));
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;

    let mut writer = create_csv_record_writer(output, &options.charset, options.with_bom)?;
    append_csv_records(
        &mut writer,
        options,
        columns,
        schema_is_empty,
        rows,
        handlers,
        0,
        0,
        true,
        Some(&holder_scope),
    )?;
    finish_csv_record_writer(writer)?;
    after_sheet(handlers, &sheet_context)?;
    after_workbook(handlers, &workbook_context)
}

#[allow(clippy::too_many_arguments)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn append_csv_records(
    writer: &mut CsvRecordWriter,
    options: &WriteOptions,
    columns: &[(usize, usize, &'static ExcelColumn)],
    schema_is_empty: bool,
    rows: &mut dyn Iterator<Item = Result<PreparedWriteRow>>,
    handlers: &mut [Box<dyn WriteHandler>],
    mut row_index: u32,
    mut data_index: usize,
    write_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress> {
    let mut csv_workbook = CsvWorkbook::new(
        "und",
        options.use_1904_windowing,
        options.use_scientific_format,
        options.charset.clone(),
        options.with_bom,
    );
    let csv_sheet = create_sheet(&mut csv_workbook, &options.sheet_name)?;
    csv_sheet.set_next_row_index(row_index);
    let head_rows = head_rows_for_columns(columns, schema_is_empty, options)?;
    if write_head && head_rows > 0 {
        let head = selected_head_paths(columns, options)?;
        for level in 0..head_rows {
            #[allow(clippy::cast_possible_truncation)]
            let level = level as usize;
            let labels = head
                .iter()
                .map(|path| normalized_head_label(path, level).to_owned())
                .collect::<Vec<_>>();
            let record = csv_header_record(
                csv_sheet,
                row_index,
                columns,
                &labels,
                &options.sheet_name,
                handlers,
                holder_scope,
            )?;
            writer.write_record(record).map_err(ExcelError::from)?;
            row_index = row_index.saturating_add(1);
        }
    }
    for prepared in rows {
        let PreparedWriteRow {
            absent,
            original_cells,
            cells,
        } = prepared?;
        if absent {
            row_index = row_index.saturating_add(1);
            data_index = data_index.saturating_add(1);
            csv_sheet.set_next_row_index(row_index);
            continue;
        }
        let dynamic_columns = dynamic_columns_for_row(schema_is_empty, cells.len(), options);
        let row_columns = dynamic_columns.as_deref().unwrap_or(columns);
        let record = csv_data_record(
            csv_sheet,
            row_index,
            data_index,
            row_columns,
            &original_cells,
            &cells,
            &options.sheet_name,
            handlers,
            holder_scope,
        )?;
        writer.write_record(record).map_err(ExcelError::from)?;
        row_index += 1;
        data_index += 1;
    }
    Ok(WriteProgress {
        next_row: row_index,
        next_data_index: data_index,
    })
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn append_csv_rows<T, I>(
    writer: &mut CsvRecordWriter,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    row_index: u32,
    data_index: usize,
    write_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let columns = selected_columns(T::schema(), options)?;
    let head_rows = if write_head {
        head_rows_for_schema(T::schema(), options)?
    } else {
        0
    };
    let first_data_row = row_index.saturating_add(head_rows);
    let csv_converters =
        crate::converters::default_converter_loader::load_default_write_converter()
            .merged_with(&options.converters)
            .with_write_target(Some(crate::core::CellDataType::String));
    let mut rows = rows.into_iter().enumerate().map(|(offset, row)| {
        prepare_write_row(
            row,
            &csv_converters,
            &options.sheet_name,
            first_data_row.saturating_add(u32::try_from(offset).unwrap_or(u32::MAX)),
            &columns,
        )
    });
    append_csv_records(
        writer,
        options,
        &columns,
        T::schema().is_empty(),
        &mut rows,
        handlers,
        row_index,
        data_index,
        write_head,
        holder_scope,
    )
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn create_csv_record_writer(
    output: Box<dyn Write + Send>,
    charset: &CsvCharset,
    with_bom: bool,
) -> Result<CsvRecordWriter> {
    CsvRecordWriter::new(output, charset, with_bom).map_err(ExcelError::from)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn create_stateful_csv_writer(
    path: &Path,
    charset: &CsvCharset,
    with_bom: bool,
) -> Result<CsvRecordWriter> {
    CsvRecordWriter::from_path(path, charset, with_bom).map_err(ExcelError::from)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn finish_csv_record_writer(writer: CsvRecordWriter) -> Result<()> {
    writer.finish().map_err(ExcelError::from)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn validate_csv_options(options: &WriteOptions) -> Result<()> {
    if options.password.is_some() {
        return Err(ExcelError::Unsupported(
            "password protection is not supported for CSV".to_owned(),
        ));
    }
    csv_encoding(&options.charset)?;
    Ok(())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Saves a workbook to `path` (optionally password-protected).
///
/// `pub(crate)` so executor integration tests can persist worksheets built via
/// [`ExcelWriteAddExecutor`] without duplicating the save path.
pub(crate) fn save_workbook(
    workbook: &mut Workbook,
    path: &Path,
    password: Option<&str>,
) -> Result<()> {
    easyexcel_xlsx::xlsx::generation::save_workbook(workbook, path, password)
        .map_err(ExcelError::from)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn save_workbook_to_writer(
    workbook: &mut Workbook,
    output: &mut (dyn Write + Send),
    password: Option<&str>,
) -> Result<()> {
    easyexcel_xlsx::xlsx::generation::save_workbook_to_writer(workbook, output, password)
        .map_err(ExcelError::from)
}

#[cfg(test)]
pub(crate) fn save_encrypted_workbook_to(
    workbook: &mut Workbook,
    password: &str,
    file: &mut dyn easyexcel_xlsx::ReadWriteSeek,
) -> Result<()> {
    easyexcel_xlsx::xlsx::generation::save_encrypted_workbook_to(workbook, password, file)
        .map_err(ExcelError::from)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn csv_header_record(
    csv_sheet: &mut CsvSheet,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
    labels: &[String],
    sheet_name: &str,
    handlers: &mut [Box<dyn WriteHandler>],
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<Vec<String>> {
    let relative = Some(usize::try_from(row_index).unwrap_or(usize::MAX));
    let row_context = WriteRowContext::new(sheet_name, row_index, relative, true);
    let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
    before_csv_row(handlers, &row_context)?;
    let row = create_row(csv_sheet, row_index)?;
    for ((physical_index, _, column), label) in columns.iter().zip(labels) {
        let column_index = to_column(*physical_index)?;
        let mut context = WriteCellContext::new(
            sheet_name,
            row_index,
            column_index,
            CellValue::String(label.clone()),
        )
        .with_column(column)
        .with_head(label.clone())
        .without_original_value()
        .with_relative_row_index(relative);
        if let Some(scope) = holder_scope {
            context = scope.cell(context);
        }
        before_csv_cell(handlers, &mut context)?;
        after_csv_cell(handlers, &mut context)?;
        if !context.skip {
            create_cell(row, column_index)?.set_value(context.value.clone());
        }
    }
    after_csv_row(handlers, &row_context)?;
    let width = csv_record_width(columns);
    Ok(csv_sheet
        .take_last_row()
        .expect("CSV row was just created")
        .into_record(width))
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn csv_data_record(
    csv_sheet: &mut CsvSheet,
    row_index: u32,
    relative_row_index: usize,
    columns: &[(usize, usize, &'static ExcelColumn)],
    original_cells: &[CellValue],
    cells: &[WriteCellData],
    sheet_name: &str,
    handlers: &mut [Box<dyn WriteHandler>],
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<Vec<String>> {
    let row_context = WriteRowContext::new(sheet_name, row_index, Some(relative_row_index), false);
    let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
    before_csv_row(handlers, &row_context)?;
    let row = create_row(csv_sheet, row_index)?;
    for (physical_index, schema_index, metadata) in columns {
        let column_index = to_column(*physical_index)?;
        let mut context = WriteCellContext::new(
            sheet_name,
            row_index,
            column_index,
            cells
                .get(*schema_index)
                .map_or(CellValue::Empty, WriteCellData::effective_value),
        )
        .with_column(metadata)
        .with_original_value(
            original_cells
                .get(*schema_index)
                .unwrap_or(&CellValue::Empty)
                .clone(),
        )
        .with_relative_row_index(Some(relative_row_index));
        if let Some(scope) = holder_scope {
            context = scope.cell(context);
        }
        before_csv_cell(handlers, &mut context)?;
        after_csv_cell(handlers, &mut context)?;
        if !context.skip {
            create_cell(row, column_index)?.set_value(context.value.clone());
        }
    }
    after_csv_row(handlers, &row_context)?;
    let width = csv_record_width(columns);
    Ok(csv_sheet
        .take_last_row()
        .expect("CSV row was just created")
        .into_record(width))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn csv_record_width(columns: &[(usize, usize, &'static ExcelColumn)]) -> usize {
    columns
        .iter()
        .map(|(physical_index, _, _)| physical_index + 1)
        .max()
        .unwrap_or(0)
}

// XLS-specific helper functions (moved from lib.rs)

