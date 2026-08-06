//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.AbstractCellValueTagHandler`.

use super::abstract_xlsx_tag_handler::AbstractXlsxTagHandler;
use super::xlsx_tag_handler::XlsxTagHandler;

/// 对应 Java：`AbstractCellValueTagHandler extends AbstractXlsxTagHandler`.
///
/// Java only overrides `characters` to append into `tempData`. Concrete
/// handlers (`CellValueTagHandler`, `CellInlineStringValueTagHandler`) inherit
/// that behaviour.
#[derive(Debug, Default)]
pub struct AbstractCellValueTagHandler {
    /// Character accumulator mirroring Java `XlsxReadSheetHolder.tempData`.
    pub temp_data: String,
}

impl AbstractCellValueTagHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.AbstractCellValueTagHandler。 Creates an idle accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.AbstractCellValueTagHandler。 Java `AbstractCellValueTagHandler.characters`.
    pub fn append(&mut self, ch: &str) {
        self.temp_data.push_str(ch);
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.AbstractCellValueTagHandler。 Takes and clears the accumulated text.
    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.temp_data)
    }
}

impl XlsxTagHandler for AbstractCellValueTagHandler {
    /// Java `AbstractCellValueTagHandler.characters`.
    fn characters(&mut self, ch: &str) {
        self.append(ch);
    }
}

include!("abstract_cell_value_tag_handler/abstract_cell_value_base.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abstract_value_handler_accumulates_and_takes() {
        // 对应 Java：AbstractCellValueTagHandler.characters 追加 tempData
        let mut handler = AbstractCellValueTagHandler::new();
        assert_eq!(handler.temp_data, "");
        handler.characters("ab");
        handler.characters("cd");
        assert_eq!(handler.temp_data, "abcd");
        assert_eq!(handler.take(), "abcd");
        assert!(handler.temp_data.is_empty());
    }
}
