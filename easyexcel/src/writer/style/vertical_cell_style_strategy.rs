//! Mirrors anonymous subclasses of Java
//! `com.alibaba.excel.write.style.AbstractVerticalCellStyleStrategy`.
//!
//! Java tests (`StyleDataTest.t03` / `t04`) override `headCellStyle(Head)` /
//! `contentCellStyle(Head)` per column. Rust cannot subclass traits, so this
//! concrete type accepts column-index closures (or fixed maps) and is
//! registerable as a [`WriteHandler`]. Nested fonts use
//! [`ExcelCellStyle::font`] (Java `WriteCellStyle.writeFont`).

use crate::core::{ExcelCellStyle, ExcelFontStyle, WriteCellContext, WriteFont, WriteHandler};

use crate::writer::metadata::style::write_font::excel_font_style_from_write_font;
use crate::writer::style::abstract_cell_style_strategy::AbstractCellStyleStrategy;
use crate::writer::style::abstract_vertical_cell_style_strategy::AbstractVerticalCellStyleStrategy;

/// Concrete, testable vertical (per-column) cell style strategy.
///
/// 对应 Java： anonymous `AbstractVerticalCellStyleStrategy` subclasses used
/// in `StyleDataTest`.
pub struct VerticalCellStyleStrategy {
    head: Box<dyn Fn(usize) -> ExcelCellStyle + Send + Sync>,
    content: Box<dyn Fn(usize) -> ExcelCellStyle + Send + Sync>,
}

impl VerticalCellStyleStrategy {
    /// Creates a strategy from head/content style factories keyed by column index.
    ///
    /// (Java anonymous class overriding `headCellStyle(Head)` / `contentCellStyle(Head)`)
    #[must_use]
    pub fn new(
        head: impl Fn(usize) -> ExcelCellStyle + Send + Sync + 'static,
        content: impl Fn(usize) -> ExcelCellStyle + Send + Sync + 'static,
    ) -> Self {
        Self {
            head: Box::new(head),
            content: Box::new(content),
        }
    }

    /// Creates a strategy with constant head and content styles for every column.
    #[must_use]
    pub fn uniform(head: ExcelCellStyle, content: ExcelCellStyle) -> Self {
        Self::new(move |_| head, move |_| content)
    }

    /// Creates a uniform strategy and attaches head/content fonts
    /// (Java `WriteCellStyle.setWriteFont` on both styles).
    #[must_use]
    pub fn uniform_with_fonts(
        mut head: ExcelCellStyle,
        head_font: ExcelFontStyle,
        mut content: ExcelCellStyle,
        content_font: ExcelFontStyle,
    ) -> Self {
        head.font = Some(head_font);
        content.font = Some(content_font);
        Self::uniform(head, content)
    }

    /// Creates a uniform strategy from runtime [`WriteFont`] values
    /// (Java `setWriteFont(WriteFont)`; owned names omitted — see
    /// [`excel_font_style_from_write_font`]).
    #[must_use]
    pub fn uniform_with_write_fonts(
        head: ExcelCellStyle,
        head_font: &WriteFont,
        content: ExcelCellStyle,
        content_font: &WriteFont,
    ) -> Self {
        Self::uniform_with_fonts(
            head,
            excel_font_style_from_write_font(head_font),
            content,
            excel_font_style_from_write_font(content_font),
        )
    }
}

impl AbstractVerticalCellStyleStrategy for VerticalCellStyleStrategy {
    fn head_cell_style(&self, context: &WriteCellContext) -> ExcelCellStyle {
        (self.head)(usize::from(context.column_index))
    }

    fn content_cell_style(&self, context: &WriteCellContext) -> ExcelCellStyle {
        (self.content)(usize::from(context.column_index))
    }
}

impl AbstractCellStyleStrategy for VerticalCellStyleStrategy {
    fn cell_style(&self, context: &WriteCellContext) -> ExcelCellStyle {
        // Java `AbstractVerticalCellStyleStrategy.setHead/ContentCellStyle`
        if context.is_head {
            AbstractVerticalCellStyleStrategy::head_cell_style(self, context)
        } else {
            AbstractVerticalCellStyleStrategy::content_cell_style(self, context)
        }
    }
}

impl WriteHandler for VerticalCellStyleStrategy {
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
    fn vertical_strategy_new_closure() {
        let s =
            VerticalCellStyleStrategy::new(|_| ExcelCellStyle::new(), |_| ExcelCellStyle::new());
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = true;
        let _ = s.cell_style(&context);
        context.is_head = false;
        let _ = s.cell_style(&context);
    }

    #[test]
    fn vertical_strategy_uniform() {
        let s = VerticalCellStyleStrategy::uniform(ExcelCellStyle::new(), ExcelCellStyle::new());
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let _ = s.cell_style(&context);
    }

    #[test]
    fn vertical_strategy_uniform_with_fonts() {
        let s = VerticalCellStyleStrategy::uniform_with_fonts(
            ExcelCellStyle::new(),
            ExcelFontStyle::default(),
            ExcelCellStyle::new(),
            ExcelFontStyle::default(),
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let _ = s.cell_style(&context);
    }

    #[test]
    fn vertical_strategy_uniform_with_write_fonts() {
        let s = VerticalCellStyleStrategy::uniform_with_write_fonts(
            ExcelCellStyle::new(),
            &WriteFont::default(),
            ExcelCellStyle::new(),
            &WriteFont::default(),
        );
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let _ = s.cell_style(&context);
    }

    #[test]
    fn vertical_strategy_order_is_50_000() {
        let s = VerticalCellStyleStrategy::uniform(ExcelCellStyle::new(), ExcelCellStyle::new());
        assert_eq!(s.order(), 50_000);
    }

    #[test]
    fn vertical_strategy_style_cell_style_head() {
        let s = VerticalCellStyleStrategy::uniform(ExcelCellStyle::new(), ExcelCellStyle::new());
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = true;
        let style = s.style_cell_style(&context);
        assert!(style.is_some());
    }

    #[test]
    fn vertical_strategy_style_cell_style_content() {
        let s = VerticalCellStyleStrategy::uniform(ExcelCellStyle::new(), ExcelCellStyle::new());
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let style = s.style_cell_style(&context);
        assert!(style.is_some());
    }

    #[test]
    fn vertical_strategy_head_cell_style() {
        let s = VerticalCellStyleStrategy::uniform(ExcelCellStyle::new(), ExcelCellStyle::new());
        let context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        let _ = AbstractVerticalCellStyleStrategy::head_cell_style(&s, &context);
    }

    #[test]
    fn vertical_strategy_content_cell_style() {
        let s = VerticalCellStyleStrategy::uniform(ExcelCellStyle::new(), ExcelCellStyle::new());
        let context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        let _ = AbstractVerticalCellStyleStrategy::content_cell_style(&s, &context);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::CellValue;

    #[test]
    fn vertical_strategy_content_closure_applies() {
        let s =
            VerticalCellStyleStrategy::new(|_| ExcelCellStyle::new(), |_| ExcelCellStyle::new());
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        context.is_head = false;
        let _ = s.cell_style(&context);
        context.is_head = true;
        let _ = s.cell_style(&context);
    }
}
