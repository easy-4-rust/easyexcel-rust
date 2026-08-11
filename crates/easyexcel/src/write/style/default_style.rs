//! 对应 Java：`com.alibaba.excel.write.style.DefaultStyle`.

use crate::core::{
    ExcelBorderStyle, ExcelCellStyle, ExcelColor, ExcelFillPattern, ExcelHorizontalAlignment,
    ExcelVerticalAlignment, WriteCellContext, WriteFont, WriteHandler,
};

/// 对应 Java：`DefaultStyle`.
///
/// The Java side is a `WorkbookWriteHandler` that pushes a default
/// `WriteCellStyle` (bold header, white background) onto every
/// worksheet. The Rust port exposes the same fields and the same
/// `WriteHandler` hook.
pub struct DefaultStyle {
    header: ExcelCellStyle,
    header_font: WriteFont,
}

impl DefaultStyle {
    /// Creates the default style with a bold header.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.DefaultStyle。
    pub fn new() -> Self {
        let header = ExcelCellStyle {
            locked: Some(true),
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            wrapped: Some(true),
            vertical_alignment: Some(ExcelVerticalAlignment::Center),
            border_left: Some(ExcelBorderStyle::Thin),
            border_right: Some(ExcelBorderStyle::Thin),
            border_top: Some(ExcelBorderStyle::Thin),
            border_bottom: Some(ExcelBorderStyle::Thin),
            fill_pattern: Some(ExcelFillPattern::Solid),
            // Apache POI `IndexedColors.GREY_25_PERCENT` 的稳定索引。
            fill_foreground_color: Some(ExcelColor::Indexed(22)),
            ..ExcelCellStyle::new()
        };
        let header_font = WriteFont::new()
            .font_name("宋体")
            .font_height_in_points(14.0)
            .bold(true);
        Self {
            header,
            header_font,
        }
    }

    /// Returns the configured header style. (Java `getHeaderStyle()` step)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.DefaultStyle。
    pub const fn header(&self) -> &ExcelCellStyle {
        &self.header
    }

    /// 返回 Java 默认表头字体。
    #[must_use]
    pub const fn header_font(&self) -> &WriteFont {
        &self.header_font
    }
}

impl Default for DefaultStyle {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteHandler for DefaultStyle {
    fn backend_capability(&self) -> crate::WriteHandlerCapability {
        crate::WriteHandlerCapability::StreamingSafe
    }

    fn requires_row_context(&self) -> bool {
        false
    }

    fn requires_cell_context(&self) -> bool {
        false
    }

    fn order(&self) -> i32 {
        crate::constant::order_constant::DEFAULT_DEFINE_STYLE
    }

    fn style_cell_style(&self, context: &WriteCellContext) -> Option<ExcelCellStyle> {
        context.is_head.then_some(self.header)
    }

    fn style_write_font(&self, context: &WriteCellContext) -> Option<WriteFont> {
        context.is_head.then(|| self.header_font.clone())
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
        // Java `OrderConstant.DEFAULT_DEFINE_STYLE` = -70_000
        assert_eq!(style.order(), -70_000);
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
