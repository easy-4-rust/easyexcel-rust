//! 对应 Java： SAX `ContentHandler` for XLSX tag dispatch.

/// XLSX 解析标签处理器：按 XML 标签分派 SAX 事件，对应 Java `ContentHandler`。
pub trait XlsxTagHandler {
    /// 判断当前处理器是否支持该标签，默认返回 `true`。
    fn support(&self) -> bool {
        true
    }
    /// 处理元素开始事件，默认实现忽略参数。
    fn start_element(&mut self, name: &str, attrs: &str) {
        let _ = (name, attrs);
    }
    /// 处理元素结束事件，默认实现忽略参数。
    fn end_element(&mut self, name: &str) {
        let _ = name;
    }
    /// 处理元素文本内容，默认实现忽略参数。
    fn characters(&mut self, ch: &str) {
        let _ = ch;
    }

    /// Java `support(XlsxReadContext)` 的上下文感知入口。
    ///
    /// 默认桥接现有无上下文实现，使自定义 Handler 可以逐步迁移而不丢失动态分派。
    fn support_with_context(&self, context: &dyn crate::XlsxReadContext) -> bool {
        let _ = context;
        self.support()
    }

    /// Java `startElement(XlsxReadContext, String, Attributes)` 的上下文感知入口。
    fn start_element_with_context(
        &mut self,
        context: &dyn crate::XlsxReadContext,
        name: &str,
        attrs: &str,
    ) {
        let _ = context;
        self.start_element(name, attrs);
    }

    /// Java `endElement(XlsxReadContext, String)` 的上下文感知入口。
    fn end_element_with_context(&mut self, context: &dyn crate::XlsxReadContext, name: &str) {
        let _ = context;
        self.end_element(name);
    }

    /// Java `characters(XlsxReadContext, char[], int, int)`，严格按字符而非 UTF-8 字节切片。
    fn characters_with_context(
        &mut self,
        context: &dyn crate::XlsxReadContext,
        ch: &[char],
        start: usize,
        length: usize,
    ) {
        let _ = context;
        if start >= ch.len() || length == 0 {
            return;
        }
        let end = start.saturating_add(length).min(ch.len());
        let value: String = ch[start..end].iter().collect();
        self.characters(&value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_trait_methods_are_noops() {
        // 对应 Java：ContentHandler 默认空实现
        struct DefaultHandler;
        impl XlsxTagHandler for DefaultHandler {}

        let mut handler = DefaultHandler;
        assert!(handler.support());
        handler.start_element("any", "k=v");
        handler.end_element("any");
        handler.characters("text");
    }
}
