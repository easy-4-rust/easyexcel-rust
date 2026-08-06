//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.AbstractXlsxTagHandler`.

use super::xlsx_tag_handler::XlsxTagHandler;

/// 对应 Java：`AbstractXlsxTagHandler implements XlsxTagHandler`.
///
/// Java provides default no-op implementations for all four methods
/// (`support` / `startElement` / `endElement` / `characters`). Rust mirrors
/// the same pattern via trait defaults on [`XlsxTagHandler`].
#[derive(Debug, Default)]
pub struct AbstractXlsxTagHandler;

impl AbstractXlsxTagHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.AbstractXlsxTagHandler。 Creates the abstract base (rarely constructed on its own).
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl XlsxTagHandler for AbstractXlsxTagHandler {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abstract_base_is_constructible_and_noop() {
        // 对应 Java：AbstractXlsxTagHandler 直接构造并继承默认实现
        let mut handler = AbstractXlsxTagHandler::new();
        assert!(handler.support());
        handler.start_element("c", "r=A1");
        handler.end_element("c");
        handler.characters("v");
    }
}
