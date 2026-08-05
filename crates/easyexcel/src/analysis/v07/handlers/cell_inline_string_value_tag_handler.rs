//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.CellInlineStringValueTagHandler`.
//!
//! Java class is empty — it inherits `characters` from
//! `AbstractCellValueTagHandler` for the inline `<t>` tag.

use super::abstract_cell_value_tag_handler::AbstractCellValueTagHandler;
use super::xlsx_tag_handler::XlsxTagHandler;

/// 对应 Java：`CellInlineStringValueTagHandler` (inline string `<t>`).
#[derive(Debug, Default)]
pub struct CellInlineStringValueTagHandler {
    inner: AbstractCellValueTagHandler,
}

impl CellInlineStringValueTagHandler {
    /// Creates an idle inline-string handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes accumulated inline `<t>` text.
    pub fn take(&mut self) -> String {
        self.inner.take()
    }
}

impl XlsxTagHandler for CellInlineStringValueTagHandler {
    /// Java `CellInlineStringValueTagHandler` inherits empty `startElement` —
    /// multiple rich-text `<t>` runs append into the same `tempData`.
    fn start_element(&mut self, name: &str, attrs: &str) {
        let _ = (name, attrs);
    }

    /// Java `AbstractCellValueTagHandler.characters`.
    fn characters(&mut self, ch: &str) {
        self.inner.append(ch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_string_handler_accumulates_rich_text_runs() {
        // 对应 Java：CellInlineStringValueTagHandler 多段 <t> 追加同一缓冲
        let mut handler = CellInlineStringValueTagHandler::new();
        handler.start_element("t", "");
        handler.characters("rich ");
        handler.start_element("t", "");
        handler.characters("text");
        assert_eq!(handler.take(), "rich text");
    }
}
