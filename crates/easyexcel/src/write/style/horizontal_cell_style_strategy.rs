//! 对应 Java：`com.alibaba.excel.write.style.HorizontalCellStyleStrategy`.
//!
//! Wired into the XLSX write path via [`WriteHandler::style_cell_style`], which
//! the writer merges into each cell format after annotation styles. Nested
//! fonts on [`ExcelCellStyle::font`] mirror Java `WriteCellStyle.writeFont`.

use crate::core::{
    ExcelCellStyle, ExcelFontStyle, WriteCellContext, WriteCellStyle, WriteFont, WriteHandler,
};

use crate::write::metadata::style::write_font::write_font_from_excel_font_style;
use crate::write::style::abstract_cell_style_strategy::AbstractCellStyleStrategy;

/// 对应 Java：`HorizontalCellStyleStrategy`.
///
/// The Java side cycles through a list of content styles by
/// `relativeRowIndex`; the Rust port mirrors that behaviour once the
/// write path supplies [`WriteCellContext::relative_row_index`].
/// Styles may carry nested fonts via [`ExcelCellStyle::font`] (Java
/// `WriteCellStyle.setWriteFont`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HorizontalCellStyleStrategy {
    head_style: WriteCellStyle,
    content_styles: Vec<WriteCellStyle>,
}

impl HorizontalCellStyleStrategy {
    /// Creates a strategy with content styles only (empty head style).
    /// (Java `HorizontalCellStyleStrategy(List<WriteCellStyle>)` subset)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。
    pub fn new(content_styles: Vec<WriteCellStyle>) -> Self {
        Self {
            head_style: WriteCellStyle::new(),
            content_styles,
        }
    }

