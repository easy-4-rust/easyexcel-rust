//! Mirrors Java `com.alibaba.excel.metadata.csv.CsvRichTextString`.

/// CSV rich text wrapper.
///
/// CSV cannot preserve font runs, so Java stores only the plain string and
/// makes formatting methods inert. Rust exposes the same meaningful state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CsvRichTextString {
    value: String,
}

impl CsvRichTextString {
    /// Creates a CSV rich-text value from plain text.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Returns the plain text. (Java `getString()`)
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Returns the UTF-8 text length in Unicode scalar values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.value.chars().count()
    }

    /// Returns whether the text is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_as_str_len_and_is_empty() {
        // 对应 Java：CsvRichTextString 纯文本包装
        let value = CsvRichTextString::new("你好");
        assert_eq!(value.as_str(), "你好");
        assert_eq!(value.len(), 2);
        assert!(!value.is_empty());

        let empty = CsvRichTextString::new("");
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
        assert_eq!(CsvRichTextString::default(), empty);
    }
}
