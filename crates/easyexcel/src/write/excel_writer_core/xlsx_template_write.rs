/// 对应 Java：WorkBookUtil.createWorkBook。 ZIP/OOXML `withTemplate` path: preserve styles/merges and append sheetData.
///
/// When the requested sheet is missing, creates a new worksheet part inside the
/// package so existing sheets keep their styles and merges. The legacy
/// calamine → `rust_xlsxwriter` seed path is used only when
/// [`WriteOptions::use_legacy_template_seed`] is set.
pub(crate) fn write_xlsx_onto_template_package<T, I>(
    path: &Path,
    output: Option<&mut (dyn Write + Send)>,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    crate::write::template_write::validate_template_source(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    let bytes = crate::write::template_write::load_template_bytes(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    if options.use_legacy_template_seed {
        let mut workbook = easyexcel_xlsx::xlsx::generation::new_workbook();
        write_sheet_onto_template::<T, I>(&mut workbook, options, rows, handlers)?;
        return match output {
            Some(writer) => {
                save_workbook_to_writer(&mut workbook, writer, options.password.as_deref())
            }
            None => save_workbook(&mut workbook, path, options.password.as_deref()),
        };
    }

    let mut package = crate::write::template_write::TemplatePackage::from_bytes(&bytes)?;
    let sheet_names = package.sheet_names()?;
    let (target_index, target_name, create_new) =
        crate::write::template_write::resolve_package_target(
            &sheet_names,
            options.sheet_index,
            &options.sheet_name,
        );
    if create_new {
        package.ensure_sheet(&target_name)?;
    }

    let mut write_options = options.clone();
    write_options.sheet_name.clone_from(&target_name);
    apply_template_holder_layout::<T>(&mut package, &target_name, &write_options, handlers, &[])?;
    let start_row = package.next_row_for_sheet(&target_name)?.saturating_sub(1);
    let head_merges = automatic_dynamic_head_merge_ranges::<T>(&write_options, start_row, true)?;
    package.apply_sheet_layout(&target_name, &[], &head_merges)?;
    let (mut append_rows, original_rows, converted_rows, absent_rows) =
        collect_template_append_rows::<T, I>(&write_options, rows, true, start_row)?;
    let mut row_heights = template_append_row_heights::<T>(
        &write_options,
        handlers,
        true,
        append_rows.len(),
        &absent_rows,
    )?;
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        path,
        i32::try_from(target_index).unwrap_or(i32::MAX),
        None,
        &write_options,
    )?;
    let sheet_context = holder_scope.sheet(WriteSheetContext::new(&target_name));
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;
    let effects = run_template_handler_callbacks::<T>(
        &write_options,
        handlers,
        &mut append_rows,
        &original_rows,
        &absent_rows,
        true,
        0,
        start_row,
        Some(&holder_scope),
    )?;
    if row_heights.is_empty() && effects.requested_row_heights.iter().any(Option::is_some) {
        row_heights.resize(effects.requested_row_heights.len(), None);
    }
    for (height, requested) in row_heights.iter_mut().zip(&effects.requested_row_heights) {
        if requested.is_some() {
            *height = *requested;
        }
    }
    let cell_styles = template_append_cell_styles::<T>(
        &mut package,
        &write_options,
        handlers,
        &append_rows,
        &original_rows,
        &converted_rows,
        &effects.ignore_styles,
        &effects.requested_styles,
        true,
        0,
    )?;
    package.append_rows_with_layout_and_absent(
        &target_name,
        &append_rows,
        &row_heights,
        &cell_styles,
        &absent_rows,
    )?;
    after_sheet(handlers, &sheet_context)?;
    save_template_package(&package, path, output, options.password.as_deref())
}

/// 对应 Java：WorkBookUtil.createWorkBook。 Resolves Java annotation/handler row-height precedence for template rows.
pub(crate) fn template_append_row_heights<T>(
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
    write_head: bool,
    row_count: usize,
    absent_rows: &[bool],
) -> Result<Vec<Option<u16>>>
where
    T: ExcelRow,
{
    let head_start = if write_head {
        usize::try_from(relative_head_start_row(options)).unwrap_or(usize::MAX)
    } else {
        0
    }
    .min(row_count);
    let head_end = head_start
        .saturating_add(if write_head {
            usize::try_from(head_rows_for_schema(T::schema(), options)?).unwrap_or(0)
        } else {
            0
        })
        .min(row_count);
    let metadata = T::write_metadata();
    let head_height = collect_handler_head_row_height(handlers).or(metadata.head_row_height);
    let content_height =
        collect_handler_content_row_height(handlers).or(metadata.content_row_height);
    if head_height.is_none() && content_height.is_none() {
        return Ok(Vec::new());
    }
    Ok((0..row_count)
        .map(|index| {
            if absent_rows.get(index).copied().unwrap_or(false) {
                None
            } else if (head_start..head_end).contains(&index) {
                head_height
            } else {
                content_height
            }
        })
        .collect())
}
/// 对应 Java：WorkBookUtil.createWorkBook。
pub(crate) struct TemplateHandlerEffects {
    pub(crate) ignore_styles: Vec<Vec<bool>>,
    pub(crate) requested_styles: Vec<Vec<Option<ExcelCellStyle>>>,
    pub(crate) requested_row_heights: Vec<Option<u16>>,
}

#[allow(clippy::too_many_arguments)]
/// 对应 Java：WorkBookUtil.createWorkBook。
pub(crate) fn run_template_handler_callbacks<T>(
    options: &WriteOptions,
    handlers: &mut [Box<dyn WriteHandler>],
    rows: &mut [Vec<(usize, CellValue)>],
    original_rows: &[Vec<(usize, CellValue)>],
    absent_rows: &[bool],
    write_head: bool,
    next_data_index: usize,
    start_row: u32,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<TemplateHandlerEffects>
where
    T: ExcelRow,
{
    let columns = selected_columns(T::schema(), options)?;
    let head_start = if write_head {
        usize::try_from(relative_head_start_row(options)).unwrap_or(usize::MAX)
    } else {
        0
    }
    .min(rows.len());
    let head_end = head_start
        .saturating_add(if write_head {
            usize::try_from(head_rows_for_schema(T::schema(), options)?).unwrap_or(0)
        } else {
            0
        })
        .min(rows.len());
    let mut ignored_styles = Vec::with_capacity(rows.len());
    let mut requested_styles = Vec::with_capacity(rows.len());
    let mut requested_row_heights = Vec::with_capacity(rows.len());
    for (row_offset, row) in rows.iter_mut().enumerate() {
        if absent_rows.get(row_offset).copied().unwrap_or(false) {
            ignored_styles.push(Vec::new());
            requested_styles.push(Vec::new());
            requested_row_heights.push(None);
            continue;
        }
        let is_head = (head_start..head_end).contains(&row_offset);
        let row_index = start_row.saturating_add(u32::try_from(row_offset).unwrap_or(u32::MAX));
        let relative_row_index = if is_head {
            Some(row_offset.saturating_sub(head_start))
        } else {
            Some(next_data_index + row_offset.saturating_sub(head_end))
        };
        let row_context =
            WriteRowContext::new(&options.sheet_name, row_index, relative_row_index, is_head);
        let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
        begin_row_lifecycle(handlers, &row_context)?;
        let mut emitted = Vec::with_capacity(row.len());
        let mut row_ignored_styles = Vec::with_capacity(row.len());
        let mut row_requested_styles = Vec::with_capacity(row.len());
        for (physical_index, value) in row.iter() {
            let column = columns
                .iter()
                .find(|(index, _, _)| index == physical_index)
                .map(|(_, _, column)| *column);
            let mut context = WriteCellContext::new(
                &options.sheet_name,
                row_index,
                to_column(*physical_index)?,
                value.clone(),
            )
            .with_relative_row_index(relative_row_index);
            if let Some(column) = column {
                context = context.with_column(column);
            }
            if is_head {
                context = context.with_head(value.as_text()).without_original_value();
            } else if let Some(original_value) = original_rows
                .get(row_offset)
                .and_then(|row| row.iter().find(|(index, _)| index == physical_index))
                .map(|(_, value)| value.clone())
            {
                context = context.with_original_value(original_value);
            }
            if let Some(scope) = holder_scope {
                context = scope.cell(context);
            }
            begin_cell_lifecycle(handlers, &mut context)?;
            finish_cell_lifecycle(handlers, &context)?;
            context.apply_cell_mutations();
            if !context.skip {
                emitted.push((*physical_index, context.value.clone()));
                row_ignored_styles.push(context.ignore_fill_style);
                row_requested_styles.push(context.cell().requested_style());
            }
        }
        *row = emitted;
        ignored_styles.push(row_ignored_styles);
        requested_styles.push(row_requested_styles);
        finish_row_lifecycle(handlers, &row_context)?;
        requested_row_heights.push(row_context.row().requested_height());
    }
    Ok(TemplateHandlerEffects {
        ignore_styles: ignored_styles,
        requested_styles,
        requested_row_heights,
    })
}

/// 对应 Java：WorkBookUtil.createWorkBook。 Compiles annotation, explicit and strategy styles with `rust_xlsxwriter`,
/// imports their OOXML records into the preserved template, and returns a
/// style-index matrix aligned with `rows`.
// 参数与 Java 样式编译流程一一对应，函数体覆盖完整样式矩阵编译，拆分会割裂上下文
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
/// 对应 Java：WorkBookUtil.createWorkBook。
pub(crate) fn template_append_cell_styles<T>(
    package: &mut crate::write::template_write::TemplatePackage,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
    rows: &[Vec<(usize, CellValue)>],
    original_rows: &[Vec<(usize, CellValue)>],
    converted_rows: &[Vec<(usize, WriteCellData)>],
    ignore_styles: &[Vec<bool>],
    requested_styles: &[Vec<Option<ExcelCellStyle>>],
    write_head: bool,
    next_data_index: usize,
) -> Result<Vec<Vec<Option<u32>>>>
where
    T: ExcelRow,
{
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let columns = selected_columns(T::schema(), options)?;
    let metadata = T::write_metadata();
    let global = WriteGlobalFlags::from(options);
    let head_start = if write_head {
        usize::try_from(relative_head_start_row(options)).unwrap_or(usize::MAX)
    } else {
        0
    }
    .min(rows.len());
    let head_end = head_start
        .saturating_add(if write_head {
            usize::try_from(head_rows_for_schema(T::schema(), options)?).unwrap_or(0)
        } else {
            0
        })
        .min(rows.len());
    let start_row = package
        .next_row_for_sheet(&options.sheet_name)?
        .saturating_sub(1);
    let mut formats = Vec::new();
    let mut format_by_key = HashMap::new();
    let mut local_styles = Vec::with_capacity(rows.len());

    for (row_offset, row) in rows.iter().enumerate() {
        let is_head = (head_start..head_end).contains(&row_offset);
        let relative_row_index = if is_head {
            Some(row_offset.saturating_sub(head_start))
        } else {
            Some(next_data_index + row_offset.saturating_sub(head_end))
        };
        let explicit = if is_head {
            Some(&options.head_style)
        } else if options.content_styles.is_empty() {
            None
        } else {
            Some(
                &options.content_styles
                    [relative_row_index.unwrap_or(0) % options.content_styles.len()],
            )
        };
        let mut row_styles = Vec::with_capacity(row.len());
        for (cell_offset, (physical_index, value)) in row.iter().enumerate() {
            let column = columns
                .iter()
                .find(|(index, _, _)| index == physical_index)
                .map(|(_, _, column)| *column);
            let (annotation_cell, annotation_font, field) = match column {
                Some(column) if is_head => (
                    column.head_style.or(metadata.head_style),
                    column.head_font_style.or(metadata.head_font_style),
                    Some(column.field),
                ),
                Some(column) => (
                    column.content_style.or(metadata.content_style),
                    column.content_font_style.or(metadata.content_font_style),
                    Some(column.field),
                ),
                None if is_head => (metadata.head_style, metadata.head_font_style, None),
                None => (metadata.content_style, metadata.content_font_style, None),
            };
            let mut context = WriteCellContext::new(
                &options.sheet_name,
                start_row.saturating_add(u32::try_from(row_offset).unwrap_or(u32::MAX)),
                to_column(*physical_index)?,
                value.clone(),
            )
            .with_relative_row_index(relative_row_index);
            if let Some(column) = column {
                context = context.with_column(column);
            } else {
                context.field = field;
            }
            if is_head {
                context = context.with_head(value.as_text()).without_original_value();
            } else if let Some(original_value) = original_rows
                .get(row_offset)
                .and_then(|row| row.iter().find(|(index, _)| index == physical_index))
                .map(|(_, value)| value.clone())
            {
                context = context.with_original_value(original_value);
            }
            context.activate_original_value();
            context.refresh_converted_data();
            context.ignore_fill_style = ignore_styles
                .get(row_offset)
                .and_then(|row| row.get(cell_offset))
                .copied()
                .unwrap_or(false);
            if context.ignore_fill_style {
                row_styles.push(None);
                continue;
            }
            let handler_cell = collect_handler_cell_style(handlers, &context);
            let handler_cell = requested_styles
                .get(row_offset)
                .and_then(|row| row.get(cell_offset))
                .copied()
                .flatten()
                .map_or(handler_cell, |requested| {
                    Some(match handler_cell {
                        Some(current) => merge_write_cell_style(&requested, current),
                        None => requested,
                    })
                });
            let converted_cell = converted_rows
                .get(row_offset)
                .and_then(|row| row.iter().find(|(index, _)| index == physical_index))
                .map(|(_, cell)| cell);
            let annotation_cell =
                annotation_cell.filter(|style| *style != ExcelCellStyle::default());
            let annotation_font = annotation_font.filter(|font| *font != ExcelFontStyle::default());
            let handler_cell = handler_cell.filter(|style| *style != ExcelCellStyle::default());
            let explicit = explicit.filter(|style| **style != CellStyle::default());
            if explicit.is_none()
                && annotation_cell.is_none()
                && annotation_font.is_none()
                && handler_cell.is_none()
                && converted_cell
                    .and_then(WriteCellData::write_cell_style)
                    .is_none()
                && converted_cell
                    .and_then(WriteCellData::data_format_data)
                    .and_then(|data| data.format())
                    .is_none()
            {
                row_styles.push(None);
                continue;
            }
            let converted_style = converted_cell.and_then(WriteCellData::write_cell_style);
            let converted_format = converted_cell
                .and_then(WriteCellData::data_format_data)
                .and_then(|data| data.format());
            let key = format!(
                "{explicit:?}|{annotation_cell:?}|{annotation_font:?}|{handler_cell:?}|\
                 {converted_style:?}|{converted_format:?}|{global:?}"
            );
            let local_index = if let Some(index) = format_by_key.get(&key).copied() {
                index
            } else {
                let index = formats.len();
                let format_context = CellFormatContext {
                    explicit,
                    cell: annotation_cell,
                    font: annotation_font,
                    handler_cell: None,
                    converted_cell: None,
                    converted_data_format: None,
                    ignore_fill_style: false,
                    global,
                }
                .with_handler_cell(handler_cell);
                let format_context = converted_cell.map_or(format_context, |cell| {
                    format_context.with_converted_cell(cell)
                });
                formats.push(cell_format(format_context));
                format_by_key.insert(key, index);
                index
            };
            row_styles.push(Some(local_index));
        }
        local_styles.push(row_styles);
    }
    if formats.is_empty() {
        return Ok(Vec::new());
    }

    let mut compiler = create_work_book(XlsxWorkBookCreator)?;
    let mut sheet_creator = XlsxSheetCreator {
        workbook: &mut compiler,
        constant_memory: false,
    };
    let worksheet = create_sheet(&mut sheet_creator, "Sheet1")?;
    for (index, format) in formats.iter().enumerate() {
        let row = u32::try_from(index)
            .map_err(|_| ExcelError::Format("too many template styles".to_owned()))?;
        generation::write_blank(worksheet, row, 0, format).map_err(format_error)?;
    }
    let bytes = generation::serialize_workbook(&mut compiler).map_err(ExcelError::from)?;
    let mapped = package.import_compiled_styles(&bytes, formats.len())?;
    Ok(local_styles
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|index| index.map(|index| mapped[index]))
                .collect()
        })
        .collect())
}

