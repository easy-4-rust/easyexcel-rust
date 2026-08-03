//! 对应 Java：`com.alibaba.excel.write.style.AbstractVerticalCellStyleStrategy`.

use crate::core::{ExcelCellStyle, WriteCellContext};

use crate::writer::style::abstract_cell_style_strategy::AbstractCellStyleStrategy;

/// 对应 Java：`AbstractVerticalCellStyleStrategy extends AbstractCellStyleStrategy`.
///
/// The Java side stores two `WriteCellStyle` fields (`headCellStyle`,
/// `contentCellStyle`) and applies them based on `isHead`. The Rust
/// port exposes the same two accessors; concrete types such as
/// [`crate::writer::style::vertical_cell_style_strategy::VerticalCellStyleStrategy`]
/// implement them and register as [`crate::core::WriteHandler`].
///
/// Default methods return an empty style (Java returns `null`), so a
/// minimal override only fills the columns that need differentiation.
pub trait AbstractVerticalCellStyleStrategy: AbstractCellStyleStrategy {
    /// Returns the head cell style. (Java `headCellStyle(CellWriteHandlerContext)` /
    /// `headCellStyle(Head)`)
    fn head_cell_style(&self, _context: &WriteCellContext) -> ExcelCellStyle {
        ExcelCellStyle::new()
    }

    /// Returns the content cell style. (Java `contentCellStyle(CellWriteHandlerContext)` /
    /// `contentCellStyle(Head)`)
    fn content_cell_style(&self, _context: &WriteCellContext) -> ExcelCellStyle {
        ExcelCellStyle::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        CellValue, ExcelCellStyle, ExcelHorizontalAlignment, WriteCellContext, WriteHandler,
    };

    use super::*;
    use crate::writer::style::abstract_cell_style_strategy::AbstractCellStyleStrategy;

    struct TestVerticalStrategy {
        head_style: ExcelCellStyle,
        content_style: ExcelCellStyle,
    }

    impl WriteHandler for TestVerticalStrategy {}

    impl AbstractCellStyleStrategy for TestVerticalStrategy {
        fn cell_style(&self, _context: &WriteCellContext) -> ExcelCellStyle {
            ExcelCellStyle::new()
        }
    }

    impl AbstractVerticalCellStyleStrategy for TestVerticalStrategy {
        fn head_cell_style(&self, _context: &WriteCellContext) -> ExcelCellStyle {
            self.head_style
        }
        fn content_cell_style(&self, _context: &WriteCellContext) -> ExcelCellStyle {
            self.content_style
        }
    }

    struct DefaultStrategy;

    impl WriteHandler for DefaultStrategy {}

    impl AbstractCellStyleStrategy for DefaultStrategy {
        fn cell_style(&self, _context: &WriteCellContext) -> ExcelCellStyle {
            ExcelCellStyle::new()
        }
    }

    impl AbstractVerticalCellStyleStrategy for DefaultStrategy {}

    #[test]
    fn default_head_cell_style_is_empty() {
        let strategy = DefaultStrategy;
        let context = WriteCellContext::new("Sheet1", 0, 0, CellValue::Empty);
        let style = strategy.head_cell_style(&context);
        assert_eq!(style, ExcelCellStyle::new());
    }

    #[test]
    fn default_content_cell_style_is_empty() {
        let strategy = DefaultStrategy;
        let context = WriteCellContext::new("Sheet1", 0, 0, CellValue::Empty);
        let style = strategy.content_cell_style(&context);
        assert_eq!(style, ExcelCellStyle::new());
    }

    #[test]
    fn custom_head_cell_style_is_returned() {
        let head_style = ExcelCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..ExcelCellStyle::new()
        };
        let strategy = TestVerticalStrategy {
            head_style,
            content_style: ExcelCellStyle::new(),
        };
        let context = WriteCellContext::new("Sheet1", 0, 0, CellValue::Empty);
        let style = strategy.head_cell_style(&context);
        assert_eq!(
            style.horizontal_alignment,
            Some(ExcelHorizontalAlignment::Center)
        );
    }

    #[test]
    fn custom_content_cell_style_is_returned() {
        let content_style = ExcelCellStyle {
            wrapped: Some(true),
            ..ExcelCellStyle::new()
        };
        let strategy = TestVerticalStrategy {
            head_style: ExcelCellStyle::new(),
            content_style,
        };
        let context = WriteCellContext::new("Sheet1", 0, 0, CellValue::Empty);
        let style = strategy.content_cell_style(&context);
        assert_eq!(style.wrapped, Some(true));
    }

    #[test]
    fn vertical_strategy_cell_style_delegates_to_impl() {
        let strategy = TestVerticalStrategy {
            head_style: ExcelCellStyle::new(),
            content_style: ExcelCellStyle::new(),
        };
        let context = WriteCellContext::new("Sheet1", 0, 0, CellValue::Empty);
        let _ = strategy.cell_style(&context);
        let default = DefaultStrategy;
        let _ = default.cell_style(&context);
    }

    #[test]
    // 语义敏感：`head` / `content` / `context` 命名与 Java 侧 getter 一一对应，
    // 保留原名便于对照，故豁免 similar_names。
    #[allow(clippy::similar_names)]
    fn different_styles_for_head_and_content() {
        let head_style = ExcelCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..ExcelCellStyle::new()
        };
        let content_style = ExcelCellStyle {
            wrapped: Some(true),
            ..ExcelCellStyle::new()
        };
        let strategy = TestVerticalStrategy {
            head_style,
            content_style,
        };
        let context = WriteCellContext::new("Sheet1", 0, 0, CellValue::Empty);

        let head = strategy.head_cell_style(&context);
        let content = strategy.content_cell_style(&context);

        assert_ne!(head, content);
        assert_eq!(
            head.horizontal_alignment,
            Some(ExcelHorizontalAlignment::Center)
        );
        assert_eq!(content.wrapped, Some(true));
    }
}
