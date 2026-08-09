//! 对应 Java：`com.alibaba.excel.write.handler.context.SheetWriteHandlerContext`.

use crate::WriteContext;
use crate::{ChartMutation, Result};

use super::write_mutation_plan::WriteMutationPlan;

/// Worksheet-level write lifecycle context.
///
/// 对应 Java：`SheetWriteHandlerContext` (`writeSheetHolder.getSheetName()`).
#[derive(Debug, Clone, PartialEq)]
pub struct WriteSheetContext {
    sheet_name: String,
    holders: WriteHolderContext,
    mutations: WriteMutationPlan,
}

impl WriteSheetContext {
    /// Returns this backend-neutral sheet context.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。
    pub const fn sheet(&self) -> &Self {
        self
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。 Creates a worksheet context.
    #[must_use]
    pub fn new(sheet_name: impl Into<String>) -> Self {
        let sheet_name = sheet_name.into();
        Self {
            holders: WriteHolderContext::new().with_sheet(WriteSheetHolderView::new(&sheet_name)),
            sheet_name,
            mutations: WriteMutationPlan::default(),
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。 Creates a sheet callback context from a live [`WriteContext`].
    ///
    /// Returns `None` before the context has selected a sheet.
    #[must_use]
    pub fn from_write_context(context: &dyn WriteContext) -> Option<Self> {
        let holder = context.current_write_holder();
        let sheet_name = holder.sheet_name()?.to_owned();
        Some(Self {
            holders: WriteHolderContext::from_write_context(context)
                .with_callback_sheet(&sheet_name, holder.last_row_index()),
            sheet_name,
            mutations: WriteMutationPlan::default(),
        })
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。 Replaces compatibility holder data with a live-context snapshot.
    #[must_use]
    pub fn with_write_context(mut self, context: &dyn WriteContext) -> Self {
        self.holders = WriteHolderContext::from_write_context(context)
            .with_callback_sheet(&self.sheet_name, None);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。 Returns the worksheet name. (Java `WriteSheetHolder.getSheetName()`)
    #[must_use]
    pub fn sheet_name(&self) -> &str {
        &self.sheet_name
    }
    /// 返回工作表名称。
    #[must_use] pub fn get_sheet_name(&self) -> &str { self.sheet_name() }

    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。 Attaches the workbook, resolved sheet number, and optional table.
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

    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。 Attaches all holder views and the resolved Java `currentWriteHolder()` state.
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
        let sheet = WriteSheetHolderView::new(&self.sheet_name).with_sheet_no(sheet_no);
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

    /// Returns the active workbook holder view, when the writer supplied one.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。
    pub const fn write_workbook_holder(&self) -> Option<&WriteWorkbookHolderView> {
        self.holders.workbook()
    }
    #[must_use] pub const fn get_write_workbook_holder(&self) -> Option<&WriteWorkbookHolderView> {
        self.write_workbook_holder()
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。 Returns the active sheet holder view.
    ///
    /// # Panics
    ///
    /// Panics when the callback was created without a sheet holder
    /// (sheet callbacks always carry one).
    #[must_use]
    pub fn write_sheet_holder(&self) -> &WriteSheetHolderView {
        self.holders
            .sheet()
            .expect("sheet contexts always carry a sheet holder")
    }
    #[must_use] pub fn get_write_sheet_holder(&self) -> &WriteSheetHolderView {
        self.write_sheet_holder()
    }

    /// Returns the active table holder view for table callbacks.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。
    pub const fn write_table_holder(&self) -> Option<&WriteTableHolderView> {
        self.holders.table()
    }
    #[must_use] pub const fn get_write_table_holder(&self) -> Option<&WriteTableHolderView> {
        self.write_table_holder()
    }

    /// Returns all holder views captured for this callback.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.SheetWriteHandlerContext。
    pub const fn write_context(&self) -> &WriteHolderContext {
        &self.holders
    }
    #[must_use] pub const fn get_write_context(&self) -> &WriteHolderContext {
        self.write_context()
    }

    /// 替换全部 holder 视图。
    pub fn set_write_context(&mut self, value: WriteHolderContext) { self.holders = value; }
    /// 替换 workbook holder 视图。
    pub fn set_write_workbook_holder(&mut self, value: WriteWorkbookHolderView) {
        self.holders = std::mem::take(&mut self.holders).with_workbook(value);
    }
    /// 替换 sheet holder 视图。
    pub fn set_write_sheet_holder(&mut self, value: WriteSheetHolderView) {
        self.sheet_name = value.sheet_name().to_owned();
        self.holders = std::mem::take(&mut self.holders).with_sheet(value);
    }

    /// 请求在保存前使用密码保护当前工作表。
    ///
    /// 对应 Java：`SheetWriteHandlerContext#getWriteSheetHolder().getSheet().protectSheet`。
    ///
    /// # Errors
    ///
    /// 当共享修改计划不可用时返回错误。
    pub fn protect_sheet(&self, password: impl Into<String>) -> Result<()> {
        self.mutations
            .protect_sheet(self.sheet_name.clone(), password)
    }

    /// 请求在保存前向当前工作表添加图表。
    ///
    /// 传入对象的 `sheet_name` 必须与当前回调工作表一致，避免 Handler
    /// 无意修改其他 Sheet。
    ///
    /// # Errors
    ///
    /// 工作表名称不一致或共享修改计划不可用时返回错误。
    pub fn add_chart(&self, chart: ChartMutation) -> Result<()> {
        if chart.sheet_name != self.sheet_name {
            return Err(crate::ExcelError::Format(format!(
                "chart sheet '{}' does not match handler sheet '{}'",
                chart.sheet_name, self.sheet_name
            )));
        }
        self.mutations.add_chart(chart)
    }

    /// 请求在保存前删除当前工作表中指定单元格的批注。
    ///
    /// 对应 Java：`HSSFCell/XSSFCell#removeCellComment()`。
    pub fn remove_comment(&self, row_index: u32, column_index: u16) -> Result<()> {
        self.mutations
            .remove_comment(self.sheet_name.clone(), row_index, column_index)
    }

    pub(crate) fn with_mutation_plan(mut self, mutations: WriteMutationPlan) -> Self {
        self.mutations = mutations;
        self
    }
}
use crate::{
    WriteHolderContext, WriteSheetHolderView, WriteTableHolderView, WriteWorkbookHolderView,
};

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::write::write_context::WriteContextHolder;

    #[test]
    fn sheet_accessor_returns_self() {
        // 对应 Java：SheetWriteHandlerContext 的 sheet 便捷访问器
        let context = WriteSheetContext::new("Sheet1");
        assert!(std::ptr::eq(context.sheet(), &raw const context));
    }

    #[test]
    fn with_holder_context_resolves_holder_views() {
        // 对应 Java：挂载 workbook/sheet/table holder 视图
        let context = WriteSheetContext::new("Sheet1").with_holder_context(
            WriteWorkbookHolderView::new("out.xlsx"),
            2,
            Some(5),
        );
        assert_eq!(
            context
                .write_workbook_holder()
                .map(WriteWorkbookHolderView::path),
            Some(std::path::Path::new("out.xlsx"))
        );
        assert_eq!(context.write_sheet_holder().sheet_no(), Some(2));
        assert_eq!(
            context
                .write_table_holder()
                .map(WriteTableHolderView::table_no),
            Some(5)
        );
        assert_eq!(context.write_context().holder_type(), crate::Holder::Table);

        // 无 table 时为 Sheet
        let plain = WriteSheetContext::new("Sheet1").with_holder_context(
            WriteWorkbookHolderView::new("out.xlsx"),
            2,
            None,
        );
        assert_eq!(plain.write_context().holder_type(), crate::Holder::Sheet);
        assert!(plain.write_table_holder().is_none());
    }
}
