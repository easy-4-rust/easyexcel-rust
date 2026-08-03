//! 对应 Java：`com.alibaba.excel.write.handler.impl.DefaultRowWriteHandler`.

use crate::WriteHandler;
use crate::core::WriteContext;
use crate::core::WriteSheetContext;

/// 对应 Java：`DefaultRowWriteHandler extends AbstractRowWriteHandler`.
///
/// Java's handler simply hooks `beforeSheetCreate` to freeze the first
/// row, which `rust_xlsxwriter` does automatically via
/// `worksheet.set_freeze_panes(...)`. The Rust shim is preserved so
/// 1:1 code references resolve.
pub struct DefaultRowWriteHandler {
    frozen: bool,
}

impl DefaultRowWriteHandler {
    /// Creates the handler.
    #[must_use]
    pub const fn new() -> Self {
        Self { frozen: false }
    }

    /// Returns whether the first row is frozen. (Java `getFreeze()` equivalent)
    #[must_use]
    pub const fn frozen(&self) -> bool {
        self.frozen
    }
}

impl Default for DefaultRowWriteHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteHandler for DefaultRowWriteHandler {
    fn before_sheet(&mut self, _context: &WriteSheetContext) -> crate::core::Result<()> {
        // The actual freeze is performed in [`crate::ExcelWriter::write`]
        // by inspecting `WriteOptions.freeze_head`.
        self.frozen = true;
        Ok(())
    }
}

/// Mirrors the Java constructor pattern that received a
/// `WriteContext` for back-reference. Kept for parity.
pub fn new_default_row_write_handler(_ctx: &dyn WriteContext) -> DefaultRowWriteHandler {
    DefaultRowWriteHandler::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WriteSheetContext;

    #[test]
    fn default_row_write_handler_new_not_frozen() {
        let handler = DefaultRowWriteHandler::new();
        assert!(!handler.frozen());
    }

    #[test]
    fn default_row_write_handler_default_not_frozen() {
        let handler = DefaultRowWriteHandler::default();
        assert!(!handler.frozen());
    }

    #[test]
    fn default_row_write_handler_before_sheet_freezes() {
        let mut handler = DefaultRowWriteHandler::new();
        let context = WriteSheetContext::new("Sheet1");
        handler.before_sheet(&context).unwrap();
        assert!(handler.frozen());
    }

    #[test]
    fn default_row_write_handler_order_is_zero() {
        let handler = DefaultRowWriteHandler::new();
        assert_eq!(handler.order(), 0);
    }

    #[test]
    fn new_default_row_write_handler_factory() {
        use crate::core::ConverterRegistry;
        use crate::core::ExcelWriteHeadProperty;
        use crate::core::Holder;
        use crate::core::WriteContext;
        use crate::core::WriteContextHolder;

        struct TestCtx;
        impl WriteContext for TestCtx {
            fn current_write_holder(&self) -> &dyn crate::core::WriteContextHolder {
                self
            }
        }
        impl crate::core::WriteContextHolder for TestCtx {
            fn path(&self) -> &std::path::Path {
                std::path::Path::new("/tmp/x")
            }
            fn holder_type(&self) -> Holder {
                Holder::Workbook
            }
            fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty {
                static P: std::sync::OnceLock<ExcelWriteHeadProperty> = std::sync::OnceLock::new();
                P.get_or_init(ExcelWriteHeadProperty::new)
            }
            fn converter_map(&self) -> &ConverterRegistry {
                static R: std::sync::OnceLock<ConverterRegistry> = std::sync::OnceLock::new();
                R.get_or_init(ConverterRegistry::default)
            }
            fn need_head(&self) -> bool {
                true
            }
            fn automatic_merge_head(&self) -> bool {
                true
            }
            fn relative_head_row_index(&self) -> i32 {
                0
            }
            fn order_by_include_column(&self) -> bool {
                false
            }
            fn include_column_indexes(&self) -> Option<&[usize]> {
                None
            }
            fn include_column_field_names(&self) -> Option<&[String]> {
                None
            }
            fn exclude_column_indexes(&self) -> &[usize] {
                &[]
            }
            fn exclude_column_field_names(&self) -> &[String] {
                &[]
            }
        }
        let ctx = TestCtx;
        let _ = ctx.current_write_holder();
        let _ = ctx.path();
        let _ = ctx.holder_type();
        let _ = ctx.excel_write_head_property();
        let _ = ctx.converter_map();
        let _ = ctx.need_head();
        let _ = ctx.automatic_merge_head();
        let _ = ctx.relative_head_row_index();
        let _ = ctx.order_by_include_column();
        let _ = ctx.include_column_indexes();
        let _ = ctx.include_column_field_names();
        let _ = ctx.exclude_column_indexes();
        let _ = ctx.exclude_column_field_names();
        let handler = new_default_row_write_handler(&ctx);
        assert!(!handler.frozen());
    }
}
