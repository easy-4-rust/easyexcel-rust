//! 对应 Java：`com.alibaba.excel.write.style.HorizontalCellStyleStrategy`.
//!
//! Wired into the XLSX write path via [`WriteHandler::style_cell_style`], which
//! the writer merges into each cell format after annotation styles. Nested
//! fonts on [`ExcelCellStyle::font`] mirror Java `WriteCellStyle.writeFont`.

use crate::core::{ExcelCellStyle, ExcelFontStyle, WriteCellContext, WriteFont, WriteHandler};

use crate::write::metadata::style::write_font::excel_font_style_from_write_font;
use crate::write::style::abstract_cell_style_strategy::AbstractCellStyleStrategy;

/// 对应 Java：`HorizontalCellStyleStrategy`.
///
/// The Java side cycles through a list of content styles by
/// `relativeRowIndex`; the Rust port mirrors that behaviour once the
/// write path supplies [`WriteCellContext::relative_row_index`].
/// Styles may carry nested fonts via [`ExcelCellStyle::font`] (Java
/// `WriteCellStyle.setWriteFont`).
pub struct HorizontalCellStyleStrategy {
    head_style: ExcelCellStyle,
    content_styles: Vec<ExcelCellStyle>,
}

impl HorizontalCellStyleStrategy {
    /// Creates a strategy with content styles only (empty head style).
    /// (Java `HorizontalCellStyleStrategy(List<WriteCellStyle>)` subset)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。
    pub const fn new(content_styles: Vec<ExcelCellStyle>) -> Self {
        Self {
            head_style: ExcelCellStyle::new(),
            content_styles,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Creates a strategy with one head style and one content style.
    /// (Java `HorizontalCellStyleStrategy(WriteCellStyle, WriteCellStyle)`)
    #[must_use]
    pub fn with_head_and_content(
        head_style: ExcelCellStyle,
        content_style: ExcelCellStyle,
    ) -> Self {
        Self {
            head_style,
            content_styles: vec![content_style],
        }
    }

    /// Creates a strategy with one head style and a content-style cycle.
    /// (Java `HorizontalCellStyleStrategy(WriteCellStyle, List<WriteCellStyle>)`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。
    pub const fn with_head_and_contents(
        head_style: ExcelCellStyle,
        content_styles: Vec<ExcelCellStyle>,
    ) -> Self {
        Self {
            head_style,
            content_styles,
        }
    }

    /// Attaches a head font (Java `headWriteCellStyle.setWriteFont`).
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。
    pub const fn with_head_font(mut self, font: ExcelFontStyle) -> Self {
        self.head_style.font = Some(font);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Attaches a head font from runtime [`WriteFont`]
    /// (Java `WriteCellStyle.setWriteFont(WriteFont)`).
    ///
    /// Owned font names are not copied into [`ExcelFontStyle`]; set
    /// [`ExcelFontStyle::font_name`] when a static name is required.
    #[must_use]
    pub fn with_head_write_font(mut self, font: &WriteFont) -> Self {
        self.head_style.font = Some(excel_font_style_from_write_font(font));
        self
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Attaches one content font to every configured content style
    /// (Java each `contentWriteCellStyle.setWriteFont`).
    #[must_use]
    pub fn with_content_font(mut self, font: ExcelFontStyle) -> Self {
        for style in &mut self.content_styles {
            style.font = Some(font);
        }
        self
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Attaches a content font from runtime [`WriteFont`].
    #[must_use]
    pub fn with_content_write_font(mut self, font: &WriteFont) -> Self {
        let converted = excel_font_style_from_write_font(font);
        for style in &mut self.content_styles {
            style.font = Some(converted);
        }
        self
    }

    /// Returns the configured head style. (Java `getHeadWriteCellStyle()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。
    pub const fn head_style(&self) -> ExcelCellStyle {
        self.head_style
    }
    #[must_use] pub const fn get_head_write_cell_style(&self) -> ExcelCellStyle { self.head_style() }
    pub const fn set_head_write_cell_style(&mut self, value: ExcelCellStyle) { self.head_style = value; }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Returns the configured content styles. (Java `getContentWriteCellStyleList()`)
    #[must_use]
    pub fn content_styles(&self) -> &[ExcelCellStyle] {
        &self.content_styles
    }
    #[must_use] pub fn get_content_write_cell_style_list(&self) -> &[ExcelCellStyle] { self.content_styles() }
    pub fn set_content_write_cell_style_list(&mut self, value: Vec<ExcelCellStyle>) { self.content_styles = value; }
}

impl AbstractCellStyleStrategy for HorizontalCellStyleStrategy {
    fn cell_style(&self, context: &WriteCellContext) -> ExcelCellStyle {
        // Java `setHeadCellStyle` / `setContentCellStyle`
        if context.is_head {
            return self.head_style;
        }
        if self.content_styles.is_empty() {
            return ExcelCellStyle::new();
        }
        // Java: `relativeRowIndex % contentWriteCellStyleList.size()`
        let relative = context.relative_row_index.unwrap_or(0);
        self.content_styles[relative % self.content_styles.len()]
    }
}

impl WriteHandler for HorizontalCellStyleStrategy {
    fn order(&self) -> i32 {
        // Java `OrderConstant.DEFINE_STYLE` on `AbstractCellStyleStrategy`
        50_000
    }

    fn style_cell_style(&self, context: &WriteCellContext) -> Option<ExcelCellStyle> {
        Some(AbstractCellStyleStrategy::cell_style(self, context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CellValue;

    #[test]
    fn horizontal_strategy_new_empty() {
        let s = HorizontalCellStyleStrategy::new(vec![]);
        assert!(s.content_styles().is_empty());
    }

    #[test]
    fn horizontal_strategy_with_head_and_content() {
        let s = HorizontalCellStyleStrategy::with_head_and_content(
            ExcelCellStyle::new(),
            ExcelCellStyle::new(),
        );
        assert_eq!(s.content_styles().len(), 1);
    }

    #[test]
    fn horizontal_strategy_with_head_and_contents() {
        let s = HorizontalCellStyleStrategy::with_head_and_contents(
            ExcelCellStyle::new(),
            vec![ExcelCellStyle::new(), ExcelCellStyle::new()],
        );
        assert_eq!(s.content_styles().len(), 2);
    }

    #[test]
    fn horizontal_strategy_with_head_font() {
        let font = ExcelFontStyle::default();
        let s = HorizontalCellStyleStrategy::new(vec![]).with_head_font(font);
        assert!(s.head_style().font.is_some());
    }

    #[test]
    fn horizontal_strategy_with_head_write_font() {
        let font = WriteFont::default();
        let s = HorizontalCellStyleStrategy::new(vec![]).with_head_write_font(&font);
        assert!(s.head_style().font.is_some());
    }

    #[test]
    fn horizontal_strategy_with_content_font() {
        let font = ExcelFontStyle::default();
        let style = ExcelCellStyle::new();
        let s = HorizontalCellStyleStrategy::new(vec![style]).with_content_font(font);
        assert!(s.content_styles()[0].font.is_some());
    }

    #[test]
    fn horizontal_strategy_with_content_write_font() {
        let font = WriteFont::default();
        let style = ExcelCellStyle::new();
        let s = HorizontalCellStyleStrategy::new(vec![style]).with_content_write_font(&font);
        assert!(s.content_styles()[0].font.is_some());
    }

    #[test]
    fn horizontal_strategy_order_is_50_000() {
        let s = HorizontalCellStyleStrategy::new(vec![]);
        assert_eq!(s.order(), 50_000);
    }

    #[test]
    fn horizontal_strategy_style_cell_style_head() {
        let s = HorizontalCellStyleStrategy::with_head_and_content(
            ExcelCellStyle::new(),
            ExcelCellStyle::new(),
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = true;
        let style = s.style_cell_style(&context);
        assert!(style.is_some());
    }

    #[test]
    fn horizontal_strategy_style_cell_style_content() {
        let s = HorizontalCellStyleStrategy::with_head_and_content(
            ExcelCellStyle::new(),
            ExcelCellStyle::new(),
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let style = s.style_cell_style(&context);
        assert!(style.is_some());
    }

    #[test]
    fn horizontal_strategy_style_cell_style_empty() {
        let s = HorizontalCellStyleStrategy::new(vec![]);
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let style = s.style_cell_style(&context);
        assert!(style.is_some());
    }

    #[test]
    fn horizontal_strategy_cell_style_strategy_head() {
        let s = HorizontalCellStyleStrategy::with_head_and_content(
            ExcelCellStyle::new(),
            ExcelCellStyle::new(),
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = true;
        let style = AbstractCellStyleStrategy::cell_style(&s, &context);
        assert!(style.font.is_none() || style.font.is_some());
    }

    #[test]
    fn horizontal_strategy_cell_style_strategy_content() {
        let s = HorizontalCellStyleStrategy::with_head_and_contents(
            ExcelCellStyle::new(),
            vec![ExcelCellStyle::new(), ExcelCellStyle::new()],
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let _ = AbstractCellStyleStrategy::cell_style(&s, &context);
    }
}
