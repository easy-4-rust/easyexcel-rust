//! 对应 Java：`com.alibaba.excel.write.handler.context.WorkbookWriteHandlerContext`.

use std::path::{Path, PathBuf};

use crate::{WriteContext, WriteHolderContext, WriteWorkbookHolderView};

/// Workbook-level write lifecycle context.
///
/// 对应 Java：`WorkbookWriteHandlerContext` (`writeContext`,
/// `writeWorkbookHolder`). Rust collapses it to the logical path because the
/// `rust_xlsxwriter::Workbook` is held privately by the [`crate::ExcelWriter`].
#[derive(Debug, Clone, PartialEq)]
pub struct WriteWorkbookContext {
    path: PathBuf,
    holders: WriteHolderContext,
}

impl WriteWorkbookContext {
    /// Returns this backend-neutral workbook context.
    #[must_use]
    pub const fn workbook(&self) -> &Self {
        self
    }

    /// Creates a workbook context for an output path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            holders: WriteHolderContext::new().with_workbook(WriteWorkbookHolderView::new(&path)),
            path,
        }
    }

    /// Creates the Java callback context from a live [`WriteContext`].
    #[must_use]
    pub fn from_write_context(context: &dyn WriteContext) -> Self {
        let holders = WriteHolderContext::from_write_context(context);
        let path = holders.current_write_holder().path().to_path_buf();
        Self { path, holders }
    }

    /// Returns the output path. (Java `WriteWorkbookHolder.getFile()`)
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the live workbook holder view carried by this callback.
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
    pub const fn write_context(&self) -> &WriteHolderContext {
        &self.holders
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
