//! 对应 Java：`com.alibaba.excel.enums.ByteOrderMarkEnum`.
//!
//! Maps CSV charset names to their leading BOM. Java uses
//! `org.apache.commons.io.ByteOrderMarkEnum`; the protocol table is provided
//! by `easyexcel-io` and this module retains only the Java-compatible enum.

use crate::ExcelError;

/// 对应 Java：com.alibaba.excel.enums.ByteOrderMarkEnum。 UTF BOM byte sequences aligned with Java's `ByteOrderMarkEnum`.
///
/// Rust port of Java `ByteOrderMarkEnum`. Stores the raw BOM bytes and the
/// associated canonical charset name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrderMarkEnum {
    /// UTF-8 BOM (`EF BB BF`).
    Utf8,
    /// UTF-16 big-endian BOM (`FE FF`).
    Utf16Be,
    /// UTF-16 little-endian BOM (`FF FE`).
    Utf16Le,
    /// UTF-32 big-endian BOM (`00 00 FE FF`).
    Utf32Be,
    /// UTF-32 little-endian BOM (`FF FE 00 00`).
    Utf32Le,
}

impl ByteOrderMarkEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 5] = [
        Self::Utf8,
        Self::Utf16Be,
        Self::Utf16Le,
        Self::Utf32Be,
        Self::Utf32Le,
    ];
    /// Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Utf8 => "UTF_8",
            Self::Utf16Be => "UTF_16BE",
            Self::Utf16Le => "UTF_16LE",
            Self::Utf32Be => "UTF_32BE",
            Self::Utf32Le => "UTF_32LE",
        }
    }
    /// Returns the BOM bytes as a slice.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.enums.ByteOrderMarkEnum。
    pub const fn bytes(self) -> &'static [u8] {
        self.engine_value().bytes()
    }

    /// 返回 BOM 字节。对应 Java：`getByteOrderMark()`。
    #[must_use]
    pub const fn get_byte_order_mark(self) -> &'static [u8] {
        self.bytes()
    }

    /// 返回按对应字符集解码后的 BOM 前缀。所有 Unicode BOM 均为 U+FEFF。
    /// 对应 Java：`getStringPrefix()`。
    #[must_use]
    pub const fn get_string_prefix(self) -> &'static str {
        "\u{feff}"
    }

    /// Canonical charset name matched against the BOM.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.enums.ByteOrderMarkEnum。
    pub const fn charset_name(self) -> &'static str {
        self.engine_value().charset_name()
    }

    /// 对应 Java：com.alibaba.excel.enums.ByteOrderMarkEnum。 Resolves a Java-style charset label to its BOM, if any.
    #[must_use]
    pub fn value_of_by_charset_name(name: &str) -> Option<Self> {
        easyexcel_io::ByteOrderMark::from_charset_name(name).map(Self::from_engine_value)
    }

    /// 对应 Java：com.alibaba.excel.enums.ByteOrderMarkEnum。 Returns an error explaining the BOM lookup failure (for `Result`-style callers).
    #[must_use]
    pub fn error_for_missing_bom(name: &str) -> ExcelError {
        ExcelError::Unsupported(format!("unsupported CSV charset: {name}"))
    }

    const fn engine_value(self) -> easyexcel_io::ByteOrderMark {
        match self {
            Self::Utf8 => easyexcel_io::ByteOrderMark::Utf8,
            Self::Utf16Be => easyexcel_io::ByteOrderMark::Utf16Be,
            Self::Utf16Le => easyexcel_io::ByteOrderMark::Utf16Le,
            Self::Utf32Be => easyexcel_io::ByteOrderMark::Utf32Be,
            Self::Utf32Le => easyexcel_io::ByteOrderMark::Utf32Le,
        }
    }

    const fn from_engine_value(value: easyexcel_io::ByteOrderMark) -> Self {
        match value {
            easyexcel_io::ByteOrderMark::Utf8 => Self::Utf8,
            easyexcel_io::ByteOrderMark::Utf16Be => Self::Utf16Be,
            easyexcel_io::ByteOrderMark::Utf16Le => Self::Utf16Le,
            easyexcel_io::ByteOrderMark::Utf32Be => Self::Utf32Be,
            easyexcel_io::ByteOrderMark::Utf32Le => Self::Utf32Le,
        }
    }
}

impl std::str::FromStr for ByteOrderMarkEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown ByteOrderMarkEnum value: {value}"))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn bytes_matches_java_bom_sequences() {
        // 对应 Java：ByteOrderMarkEnum 的 BOM 字节序列
        assert_eq!(ByteOrderMarkEnum::Utf8.bytes(), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(ByteOrderMarkEnum::Utf16Be.bytes(), &[0xFE, 0xFF]);
        assert_eq!(ByteOrderMarkEnum::Utf16Le.bytes(), &[0xFF, 0xFE]);
        assert_eq!(
            ByteOrderMarkEnum::Utf32Be.bytes(),
            &[0x00, 0x00, 0xFE, 0xFF]
        );
        assert_eq!(
            ByteOrderMarkEnum::Utf32Le.bytes(),
            &[0xFF, 0xFE, 0x00, 0x00]
        );
    }

    #[test]
    fn charset_name_matches_java_labels() {
        // 对应 Java：BOM 对应的规范化字符集名
        assert_eq!(ByteOrderMarkEnum::Utf8.charset_name(), "UTF-8");
        assert_eq!(ByteOrderMarkEnum::Utf16Be.charset_name(), "UTF-16BE");
        assert_eq!(ByteOrderMarkEnum::Utf16Le.charset_name(), "UTF-16LE");
        assert_eq!(ByteOrderMarkEnum::Utf32Be.charset_name(), "UTF-32BE");
        assert_eq!(ByteOrderMarkEnum::Utf32Le.charset_name(), "UTF-32LE");
    }

    #[test]
    fn value_of_by_charset_name_resolves_labels() {
        // 对应 Java：按字符集名解析 BOM，大小写不敏感
        assert_eq!(
            ByteOrderMarkEnum::value_of_by_charset_name("utf-8"),
            Some(ByteOrderMarkEnum::Utf8)
        );
        assert_eq!(
            ByteOrderMarkEnum::value_of_by_charset_name("UTF8"),
            Some(ByteOrderMarkEnum::Utf8)
        );
        assert_eq!(
            ByteOrderMarkEnum::value_of_by_charset_name("UTF-16BE"),
            Some(ByteOrderMarkEnum::Utf16Be)
        );
        assert_eq!(
            ByteOrderMarkEnum::value_of_by_charset_name("UTF-16LE"),
            Some(ByteOrderMarkEnum::Utf16Le)
        );
        assert_eq!(
            ByteOrderMarkEnum::value_of_by_charset_name("UTF-32BE"),
            Some(ByteOrderMarkEnum::Utf32Be)
        );
        assert_eq!(
            ByteOrderMarkEnum::value_of_by_charset_name("UTF-32LE"),
            Some(ByteOrderMarkEnum::Utf32Le)
        );
        assert_eq!(ByteOrderMarkEnum::value_of_by_charset_name("GBK"), None);
        assert_eq!(ByteOrderMarkEnum::value_of_by_charset_name(""), None);
    }

    #[test]
    fn error_for_missing_bom_reports_charset() {
        // 对应 Java：不支持的字符集返回明确错误
        let error = ByteOrderMarkEnum::error_for_missing_bom("GBK");
        assert!(error.to_string().contains("unsupported CSV charset: GBK"));
    }
}
