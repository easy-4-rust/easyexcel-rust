//! CSV 字符集值对象。

/// 对应 Java：无直接对应对象；Rust 架构扩展。 CSV 读取和写入使用的字符编码。
///
/// 名称遵循 Java `Charset.forName` 约定，并接受 WHATWG 不区分大小写的别名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvCharset(String);

impl CsvCharset {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建 Java 风格字符集名称。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回配置的字符集名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回 UTF-8 默认字符集。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_name() {
        let cs = CsvCharset::new("GBK");
        assert_eq!(cs.name(), "GBK");
    }

    #[test]
    fn utf8_default() {
        let cs = CsvCharset::utf8();
        assert_eq!(cs.name(), "UTF-8");
    }

    #[test]
    fn default_is_utf8() {
        let cs = CsvCharset::default();
        assert_eq!(cs.name(), "UTF-8");
    }

    #[test]
    fn from_str_ref() {
        let cs: CsvCharset = "ISO-8859-1".into();
        assert_eq!(cs.name(), "ISO-8859-1");
    }

    #[test]
    fn from_string() {
        let cs: CsvCharset = String::from("Windows-1252").into();
        assert_eq!(cs.name(), "Windows-1252");
    }

    #[test]
    fn equality() {
        assert_eq!(CsvCharset::utf8(), CsvCharset::utf8());
        assert_ne!(CsvCharset::utf8(), CsvCharset::new("GBK"));
    }
}
