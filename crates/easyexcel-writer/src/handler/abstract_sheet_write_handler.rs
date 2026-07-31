//! Mirrors Java `com.alibaba.excel.write.handler.AbstractSheetWriteHandler`.

use easyexcel_core::WriteHandler;

use crate::handler::sheet_write_handler::SheetWriteHandler;

/// Mirrors Java `AbstractSheetWriteHandler implements SheetWriteHandler`.
#[allow(dead_code)]
#[deprecated(note = "Use `easyexcel_core::WriteHandler` directly")]
pub struct AbstractSheetWriteHandler;

#[allow(deprecated)]
impl WriteHandler for AbstractSheetWriteHandler {
    fn order(&self) -> i32 {
        0
    }
}

#[allow(deprecated)]
impl SheetWriteHandler for AbstractSheetWriteHandler {}
