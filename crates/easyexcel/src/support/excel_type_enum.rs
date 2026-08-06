//! 对应 Java：`com.alibaba.excel.support.ExcelTypeEnum`.
//!
//! Java distinguishes three Excel types by file extension and magic bytes:
//! `XLSX` (`PK\x03\x04`), `XLS` (`D0CF11E0A1B11AE1`), and `CSV` (no magic).
//! Rust mirrors the same three variants.

/// 对应 Java：`ExcelTypeEnum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExcelTypeEnum {
    /// CSV format. (Java `CSV`)
    Csv,
    /// Legacy XLS (BIFF) format. (Java `XLS`)
    Xls,
    /// XLSX (OOXML) format. (Java `XLSX`)
    #[default]
    Xlsx,
}

impl ExcelTypeEnum {
    /// Returns the file extension. (Java `getValue()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.support.ExcelTypeEnum。
    pub const fn value(self) -> &'static str {
        match self {
            Self::Csv => ".csv",
            Self::Xls => ".xls",
            Self::Xlsx => ".xlsx",
        }
    }

    /// 对应 Java：com.alibaba.excel.support.ExcelTypeEnum。 Sniffs the type from magic bytes. (Java `recognitionExcelType(InputStream)`)
    #[must_use]
    pub fn from_magic(bytes: &[u8]) -> Self {
        match easyexcel_io::Format::from_magic(bytes) {
            easyexcel_io::Format::Xls => Self::Xls,
            easyexcel_io::Format::Xlsx => Self::Xlsx,
            _ => Self::Csv,
        }
    }

    /// 对应 Java：com.alibaba.excel.support.ExcelTypeEnum。 Sniffs the type from a file extension.
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        match easyexcel_io::Format::from_extension(extension) {
            Some(easyexcel_io::Format::Csv) if extension.eq_ignore_ascii_case("csv") => {
                Some(Self::Csv)
            }
            Some(easyexcel_io::Format::Xls) => Some(Self::Xls),
            Some(easyexcel_io::Format::Xlsx) if extension.eq_ignore_ascii_case("xlsx") => {
                Some(Self::Xlsx)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn value_returns_extensions() {
        // 对应 Java：ExcelTypeEnum.getValue
        assert_eq!(ExcelTypeEnum::Csv.value(), ".csv");
        assert_eq!(ExcelTypeEnum::Xls.value(), ".xls");
        assert_eq!(ExcelTypeEnum::Xlsx.value(), ".xlsx");
        assert_eq!(ExcelTypeEnum::default(), ExcelTypeEnum::Xlsx);
    }

    #[test]
    fn from_magic_sniffs_formats() {
        // 对应 Java：按魔数识别 XLSX/XLS，其余为 CSV
        assert_eq!(
            ExcelTypeEnum::from_magic(&[0x50, 0x4B, 0x03, 0x04, 0x00]),
            ExcelTypeEnum::Xlsx
        );
        assert_eq!(
            ExcelTypeEnum::from_magic(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]),
            ExcelTypeEnum::Xls
        );
        assert_eq!(ExcelTypeEnum::from_magic(&[0x50, 0x4B]), ExcelTypeEnum::Csv);
        assert_eq!(ExcelTypeEnum::from_magic(&[]), ExcelTypeEnum::Csv);
        assert_eq!(ExcelTypeEnum::from_magic(b"hello"), ExcelTypeEnum::Csv);
    }

    #[test]
    fn from_extension_resolves_case_insensitively() {
        // 对应 Java：按扩展名识别，大小写不敏感
        assert_eq!(
            ExcelTypeEnum::from_extension("csv"),
            Some(ExcelTypeEnum::Csv)
        );
        assert_eq!(
            ExcelTypeEnum::from_extension("XLS"),
            Some(ExcelTypeEnum::Xls)
        );
        assert_eq!(
            ExcelTypeEnum::from_extension("xlsx"),
            Some(ExcelTypeEnum::Xlsx)
        );
        assert_eq!(ExcelTypeEnum::from_extension("txt"), None);
        assert_eq!(ExcelTypeEnum::from_extension(""), None);
    }
}
