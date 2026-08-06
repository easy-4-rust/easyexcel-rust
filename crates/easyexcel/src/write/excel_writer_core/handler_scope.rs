#[cfg(test)]
pub(crate) fn csv_record(columns: &[(usize, usize, &'static ExcelColumn)]) -> Vec<String> {
    vec![String::new(); csv_record_width(columns)]
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn before_csv_row(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteRowContext,
) -> Result<()> {
    begin_row_lifecycle(handlers, context)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn after_csv_row(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteRowContext,
) -> Result<()> {
    finish_row_lifecycle(handlers, context)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn before_csv_cell(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &mut WriteCellContext,
) -> Result<()> {
    begin_cell_lifecycle(handlers, context)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn after_csv_cell(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &mut WriteCellContext,
) -> Result<()> {
    finish_cell_lifecycle(handlers, context)?;
    context.apply_cell_mutations();
    Ok(())
}

/// Tracks the next physical row / data-row index while appending.
///
/// Immutable Java-holder state shared by row/cell callback construction.
#[derive(Debug, Clone)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct HandlerHolderScope {
    workbook: WriteWorkbookHolderView,
    sheet_no: i32,
    table_no: Option<i32>,
    current_holder_state: WriteContextHolderState,
    mutation_plan: crate::context::write_mutation_plan::WriteMutationPlan,
}

impl HandlerHolderScope {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn new_resolved<T>(
        path: &Path,
        sheet_no: i32,
        table_no: Option<i32>,
        options: &WriteOptions,
    ) -> Result<Self>
    where
        T: ExcelRow,
    {
        Self::new_resolved_with_plan::<T>(
            path,
            sheet_no,
            table_no,
            options,
            crate::context::write_mutation_plan::WriteMutationPlan::default(),
        )
    }

    pub(crate) fn new_resolved_with_plan<T>(
        path: &Path,
        sheet_no: i32,
        table_no: Option<i32>,
        options: &WriteOptions,
        mutation_plan: crate::context::write_mutation_plan::WriteMutationPlan,
    ) -> Result<Self>
    where
        T: ExcelRow,
    {
        Ok(Self {
            workbook: WriteWorkbookHolderView::new(path),
            sheet_no,
            table_no,
            current_holder_state: resolved_write_context_holder_state::<T>(options, table_no)?,
            mutation_plan,
        })
    }

    fn row(&self, context: WriteRowContext) -> WriteRowContext {
        // 跳过临时 live_context 构造与解构——直接注入字段，省去中间全套克隆
        context.with_resolved_holder_context(
            self.workbook.clone(),
            self.sheet_no,
            self.table_no,
            self.current_holder_state.clone(),
        )
    }

    /// 每单元格注入 holder 状态。跳过临时 `live_context` 值构造与解构，
    /// 直接调用 `with_resolved_holder_context` 省去中间全套克隆。
    fn cell(&self, context: WriteCellContext) -> WriteCellContext {
        context.with_resolved_holder_context(
            self.workbook.clone(),
            self.sheet_no,
            self.table_no,
            self.current_holder_state.clone(),
        )
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn sheet(&self, context: WriteSheetContext) -> WriteSheetContext {
        context.with_resolved_holder_context(
            self.workbook.clone(),
            self.sheet_no,
            self.table_no,
            self.current_holder_state.clone(),
        ).with_mutation_plan(self.mutation_plan.clone())
    }
}
