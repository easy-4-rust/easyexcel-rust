//! Unicode 字节顺序标记（BOM）协议表。

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Unicode 文本流可识别的字节顺序标记。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrderMark {
    /// UTF-8 BOM。
    Utf8,
    /// UTF-16 大端 BOM。
    Utf16Be,
    /// UTF-16 小端 BOM。
    Utf16Le,
    /// UTF-32 大端 BOM。
    Utf32Be,
    /// UTF-32 小端 BOM。
    Utf32Le,
}

impl ByteOrderMark {
    /// 返回该标记的原始协议字节。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn bytes(self) -> &'static [u8] {
        match self {
            Self::Utf8 => &[0xEF, 0xBB, 0xBF],
            Self::Utf16Be => &[0xFE, 0xFF],
            Self::Utf16Le => &[0xFF, 0xFE],
            Self::Utf32Be => &[0x00, 0x00, 0xFE, 0xFF],
            Self::Utf32Le => &[0xFF, 0xFE, 0x00, 0x00],
        }
    }

    /// 返回与该标记对应的规范字符集名称。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn charset_name(self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf16Be => "UTF-16BE",
            Self::Utf16Le => "UTF-16LE",
            Self::Utf32Be => "UTF-32BE",
            Self::Utf32Le => "UTF-32LE",
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 根据常见 Java 字符集名称解析字节顺序标记。
    #[must_use]
    pub fn from_charset_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("UTF-8") || name.eq_ignore_ascii_case("UTF8") {
            return Some(Self::Utf8);
        }
        if name.eq_ignore_ascii_case("UTF-16BE") {
            return Some(Self::Utf16Be);
        }
        if name.eq_ignore_ascii_case("UTF-16LE") {
            return Some(Self::Utf16Le);
        }
        if name.eq_ignore_ascii_case("UTF-32BE") {
            return Some(Self::Utf32Be);
        }
        if name.eq_ignore_ascii_case("UTF-32LE") {
            return Some(Self::Utf32Le);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::ByteOrderMark;

    #[test]
    fn exposes_unicode_bom_protocol_bytes() {
        assert_eq!(ByteOrderMark::Utf8.bytes(), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(ByteOrderMark::Utf16Be.bytes(), &[0xFE, 0xFF]);
        assert_eq!(ByteOrderMark::Utf16Le.bytes(), &[0xFF, 0xFE]);
        assert_eq!(ByteOrderMark::Utf32Be.bytes(), &[0x00, 0x00, 0xFE, 0xFF]);
        assert_eq!(ByteOrderMark::Utf32Le.bytes(), &[0xFF, 0xFE, 0x00, 0x00]);
    }

    #[test]
    fn resolves_java_charset_aliases_without_allocating() {
        assert_eq!(
            ByteOrderMark::from_charset_name("utf8"),
            Some(ByteOrderMark::Utf8)
        );
        assert_eq!(
            ByteOrderMark::from_charset_name("UTF-16LE"),
            Some(ByteOrderMark::Utf16Le)
        );
        assert_eq!(ByteOrderMark::from_charset_name("GBK"), None);
    }
}
