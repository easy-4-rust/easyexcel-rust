//! Mirrors Java SAX ContentHandler for XLSX tag dispatch.

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
}
