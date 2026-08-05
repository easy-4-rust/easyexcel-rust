//! CSV 纯文本包装。
//!
//! 语义对应 Java：`com.alibaba.excel.metadata.csv.CsvRichTextString`。CSV
//! 无法保留字体区间，因此只保存最终文本。

/// CSV 富文本兼容值。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CsvRichTextString {
    value: String,
}

impl CsvRichTextString {
    /// 从纯文本创建值。
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// 返回纯文本。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// 返回 Unicode 标量数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.value.chars().count()
    }

    /// 返回文本是否为空。
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
}
