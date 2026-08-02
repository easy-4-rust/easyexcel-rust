//! Mirrors Java `com.alibaba.excel.write.handler.AbstractWorkbookWriteHandler`.

use easyexcel_core::WriteHandler;

use crate::handler::workbook_write_handler::WorkbookWriteHandler;

/// Mirrors Java `AbstractWorkbookWriteHandler implements WorkbookWriteHandler`.
#[allow(dead_code)]
#[deprecated(note = "Use `easyexcel_core::WriteHandler` directly")]
pub struct AbstractWorkbookWriteHandler;

#[allow(deprecated)]
impl WriteHandler for AbstractWorkbookWriteHandler {
    fn order(&self) -> i32 {
        0
    }
}

#[allow(deprecated)]
impl WorkbookWriteHandler for AbstractWorkbookWriteHandler {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn abstract_workbook_write_handler_order_is_zero() {
        let handler = AbstractWorkbookWriteHandler;
        assert_eq!(handler.order(), 0);
    }

    #[test]
    #[allow(deprecated)]
    fn abstract_workbook_write_handler_unit_construction() {
        let handler = AbstractWorkbookWriteHandler;
        let _: AbstractWorkbookWriteHandler = handler;
    }
}
