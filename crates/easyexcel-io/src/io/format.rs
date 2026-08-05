use std::path::Path;

const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// 支持的电子表格文件格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Format {
    /// Office Open XML 工作簿。
    Xlsx,
    /// BIFF8/OLE2 工作簿。
    Xls,
    /// CSV、TSV 或纯文本分隔表格。
    Csv,
}

impl Format {
    /// 根据文件扩展名识别格式。
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("xlsx" | "xlsm") => Some(Self::Xlsx),
            Some("xls") => Some(Self::Xls),
            Some("csv" | "tsv" | "txt") => Some(Self::Csv),
            _ => None,
        }
    }

    /// 根据文件头识别格式；无法识别时按 CSV 处理。
    #[must_use]
    pub fn from_magic(magic: &[u8]) -> Self {
        if looks_like_cfb(magic) {
            Self::Xls
        } else if looks_like_zip(magic) {
            Self::Xlsx
        } else {
            Self::Csv
        }
    }
}

/// 判断路径扩展名是否与给定 ASCII 名称一致（忽略大小写）。
#[must_use]
pub fn path_has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

/// 判断文件头是否为 OLE2/CFB。
#[must_use]
pub fn looks_like_cfb(magic: &[u8]) -> bool {
    magic.starts_with(&CFB_MAGIC)
}

/// 判断文件头是否为 ZIP/OOXML。
#[must_use]
pub fn looks_like_zip(magic: &[u8]) -> bool {
    magic.starts_with(b"PK\x03\x04")
        || magic.starts_with(b"PK\x05\x06")
        || magic.starts_with(b"PK\x07\x08")
}

/// 判断字节是否可能是 CSV、TSV 或其他纯文本分隔数据。
///
/// 该探测只用于选择文本解析器，不承诺验证完整 CSV 语法。
#[must_use]
pub fn looks_like_delimited_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return true;
    }
    bytes[0].is_ascii_graphic() || bytes[0].is_ascii_whitespace()
}
