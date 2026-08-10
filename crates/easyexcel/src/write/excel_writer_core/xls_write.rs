/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_xls_onto_template<T, I>(
    path: &Path,
    output: Option<&mut (dyn Write + Send)>,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    workbook_context: &WriteWorkbookContext,
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let bytes = crate::write::template_write::load_template_bytes(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    if !crate::write::xls_adapter::looks_like_xls(&bytes) {
        return Err(ExcelError::Format(
            "xls with_template requires an OLE .xls workbook".to_owned(),
        ));
    }
    let mut package = crate::write::xls_adapter::Biff8TemplatePackage::from_bytes_with_password(
        &bytes,
        options.password.as_deref(),
    )?;
    let sheet_names = package.sheet_names();
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
    let start_row = package.next_row_for_sheet(&target_name)?;
    for range in automatic_dynamic_head_merge_ranges::<T>(&write_options, start_row, true)? {
        package.add_merge_range(&target_name, merge_range_to_biff8(range)?)?;
    }
    let (mut append_rows, original_rows, _converted_rows, absent_rows) =
        collect_template_append_rows::<T, I>(&write_options, rows, true, start_row)?;
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        path,
        i32::try_from(target_index).unwrap_or(i32::MAX),
        None,
        &write_options,
    )?;
    let sheet_context = holder_scope.sheet(WriteSheetContext::new(&target_name));
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;
    let _ignore_styles = run_template_handler_callbacks::<T>(
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
    package.append_rows(&target_name, &append_rows)?;
    after_sheet(handlers, &sheet_context)?;
    after_workbook(handlers, workbook_context)?;
    package.apply_mutations(workbook_context.mutation_plan())?;
    match output {
        Some(writer) => package.save_to_writer_with_password_and_macro_policy(
            writer,
            options.password.as_deref(),
            &options.biff8_macro_policy,
        ),
        None => package.save_to_path_with_password_and_macro_policy(
            path,
            options.password.as_deref(),
            &options.biff8_macro_policy,
        ),
    }
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn save_xls_book(book: &Biff8Book, path: &Path) -> Result<()> {
    book.save_to_path(path).map_err(ExcelError::from)
}

/// 在 BIFF8 工作簿保存前执行 Handler 提交的后端中立修改。
pub(crate) fn apply_xls_mutations(
    book: &mut Biff8Book,
    plan: &crate::context::write_mutation_plan::WriteMutationPlan,
) -> Result<()> {
    let global = WriteGlobalFlags::for_biff8_mutation(book.use_1904_windowing);
    for mutation in plan.snapshot()? {
        match mutation {
            crate::context::write_mutation::WriteMutation::SetCell {
                sheet_name,
                row_index,
                column_index,
                value,
            } => {
                let cell = cell_value_to_biff8_with_styles(&value, &mut book.styles, global)?;
                book.sheet_mut(&sheet_name)
                    .set(row_index, usize::from(column_index), cell)?;
                add_biff8_cell_metadata(
                    book,
                    &sheet_name,
                    row_index,
                    usize::from(column_index),
                    &value,
                )?;
            }
            crate::context::write_mutation::WriteMutation::ProtectSheet {
                sheet_name,
                password,
            } => book.sheet_mut(&sheet_name).protect_sheet(&password),
            crate::context::write_mutation::WriteMutation::AddChart(chart) => {
                add_biff8_chart(book, &chart)?;
            }
            crate::context::write_mutation::WriteMutation::AddMerge { sheet_name, range } => {
                add_biff8_merge_range(book.sheet_mut(&sheet_name), range)?;
            }
            crate::context::write_mutation::WriteMutation::RemoveComment {
                sheet_name,
                row_index,
                column_index,
            } => {
                book.sheet_mut(&sheet_name)
                    .remove_comment(row_index, usize::from(column_index))?;
            }
        }
    }
    Ok(())
}

pub(crate) fn add_biff8_chart(book: &mut Biff8Book, mutation: &crate::ChartMutation) -> Result<()> {
    book.add_chart_mutation(mutation).map_err(ExcelError::from)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_sheet_to_biff8_book<T, I>(
    book: &mut Biff8Book,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let sheet_name = effective_sheet_name(options);
    let mut write_options = options.clone();
    write_options.sheet_name.clone_from(&sheet_name);
    book.use_1904_windowing = write_options.use_1904_windowing;
    create_sheet(book, &sheet_name)?;
    let sheet_context = WriteSheetContext::new(&sheet_name);
    let sheet_context =
        holder_scope.map_or(sheet_context.clone(), |scope| scope.sheet(sheet_context));
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;
    let progress = append_rows_to_biff8_sheet::<T, I>(
        book,
        &sheet_name,
        &write_options,
        rows,
        handlers,
        WriteProgress {
            next_row: relative_head_start_row(&write_options),
            next_data_index: 0,
        },
        true,
        holder_scope,
    )?;
    after_sheet(handlers, &sheet_context)?;
    Ok(progress)
}

// 参数与 Java 写入路径一一对应且函数体覆盖完整 BIFF8 追加流程，拆分破坏可追溯性
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn append_rows_to_biff8_sheet<T, I>(
    book: &mut Biff8Book,
    sheet_name: &str,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    progress: WriteProgress,
    write_head: bool,
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
    let metadata = T::write_metadata();
    let head_rows = head_rows_for_schema(T::schema(), options)?;
    let loop_merges = effective_loop_merges(&columns, options, handlers)?;

    if write_head {
        apply_biff8_column_widths::<T>(book.sheet_mut(sheet_name), options, handlers)?;
        apply_biff8_once_absolute_merges::<T>(book.sheet_mut(sheet_name), handlers)?;
        for range in &options.merge_ranges {
            add_biff8_merge_range(book.sheet_mut(sheet_name), *range)?;
        }
        // Java `Sheet.createFreezePane(row, col)` — 与 XLSX 路径同一表达式
        // (`freeze_head && need_head` → 冻结表头行数)。BIFF8 边界：行≤65535、
        // 列≤255，越界与 rust_xlsxwriter `set_freeze_panes` 一样返回错误。
        let freeze_panes = options
            .freeze_panes
            .or_else(|| (options.freeze_head && options.need_head).then_some((head_rows, 0)));
        if let Some((rows, cols)) = freeze_panes {
            book.sheet_mut(sheet_name).set_freeze_panes(rows, cols)?;
        }
    }

    if write_head && head_rows > 0 {
        write_biff8_headers(
            book,
            sheet_name,
            &columns,
            options,
            metadata,
            handlers,
            row_index,
            holder_scope,
        )?;
        // Annotation `@HeadRowHeight` / `SimpleRowHeightStyleStrategy`
        let head_height = collect_handler_head_row_height(handlers).or(metadata.head_row_height);
        if let Some(height) = head_height {
            let sheet = book.sheet_mut(sheet_name);
            for head_row in row_index..row_index + head_rows {
                sheet.set_row_height_at(head_row, height)?;
            }
        }
        if options.automatic_merge_head {
            let head = selected_head_paths(&columns, options)?;
            merge_biff8_dynamic_head_groups(
                book.sheet_mut(sheet_name),
                &columns,
                &head,
                row_index,
            )?;
        }
        row_index = row_index
            .checked_add(head_rows)
            .ok_or_else(|| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
    }

    let row_list: Vec<T> = rows.into_iter().collect();
    for row in row_list {
        if row.is_absent_row() {
            row_index = row_index
                .checked_add(1)
                .ok_or_else(|| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
            data_index = data_index.saturating_add(1);
            continue;
        }
        let content_height =
            collect_handler_content_row_height(handlers).or(metadata.content_row_height);
        if let Some(height) = content_height {
            book.sheet_mut(sheet_name)
                .set_row_height_at(row_index, height)?;
        }
        let (original_cells, cells) =
            convert_row_at(&row, &options.converters, sheet_name, row_index, &columns)?;
        let dynamic_columns = dynamic_columns_for_row(T::schema().is_empty(), cells.len(), options);
        let row_columns = dynamic_columns.as_deref().unwrap_or(&columns);
        let explicit_style = (!options.content_styles.is_empty())
            .then(|| &options.content_styles[data_index % options.content_styles.len()]);
        apply_biff8_loop_merges(
            book.sheet_mut(sheet_name),
            row_index,
            data_index,
            &loop_merges,
        )?;
        let row_context = WriteRowContext::new(sheet_name, row_index, Some(data_index), false);
        let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
        // 样式上下文按行构建一次：`content` 是常量构造，但移出单元格循环与
        // XLSX 路径保持一致，避免每单元格重复构造。
        let style_ctx = SheetStyleContext::content(explicit_style, metadata, global);
        begin_row_lifecycle(handlers, &row_context)?;
        for (physical_index, schema_index, column) in row_columns {
            let cell_data = cells.get(*schema_index);
            let value = cell_data.map_or(CellValue::Empty, WriteCellData::effective_value);
            let mut context =
                WriteCellContext::new(sheet_name, row_index, to_column(*physical_index)?, value)
                    .with_column(column)
                    .with_original_value(
                        original_cells
                            .get(*schema_index)
                            .unwrap_or(&CellValue::Empty)
                            .clone(),
                    )
                    .with_relative_row_index(Some(data_index));
            if let Some(scope) = holder_scope {
                context = scope.cell(context);
            }
            begin_cell_lifecycle(handlers, &mut context)?;
            finish_cell_lifecycle(handlers, &context)?;
            context.apply_cell_mutations();
            if !context.skip {
                let format_ctx = if context.ignore_fill_style {
                    style_ctx.column(column).without_fill_style()
                } else {
                    let format_ctx = style_ctx
                        .column(column)
                        .with_handler_cell(effective_handler_cell_style(handlers, &context))
                        .with_handler_font(collect_handler_write_font(handlers, &context));
                    match cell_data {
                        Some(cell) => format_ctx.with_converted_cell(cell),
                        None => format_ctx,
                    }
                };
                let cell =
                    cell_value_to_biff8_styled(&context.value, &mut book.styles, format_ctx)?;
                {
                    let mut row_creator = Biff8RowCreator {
                        sheet: book.sheet_mut(sheet_name),
                    };
                    let mut row = create_row(&mut row_creator, row_index)?;
                    let column = Biff8Sheet::column_index(*physical_index)?;
                    create_cell(&mut row, column)?.set(cell)?;
                }
                add_biff8_cell_metadata(
                    book,
                    sheet_name,
                    row_index,
                    *physical_index,
                    &context.value,
                )?;
            }
        }
        finish_row_lifecycle(handlers, &row_context)?;
        if let Some(height) = row_context.row().requested_height() {
            book.sheet_mut(sheet_name)
                .set_row_height_at(row_index, height)?;
        }
        row_index = row_index
            .checked_add(1)
            .ok_or_else(|| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
        data_index += 1;
    }
    // LongestMatch / strategy widths may update after cells (Java afterCellDispose).
    apply_biff8_handler_column_widths::<T>(book.sheet_mut(sheet_name), options, handlers)?;
    let sheet = book.sheet_mut(sheet_name);
    sheet.next_row = row_index;
    sheet.next_data_index = data_index;
    Ok(WriteProgress {
        next_row: row_index,
        next_data_index: data_index,
    })
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_biff8_headers(
    book: &mut Biff8Book,
    sheet_name: &str,
    columns: &[(usize, usize, &'static ExcelColumn)],
    options: &WriteOptions,
    metadata: &ExcelWriteMetadata,
    handlers: &mut [Box<dyn WriteHandler>],
    start_row: u32,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let global = WriteGlobalFlags::from(options);
    let style_ctx = SheetStyleContext::head(&options.head_style, metadata, global);
    let head = selected_head_paths(columns, options)?;
    let levels = head.iter().map(Vec::len).max().unwrap_or(0);
    for level in 0..levels {
        let row = start_row
            .checked_add(
                u32::try_from(level)
                    .map_err(|_| ExcelError::Format("head is too deep".to_owned()))?,
            )
            .ok_or_else(|| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
        let row_context = WriteRowContext::new(sheet_name, row, Some(level), true);
        let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
        begin_row_lifecycle(handlers, &row_context)?;
        for ((physical_index, _, column), path) in columns.iter().zip(&head) {
            write_biff8_styled_text_cell(
                book,
                sheet_name,
                row,
                *physical_index,
                normalized_head_label(path, level).to_owned(),
                column,
                Some(level),
                style_ctx.column(column),
                handlers,
                true,
                holder_scope,
            )?;
        }
        finish_row_lifecycle(handlers, &row_context)?;
        if let Some(height) = row_context.row().requested_height() {
            book.sheet_mut(sheet_name).set_row_height_at(row, height)?;
        }
    }
    Ok(())
}

// 参数与 Java BIFF8 单元格写入签名一一对应；label/format_ctx 按值传入是调用点惯例
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
// CellFormatContext 是 Java 写入上下文 1:1 映射的聚合值类型，borrow 化会牵动整条调用链。
#[allow(clippy::large_types_passed_by_value)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn write_biff8_styled_text_cell(
    book: &mut Biff8Book,
    sheet_name: &str,
    row_index: u32,
    physical_index: usize,
    label: String,
    column: &'static ExcelColumn,
    relative_row_index: Option<usize>,
    format_ctx: CellFormatContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    is_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let column_index = to_column(physical_index)?;
    let mut context = WriteCellContext::new(
        sheet_name,
        row_index,
        column_index,
        CellValue::String(label.clone()),
    )
    .with_column(column)
    .with_relative_row_index(relative_row_index);
    if is_head {
        context = context.with_head(label.clone()).without_original_value();
    }
    if let Some(scope) = holder_scope {
        context = scope.cell(context);
    }
    begin_cell_lifecycle(handlers, &mut context)?;
    finish_cell_lifecycle(handlers, &context)?;
    context.apply_cell_mutations();
    if !context.skip {
        let format_ctx = if context.ignore_fill_style {
            format_ctx.without_fill_style()
        } else {
            format_ctx
                .with_handler_cell(effective_handler_cell_style(handlers, &context))
                .with_handler_font(collect_handler_write_font(handlers, &context))
        };
        let cell = cell_value_to_biff8_styled(&context.value, &mut book.styles, format_ctx)?;
        {
            let mut row_creator = Biff8RowCreator {
                sheet: book.sheet_mut(sheet_name),
            };
            let mut row = create_row(&mut row_creator, row_index)?;
            let column = Biff8Sheet::column_index(physical_index)?;
            create_cell(&mut row, column)?.set(cell)?;
        }
        add_biff8_cell_metadata(book, sheet_name, row_index, physical_index, &context.value)?;
    }
    Ok(())
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn cell_value_to_biff8(
    value: &CellValue,
    global: WriteGlobalFlags,
) -> Result<Biff8Cell> {
    let generated = match value {
        CellValue::Empty => GeneratedBiff8CellValue::Blank,
        CellValue::String(text)
        | CellValue::Error(text)
        | CellValue::Hyperlink { text, .. }
        | CellValue::HyperlinkWithMetadata { text, .. } => GeneratedBiff8CellValue::Text(
            easyexcel_utils::string_utils::maybe_trim(text, global.auto_trim).into_owned(),
        ),
        CellValue::Formula(text) => GeneratedBiff8CellValue::Formula(text.clone()),
        CellValue::Bool(flag) => GeneratedBiff8CellValue::Bool(*flag),
        CellValue::Int(number) => {
            #[allow(clippy::cast_precision_loss)]
            GeneratedBiff8CellValue::Number(*number as f64)
        }
        CellValue::Float(number) => GeneratedBiff8CellValue::Number(*number),
        CellValue::Decimal(number) => {
            let numeric = finite_decimal_f64(number, "BIFF8")?;
            if decimal_integer_requires_text(number)? {
                GeneratedBiff8CellValue::Text(number.to_plain_string())
            } else {
                GeneratedBiff8CellValue::Number(numeric)
            }
        }
        CellValue::Date(date) => GeneratedBiff8CellValue::DateSerial(
            date_to_excel_serial_with_windowing(*date, global.use_1904_windowing),
        ),
        CellValue::DateTime(date_time) => GeneratedBiff8CellValue::DateTimeSerial(
            datetime_to_excel_serial_with_windowing(*date_time, global.use_1904_windowing),
        ),
        CellValue::Comment { value, .. }
        | CellValue::CommentWithMetadata { value, .. } => return cell_value_to_biff8(value, global),
        CellValue::Images { .. } => {
            return Err(ExcelError::Unsupported(
                "legacy XLS writing does not support CellValue::Images".to_owned(),
            ));
        }
        CellValue::RichText(rich) => GeneratedBiff8CellValue::Text(
            easyexcel_utils::string_utils::maybe_trim(rich.text_string(), global.auto_trim)
                .into_owned(),
        ),
        CellValue::Image(_) => {
            return Err(ExcelError::Unsupported(
                "legacy XLS writing does not support CellValue::Image".to_owned(),
            ));
        }
    };
    Ok(generated.into_cell())
}

fn add_biff8_cell_metadata(
    book: &mut Biff8Book,
    sheet_name: &str,
    row: u32,
    column: usize,
    value: &CellValue,
) -> Result<()> {
    match value {
        CellValue::Hyperlink { url, text } => {
            book.sheet_mut(sheet_name)
                .add_hyperlink(row, column, url.clone(), text.clone())?;
            Ok(())
        }
        CellValue::HyperlinkWithMetadata {
            address,
            text,
            hyperlink_type,
            coordinates,
        } => {
            let Some(kind) = biff8_hyperlink_kind(*hyperlink_type) else {
                return Ok(());
            };
            let first_row = resolve_hyperlink_coordinate(
                row,
                coordinates.get_first_row_index(),
                coordinates.get_relative_first_row_index(),
                "first row",
            )?;
            let last_row = resolve_hyperlink_coordinate(
                row,
                coordinates.get_last_row_index(),
                coordinates.get_relative_last_row_index(),
                "last row",
            )?;
            let current_column = u32::try_from(column).map_err(|_| {
                ExcelError::Format("BIFF8 hyperlink column exceeds u32".to_owned())
            })?;
            let first_column = resolve_hyperlink_coordinate(
                current_column,
                coordinates.get_first_column_index().map(u32::from),
                coordinates.get_relative_first_column_index(),
                "first column",
            )?;
            let last_column = resolve_hyperlink_coordinate(
                current_column,
                coordinates.get_last_column_index().map(u32::from),
                coordinates.get_relative_last_column_index(),
                "last column",
            )?;
            let address = kind.normalized_target(address);
            book.sheet_mut(sheet_name).add_typed_hyperlink(
                first_row,
                last_row,
                usize::try_from(first_column).map_err(|_| {
                    ExcelError::Format("BIFF8 hyperlink first column exceeds usize".to_owned())
                })?,
                usize::try_from(last_column).map_err(|_| {
                    ExcelError::Format("BIFF8 hyperlink last column exceeds usize".to_owned())
                })?,
                address,
                text.clone(),
                kind,
            )?;
            Ok(())
        }
        CellValue::Comment { value, text } => {
            book.sheet_mut(sheet_name)
                .add_comment(row, column, text.clone(), "easyexcel-rust")?;
            add_biff8_cell_metadata(book, sheet_name, row, column, value)
        }
        CellValue::CommentWithMetadata { value, comment } => {
            let text = comment.note_text();
            let author = comment.get_author().unwrap_or("").to_owned();
            let visible = comment.get_visible();
            if text.contains('\0') || author.contains('\0') {
                return Err(ExcelError::Format(
                    "BIFF8 comment text and author cannot contain NUL".to_owned(),
                ));
            }
            if text.encode_utf16().count() > usize::from(u16::MAX)
                || author.encode_utf16().count() > usize::from(u16::MAX)
            {
                return Err(ExcelError::Format(
                    "BIFF8 comment text or author exceeds 65535 UTF-16 units".to_owned(),
                ));
            }
            let anchor = comment.get_anchor();
            let coordinates = anchor.get_coordinates();
            let current_column = u32::try_from(column)
                .map_err(|_| ExcelError::Format("BIFF8 comment column exceeds u32".to_owned()))?;
            let first_row = resolve_hyperlink_coordinate(
                row,
                coordinates.get_first_row_index(),
                coordinates.get_relative_first_row_index(),
                "comment first row",
            )?;
            let first_column = resolve_hyperlink_coordinate(
                current_column,
                coordinates.get_first_column_index().map(u32::from),
                coordinates.get_relative_first_column_index(),
                "comment first column",
            )?;
            let last_row = resolve_hyperlink_coordinate(
                row,
                coordinates.get_last_row_index(),
                coordinates.get_relative_last_row_index(),
                "comment last row",
            )?
            .saturating_add(1);
            let last_column = resolve_hyperlink_coordinate(
                current_column,
                coordinates.get_last_column_index().map(u32::from),
                coordinates.get_relative_last_column_index(),
                "comment last column",
            )?
            .saturating_add(1);
            let formatting_runs = comment
                .get_rich_text_string_data()
                .map(|rich| resolve_biff8_rich_text_runs(rich, &mut book.styles))
                .transpose()?;
            if formatting_runs.as_ref().is_some_and(|runs| runs.len() > 8_190) {
                return Err(ExcelError::Unsupported(
                    "legacy XLS comment rich text exceeds the TXO 65535-byte formatting limit"
                        .to_owned(),
                ));
            }
            let mut comment = Biff8Comment::new(
                u16::try_from(row).map_err(|_| {
                    ExcelError::Format("BIFF8 comment row exceeds 65535".to_owned())
                })?,
                u8::try_from(column).map_err(|_| {
                    ExcelError::Format("BIFF8 comment column exceeds 255".to_owned())
                })?,
                text,
                author,
            )
            .with_anchor(
                u16::try_from(first_row).map_err(|_| {
                    ExcelError::Format("BIFF8 comment first row exceeds 65535".to_owned())
                })?,
                u8::try_from(first_column).map_err(|_| {
                    ExcelError::Format("BIFF8 comment first column exceeds 255".to_owned())
                })?,
                u16::try_from(last_row).map_err(|_| {
                    ExcelError::Format("BIFF8 comment last row exceeds 65535".to_owned())
                })?,
                u8::try_from(last_column).map_err(|_| {
                    ExcelError::Format("BIFF8 comment last column exceeds 255".to_owned())
                })?,
                anchor.get_top().map(biff8_comment_offset),
                anchor.get_right().map(biff8_comment_offset),
                anchor.get_bottom().map(biff8_comment_offset),
                anchor.get_left().map(biff8_comment_offset),
            );
            if let Some(formatting_runs) = formatting_runs {
                comment = comment.with_formatting_runs(formatting_runs);
            }
            if let Some(visible) = visible {
                comment = comment.with_visible(visible);
            }
            book.sheet_mut(sheet_name).set_comment(comment);
            // Java WriteCellData 可同时携带 comment 与 hyperlink。effective_value
            // 以包装值表达组合装饰，写完当前 NOTE/TXO/OBJ 后继续处理底层 HLINK。
            add_biff8_cell_metadata(book, sheet_name, row, column, value)
        }
        _ => Ok(()),
    }
}

fn cell_value_to_biff8_with_styles(
    value: &CellValue,
    styles: &mut Biff8StyleTable,
    global: WriteGlobalFlags,
) -> Result<Biff8Cell> {
    let mut scalar = value;
    loop {
        scalar = match scalar {
            CellValue::Comment { value, .. }
            | CellValue::CommentWithMetadata { value, .. } => value.as_ref(),
            CellValue::Images { .. } => {
                return Err(ExcelError::Unsupported(
                    "legacy XLS writing does not support CellValue::Images".to_owned(),
                ));
            }
            _ => break,
        };
    }
    if let CellValue::RichText(rich) = scalar {
        let runs = resolve_biff8_rich_text_runs(rich, styles)?;
        return Ok(GeneratedBiff8CellValue::RichText {
            // 富文本 run 使用原始 UTF-16 坐标，不能套用普通字符串 autoTrim。
            text: rich.text_string().to_owned(),
            runs,
        }
        .into_cell());
    }
    cell_value_to_biff8(value, global)
}

fn resolve_biff8_rich_text_runs(
    rich: &RichTextStringData,
    styles: &mut Biff8StyleTable,
) -> Result<Vec<(u16, u16)>> {
    let intervals = rich
        .interval_fonts()
        .iter()
        .map(|interval| (interval.start_index(), interval.end_index()))
        .collect::<Vec<_>>();
    let segments = easyexcel_model::segment_utf16_text(rich.text_string(), &intervals)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut runs = Vec::new();
    let mut utf16_start = 0usize;
    let mut previous_font = None;
    for segment in segments {
        let font = segment.interval_index.map_or(rich.write_font(), |index| {
            Some(rich.interval_fonts()[index].write_font())
        });
        let font_index = font.map_or(0, |font| {
            let mut request = Biff8StyleRequest::default();
            apply_write_font(&mut request, font);
            styles.resolve_font_index(&request)
        });
        if previous_font != Some(font_index) {
            runs.push((
                u16::try_from(utf16_start).map_err(|_| {
                    ExcelError::Unsupported(
                        "legacy XLS rich text exceeds 65535 UTF-16 units".to_owned(),
                    )
                })?,
                font_index,
            ));
            previous_font = Some(font_index);
        }
        utf16_start = utf16_start.saturating_add(segment.text.encode_utf16().count());
    }
    Ok(runs)
}

const fn biff8_comment_offset(pixels: u32) -> u16 {
    let emu = pixels.saturating_mul(9_525);
    if emu > u16::MAX as u32 { u16::MAX } else { emu as u16 }
}

const fn biff8_hyperlink_kind(value: HyperlinkType) -> Option<Biff8HyperlinkKind> {
    match value {
        HyperlinkType::None => None,
        HyperlinkType::Url => Some(Biff8HyperlinkKind::Url),
        HyperlinkType::Document => Some(Biff8HyperlinkKind::Document),
        HyperlinkType::Email => Some(Biff8HyperlinkKind::Email),
        HyperlinkType::File => Some(Biff8HyperlinkKind::File),
    }
}

fn resolve_hyperlink_coordinate(
    current: u32,
    absolute: Option<u32>,
    relative: Option<i32>,
    name: &str,
) -> Result<u32> {
    let current = i32::try_from(current)
        .map_err(|_| ExcelError::Format(format!("BIFF8 hyperlink {name} exceeds i32")))?;
    let absolute = absolute
        .map(i32::try_from)
        .transpose()
        .map_err(|_| ExcelError::Format(format!("BIFF8 hyperlink absolute {name} exceeds i32")))?;
    let resolved = crate::util::style_util::get_cell_coordinate(current, absolute, relative);
    u32::try_from(resolved)
        .map_err(|_| ExcelError::Format(format!("BIFF8 hyperlink resolved {name} is negative")))
}

// 按值传入与调用点构造惯例一致，改引用会增加不必要的借用链
#[allow(clippy::large_types_passed_by_value)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn cell_value_to_biff8_styled(
    value: &CellValue,
    styles: &mut Biff8StyleTable,
    format_ctx: CellFormatContext<'_>,
) -> Result<Biff8Cell> {
    let cell = cell_value_to_biff8_with_styles(value, styles, format_ctx.global)?;
    let request = biff8_style_request(format_ctx);
    let xf = styles.resolve_xf(&request, cell.xf);
    Ok(cell.with_xf(xf))
}

// 按值传入与调用点构造惯例一致，改引用会增加不必要的借用链
#[allow(clippy::large_types_passed_by_value)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn biff8_style_request(context: CellFormatContext<'_>) -> Biff8StyleRequest {
    let mut request = Biff8StyleRequest::default();
    let mut annotation_cell = context.converted_cell.cloned();
    if let Some(annotation_style) = context.cell {
        let annotation_style = crate::WriteCellStyle::from(annotation_style);
        annotation_cell = Some(merge_write_cell_style(
            &annotation_style,
            annotation_cell.unwrap_or_default(),
        ));
    }
    if let Some(handler_style) = context.handler_cell {
        let handler_style = crate::WriteCellStyle::from(handler_style);
        annotation_cell = Some(merge_write_cell_style(
            &handler_style,
            annotation_cell.unwrap_or_default(),
        ));
    }
    let mut font = context.font.map(write_font_from_excel_font_style);
    if let Some(style) = annotation_cell {
        if let Some(style_font) = style.font.as_ref() {
            font = Some(match font {
                Some(target) => merge_write_font(style_font, target),
                None => style_font.clone(),
            });
        }
        apply_excel_cell_style(&mut request, style.engine_cell_style());
    }
    if let Some(handler_font) = context.handler_font {
        font = Some(match font {
            Some(target) => merge_write_font(&handler_font, target),
            None => handler_font,
        });
    }
    if let Some(font) = font {
        apply_write_font(&mut request, &font);
    }
    if let Some(style) = context.explicit {
        apply_writer_cell_style_to_request(&mut request, style);
    }
    request
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_writer_cell_style_to_request(
    request: &mut Biff8StyleRequest,
    style: &CellStyle,
) {
    if style.bold {
        request.bold = true;
    }
    if style.italic {
        request.italic = true;
    }
    if let Some(color) = style.font_color {
        request.font_color = Some(Biff8Color::Rgb(color));
    }
    if let Some(color) = style.background_color {
        request.fill_pattern = Some(Biff8FillPattern::Solid);
        request.fill_foreground_color = Some(Biff8Color::Rgb(color));
    }
    if let Some(alignment) = style.horizontal_alignment {
        request.horizontal_alignment = Some(writer_horizontal_alignment(alignment));
    }
    if let Some(alignment) = style.vertical_alignment {
        request.vertical_alignment = Some(writer_vertical_alignment(alignment));
    }
    if style.wrap_text {
        request.wrap = true;
    }
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_biff8_column_widths<T>(
    sheet: &mut Biff8Sheet,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    for (column, width) in &options.column_widths {
        sheet.set_column_width_at(usize::from(*column), *width)?;
    }
    let type_width = T::write_metadata().column_width;
    for (physical_index, _, column) in selected_columns(T::schema(), options)? {
        if options
            .column_widths
            .iter()
            .any(|(explicit, _)| usize::from(*explicit) == physical_index)
        {
            continue;
        }
        if let Some(width) = column.column_width.or(type_width) {
            sheet.set_column_width_at(physical_index, width)?;
        }
    }
    apply_biff8_handler_column_widths::<T>(sheet, options, handlers)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_biff8_handler_column_widths<T>(
    sheet: &mut Biff8Sheet,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    for (physical_index, _, _) in selected_columns(T::schema(), options)? {
        if options
            .column_widths
            .iter()
            .any(|(explicit, _)| usize::from(*explicit) == physical_index)
        {
            continue;
        }
        for handler in handlers {
            if let Some(width) = handler.style_column_width(physical_index) {
                sheet.set_column_width_at(physical_index, width)?;
            }
        }
    }
    Ok(())
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_biff8_once_absolute_merges<T>(
    sheet: &mut Biff8Sheet,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    for merge in collect_once_absolute_merges::<T>(handlers) {
        apply_biff8_once_absolute_merge_property(sheet, merge)?;
    }
    Ok(())
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_biff8_once_absolute_merge_property(
    sheet: &mut Biff8Sheet,
    merge: crate::core::OnceAbsoluteMergeProperty,
) -> Result<()> {
    if merge.first_row_index < 0
        || merge.last_row_index < 0
        || merge.first_column_index < 0
        || merge.last_column_index < 0
    {
        return Ok(());
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    add_biff8_merge_range(
        sheet,
        MergeRange::new(
            merge.first_row_index as u32,
            merge.last_row_index as u32,
            merge.first_column_index as u16,
            merge.last_column_index as u16,
        ),
    )
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn add_biff8_merge_range(sheet: &mut Biff8Sheet, range: MergeRange) -> Result<()> {
    sheet.add_merge_range(range).map_err(ExcelError::from)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn merge_range_to_biff8(range: MergeRange) -> Result<Biff8Merge> {
    Biff8Merge::try_from_bounds(
        range.first_row,
        range.last_row,
        range.first_column,
        range.last_column,
    )
    .map_err(ExcelError::from)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_biff8_loop_merges(
    sheet: &mut Biff8Sheet,
    row_index: u32,
    data_index: usize,
    strategies: &[MirroredLoopMergeStrategy],
) -> Result<()> {
    for strategy in strategies {
        #[allow(clippy::cast_possible_truncation)]
        let each_rows = strategy.each_rows as usize;
        if !data_index.is_multiple_of(each_rows) {
            continue;
        }
        let last_row = row_index
            .checked_add(strategy.each_rows - 1)
            .ok_or_else(|| ExcelError::Format("loop merge row overflow".to_owned()))?;
        let last_column = strategy
            .column_index
            .checked_add(strategy.column_extend - 1)
            .ok_or_else(|| ExcelError::Format("loop merge column overflow".to_owned()))?;
        add_biff8_merge_range(
            sheet,
            MergeRange::new(row_index, last_row, strategy.column_index, last_column),
        )?;
    }
    Ok(())
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn merge_biff8_dynamic_head_groups(
    sheet: &mut Biff8Sheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
    start_row: u32,
) -> Result<()> {
    for range in dynamic_head_merge_ranges(columns, head, start_row)? {
        add_biff8_merge_range(sheet, range)?;
    }
    Ok(())
}