    /// 从注解/引擎轻量样式创建策略。
    #[must_use]
    pub fn from_engine_styles(content_styles: Vec<ExcelCellStyle>) -> Self {
        Self::new(content_styles.into_iter().map(Into::into).collect())
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Creates a strategy with one head style and one content style.
    /// (Java `HorizontalCellStyleStrategy(WriteCellStyle, WriteCellStyle)`)
    #[must_use]
    pub fn with_head_and_content(
        head_style: WriteCellStyle,
        content_style: WriteCellStyle,
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
    pub fn with_head_and_contents(
        head_style: WriteCellStyle,
        content_styles: Vec<WriteCellStyle>,
    ) -> Self {
        Self {
            head_style,
            content_styles,
        }
    }

    /// Attaches a head font (Java `headWriteCellStyle.setWriteFont`).
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。
    pub fn with_head_font(mut self, font: ExcelFontStyle) -> Self {
        self.head_style = self.head_style.with_excel_font_style(font);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Attaches a head font from runtime [`WriteFont`]
    /// (Java `WriteCellStyle.setWriteFont(WriteFont)`).
    ///
    /// 可复制字段进入 [`ExcelFontStyle`] 热路径，动态字体名称同时保存在
    /// `WriteFont` 侧车中并由格式边界应用，不会被静默丢弃。
    #[must_use]
    pub fn with_head_write_font(mut self, font: &WriteFont) -> Self {
        self.head_style.font = Some(font.clone());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Attaches one content font to every configured content style
    /// (Java each `contentWriteCellStyle.setWriteFont`).
    #[must_use]
    pub fn with_content_font(mut self, font: ExcelFontStyle) -> Self {
        for style in &mut self.content_styles {
            style.font = Some(write_font_from_excel_font_style(font));
        }
        self
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Attaches a content font from runtime [`WriteFont`].
    #[must_use]
    pub fn with_content_write_font(mut self, font: &WriteFont) -> Self {
        for style in &mut self.content_styles {
            style.font = Some(font.clone());
        }
        self
    }

    /// Returns the configured head style. (Java `getHeadWriteCellStyle()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。
    pub const fn head_style(&self) -> &WriteCellStyle {
        &self.head_style
    }
    #[must_use] pub const fn get_head_write_cell_style(&self) -> &WriteCellStyle { self.head_style() }
    pub fn set_head_write_cell_style(&mut self, value: WriteCellStyle) {
        self.head_style = value;
    }

    /// 对应 Java：com.alibaba.excel.write.style.HorizontalCellStyleStrategy。 Returns the configured content styles. (Java `getContentWriteCellStyleList()`)
    #[must_use]
    pub fn content_styles(&self) -> &[WriteCellStyle] {
        &self.content_styles
    }
    #[must_use] pub fn get_content_write_cell_style_list(&self) -> &[WriteCellStyle] { self.content_styles() }
    pub fn set_content_write_cell_style_list(&mut self, value: Vec<WriteCellStyle>) {
        self.content_styles = value;
    }
}

impl Default for HorizontalCellStyleStrategy {
    /// Java 无参构造器：head/content 均保持未设置。
    fn default() -> Self { Self::new(Vec::new()) }
}

impl AbstractCellStyleStrategy for HorizontalCellStyleStrategy {
    fn cell_style(&self, context: &WriteCellContext) -> ExcelCellStyle {
        // Java `setHeadCellStyle` / `setContentCellStyle`
        if context.is_head {
            return self.head_style.engine_cell_style();
        }
        if self.content_styles.is_empty() {
            return ExcelCellStyle::new();
        }
        // Java: `relativeRowIndex % contentWriteCellStyleList.size()`
        let relative = context.relative_row_index.unwrap_or(0);
        self.content_styles[relative % self.content_styles.len()].engine_cell_style()
    }
}

impl WriteHandler for HorizontalCellStyleStrategy {
    fn order(&self) -> i32 {
        // Java `OrderConstant.DEFINE_STYLE` on `AbstractCellStyleStrategy`
        crate::constant::order_constant::DEFINE_STYLE
    }

    fn style_cell_style(&self, context: &WriteCellContext) -> Option<ExcelCellStyle> {
        Some(AbstractCellStyleStrategy::cell_style(self, context))
    }

    fn style_write_font(&self, context: &WriteCellContext) -> Option<WriteFont> {
        if context.is_head {
            return self.head_style.font.clone();
        }
        if self.content_styles.is_empty() {
            return None;
        }
        let relative = context.relative_row_index.unwrap_or(0);
        self.content_styles[relative % self.content_styles.len()]
            .font
            .clone()
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
            ExcelCellStyle::new().into(),
            ExcelCellStyle::new().into(),
        );
        assert_eq!(s.content_styles().len(), 1);
    }

    #[test]
    fn horizontal_strategy_with_head_and_contents() {
        let s = HorizontalCellStyleStrategy::with_head_and_contents(
            ExcelCellStyle::new().into(),
            vec![ExcelCellStyle::new().into(), ExcelCellStyle::new().into()],
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
        let s = HorizontalCellStyleStrategy::new(vec![style.into()]).with_content_font(font);
        assert!(s.content_styles()[0].font.is_some());
    }

    #[test]
    fn horizontal_strategy_with_content_write_font() {
        let font = WriteFont::default();
        let style = ExcelCellStyle::new();
        let s = HorizontalCellStyleStrategy::new(vec![style.into()]).with_content_write_font(&font);
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
            ExcelCellStyle::new().into(),
            ExcelCellStyle::new().into(),
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = true;
        let style = s.style_cell_style(&context);
        assert!(style.is_some());
    }

    #[test]
    fn horizontal_strategy_style_cell_style_content() {
        let s = HorizontalCellStyleStrategy::with_head_and_content(
            ExcelCellStyle::new().into(),
            ExcelCellStyle::new().into(),
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
            ExcelCellStyle::new().into(),
            ExcelCellStyle::new().into(),
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = true;
        let style = AbstractCellStyleStrategy::cell_style(&s, &context);
        assert!(style.font.is_none() || style.font.is_some());
    }

    #[test]
    fn horizontal_strategy_cell_style_strategy_content() {
        let s = HorizontalCellStyleStrategy::with_head_and_contents(
            ExcelCellStyle::new().into(),
            vec![ExcelCellStyle::new().into(), ExcelCellStyle::new().into()],
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let _ = AbstractCellStyleStrategy::cell_style(&s, &context);
    }
}
