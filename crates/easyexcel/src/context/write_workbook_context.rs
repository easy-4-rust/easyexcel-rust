//! 对应 Java：`com.alibaba.excel.write.handler.context.WorkbookWriteHandlerContext`.

use std::path::{Path, PathBuf};

use crate::{CellValue, ChartMutation, Result};
use crate::{WriteContext, WriteHolderContext, WriteWorkbookHolderView};

use super::write_mutation_plan::WriteMutationPlan;

/// Workbook-level write lifecycle context.
///
/// 对应 Java：`WorkbookWriteHandlerContext` (`writeContext`,
/// `writeWorkbookHolder`). Rust collapses it to the logical path because the
/// `rust_xlsxwriter::Workbook` is held privately by the [`crate::ExcelWriter`].
#[derive(Debug, Clone, PartialEq)]
pub struct WriteWorkbookContext {
    path: PathBuf,
    holders: WriteHolderContext,
    mutations: WriteMutationPlan,
}

impl WriteWorkbookContext {
    /// Returns this backend-neutral workbook context.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.WorkbookWriteHandlerContext。
    pub const fn workbook(&self) -> &Self {
        self
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.WorkbookWriteHandlerContext。 Creates a workbook context for an output path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            holders: WriteHolderContext::new().with_workbook(WriteWorkbookHolderView::new(&path)),
            path,
            mutations: WriteMutationPlan::default(),
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.WorkbookWriteHandlerContext。 Creates the Java callback context from a live [`WriteContext`].
    #[must_use]
    pub fn from_write_context(context: &dyn WriteContext) -> Self {
        let holders = WriteHolderContext::from_write_context(context);
        let path = holders.current_write_holder().path().to_path_buf();
        Self {
            path,
            holders,
            mutations: WriteMutationPlan::default(),
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.WorkbookWriteHandlerContext。 Returns the output path. (Java `WriteWorkbookHolder.getFile()`)
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 对应 Java：com.alibaba.excel.write.handler.context.WorkbookWriteHandlerContext。 Returns the live workbook holder view carried by this callback.
    ///
    /// # Panics
    ///
    /// Panics when the callback was created without a workbook holder
    /// (workbook callbacks always carry one).
    #[must_use]
    pub fn write_workbook_holder(&self) -> &WriteWorkbookHolderView {
        self.holders
            .workbook()
            .expect("workbook contexts always carry a workbook holder")
    }

    /// Returns all holder views captured for this callback.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.context.WorkbookWriteHandlerContext。
    pub const fn write_context(&self) -> &WriteHolderContext {
        &self.holders
    }

    /// 请求在保存前设置指定工作表的单元格值。
    ///
    /// 对应 Java：`WorkbookWriteHandlerContext#getWriteWorkbookHolder().getWorkbook()`
    /// 后对 `Sheet` / `Row` / `Cell` 的修改。
    ///
    /// # Errors
    ///
    /// 当共享修改计划不可用时返回错误。
    pub fn set_cell(
        &self,
        sheet_name: impl Into<String>,
        row_index: u32,
        column_index: u16,
        value: CellValue,
    ) -> Result<()> {
        self.mutations
            .set_cell(sheet_name, row_index, column_index, value)
    }

    /// 请求在保存前创建一个图表。
    ///
    /// 对应 Java：通过 `WorkbookWriteHandlerContext` 获取 POI 工作簿后调用
    /// `Drawing#createChart(ClientAnchor)`。
    ///
    /// # Errors
    ///
    /// 当共享修改计划不可用时返回错误。
    pub fn add_chart(&self, chart: ChartMutation) -> Result<()> {
        self.mutations.add_chart(chart)
    }

    pub(crate) const fn mutation_plan(&self) -> &WriteMutationPlan {
        &self.mutations
    }

    pub(crate) fn with_mutation_plan(mut self, mutations: WriteMutationPlan) -> Self {
        self.mutations = mutations;
        self
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn path_and_workbook_accessors() {
        // 对应 Java：WorkbookWriteHandlerContext 路径访问器
        let context = WriteWorkbookContext::new("out.xlsx");
        assert_eq!(context.path(), Path::new("out.xlsx"));
        assert!(std::ptr::eq(context.workbook(), &raw const context));
        assert_eq!(
            context.write_workbook_holder().path(),
            Path::new("out.xlsx")
        );
    }

    #[test]
    fn from_write_context_captures_path() {
        // 对应 Java：从 live WriteContext 创建回调上下文
        let live = crate::WriteContextImpl::new("live.xlsx");
        let context = WriteWorkbookContext::from_write_context(&live);
        assert_eq!(context.path(), Path::new("live.xlsx"));
    }
}
