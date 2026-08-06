//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.sax.XlsxRowHandler`.
//!
//! Java routes each worksheet tag through a static `XLSX_CELL_HANDLER_MAP`.
//! This Rust port keeps the same map and forwards SAX-style callbacks; the
//! production reader still uses `xlsx_rows::XlsxDisplayCellReader` as the
//! primary event loop and may call individual handlers from that path.

use std::collections::HashMap;

use crate::constant::excel_xml_constants::{
    CELL_FORMULA_TAG, CELL_INLINE_STRING_VALUE_TAG, CELL_TAG, CELL_VALUE_TAG, DIMENSION_TAG,
    HYPERLINK_TAG, MERGE_CELL_TAG, ROW_TAG,
};

use crate::analysis::v07::handlers::cell_formula_tag_handler::CellFormulaTagHandler;
use crate::analysis::v07::handlers::cell_inline_string_value_tag_handler::CellInlineStringValueTagHandler;
use crate::analysis::v07::handlers::cell_tag_handler::CellTagHandler;
use crate::analysis::v07::handlers::cell_value_tag_handler::CellValueTagHandler;
use crate::analysis::v07::handlers::count_tag_handler::CountTagHandler;
use crate::analysis::v07::handlers::hyperlink_tag_handler::HyperlinkTagHandler;
use crate::analysis::v07::handlers::merge_cell_tag_handler::MergeCellTagHandler;
use crate::analysis::v07::handlers::row_tag_handler::RowTagHandler;
use crate::analysis::v07::handlers::xlsx_tag_handler::XlsxTagHandler;

include!("xlsx_row_handler/routed_handler.rs");

/// 对应 Java：`XlsxRowHandler extends DefaultHandler`.
pub struct XlsxRowHandler {
    /// Active handlers keyed by local tag name.
    handlers: HashMap<&'static str, RoutedHandler>,
    /// Open-tag stack. (Java `XlsxReadSheetHolder.tagDeque`)
    tag_stack: Vec<String>,
}

