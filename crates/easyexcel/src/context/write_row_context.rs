//! 对应 Java：`com.alibaba.excel.write.handler.context.RowWriteHandlerContext`.

/// Row-level write lifecycle context.
///
/// 对应 Java：`RowWriteHandlerContext` (`writeSheetHolder`, `writeTableHolder`,
/// `rowIndex`, `relativeRowIndex`, `head`). Rust keeps only the fields a
/// handler needs and drops the `Row` POI object because `rust_xlsxwriter`
/// does not expose it for handler interception.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteRowContext {
    /// Worksheet name.
    pub sheet_name: String,
    /// Physical zero-based row index.
    pub row_index: u32,
    /// Relative index within the current head or content block.
    ///
    /// 对应 Java：`RowWriteHandlerContext.relativeRowIndex`.
    pub relative_row_index: Option<usize>,
    /// Whether this is a header row.
    pub is_head: bool,
    row: WriteRowHandle,
    holders: WriteHolderContext,
}

impl WriteRowContext {
    /// Returns the mutable logical row handle.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。
    pub const fn row(&self) -> &WriteRowHandle {
        &self.row
    }
    #[must_use]
    pub const fn get_row(&self) -> &WriteRowHandle {
        self.row()
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。 Creates a row handler context.
    #[must_use]
    pub fn new(
        sheet_name: impl Into<String>,
        row_index: u32,
        relative_row_index: Option<usize>,
        is_head: bool,
    ) -> Self {
        let sheet_name = sheet_name.into();
        Self {
            row: WriteRowHandle::new(row_index),
            holders: WriteHolderContext::new()
                .with_sheet(WriteSheetHolderView::new(&sheet_name).with_last_row_index(row_index)),
            sheet_name,
            row_index,
            relative_row_index,
            is_head,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。 Attaches the real writer holder state visible for this row callback.
    #[must_use]
    pub fn with_holder_context(
        mut self,
        workbook: WriteWorkbookHolderView,
        sheet_no: i32,
        table_no: Option<i32>,
    ) -> Self {
        let holder_type = if table_no.is_some() {
            crate::Holder::Table
        } else {
            crate::Holder::Sheet
        };
        self = self.with_resolved_holder_context(
            workbook,
            sheet_no,
            table_no,
            crate::WriteContextHolderState {
                holder_type,
                ..crate::WriteContextHolderState::default()
            },
        );
        self
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。 Replaces compatibility holder data with a live-context snapshot.
    #[must_use]
    pub fn with_write_context(mut self, context: &dyn WriteContext) -> Self {
        self.holders = WriteHolderContext::from_write_context(context)
            .with_callback_sheet(&self.sheet_name, Some(self.row_index));
        self
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。 Attaches all holder views and the resolved Java `currentWriteHolder()` state.
    #[must_use]
    pub fn with_resolved_holder_context(
        self,
        workbook: WriteWorkbookHolderView,
        sheet_no: i32,
        table_no: Option<i32>,
        current_holder_state: crate::WriteContextHolderState,
    ) -> Self {
        self.with_shared_resolved_holder_context(
            workbook,
            sheet_no,
            table_no,
            std::sync::Arc::new(current_holder_state),
        )
    }

    pub(crate) fn with_shared_resolved_holder_context(
        mut self,
        workbook: WriteWorkbookHolderView,
        sheet_no: i32,
        table_no: Option<i32>,
        current_holder_state: std::sync::Arc<crate::WriteContextHolderState>,
    ) -> Self {
        let sheet = WriteSheetHolderView::new(&self.sheet_name)
            .with_sheet_no(sheet_no)
            .with_last_row_index(self.row_index);
        self.holders = WriteHolderContext::new()
            .with_workbook(workbook)
            .with_sheet(sheet)
            .with_shared_current_holder_state(current_holder_state);
        if let Some(table_no) = table_no {
            self.holders = self
                .holders
                .with_table(WriteTableHolderView::new(table_no, &self.sheet_name));
        }
        self
    }

    /// Returns the active workbook holder view, when supplied by the writer.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。
    pub const fn write_workbook_holder(&self) -> Option<&WriteWorkbookHolderView> {
        self.holders.workbook()
    }
    #[must_use]
    pub const fn get_write_workbook_holder(&self) -> Option<&WriteWorkbookHolderView> {
        self.write_workbook_holder()
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。 Returns the active sheet holder view.
    ///
    /// # Panics
    ///
    /// Panics when the callback was created without a sheet holder
    /// (row callbacks always carry one).
    #[must_use]
    pub fn write_sheet_holder(&self) -> &WriteSheetHolderView {
        self.holders
            .sheet()
            .expect("row contexts always carry a sheet holder")
    }
    #[must_use]
    pub fn get_write_sheet_holder(&self) -> &WriteSheetHolderView {
        self.write_sheet_holder()
    }

    /// Returns the active table holder view for table callbacks.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。
    pub const fn write_table_holder(&self) -> Option<&WriteTableHolderView> {
        self.holders.table()
    }
    #[must_use]
    pub const fn get_write_table_holder(&self) -> Option<&WriteTableHolderView> {
        self.write_table_holder()
    }

    /// Returns all holder views captured for this callback.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.RowWriteHandlerContext。
    pub const fn write_context(&self) -> &WriteHolderContext {
        &self.holders
    }
    #[must_use]
    pub const fn get_write_context(&self) -> &WriteHolderContext {
        self.write_context()
    }

    /// 返回物理行号。
    #[must_use]
    pub const fn get_row_index(&self) -> u32 {
        self.row_index
    }
    /// 设置物理行号并重建逻辑行句柄。
    pub fn set_row_index(&mut self, value: u32) {
        self.row_index = value;
        self.row = WriteRowHandle::new(value);
    }
    /// 替换逻辑行句柄。
    pub fn set_row(&mut self, row: WriteRowHandle) {
        self.row_index = row.row_index();
        self.row = row;
    }
    /// 返回相对行号。
    #[must_use]
    pub const fn relative_row_index(&self) -> Option<usize> {
        self.relative_row_index
    }
    #[must_use]
    pub const fn get_relative_row_index(&self) -> Option<usize> {
        self.relative_row_index()
    }
    /// 设置相对行号。
    pub const fn set_relative_row_index(&mut self, value: Option<usize>) {
        self.relative_row_index = value;
    }
    /// 返回表头标志。
    #[must_use]
    pub const fn head(&self) -> bool {
        self.is_head
    }
    #[must_use]
    pub const fn get_head(&self) -> bool {
        self.head()
    }
    /// 设置表头标志。
    pub const fn set_head(&mut self, value: bool) {
        self.is_head = value;
    }
    /// 替换全部 holder 视图。
    pub fn set_write_context(&mut self, value: WriteHolderContext) {
        self.holders = value;
    }
    /// 替换 workbook holder 视图。
    pub fn set_write_workbook_holder(&mut self, value: WriteWorkbookHolderView) {
        self.holders = std::mem::take(&mut self.holders).with_workbook(value);
    }
    /// 替换 sheet holder 视图。
    pub fn set_write_sheet_holder(&mut self, value: WriteSheetHolderView) {
        self.holders = std::mem::take(&mut self.holders).with_sheet(value);
    }
    /// 替换 table holder 视图。
    pub fn set_write_table_holder(&mut self, value: WriteTableHolderView) {
        self.holders = std::mem::take(&mut self.holders).with_table(value);
    }
}
use crate::{
    WriteContext, WriteHolderContext, WriteRowHandle, WriteSheetHolderView, WriteTableHolderView,
    WriteWorkbookHolderView,
};

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::WriteContextHolder;

    #[test]
    fn with_holder_context_sheet_branch() {
        // 对应 Java：无 table 时为 Sheet holder
        let context = WriteRowContext::new("Sheet1", 0, None, false).with_holder_context(
            WriteWorkbookHolderView::new("out.xlsx"),
            1,
            None,
        );
        assert_eq!(context.write_context().holder_type(), crate::Holder::Sheet);
        assert_eq!(context.write_context().sheet_no(), Some(1));
        assert_eq!(context.row().row_index(), 0);
    }
}
