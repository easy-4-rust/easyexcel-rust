//! 对应 Java：`com.alibaba.excel.write.handler.AbstractRowWriteHandler`.

use crate::core::WriteHandler;

use crate::write::handler::row_write_handler::RowWriteHandler;

/// 对应 Java：`AbstractRowWriteHandler implements RowWriteHandler`.
#[allow(dead_code)]
#[deprecated(note = "Use `crate::core::WriteHandler` directly")]
pub struct AbstractRowWriteHandler;

#[allow(deprecated)]
impl WriteHandler for AbstractRowWriteHandler {
    fn order(&self) -> i32 {
        0
    }
}

#[allow(deprecated)]
impl RowWriteHandler for AbstractRowWriteHandler {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn abstract_row_write_handler_order_is_zero() {
        let handler = AbstractRowWriteHandler;
        assert_eq!(handler.order(), 0);
    }

    #[test]
    #[allow(deprecated)]
    fn abstract_row_write_handler_unit_construction() {
        let handler = AbstractRowWriteHandler;
        let _: AbstractRowWriteHandler = handler;
    }
}
