/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_loop_merges(
    worksheet: &mut Worksheet,
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
        generation::merge_range(
            worksheet,
            row_index,
            strategy.column_index,
            last_row,
            last_column,
            "",
            &generation::new_format(),
        )
        .map_err(format_error)?;
    }
    Ok(())
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn sort_handlers(handlers: &mut [Box<dyn WriteHandler>]) {
    handlers.sort_by_key(|handler| handler.order());
    let mut unique_values = HashSet::new();
    for handler in handlers {
        let duplicate = handler
            .as_not_repeat_executor()
            .is_some_and(|executor| !unique_values.insert(executor.unique_value().to_owned()));
        if duplicate {
            *handler = Box::new(NoopWriteHandler);
        }
    }
}

struct NoopWriteHandler;

impl WriteHandler for NoopWriteHandler {}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn begin_row_lifecycle(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteRowContext,
) -> Result<()> {
    crate::util::write_handler_utils::before_row_create(handlers, context)?;
    crate::util::write_handler_utils::after_row_create(handlers, context)?;
    Ok(())
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn finish_row_lifecycle(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteRowContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_row_dispose(handlers, context)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn begin_cell_lifecycle(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &mut WriteCellContext,
) -> Result<()> {
    crate::util::write_handler_utils::before_cell_create(handlers, context)?;
    context.apply_cell_mutations();
    crate::util::write_handler_utils::after_cell_create(handlers, context)?;
    context.apply_cell_mutations();
    crate::util::write_handler_utils::after_cell_data_converted(handlers, context)?;
    context.apply_cell_mutations();
    context.sync_cell_handle();
    Ok(())
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn finish_cell_lifecycle(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteCellContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_cell_dispose(handlers, context)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn before_workbook(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteWorkbookContext,
) -> Result<()> {
    crate::util::write_handler_utils::before_workbook_create(handlers, context)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn after_workbook_create(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteWorkbookContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_workbook_create(handlers, context)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn after_workbook(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteWorkbookContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_workbook_dispose(handlers, context)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn run_own_workbook_callbacks(scope: &HandlerExecutionScope, path: &Path) -> Result<()> {
    let mut own = scope.own_boxed();
    let context = WriteWorkbookContext::new(path);
    before_workbook(&mut own, &context)?;
    after_workbook_create(&mut own, &context)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn before_sheet(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteSheetContext,
) -> Result<()> {
    crate::util::write_handler_utils::before_sheet_create(handlers, context)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn after_sheet_create(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteSheetContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_sheet_create(handlers, context)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn after_sheet(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteSheetContext,
) -> Result<()> {
    for handler in handlers.iter_mut() {
        handler.after_sheet_dispose(context)?;
    }
    Ok(())
}

