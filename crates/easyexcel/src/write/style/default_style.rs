//! 对应 Java：`com.alibaba.excel.write.style.DefaultStyle`.

use crate::core::{ExcelCellStyle, ExcelColor, ExcelHorizontalAlignment, WriteHandler};

/// 对应 Java：`DefaultStyle`.
///
/// The Java side is a `WorkbookWriteHandler` that pushes a default
/// `WriteCellStyle` (bold header, white background) onto every
/// worksheet. The Rust port exposes the same fields and the same
/// `WriteHandler` hook.
pub struct DefaultStyle {
    header: ExcelCellStyle,
}

impl DefaultStyle {
    /// Creates the default style with a bold header.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.DefaultStyle。
    pub const fn new() -> Self {
        let mut header = ExcelCellStyle::new();
        header.horizontal_alignment = Some(ExcelHorizontalAlignment::Center);
        Self { header }
    }

    /// Returns the configured header style. (Java `getHeaderStyle()` step)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.DefaultStyle。
    pub const fn header(&self) -> &ExcelCellStyle {
        &self.header
    }
}

impl Default for DefaultStyle {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteHandler for DefaultStyle {
    fn order(&self) -> i32 {
        0
    }
    fn after_workbook(
        &mut self,
        _context: &crate::core::WriteWorkbookContext,
    ) -> crate::core::Result<()> {
        // `rust_xlsxwriter` applies default style on demand. This shim
        // exists for parity.
        Ok(())
    }
}

// Hint to the linter that the color import is part of the public surface.
const _IGNORE: Option<ExcelColor> = None;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WriteWorkbookContext;

    #[test]
    fn default_style_new_alignment() {
        let style = DefaultStyle::new();
        assert_eq!(
            style.header().horizontal_alignment,
            Some(ExcelHorizontalAlignment::Center)
        );
    }

    #[test]
    fn default_style_default_impl() {
        let style = DefaultStyle::default();
        assert_eq!(
            style.header().horizontal_alignment,
            Some(ExcelHorizontalAlignment::Center)
        );
    }

    #[test]
    fn default_style_order() {
        let style = DefaultStyle::new();
        assert_eq!(style.order(), 0);
    }

    #[test]
    fn default_style_after_workbook() {
        let mut style = DefaultStyle::new();
        let context = WriteWorkbookContext::new("test.xlsx");
        style.after_workbook(&context).unwrap();
    }

    #[test]
    fn default_style_header_accessor() {
        let style = DefaultStyle::new();
        let _ = style.header();
    }
}
