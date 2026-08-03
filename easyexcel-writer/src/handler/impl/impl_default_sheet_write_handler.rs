//! 对应 Java：`com.alibaba.excel.write.handler.impl.DefaultWriteSheetHandler`.

use easyexcel_core::{Result, WriteSheetContext};

use crate::WriteHandler;

/// 对应 Java：`DefaultWriteSheetHandler`.
///
/// Tracks whether the sheet has been initialized for writing so the
/// builder can defer dimension calculation until the first row arrives.
pub struct DefaultWriteSheetHandler {
    initialized: bool,
}

impl DefaultWriteSheetHandler {
    /// Creates the handler. (Java `DefaultWriteSheetHandler()`)
    #[must_use]
    pub const fn new() -> Self {
        Self { initialized: false }
    }

    /// Returns whether the sheet has been initialized.
    #[must_use]
    pub const fn initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for DefaultWriteSheetHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteHandler for DefaultWriteSheetHandler {
    fn after_sheet(&mut self, _context: &WriteSheetContext) -> Result<()> {
        // Java: `DefaultWriteSheetHandler.afterSheetCreate` just marks
        // the sheet as initialized so subsequent rows can be appended.
        self.initialized = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_core::WriteSheetContext;

    #[test]
    fn default_sheet_write_handler_new_initialized_false() {
        let handler = DefaultWriteSheetHandler::new();
        assert!(!handler.initialized());
    }

    #[test]
    fn default_sheet_write_handler_default_impl() {
        let handler = DefaultWriteSheetHandler::default();
        assert!(!handler.initialized());
    }

    #[test]
    fn default_sheet_write_handler_after_sheet_sets_initialized() {
        let mut handler = DefaultWriteSheetHandler::new();
        assert!(!handler.initialized());
        let context = WriteSheetContext::new("Sheet1");
        handler.after_sheet(&context).unwrap();
        assert!(handler.initialized());
    }

    #[test]
    fn default_sheet_write_handler_order_is_zero() {
        let handler = DefaultWriteSheetHandler::new();
        assert_eq!(handler.order(), 0);
    }
}
