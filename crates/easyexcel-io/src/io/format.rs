use std::io::Read;
use std::path::Path;

use crate::Result;

const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// 对应 Java：无直接对应对象；Rust 架构扩展。 支持的电子表格文件格式。
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
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 根据不含点号的文件扩展名识别格式。
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "xlsx" | "xlsm" => Some(Self::Xlsx),
            "xls" => Some(Self::Xls),
            "csv" | "tsv" | "txt" => Some(Self::Csv),
            _ => None,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 根据文件扩展名识别格式。
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        Self::from_extension(
            path.extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or_default(),
        )
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按 Java `EasyExcel` 的 `ExcelTypeEnum.valueOf(ReadWorkbook)` 规则识别路径格式。
    ///
    /// 已知扩展名直接决定格式；未知或无扩展名时读取最多八个文件头字节，
    /// `XLSX`/`XLS` 按 magic 识别，其余内容默认作为 `CSV`。该入口同时保证需要
    /// 探测文件头时路径可读。
    ///
    /// # Errors
    ///
    /// 未知扩展名需要读取文件头，而路径无法打开或读取时返回 I/O 错误。
    pub fn detect_path(path: &Path) -> Result<Self> {
        if let Some(format) = Self::from_path(path) {
            return Ok(format);
        }
        let mut file = std::fs::File::open(path)?;
        let mut magic = [0_u8; 8];
        let read = file.read(&mut magic)?;
        Ok(Self::from_magic(&magic[..read]))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 根据文件头识别格式；无法识别时按 CSV 处理。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断路径扩展名是否与给定 ASCII 名称一致（忽略大小写）。
#[must_use]
pub fn path_has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断文件头是否为 OLE2/CFB。
#[must_use]
pub fn looks_like_cfb(magic: &[u8]) -> bool {
    magic.starts_with(&CFB_MAGIC)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断文件头是否为 ZIP/OOXML。
#[must_use]
pub fn looks_like_zip(magic: &[u8]) -> bool {
    magic.starts_with(b"PK\x03\x04")
        || magic.starts_with(b"PK\x05\x06")
        || magic.starts_with(b"PK\x07\x08")
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断字节是否可能是 CSV、TSV 或其他纯文本分隔数据。
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn unknown_extension_uses_magic_and_defaults_to_csv() {
        let mut xls = tempfile::Builder::new()
            .suffix(".unknown")
            .tempfile()
            .expect("temp xls");
        xls.write_all(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
            .expect("magic");
        assert_eq!(
            Format::detect_path(xls.path()).expect("detect"),
            Format::Xls
        );

        let mut csv = tempfile::Builder::new()
            .suffix(".unknown")
            .tempfile()
            .expect("temp csv");
        csv.write_all(b"a,b\n1,2\n").expect("csv");
        assert_eq!(
            Format::detect_path(csv.path()).expect("detect"),
            Format::Csv
        );
    }

    #[test]
    fn from_extension_resolves_known_types() {
        assert_eq!(Format::from_extension("xlsx"), Some(Format::Xlsx));
        assert_eq!(Format::from_extension("XLSX"), Some(Format::Xlsx));
        assert_eq!(Format::from_extension("xlsm"), Some(Format::Xlsx));
        assert_eq!(Format::from_extension("xls"), Some(Format::Xls));
        assert_eq!(Format::from_extension("XLS"), Some(Format::Xls));
        assert_eq!(Format::from_extension("csv"), Some(Format::Csv));
        assert_eq!(Format::from_extension("CSV"), Some(Format::Csv));
        assert_eq!(Format::from_extension("tsv"), Some(Format::Csv));
        assert_eq!(Format::from_extension("txt"), Some(Format::Csv));
        assert_eq!(Format::from_extension("pdf"), None);
        assert_eq!(Format::from_extension(""), None);
    }

    #[test]
    fn from_path_extracts_extension() {
        assert_eq!(
            Format::from_path(std::path::Path::new("test.xlsx")),
            Some(Format::Xlsx)
        );
        assert_eq!(
            Format::from_path(std::path::Path::new("test.xls")),
            Some(Format::Xls)
        );
        assert_eq!(
            Format::from_path(std::path::Path::new("test.csv")),
            Some(Format::Csv)
        );
        assert_eq!(Format::from_path(std::path::Path::new("test")), None);
        assert_eq!(Format::from_path(std::path::Path::new("test.pdf")), None);
    }

    #[test]
    fn from_magic_detects_xlsx_zip() {
        assert_eq!(Format::from_magic(b"PK\x03\x04extra"), Format::Xlsx);
        assert_eq!(Format::from_magic(b"PK\x05\x06"), Format::Xlsx);
        assert_eq!(Format::from_magic(b"PK\x07\x08"), Format::Xlsx);
    }

    #[test]
    fn from_magic_detects_xls_cfb() {
        assert_eq!(
            Format::from_magic(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]),
            Format::Xls
        );
    }

    #[test]
    fn from_magic_defaults_to_csv() {
        assert_eq!(Format::from_magic(b"hello,world"), Format::Csv);
        assert_eq!(Format::from_magic(&[]), Format::Csv);
    }

    #[test]
    fn path_has_extension_checks_case_insensitively() {
        assert!(path_has_extension(
            std::path::Path::new("test.xlsx"),
            "xlsx"
        ));
        assert!(path_has_extension(
            std::path::Path::new("test.XLSX"),
            "xlsx"
        ));
        assert!(!path_has_extension(
            std::path::Path::new("test.xls"),
            "xlsx"
        ));
        assert!(!path_has_extension(std::path::Path::new("test"), "xlsx"));
    }

    #[test]
    fn looks_like_cfb_identifies_ole2() {
        assert!(looks_like_cfb(&[
            0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1
        ]));
        assert!(!looks_like_cfb(b"PK\x03\x04"));
        assert!(!looks_like_cfb(&[]));
    }

    #[test]
    fn looks_like_zip_identifies_pk_signatures() {
        assert!(looks_like_zip(b"PK\x03\x04rest"));
        assert!(looks_like_zip(b"PK\x05\x06"));
        assert!(looks_like_zip(b"PK\x07\x08"));
        assert!(!looks_like_zip(b"notzip"));
        assert!(!looks_like_zip(&[]));
    }

    #[test]
    fn looks_like_delimited_text_accepts_ascii_and_bom() {
        assert!(looks_like_delimited_text(b"a,b,c"));
        assert!(looks_like_delimited_text(b"1\t2\t3"));
        assert!(looks_like_delimited_text(b" hello"));
        assert!(looks_like_delimited_text(&[0xEF, 0xBB, 0xBF, b'a']));
        assert!(!looks_like_delimited_text(&[]));
        // Binary data with non-ASCII first byte
        assert!(!looks_like_delimited_text(&[0x80, 0x01, 0x02]));
    }

    #[test]
    fn detect_path_with_known_extension_skips_magic() {
        let mut file = tempfile::Builder::new()
            .suffix(".xlsx")
            .tempfile()
            .expect("temp xlsx");
        file.write_all(b"not really xlsx").expect("write");
        assert_eq!(
            Format::detect_path(file.path()).expect("detect"),
            Format::Xlsx
        );
    }

    #[test]
    fn detect_path_xls_magic() {
        let mut file = tempfile::Builder::new()
            .suffix(".unknown")
            .tempfile()
            .expect("temp");
        file.write_all(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
            .expect("write");
        assert_eq!(
            Format::detect_path(file.path()).expect("detect"),
            Format::Xls
        );
    }
}