/// 对应 Java：WorkBookUtil.createWorkBook。 Builds sparse `(physical_column, value)` rows for ZIP `sheetData` append.
// 四元组返回值与 Java 追加行的多路数据一一对应，提取别名反而割裂阅读
#[allow(clippy::type_complexity)]
/// 对应 Java：WorkBookUtil.createWorkBook。
pub(crate) fn collect_template_append_rows<T, I>(
    options: &WriteOptions,
    rows: I,
    write_head: bool,
    start_row: u32,
) -> Result<(
    Vec<Vec<(usize, CellValue)>>,
    Vec<Vec<(usize, CellValue)>>,
    Vec<Vec<(usize, WriteCellData)>>,
    Vec<bool>,
)>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let columns = selected_columns(T::schema(), options)?;
    let mut output = Vec::new();
    let mut original_output = Vec::new();
    let mut converted_output = Vec::new();
    let mut absent_rows = Vec::new();
    let head_rows = head_rows_for_schema(T::schema(), options)?;
    if write_head {
        for _ in 0..relative_head_start_row(options) {
            output.push(Vec::new());
            original_output.push(Vec::new());
            converted_output.push(Vec::new());
            absent_rows.push(true);
        }
    }
    if write_head && head_rows > 0 {
        let head = selected_head_paths(&columns, options)?;
        for level in 0..usize::try_from(head_rows).unwrap_or(0) {
            let mut row = Vec::with_capacity(columns.len());
            for ((physical_index, _, _), path) in columns.iter().zip(&head) {
                let label = normalized_head_label(path, level).to_owned();
                row.push((*physical_index, CellValue::String(label)));
            }
            output.push(row);
            original_output.push(Vec::new());
            converted_output.push(Vec::new());
            absent_rows.push(false);
        }
    }
    for row in rows {
        if row.is_absent_row() {
            output.push(Vec::new());
            original_output.push(Vec::new());
            converted_output.push(Vec::new());
            absent_rows.push(true);
            continue;
        }
        let row_index = start_row.saturating_add(u32::try_from(output.len()).unwrap_or(u32::MAX));
        let (original_cells, cells) = convert_row_at(
            &row,
            &options.converters,
            &options.sheet_name,
            row_index,
            &columns,
        )?;
        let dynamic_columns = dynamic_columns_for_row(T::schema().is_empty(), cells.len(), options);
        let row_columns = dynamic_columns.as_deref().unwrap_or(&columns);
        let mut sparse = Vec::with_capacity(row_columns.len());
        let mut original_sparse = Vec::with_capacity(row_columns.len());
        let mut converted_sparse = Vec::with_capacity(row_columns.len());
        for (physical_index, schema_index, _) in row_columns {
            let cell = cells
                .get(*schema_index)
                .cloned()
                .unwrap_or_else(|| WriteCellData::new(CellValue::Empty));
            let value = cell.effective_value();
            sparse.push((*physical_index, value));
            converted_sparse.push((*physical_index, cell));
            original_sparse.push((
                *physical_index,
                original_cells
                    .get(*schema_index)
                    .cloned()
                    .unwrap_or(CellValue::Empty),
            ));
        }
        output.push(sparse);
        original_output.push(original_sparse);
        converted_output.push(converted_sparse);
        absent_rows.push(false);
    }
    Ok((output, original_output, converted_output, absent_rows))
}

