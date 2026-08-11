//! CSV 纯文本包装。
//!
//! 语义对应 Java：`com.alibaba.excel.metadata.csv.CsvRichTextString`。CSV
//! 无法保留字体区间，因此只保存最终文本。

/// CSV 富文本兼容值。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.metadata.csv.CsvRichTextString。
pub struct CsvRichTextString {
    value: String,
}

impl CsvRichTextString {
    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvRichTextString。 从纯文本创建值。
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvRichTextString。 返回纯文本。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Java `RichTextString#getString` 兼容入口。
    #[must_use]
    pub fn get_string(&self) -> &str {
        self.as_str()
    }

    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvRichTextString。 返回 Unicode 标量数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.value.chars().count()
    }

    /// Java `RichTextString#length` 兼容入口，按 UTF-16 单元计数。
    #[must_use]
    pub fn length(&self) -> usize {
        self.value.encode_utf16().count()
    }

    /// CSV 无法保存格式 run，固定返回零。
    #[must_use]
    pub const fn num_formatting_runs(&self) -> usize {
        0
    }

    /// CSV 无格式 run，任意查询均返回 `None`。
    #[must_use]
    pub const fn index_of_formatting_run(&self, _index: usize) -> Option<usize> {
        None
    }
    /// Java `getIndexOfFormattingRun()` returns zero for CSV.
    pub const fn get_index_of_formatting_run(&self, _index: usize) -> usize {
        0
    }
    /// Java `getFontOfFormattingRun()` has no CSV font backing.
    pub const fn get_font_of_formatting_run(&self, _index: usize) -> Option<u16> {
        None
    }

    /// CSV 对字体应用采取 Java 实现相同的 no-op 语义。
    pub const fn apply_font(&mut self, _start: usize, _end: usize, _font_index: u16) {}

    /// 清理格式在 CSV 中是 no-op。
    pub const fn clear_formatting(&mut self) {}

    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvRichTextString。 返回文本是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_unicode_text_length_and_empty_state() {
        let value = CsvRichTextString::new("你好");
        assert_eq!(value.as_str(), "你好");
        assert_eq!(value.len(), 2);
        assert!(!value.is_empty());

        let empty = CsvRichTextString::default();
        assert_eq!(empty.as_str(), "");
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn get_string_alias() {
        let value = CsvRichTextString::new("test");
        assert_eq!(value.get_string(), "test");
    }

    #[test]
    fn utf16_length() {
        // ASCII: 每字符 1 个 UTF-16 单元
        let ascii = CsvRichTextString::new("AB");
        assert_eq!(ascii.length(), 2);
        // 中文: 每字符 1 个 UTF-16 单元
        let cjk = CsvRichTextString::new("你好");
        assert_eq!(cjk.length(), 2);
        // emoji（U+1F600）: 2 个 UTF-16 单元（代理对）
        let emoji = CsvRichTextString::new("\u{1F600}");
        assert_eq!(emoji.length(), 2);
    }

    #[test]
    fn num_formatting_runs_always_zero() {
        let value = CsvRichTextString::new("test");
        assert_eq!(value.num_formatting_runs(), 0);
    }

    #[test]
    fn index_of_formatting_run_always_none() {
        let value = CsvRichTextString::new("test");
        assert_eq!(value.index_of_formatting_run(0), None);
        assert_eq!(value.index_of_formatting_run(99), None);
    }

    #[test]
    fn get_index_of_formatting_run_always_zero() {
        let value = CsvRichTextString::new("test");
        assert_eq!(value.get_index_of_formatting_run(0), 0);
    }

    #[test]
    fn get_font_of_formatting_run_always_none() {
        let value = CsvRichTextString::new("test");
        assert_eq!(value.get_font_of_formatting_run(0), None);
    }

    #[test]
    fn apply_font_is_noop() {
        let mut value = CsvRichTextString::new("test");
        value.apply_font(0, 4, 0);
        assert_eq!(value.as_str(), "test");
    }

    #[test]
    fn clear_formatting_is_noop() {
        let mut value = CsvRichTextString::new("test");
        value.clear_formatting();
        assert_eq!(value.as_str(), "test");
    }

    #[test]
    fn clone_and_eq() {
        let a = CsvRichTextString::new("hello");
        let b = a.clone();
        assert_eq!(a, b);
    }
}
