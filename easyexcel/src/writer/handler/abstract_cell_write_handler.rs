//! 对应 Java：`com.alibaba.excel.write.handler.AbstractCellWriteHandler`.

use crate::core::WriteHandler;

use crate::writer::handler::cell_write_handler::CellWriteHandler;

/// 对应 Java：`AbstractCellWriteHandler implements CellWriteHandler`.
///
/// Java declares the type as `@Deprecated`; Rust keeps the same
/// name and delegates the three callbacks to default no-ops so older
/// user code that imports it still compiles.
#[allow(dead_code)]
#[deprecated(note = "Use `crate::core::WriteHandler` directly")]
pub struct AbstractCellWriteHandler;

#[allow(deprecated)]
impl WriteHandler for AbstractCellWriteHandler {
    fn order(&self) -> i32 {
        0
    }
}

#[allow(deprecated)]
impl CellWriteHandler for AbstractCellWriteHandler {
    // All three callbacks remain no-ops — the trait provides sensible
    // defaults; we just need a concrete type for the deprecated shim.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn abstract_cell_write_handler_order_is_zero() {
        let handler = AbstractCellWriteHandler;
        assert_eq!(handler.order(), 0);
    }

    #[test]
    #[allow(deprecated)]
    fn abstract_cell_write_handler_unit_construction() {
        let handler = AbstractCellWriteHandler;
        let _: AbstractCellWriteHandler = handler;
    }
}
