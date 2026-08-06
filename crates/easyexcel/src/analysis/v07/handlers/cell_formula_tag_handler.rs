//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.CellFormulaTagHandler`.

use super::xlsx_tag_handler::XlsxTagHandler;

/// 对应 Java：`CellFormulaTagHandler`.
#[derive(Debug, Default)]
pub struct CellFormulaTagHandler {
    /// Accumulated formula text. (Java `XlsxReadSheetHolder.tempFormula`)
    pub temp_formula: String,
}

impl CellFormulaTagHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.CellFormulaTagHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.CellFormulaTagHandler。 Java `CellFormulaTagHandler.startElement`.
    pub fn begin_formula(&mut self) {
        self.temp_formula.clear();
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.CellFormulaTagHandler。 Java `CellFormulaTagHandler.endElement` — returns the formula string.
    pub fn finish_formula(&mut self) -> String {
        std::mem::take(&mut self.temp_formula)
    }
}

impl XlsxTagHandler for CellFormulaTagHandler {
    /// Java `CellFormulaTagHandler.startElement`.
    fn start_element(&mut self, name: &str, _attrs: &str) {
        let local = easyexcel_xlsx::local_tag_name(name);
        if local == "f" {
            self.begin_formula();
        }
    }

    /// Java `CellFormulaTagHandler.endElement`.
    fn end_element(&mut self, name: &str) {
        let local = easyexcel_xlsx::local_tag_name(name);
        if local == "f" {
            let _ = self.finish_formula();
        }
    }

    /// Java `CellFormulaTagHandler.characters`.
    fn characters(&mut self, ch: &str) {
        self.temp_formula.push_str(ch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_handler_accumulates_f_formula_text() {
        // 对应 Java：CellFormulaTagHandler 累积 <f> 公式文本
        let mut handler = CellFormulaTagHandler::new();
        handler.start_element("f", "");
        handler.characters("SUM(");
        handler.characters("A1:A3)");
        assert_eq!(handler.temp_formula, "SUM(A1:A3)");
        assert_eq!(handler.finish_formula(), "SUM(A1:A3)");
        // 非 f 标签不触发 begin/finish
        handler.start_element("v", "");
        handler.end_element("v");
        assert_eq!(handler.temp_formula, "");
    }
}
