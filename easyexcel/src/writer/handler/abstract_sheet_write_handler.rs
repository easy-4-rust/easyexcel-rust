//! 对应 Java：`com.alibaba.excel.write.handler.AbstractSheetWriteHandler`.

use crate::core::WriteHandler;

use crate::writer::handler::sheet_write_handler::SheetWriteHandler;

/// 对应 Java：`AbstractSheetWriteHandler implements SheetWriteHandler`.
#[allow(dead_code)]
#[deprecated(note = "Use `crate::core::WriteHandler` directly")]
pub struct AbstractSheetWriteHandler;

#[allow(deprecated)]
impl WriteHandler for AbstractSheetWriteHandler {
    fn order(&self) -> i32 {
        0
    }
}

#[allow(deprecated)]
impl SheetWriteHandler for AbstractSheetWriteHandler {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn abstract_sheet_write_handler_order_is_zero() {
        let handler = AbstractSheetWriteHandler;
        assert_eq!(handler.order(), 0);
    }

    #[test]
    #[allow(deprecated)]
    fn abstract_sheet_write_handler_unit_construction() {
        let handler = AbstractSheetWriteHandler;
        let _: AbstractSheetWriteHandler = handler;
    }
}
