//! CSV 字符集值对象。

/// CSV 读取和写入使用的字符编码。
///
/// 名称遵循 Java `Charset.forName` 约定，并接受 WHATWG 不区分大小写的别名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvCharset(String);

impl CsvCharset {
    /// 创建 Java 风格字符集名称。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// 返回配置的字符集名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }

    /// 返回 UTF-8 默认字符集。
    #[must_use]
    pub fn utf8() -> Self {
        Self("UTF-8".to_owned())
    }
}

impl Default for CsvCharset {
    fn default() -> Self {
        Self::utf8()
    }
}

impl From<&str> for CsvCharset {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for CsvCharset {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