/// 对应 Java：WorkBookUtil.createWorkBook。 Persists a template package to a path or stream, optionally encrypting.
pub(crate) fn save_template_package(
    package: &crate::write::template_write::TemplatePackage,
    path: &Path,
    output: Option<&mut (dyn Write + Send)>,
    password: Option<&str>,
) -> Result<()> {
    let plaintext = package.to_bytes()?;
    if let Some(writer) = output {
        generation::save_package_bytes_to_writer(&plaintext, writer, password)
            .map_err(ExcelError::from)
    } else {
        generation::save_package_bytes_to_path(&plaintext, path, password).map_err(ExcelError::from)
    }
}

/// Seeds a workbook from `withTemplate` then appends typed rows to the target sheet.
///
/// **Legacy path only** — enabled via [`WriteOptions::use_legacy_template_seed`].
/// Value replay does not preserve styles/merges; prefer the ZIP package path.
///
/// 对应 Java：`WorkBookUtil.createWorkBook` (template branch) + `ExcelWriteAddExecutor`.
///
/// # Errors
///
/// Returns template validation/load errors, or standard XLSX write errors.
pub(crate) fn write_sheet_onto_template<T, I>(
    workbook: &mut Workbook,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    crate::write::template_write::validate_template_source(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    let bytes = crate::write::template_write::load_template_bytes(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    let sheets = easyexcel_xlsx::load_legacy_template_sheets(&bytes)?;
    let (target_index, target_name, create_new) =
        crate::write::template_write::resolve_template_target(
            &sheets,
            options.sheet_index,
            &options.sheet_name,
        );
    easyexcel_xlsx::seed_legacy_template_workbook(workbook, &sheets)?;

    let mut write_options = options.clone();
    write_options.sheet_name.clone_from(&target_name);

    if create_new {
        // Java creates a new sheet when the requested name/index is absent.
        return write_sheet_to_workbook::<T, I>(workbook, &write_options, rows, handlers, None);
    }

    let start_row = sheets.get(target_index).map_or(0, |sheet| sheet.next_row);
    let worksheet = workbook
        .worksheet_from_name(&target_name)
        .map_err(format_error)?;
    for (column, width) in &write_options.column_widths {
        set_xlsx_column_width_chars(worksheet, *column, *width)?;
    }
    apply_annotation_column_widths::<T>(worksheet, &write_options)?;
    apply_handler_column_widths::<T>(worksheet, &write_options, handlers)?;
    apply_annotation_once_absolute_merge::<T>(worksheet, handlers)?;
    apply_handler_once_absolute_merge(worksheet, handlers)?;
    for range in &write_options.merge_ranges {
        let offset = start_row;
        generation::merge_range(
            worksheet,
            range.first_row.saturating_add(offset),
            range.first_column,
            range.last_row.saturating_add(offset),
            range.last_column,
            "",
            &generation::new_format(),
        )
        .map_err(format_error)?;
    }

    let sheet_context = WriteSheetContext::new(&target_name);
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;
    let mut spill = if write_options.compress_temp_files {
        Some(crate::write::gzip_spill::GzipSheetDataWriter::create_owned(
            &target_name,
        )?)
    } else {
        None
    };
    let progress = append_rows_to_worksheet_with_gzip::<T, I>(
        worksheet,
        &write_options,
        rows,
        handlers,
        WriteProgress {
            next_row: start_row,
            next_data_index: 0,
        },
        true,
        T::write_metadata(),
        spill.as_mut(),
    )?;
    after_sheet(handlers, &sheet_context)?;
    if write_options.auto_width || handlers_request_auto_width(handlers) {
        generation::autofit(worksheet);
    }
    // Byte-length widths win over optional autofit fallback.
    apply_handler_column_widths::<T>(worksheet, &write_options, handlers)?;
    Ok(progress)
}
