//! 对应 Java：`com.alibaba.excel.write.handler.impl.DimensionWorkbookWriteHandler`.

use crate::core::WriteWorkbookContext;

/// 对应 Java：`DimensionWorkbookWriteHandler implements WorkbookWriteHandler`.
///
/// Java's handler fixes the `<dimension ref="A1:..."/>` field on
/// `SXSSFWorkbook` because POI's streaming writer skips it. The Rust
/// port delegates this to `rust_xlsxwriter` which always sets the
/// dimension when saving; this marker type exists for parity.
pub struct DimensionWorkbookWriteHandler {
    last_ref: Option<String>,
}

impl DimensionWorkbookWriteHandler {
    /// Creates the handler.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.impl.DimensionWorkbookWriteHandler。
    pub const fn new() -> Self {
        Self { last_ref: None }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.impl.DimensionWorkbookWriteHandler。 Returns the last written dimension reference. (Java `getDimension()` step)
    #[must_use]
    pub fn last_ref(&self) -> Option<&str> {
        self.last_ref.as_deref()
    }
}

impl Default for DimensionWorkbookWriteHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::core::WriteHandler for DimensionWorkbookWriteHandler {
    fn after_workbook(&mut self, context: &WriteWorkbookContext) -> crate::core::Result<()> {
        // `rust_xlsxwriter` writes the dimension automatically based on
        // the worksheet bounds. The shim records the path for parity.
        self.last_ref = Some(context.path().display().to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_workbook_write_handler_new_no_ref() {
        let handler = DimensionWorkbookWriteHandler::new();
        assert!(handler.last_ref().is_none());
    }

    #[test]
    fn dimension_workbook_write_handler_default_no_ref() {
        let handler = DimensionWorkbookWriteHandler::default();
        assert!(handler.last_ref().is_none());
    }

    #[test]
    fn dimension_workbook_write_handler_after_workbook_sets_ref() {
        use crate::core::WriteHandler;
        let mut handler = DimensionWorkbookWriteHandler::new();
        let context = WriteWorkbookContext::new("/tmp/wb.xlsx");
        handler.after_workbook(&context).unwrap();
        assert_eq!(handler.last_ref(), Some("/tmp/wb.xlsx"));
    }

    #[test]
    fn dimension_workbook_write_handler_order_is_zero() {
        use crate::core::WriteHandler;
        let handler = DimensionWorkbookWriteHandler::new();
        assert_eq!(handler.order(), 0);
    }
}
