#[cfg(test)]
fn write_headers(
    worksheet: &mut Worksheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> Result<()> {
    const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
    let layout = ImageLayout::default();
    write_headers_with_handlers(
        worksheet,
        columns,
        "",
        SheetStyleContext::head(&CellStyle::new(), &METADATA, WriteGlobalFlags::default()),
        &mut [],
        &layout,
        0,
        None,
    )
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(crate) fn write_headers_with_handlers(
    worksheet: &mut Worksheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
    sheet_name: &str,
    style: SheetStyleContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    image_layout: &ImageLayout,
    start_row: u32,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let labels = columns
        .iter()
        .map(|(_, _, column)| column.name.to_owned())
        .collect::<Vec<_>>();
    write_header_row_with_handlers(
        worksheet,
        start_row,
        columns,
        &labels,
        sheet_name,
        style,
        handlers,
        image_layout,
        holder_scope,
    )
    .map(|_| ())
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_dynamic_headers_with_handlers(
    worksheet: &mut Worksheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
    sheet_name: &str,
    style: SheetStyleContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    image_layout: &ImageLayout,
    start_row: u32,
    automatic_merge_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<Vec<crate::write::gzip_spill::JournalRow>> {
    let head = selected_dynamic_head_paths(columns, head)?;
    let levels = head.iter().map(Vec::len).max().unwrap_or(0);
    let mut journal_rows = Vec::with_capacity(levels);
    for level in 0..levels {
        #[allow(clippy::cast_possible_truncation)]
        let row_index = start_row.saturating_add(level as u32);
        let labels = head
            .iter()
            .map(|path| normalized_head_label(path, level).to_owned())
            .collect::<Vec<_>>();
        journal_rows.push(write_header_row_with_handlers(
            worksheet,
            row_index,
            columns,
            &labels,
            sheet_name,
            style,
            handlers,
            image_layout,
            holder_scope,
        )?);
    }
    if automatic_merge_head {
        merge_dynamic_head_groups(worksheet, columns, &head, style, start_row)?;
    }
    Ok(journal_rows)
}

#[allow(clippy::too_many_arguments)]
fn write_header_row_with_handlers(
    worksheet: &mut Worksheet,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
    labels: &[String],
    sheet_name: &str,
    style: SheetStyleContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    image_layout: &ImageLayout,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<crate::write::gzip_spill::JournalRow> {
    let relative = Some(usize::try_from(row_index).unwrap_or(usize::MAX));
    let row_context = WriteRowContext::new(sheet_name, row_index, relative, true);
    let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
    begin_row_lifecycle(handlers, &row_context)?;
    let journal_width = columns
        .iter()
        .map(|(physical, _, _)| physical.saturating_add(1))
        .max()
        .unwrap_or(0);
    let mut final_cells = vec![
        crate::write::gzip_spill::JournalCell::plain(CellValue::Empty);
        journal_width
    ];
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
        begin_cell_lifecycle(handlers, &mut context)?;
        finish_cell_lifecycle(handlers, &context)?;
        context.apply_cell_mutations();
        if !context.skip {
            let format_context = if context.ignore_fill_style {
                style.column(column).without_fill_style()
            } else {
                style
                    .column(column)
                    .with_handler_cell(effective_handler_cell_style(handlers, &context))
            };
            let format = cell_format(format_context);
            match &context.value {
                CellValue::String(value) | CellValue::Error(value) => {
                    generation::write_string_with_format(
                        worksheet,
                        row_index,
                        context.column_index,
                        value,
                        &format,
                    )
                    .map_err(format_error)?;
                }
                value => write_cell(
                    worksheet,
                    row_index,
                    context.column_index,
                    column,
                    value,
                    format_context,
                    image_layout,
                )?,
            }
            let final_column = usize::from(context.column_index);
            if final_cells.len() <= final_column {
                final_cells.resize(
                    final_column.saturating_add(1),
                    crate::write::gzip_spill::JournalCell::plain(CellValue::Empty),
                );
            }
            final_cells[final_column] = crate::write::gzip_spill::JournalCell {
                value: context.value.clone(),
                style: crate::write::gzip_spill::JournalCellStyle::from_context(&format_context),
            };
        }
    }
    finish_row_lifecycle(handlers, &row_context)?;
    let row_height = row_context.row().requested_height();
    if let Some(height) = row_height {
        generation::set_row_height(worksheet, row_index, height).map_err(format_error)?;
    }
    Ok(crate::write::gzip_spill::JournalRow {
        cells: final_cells,
        row_height,
    })
}

fn merge_dynamic_head_groups(
    worksheet: &mut Worksheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
    style: SheetStyleContext<'_>,
    start_row: u32,
) -> Result<()> {
    for range in dynamic_head_merge_ranges(columns, head, start_row)? {
        let column_position = columns
            .iter()
            .position(|(physical, _, _)| u16::try_from(*physical).ok() == Some(range.first_column))
            .ok_or_else(|| ExcelError::Format("dynamic head merge column is absent".to_owned()))?;
        let relative_level =
            usize::try_from(range.first_row.saturating_sub(start_row)).unwrap_or(usize::MAX);
        let label = normalized_head_label(&head[column_position], relative_level);
        let format = cell_format(style.column(columns[column_position].2));
        generation::merge_range(
            worksheet,
            range.first_row,
            range.first_column,
            range.last_row,
            range.last_column,
            label,
            &format,
        )
        .map_err(format_error)?;
    }
    Ok(())
}

#[cfg(test)]
fn write_data_row(
    worksheet: &mut Worksheet,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
    cells: &[CellValue],
) -> Result<()> {
    let image_layout = ImageLayout::default();
    let write_cells = cells
        .iter()
        .cloned()
        .map(WriteCellData::new)
        .collect::<Vec<_>>();
    write_data_row_with_handlers(
        worksheet,
        row_index,
        0,
        columns,
        cells,
        &write_cells,
        "",
        SheetStyleContext {
            explicit: None,
            metadata: &ExcelWriteMetadata::new(),
            is_head: false,
            global: WriteGlobalFlags::default(),
        },
        &mut [],
        &image_layout,
        None,
    )
    .map(|_| ())
}

/// 无 Row/Cell Handler 且无需 stateful journal 时的直接写入路径。
///
/// 对应 Java：`ExcelWriteAddExecutor#addOneRowOfDataToExcel` 的无用户
/// Handler 分支。该路径不会构造 `WriteRowContext`、`WriteCellContext` 或
/// `JournalRow`，但保留 converter 样式、注解、图片布局和物理列映射。
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_data_row_fast(
    worksheet: &mut Worksheet,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
    cells: &[WriteCellData],
    style: SheetStyleContext<'_>,
    image_layout: &ImageLayout,
) -> Result<()> {
    for (physical_index, schema_index, metadata) in columns {
        let cell_data = cells.get(*schema_index);
        let effective_value;
        let value = match cell_data {
            Some(cell) if cell.is_plain() => cell.value(),
            Some(cell) => {
                effective_value = cell.effective_value();
                &effective_value
            }
            None => &CellValue::Empty,
        };
        let format_context = style.column(metadata);
        let format_context = cell_data.map_or(format_context, |cell| {
            format_context.with_converted_cell(cell)
        });
        write_cell(
            worksheet,
            row_index,
            to_column(*physical_index)?,
            metadata,
            value,
            format_context,
            image_layout,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_data_row_with_handlers(
    worksheet: &mut Worksheet,
    row_index: u32,
    relative_row_index: usize,
    columns: &[(usize, usize, &'static ExcelColumn)],
    original_cells: &[CellValue],
    cells: &[WriteCellData],
    sheet_name: &str,
    style: SheetStyleContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    image_layout: &ImageLayout,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<crate::write::gzip_spill::JournalRow> {
    let journal_width = columns
        .iter()
        .map(|(physical, _, _)| physical.saturating_add(1))
        .max()
        .unwrap_or(0);
    let mut final_cells = vec![
        crate::write::gzip_spill::JournalCell::plain(CellValue::Empty);
        journal_width
    ];
    let requires_row_context = handlers.iter().any(|handler| handler.requires_row_context());
    let requires_cell_context = handlers
        .iter()
        .any(|handler| handler.requires_cell_context());
    if !requires_row_context && !requires_cell_context {
        for (physical_index, schema_index, metadata) in columns {
            let cell_data = cells.get(*schema_index);
            let column = to_column(*physical_index)?;
            let effective_value;
            let value = match cell_data {
                Some(cell) if cell.is_plain() => cell.value(),
                Some(cell) => {
                    effective_value = cell.effective_value();
                    &effective_value
                }
                None => &CellValue::Empty,
            };
            let format_context = style.column(metadata);
            let format_context = cell_data.map_or(format_context, |cell| {
                format_context.with_converted_cell(cell)
            });
            write_cell(
                worksheet,
                row_index,
                column,
                metadata,
                value,
                format_context,
                image_layout,
            )?;
            final_cells[*physical_index] = crate::write::gzip_spill::JournalCell {
                value: value.clone(),
                style: crate::write::gzip_spill::JournalCellStyle::from_context(&format_context),
            };
        }
        return Ok(crate::write::gzip_spill::JournalRow {
            cells: final_cells,
            row_height: None,
        });
    }

    let row_context = WriteRowContext::new(sheet_name, row_index, Some(relative_row_index), false);
    let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
    begin_row_lifecycle(handlers, &row_context)?;
    for (physical_index, schema_index, metadata) in columns {
        let cell_data = cells.get(*schema_index);
        let value = cell_data.map_or(CellValue::Empty, WriteCellData::effective_value);
        let column = to_column(*physical_index)?;
        let mut context = WriteCellContext::new(sheet_name, row_index, column, value)
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
        begin_cell_lifecycle(handlers, &mut context)?;
        finish_cell_lifecycle(handlers, &context)?;
        context.apply_cell_mutations();
        if !context.skip {
            let format_context = if context.ignore_fill_style {
                style.column(metadata).without_fill_style()
            } else {
                let format_context = style
                    .column(metadata)
                    .with_handler_cell(effective_handler_cell_style(handlers, &context));
                cell_data.map_or(format_context, |cell| {
                    format_context.with_converted_cell(cell)
                })
            };
            write_cell(
                worksheet,
                row_index,
                context.column_index,
                metadata,
                &context.value,
                format_context,
                image_layout,
            )?;
            let final_column = usize::from(context.column_index);
            if final_cells.len() <= final_column {
                final_cells.resize(
                    final_column.saturating_add(1),
                    crate::write::gzip_spill::JournalCell::plain(CellValue::Empty),
                );
            }
            final_cells[final_column] = crate::write::gzip_spill::JournalCell {
                value: context.value.clone(),
                style: crate::write::gzip_spill::JournalCellStyle::from_context(&format_context),
            };
        }
    }
    finish_row_lifecycle(handlers, &row_context)?;
    let row_height = row_context.row().requested_height();
    if let Some(height) = row_height {
        generation::set_row_height(worksheet, row_index, height).map_err(format_error)?;
    }
    Ok(crate::write::gzip_spill::JournalRow {
        cells: final_cells,
        row_height,
    })
}

/// 从 `AutoStreaming` journal 重建一个完整内存 Sheet，不重新执行用户 Handler。
#[allow(clippy::too_many_lines)]
pub(crate) fn replay_stateful_sheet_journal(
    workbook: &mut Workbook,
    state: &StatefulSheetState,
    mut journal: Option<&mut crate::write::gzip_spill::GzipSpillReader>,
) -> Result<()> {
    let mut options = state.options.clone();
    options.constant_memory = false;
    options.compress_temp_files = false;
    let worksheet = generation::create_worksheet(workbook, &options.sheet_name, false)
        .map_err(format_error)?;
    let columns = selected_columns(state.schema, &options)?;
    for (column, width) in &options.column_widths {
        set_xlsx_column_width_chars(worksheet, *column, *width)?;
    }
    for (physical, _, column) in &columns {
        if let Some(width) = column.column_width {
            set_xlsx_column_width_chars(worksheet, to_column(*physical)?, width)?;
        }
    }
    let head_rows = head_rows_for_schema(state.schema, &options)?;
    let freeze_panes = options
        .freeze_panes
        .or_else(|| (options.freeze_head && options.need_head).then_some((head_rows, 0)));
    if let Some((row, column)) = freeze_panes {
        generation::freeze_panes(worksheet, row, column).map_err(format_error)?;
    }
    let global = WriteGlobalFlags::from(&options);
    let image_layout = ImageLayout::new(
        &columns,
        &options,
        &state.metadata,
        head_rows,
        &[],
    )?;
    let mut row_index = relative_head_start_row(&options);
    if options.need_head && head_rows > 0 {
        let schema_head;
        let head = if let Some(head) = options.dynamic_head.as_deref() {
            head
        } else {
            schema_head = state
                .schema
                .iter()
                .map(ExcelColumn::head_path)
                .collect::<Vec<_>>();
            &schema_head
        };
        if let Some(reader) = journal.as_mut() {
            let head_style = SheetStyleContext::head(&options.head_style, &state.metadata, global);
            for level in 0..head_rows {
                let journal_row = reader.next_journal_row()?.ok_or_else(|| {
                    ExcelError::Format(format!(
                        "stateful journal ended before header row {level} for '{}'",
                        options.sheet_name
                    ))
                })?;
                for (physical, _, column) in &columns {
                    let cell = journal_row.cells.get(*physical);
                    let value = cell.map_or(&CellValue::Empty, |cell| &cell.value);
                    if !matches!(value, CellValue::Empty)
                        || cell.is_some_and(|cell| cell.style.is_some())
                    {
                        let format_context = cell
                            .and_then(|cell| cell.style.as_ref())
                            .map_or(head_style.column(column), |delta| {
                                delta.apply(head_style.column(column))
                            });
                        write_cell(
                            worksheet,
                            row_index.saturating_add(level),
                            to_column(*physical)?,
                            column,
                            value,
                            format_context,
                            &image_layout,
                        )?;
                    }
                }
                if let Some(height) = journal_row.row_height {
                    generation::set_row_height(
                        worksheet,
                        row_index.saturating_add(level),
                        height,
                    )
                    .map_err(format_error)?;
                }
            }
            if options.automatic_merge_head {
                let selected_head = selected_dynamic_head_paths(&columns, head)?;
                merge_dynamic_head_groups(worksheet, &columns, &selected_head, head_style, row_index)?;
            }
        } else {
            write_dynamic_headers_with_handlers(
                worksheet,
                &columns,
                head,
                &options.sheet_name,
                SheetStyleContext::head(&options.head_style, &state.metadata, global),
                &mut [],
                &image_layout,
                row_index,
                options.automatic_merge_head,
                None,
            )?;
        }
        row_index = row_index.saturating_add(head_rows);
    }
    let mut data_index = 0usize;
    while let Some(journal_row) = journal
        .as_mut()
        .map_or(Ok(None), |reader| reader.next_journal_row())?
    {
        let style = (!options.content_styles.is_empty())
            .then(|| &options.content_styles[data_index % options.content_styles.len()]);
        let style = SheetStyleContext::content(style, &state.metadata, global);
        for (physical, _, column) in &columns {
            let cell = journal_row.cells.get(*physical);
            let value = cell.map_or(&CellValue::Empty, |cell| &cell.value);
            if !matches!(value, CellValue::Empty)
                || cell.is_some_and(|cell| cell.style.is_some())
            {
                let format_context = cell
                    .and_then(|cell| cell.style.as_ref())
                    .map_or(style.column(column), |delta| delta.apply(style.column(column)));
                write_cell(
                    worksheet,
                    row_index,
                    to_column(*physical)?,
                    column,
                    value,
                    format_context,
                    &image_layout,
                )?;
            }
        }
        if let Some(height) = journal_row
            .row_height
            .or(state.metadata.content_row_height)
        {
            generation::set_row_height(worksheet, row_index, height).map_err(format_error)?;
        }
        row_index = row_index.saturating_add(1);
        data_index = data_index.saturating_add(1);
    }
    if row_index != state.next_row {
        return Err(ExcelError::Format(format!(
            "stateful journal row mismatch for '{}': replayed {row_index}, expected {}",
            options.sheet_name, state.next_row
        )));
    }
    Ok(())
}

// CellFormatContext 是 Java 写入上下文 1:1 映射的聚合值类型，borrow 化会牵动
// 整条调用链；函数体端到端覆盖单元格写入流程，故豁免 too_many_lines /
// large_types_passed_by_value。
#[allow(clippy::too_many_lines, clippy::large_types_passed_by_value)]
fn write_cell(
    worksheet: &mut Worksheet,
    row_index: u32,
    column: u16,
    metadata: &ExcelColumn,
    value: &CellValue,
    style: CellFormatContext<'_>,
    image_layout: &ImageLayout,
) -> Result<()> {
    // Java creates the POI Row and Cell through WorkBookUtil before assigning
    // the typed value. rust_xlsxwriter materialises them on the first write,
    // so the adapter creates and validates the same logical handles here.
    let mut row_creator = XlsxRowCreator { worksheet };
    let mut row = create_row(&mut row_creator, row_index)?;
    let cell = create_cell(&mut row, column)?;
    let XlsxCell {
        worksheet,
        row_index,
        column_index: column,
    } = cell;
    let global = style.global;
    if global.constant_memory
        && matches!(
            value,
            CellValue::Comment { .. }
                | CellValue::CommentWithMetadata { .. }
                | CellValue::Image(_)
                | CellValue::Images { .. }
                | CellValue::RichText(_)
        )
    {
        return Err(ExcelError::Unsupported(
            "constant-memory XLSX does not support comment, image, images, or rich-text cells"
                .to_owned(),
        ));
    }
    // 无样式快速路径：CellFormatContext 全字段为空时，cell_format 的结果恒
    // 等于 rust_xlsxwriter 默认格式（xf 0），直接调用无格式写方法可跳过每个
    // 单元格的 Format 构造与格式表哈希查找（RwLock + Format 哈希）。输出字节
    // 完全一致：默认格式在 workbook 创建时预置为 xf 0，两种路径的单元格 XML
    // 均不带 s 属性，styles.xml 亦不受影响。
    if style.explicit.is_none()
        && style.cell.is_none()
        && style.font.is_none()
        && style.handler_cell.is_none()
        && style.converted_cell.is_none()
        && style.converted_data_format.is_none()
    {
        match value {
            CellValue::String(text) | CellValue::Error(text) => {
                let text = easyexcel_utils::string_utils::maybe_trim(text, global.auto_trim);
                if text.is_empty() {
                    // 空字符串经带格式写入会落成空白单元格（store_string 语义），
                    // 无格式写入则整格跳过——为保持优化前输出，回退带格式路径。
                    let format = generation::new_format();
                    return generation::write_string_with_format(
                        worksheet, row_index, column, text, &format,
                    )
                    .map_err(format_error);
                }
                return generation::write_string(worksheet, row_index, column, text)
                    .map_err(format_error);
            }
            CellValue::Bool(flag) => {
                return generation::write_boolean(worksheet, row_index, column, *flag)
                    .map_err(format_error);
            }
            CellValue::Int(number) => {
                return write_integer_unformatted(worksheet, row_index, column, *number);
            }
            CellValue::Float(number) => {
                if global.use_scientific_format
                    && metadata.effective_number_format().is_none()
                    && easyexcel_format::is_scientific_magnitude(*number)
                {
                    // 科学计数法需要数字格式，落入下方带格式路径。
                } else {
                    return generation::write_number(worksheet, row_index, column, *number)
                        .map_err(format_error);
                }
            }
            CellValue::Decimal(number) => {
                let numeric = finite_decimal_f64(number, "XLSX")?;
                if decimal_integer_requires_text(number)? {
                    return generation::write_string(
                        worksheet,
                        row_index,
                        column,
                        number.to_plain_string(),
                    )
                    .map_err(format_error);
                }
                if global.use_scientific_format
                    && metadata.effective_number_format().is_none()
                    && easyexcel_format::is_scientific_magnitude(numeric)
                {
                    // 科学计数法需要数字格式，落入下方带格式路径。
                } else {
                    return generation::write_number(worksheet, row_index, column, numeric)
                        .map_err(format_error);
                }
            }
            CellValue::Formula(text) => {
                return generation::write_formula(worksheet, row_index, column, text.as_str())
                    .map_err(format_error);
            }
            // 其余类型（Empty/Date/DateTime/Hyperlink/Comment/Image/RichText/
            // Images）必然携带格式或特殊语义（如 Hyperlink 无格式写入会套用
            // 超链接样式），一律走带格式路径。
            _ => {}
        }
    }
    let format = cell_format(style);
    match value {
        CellValue::Empty => {
            generation::write_blank(worksheet, row_index, column, &format).map_err(format_error)?;
        }
        CellValue::String(value) | CellValue::Error(value) => {
            let text = easyexcel_utils::string_utils::maybe_trim(value, global.auto_trim);
            generation::write_string_with_format(
                worksheet,
                row_index,
                column,
                text.as_ref(),
                &format,
            )
            .map_err(format_error)?;
        }
        CellValue::Bool(value) => {
            generation::write_boolean_with_format(worksheet, row_index, column, *value, &format)
                .map_err(format_error)?;
        }
        CellValue::Int(value) => {
            write_integer(worksheet, row_index, column, *value, &format)?;
        }
        CellValue::Float(value) => {
            let mut cell_format = format.clone();
            if global.use_scientific_format
                && metadata.effective_number_format().is_none()
                && easyexcel_format::is_scientific_magnitude(*value)
            {
                cell_format = generation::with_number_format(cell_format, "0.#####E0");
            }
            generation::write_number_with_format(
                worksheet,
                row_index,
                column,
                *value,
                &cell_format,
            )
            .map_err(format_error)?;
        }
        CellValue::Decimal(value) => {
            let numeric = finite_decimal_f64(value, "XLSX")?;
            if decimal_integer_requires_text(value)? {
                generation::write_string_with_format(
                    worksheet,
                    row_index,
                    column,
                    value.to_plain_string(),
                    &format,
                )
                .map_err(format_error)?;
                return Ok(());
            }
            let mut cell_format = format.clone();
            if global.use_scientific_format
                && metadata.effective_number_format().is_none()
                && easyexcel_format::is_scientific_magnitude(numeric)
            {
                cell_format = generation::with_number_format(cell_format, "0.#####E0");
            }
            generation::write_number_with_format(
                worksheet,
                row_index,
                column,
                numeric,
                &cell_format,
            )
            .map_err(format_error)?;
        }
        CellValue::Date(value) => {
            let number_format = easyexcel_format::excel_date_format_code(
                metadata.effective_date_time_format(),
                "yyyy-mm-dd",
            );
            let format = generation::with_number_format(format.clone(), &number_format);
            if global.use_1904_windowing {
                let serial = date_to_excel_serial_with_windowing(*value, true);
                generation::write_number_with_format(worksheet, row_index, column, serial, &format)
                    .map_err(format_error)?;
            } else {
                generation::write_date_with_format(worksheet, row_index, column, *value, &format)
                    .map_err(format_error)?;
            }
        }
        CellValue::DateTime(value) => {
            let number_format = easyexcel_format::excel_date_format_code(
                metadata.effective_date_time_format(),
                "yyyy-mm-dd hh:mm:ss",
            );
            let format = generation::with_number_format(format.clone(), &number_format);
            if global.use_1904_windowing {
                let serial = datetime_to_excel_serial_with_windowing(*value, true);
                generation::write_number_with_format(worksheet, row_index, column, serial, &format)
                    .map_err(format_error)?;
            } else {
                generation::write_datetime_with_format(
                    worksheet, row_index, column, *value, &format,
                )
                .map_err(format_error)?;
            }
        }
        CellValue::Formula(value) => {
            generation::write_formula_with_format(
                worksheet,
                row_index,
                column,
                value.as_str(),
                &format,
            )
            .map_err(format_error)?;
        }
        CellValue::Hyperlink { url, text } => {
            generation::write_url_with_options(
                worksheet,
                row_index,
                column,
                url.as_str(),
                text,
                &format,
            )
            .map_err(format_error)?;
        }
        CellValue::HyperlinkWithMetadata {
            address,
            text,
            hyperlink_type,
            ..
        } => {
            if *hyperlink_type == HyperlinkType::None {
                generation::write_string_with_format(
                    worksheet,
                    row_index,
                    column,
                    text,
                    &format,
                )
                .map_err(format_error)?;
            } else {
                let target = xlsx_hyperlink_target(address, *hyperlink_type);
                generation::write_url_with_options(
                    worksheet,
                    row_index,
                    column,
                    &target,
                    text,
                    &format,
                )
                .map_err(format_error)?;
            }
        }
        CellValue::Comment { value, text } => {
            write_cell(
                worksheet,
                row_index,
                column,
                metadata,
                value,
                style,
                image_layout,
            )?;
            generation::insert_note(worksheet, row_index, column, text).map_err(format_error)?;
        }
        CellValue::CommentWithMetadata { value, comment } => {
            write_cell(
                worksheet,
                row_index,
                column,
                metadata,
                value,
                style,
                image_layout,
            )?;
            generation::insert_note_with_metadata(
                worksheet,
                row_index,
                column,
                &comment.note_text(),
                comment.get_author(),
                xlsx_comment_movement(comment.get_anchor().get_anchor_type()),
            )
            .map_err(format_error)?;
        }
        CellValue::Image(bytes) => {
            generation::insert_image_fit_to_cell(worksheet, row_index, column, bytes, true)
                .map_err(format_error)?;
        }
        CellValue::RichText(value) => {
            write_rich_text(worksheet, row_index, column, value, &format)?;
        }
        CellValue::Images { value, images } => {
            write_cell(
                worksheet,
                row_index,
                column,
                metadata,
                value,
                style,
                image_layout,
            )?;
            for image in images {
                insert_image_data(worksheet, row_index, column, image, image_layout)?;
            }
        }
    }
    Ok(())
}

const fn xlsx_comment_movement(
    value: Option<crate::core::AnchorType>,
) -> Option<generation::ObjectMovement> {
    match value {
        Some(crate::core::AnchorType::MoveAndResize)
        | Some(crate::core::AnchorType::DontMoveDoResize) => {
            Some(generation::ObjectMovement::MoveAndSizeWithCells)
        }
        Some(crate::core::AnchorType::MoveDontResize) => {
            Some(generation::ObjectMovement::MoveButDontSizeWithCells)
        }
        Some(crate::core::AnchorType::DontMoveAndResize) => {
            Some(generation::ObjectMovement::DontMoveOrSizeWithCells)
        }
        None => None,
    }
}

fn xlsx_hyperlink_target(address: &str, hyperlink_type: HyperlinkType) -> String {
    match hyperlink_type {
        HyperlinkType::None | HyperlinkType::Url => address.to_owned(),
        HyperlinkType::Document if address.starts_with("internal:") => address.to_owned(),
        HyperlinkType::Document => format!("internal:{address}"),
        HyperlinkType::Email if address.to_ascii_lowercase().starts_with("mailto:") => {
            address.to_owned()
        }
        HyperlinkType::Email => format!("mailto:{address}"),
        HyperlinkType::File if address.starts_with("external:") => address.to_owned(),
        HyperlinkType::File => format!("external:{address}"),
    }
}
