//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.CellValueTagHandler`.
//!
//! Java class is empty — it inherits `characters` from
//! `AbstractCellValueTagHandler` for the `<v>` tag.

use super::abstract_cell_value_tag_handler::AbstractCellValueTagHandler;
use super::xlsx_tag_handler::XlsxTagHandler;

/// 对应 Java：`CellValueTagHandler` (`<v>` cell value tag).
#[derive(Debug, Default)]
pub struct CellValueTagHandler {
    inner: AbstractCellValueTagHandler,
}

impl CellValueTagHandler {
    /// Creates an idle `<v>` handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the shared temp buffer.
    #[must_use]
    pub fn temp_data(&self) -> &str {
        &self.inner.temp_data
    }

    /// Takes accumulated `<v>` text.
    pub fn take(&mut self) -> String {
        self.inner.take()
    }
}

impl XlsxTagHandler for CellValueTagHandler {
    /// Java `CellValueTagHandler` inherits empty `startElement` — buffer is
    /// cleared by `CellTagHandler.startElement` when `<c>` opens.
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
    fn cell_value_handler_accumulates_v_text() {
        // 对应 Java：CellValueTagHandler 继承 characters 累积 <v> 文本
        let mut handler = CellValueTagHandler::new();
        assert_eq!(handler.temp_data(), "");
        handler.start_element("v", "");
        handler.characters("12");
        handler.characters(".5");
        assert_eq!(handler.temp_data(), "12.5");
        assert_eq!(handler.take(), "12.5");
        assert_eq!(handler.take(), "");
    }
}