impl XlsxRowHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.sax.XlsxRowHandler。 Java `XlsxRowHandler(XlsxReadContext)` static map initialisation.
    #[must_use]
    pub fn new(read_merge: bool, read_hyperlink: bool) -> Self {
        let mut handlers = HashMap::new();
        handlers.insert(CELL_TAG, RoutedHandler::Cell(CellTagHandler::new()));
        handlers.insert(ROW_TAG, RoutedHandler::Row(RowTagHandler::new()));
        handlers.insert(
            CELL_VALUE_TAG,
            RoutedHandler::CellValue(CellValueTagHandler::new()),
        );
        handlers.insert(
            CELL_INLINE_STRING_VALUE_TAG,
            RoutedHandler::InlineString(CellInlineStringValueTagHandler::new()),
        );
        handlers.insert(
            CELL_FORMULA_TAG,
            RoutedHandler::Formula(CellFormulaTagHandler::new()),
        );
        handlers.insert(DIMENSION_TAG, RoutedHandler::Count(CountTagHandler::new()));
        handlers.insert(
            MERGE_CELL_TAG,
            RoutedHandler::Merge(MergeCellTagHandler::new(read_merge)),
        );
        handlers.insert(
            HYPERLINK_TAG,
            RoutedHandler::Hyperlink(HyperlinkTagHandler::new(read_hyperlink)),
        );
        Self {
            handlers,
            tag_stack: Vec::new(),
        }
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.sax.XlsxRowHandler。 Java `XlsxRowHandler.startElement`.
    pub fn start_element(&mut self, name: &str, attrs: &str) {
        let local = easyexcel_xlsx::local_tag_name(name);
        let Some(handler) = self.handlers.get_mut(local) else {
            return;
        };
        if !handler.as_mut().support() {
            return;
        }
        self.tag_stack.push(local.to_owned());
        handler.as_mut().start_element(name, attrs);
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.sax.XlsxRowHandler。 Java `XlsxRowHandler.characters`.
    pub fn characters(&mut self, ch: &str) {
        let Some(current) = self.tag_stack.last() else {
            return;
        };
        let key = current.clone();
        let Some(handler) = self.handlers.get_mut(key.as_str()) else {
            return;
        };
        if !handler.as_mut().support() {
            return;
        }
        handler.as_mut().characters(ch);
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.sax.XlsxRowHandler。 Java `XlsxRowHandler.endElement`.
    pub fn end_element(&mut self, name: &str) {
        let local = easyexcel_xlsx::local_tag_name(name);
        let Some(handler) = self.handlers.get_mut(local) else {
            return;
        };
        if !handler.as_mut().support() {
            return;
        }
        handler.as_mut().end_element(name);
        let _ = self.tag_stack.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::v07::handlers::cell_tag_handler::CellTagHandler;

    #[test]
    fn routes_dimension_row_cell_value_formula_merge_and_hyperlink() {
        // 对应 Java：XlsxRowHandler 按 XLSX_CELL_HANDLER_MAP 全量路由
        let mut handler = XlsxRowHandler::new(true, true);

        // <dimension> 统计近似总行数
        handler.start_element("dimension", "ref=A1:C10");
        let RoutedHandler::Count(count) = handler.handlers.get(DIMENSION_TAG).unwrap() else {
            panic!("dimension routes to CountTagHandler");
        };
        assert_eq!(count.approximate_total_row_number, Some(10));
        handler.end_element("dimension");

        // <row> 行索引
        handler.start_element("row", "r=2");
        let RoutedHandler::Row(row) = handler.handlers.get(ROW_TAG).unwrap() else {
            panic!("row routes to RowTagHandler");
        };
        assert_eq!(row.row_index, Some(1));
        handler.characters("");
        handler.end_element("row");

        // <c> + <v> 数值单元格
        handler.start_element("c", "r=B2 t=n");
        handler.start_element("v", "");
        handler.characters("42");
        handler.end_element("v");
        handler.end_element("c");
        let RoutedHandler::Cell(cell) = handler.handlers.get(CELL_TAG).unwrap() else {
            panic!("c routes to CellTagHandler");
        };
        assert_eq!(cell.column_index, Some(1));
        assert!(cell.temp_data.is_empty(), "endElement resets temp data");

        // inline string <is><t>
        handler.start_element("c", "r=C2 t=inlineStr");
        handler.start_element("is", "");
        handler.start_element("t", "");
        handler.characters("hello");
        handler.end_element("t");
        handler.end_element("is");
        handler.end_element("c");

        // 公式 <f>
        handler.start_element("c", "r=D2");
        handler.start_element("f", "");
        handler.characters("SUM(A1:A2)");
        handler.end_element("f");
        handler.end_element("c");
        let RoutedHandler::Formula(formula) = handler.handlers.get(CELL_FORMULA_TAG).unwrap()
        else {
            panic!("f routes to CellFormulaTagHandler");
        };
        assert!(formula.temp_formula.is_empty(), "finish_formula takes text");

        // <mergeCell> 与 <hyperlink>
        handler.start_element("mergeCell", "ref=A1:B2");
        handler.end_element("mergeCell");
        handler.start_element("hyperlink", "ref=A1 location=example.com");
        handler.end_element("hyperlink");
        let RoutedHandler::Merge(merge) = handler.handlers.get(MERGE_CELL_TAG).unwrap() else {
            panic!("mergeCell routes to MergeCellTagHandler");
        };
        assert!(merge.last_extra.is_some());
        let RoutedHandler::Hyperlink(hyperlink) = handler.handlers.get(HYPERLINK_TAG).unwrap()
        else {
            panic!("hyperlink routes to HyperlinkTagHandler");
        };
        assert!(hyperlink.last_extra.is_some());

        // 带命名空间前缀的标签同样路由
        handler.start_element("x:row", "r=3");
        handler.end_element("x:row");

        // 未知标签：不 panic、不进栈
        handler.start_element("unknown", "foo=bar");
        handler.characters("x");
        handler.end_element("unknown");
        assert!(handler.tag_stack.is_empty());
    }

    #[test]
    fn skips_disabled_handlers_and_orphan_characters() {
        // 对应 Java：support()=false 的处理器被跳过
        let mut handler = XlsxRowHandler::new(false, false);
        handler.start_element("mergeCell", "ref=A1:B2");
        let RoutedHandler::Merge(merge) = handler.handlers.get(MERGE_CELL_TAG).unwrap() else {
            panic!("mergeCell routes to MergeCellTagHandler");
        };
        assert!(merge.last_extra.is_none(), "disabled merge is skipped");

        handler.start_element("hyperlink", "ref=A1 location=x");
        // 空栈 characters 直接返回
        handler.characters("orphan");
        // 关闭未知标签直接返回
        handler.end_element("whatever");
        // 关闭未入栈的已知标签（support=false）不弹栈
        handler.end_element("mergeCell");
        assert!(handler.tag_stack.is_empty());
    }

    #[test]
    fn characters_target_the_current_open_tag() {
        // 对应 Java：characters 路由到栈顶标签的处理器
        let mut handler = XlsxRowHandler::new(false, false);
        handler.characters("before-open");
        handler.start_element("row", "r=1");
        handler.characters("inside");
        handler.end_element("row");
        handler.characters("after-close");

        // CellTagHandler 独立使用时 characters 累积（对应 Java：tempData）
        let mut cell = CellTagHandler::new();
        cell.characters("abc");
        assert_eq!(cell.temp_data, "abc");
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    #[test]
    fn characters_with_unknown_stack_entry_is_ignored() {
        // 对应 Java：栈顶标签不在 handler 表内时 characters 直接返回
        let mut handler = XlsxRowHandler::new(true, true);
        handler.tag_stack.push("zzz".to_owned());
        handler.characters("orphan");
        assert_eq!(handler.tag_stack, vec!["zzz".to_owned()]);
    }

    #[test]
    fn characters_skip_disabled_handlers_even_when_pushed() {
        // 对应 Java：support()=false 的处理器 characters 被跳过
        let mut handler = XlsxRowHandler::new(false, false);
        handler.tag_stack.push(MERGE_CELL_TAG.to_owned());
        handler.characters("x");
        handler.tag_stack.pop();
        assert!(handler.tag_stack.is_empty());
    }

    #[test]
    fn start_and_end_skip_disabled_handlers_without_pushing() {
        // 对应 Java：禁用处理器 startElement 不进栈、endElement 不弹栈
        let mut handler = XlsxRowHandler::new(false, false);
        handler.start_element("mergeCell", "ref=A1:B2");
        handler.end_element("mergeCell");
        handler.end_element("hyperlink");
        assert!(handler.tag_stack.is_empty());
    }
}
