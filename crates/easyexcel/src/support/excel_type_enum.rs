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
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 3] = [Self::Csv, Self::Xls, Self::Xlsx];
    /// Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Csv => "CSV",
            Self::Xls => "XLS",
            Self::Xlsx => "XLSX",
        }
    }
    /// 返回文件魔数。CSV 没有固定魔数。对应 Java：`getMagic()`。
    #[must_use]
    pub const fn magic(self) -> &'static [u8] {
        match self {
            Self::Xlsx => &[0x50, 0x4b, 0x03, 0x04],
            Self::Xls => &[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1],
            Self::Csv => &[],
        }
    }

    /// Java `getMagic()` 兼容别名。
    #[must_use]
    pub const fn get_magic(self) -> &'static [u8] {
        self.magic()
    }

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

    /// 解析 Java `ExcelTypeEnum.valueOf(ReadWorkbook)`。
    ///
    /// 显式 `excelType` 优先；无密码文件先按 Java 的小写扩展名判断，随后复用
    /// `easyexcel-io` 魔数探测；输入流已由 Rust 门面物化为字节，因此无需修改
    /// 调用方 stream 的 mark/reset 状态。
    ///
    /// # Errors
    ///
    /// 文件与输入字节均缺失，或文件无法读取时返回可见的格式错误。
    pub fn value_of(read_workbook: &crate::read::metadata::ReadWorkbook) -> crate::Result<Self> {
        if let Some(excel_type) = read_workbook.get_excel_type() {
            return Ok(excel_type);
        }
        if let Some(file) = read_workbook.get_file() {
            if read_workbook.get_password().is_none() {
                let name = file
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                if name.ends_with(Self::Xlsx.value()) {
                    return Ok(Self::Xlsx);
                }
                if name.ends_with(Self::Xls.value()) {
                    return Ok(Self::Xls);
                }
                if name.ends_with(Self::Csv.value()) {
                    return Ok(Self::Csv);
                }
            }
            return match easyexcel_io::Format::detect_path(file).map_err(crate::ExcelError::from)? {
                easyexcel_io::Format::Xlsx => Ok(Self::Xlsx),
                easyexcel_io::Format::Xls => Ok(Self::Xls),
                easyexcel_io::Format::Csv => Ok(Self::Csv),
                _ => Err(crate::ExcelError::Format(
                    "Convert excel format exception.You can try specifying the 'excelType' yourself".to_owned(),
                )),
            };
        }
        if let Some(input) = read_workbook.get_input_stream() {
            return Ok(Self::from_magic(input));
        }
        Err(crate::ExcelError::Format(
            "File and inputStream must be a non-null.".to_owned(),
        ))
    }
}

impl std::str::FromStr for ExcelTypeEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown ExcelTypeEnum value: {value}"))
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

    #[test]
    fn java_name_returns_enum_constant_names() {
        assert_eq!(ExcelTypeEnum::Csv.java_name(), "CSV");
        assert_eq!(ExcelTypeEnum::Xls.java_name(), "XLS");
        assert_eq!(ExcelTypeEnum::Xlsx.java_name(), "XLSX");
    }

    #[test]
    fn magic_returns_correct_bytes() {
        assert_eq!(ExcelTypeEnum::Xlsx.magic(), &[0x50, 0x4b, 0x03, 0x04]);
        assert_eq!(
            ExcelTypeEnum::Xls.magic(),
            &[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]
        );
        assert!(ExcelTypeEnum::Csv.magic().is_empty());
    }

    #[test]
    fn get_magic_is_alias_for_magic() {
        assert_eq!(ExcelTypeEnum::Xlsx.get_magic(), ExcelTypeEnum::Xlsx.magic());
        assert_eq!(ExcelTypeEnum::Xls.get_magic(), ExcelTypeEnum::Xls.magic());
    }

    #[test]
    fn all_contains_three_variants() {
        assert_eq!(ExcelTypeEnum::ALL.len(), 3);
        assert!(ExcelTypeEnum::ALL.contains(&ExcelTypeEnum::Csv));
        assert!(ExcelTypeEnum::ALL.contains(&ExcelTypeEnum::Xls));
        assert!(ExcelTypeEnum::ALL.contains(&ExcelTypeEnum::Xlsx));
    }

    #[test]
    fn from_str_parses_java_names() {
        use std::str::FromStr;
        assert_eq!(ExcelTypeEnum::from_str("CSV").unwrap(), ExcelTypeEnum::Csv);
        assert_eq!(ExcelTypeEnum::from_str("XLS").unwrap(), ExcelTypeEnum::Xls);
        assert_eq!(
            ExcelTypeEnum::from_str("XLSX").unwrap(),
            ExcelTypeEnum::Xlsx
        );
        assert!(ExcelTypeEnum::from_str("UNKNOWN").is_err());
        assert!(ExcelTypeEnum::from_str("").is_err());
    }

    #[test]
    fn value_of_with_input_stream() {
        let mut workbook = crate::ReadWorkbook::new();
        workbook.set_input_stream(Some(vec![0x50, 0x4B, 0x03, 0x04, 0x00]));
        let result = ExcelTypeEnum::value_of(&workbook).unwrap();
        assert_eq!(result, ExcelTypeEnum::Xlsx);
    }

    #[test]
    fn value_of_returns_error_without_file_or_stream() {
        let workbook = crate::ReadWorkbook::new();
        assert!(ExcelTypeEnum::value_of(&workbook).is_err());
    }

    #[test]
    fn value_of_prefers_explicit_type() {
        let mut workbook = crate::ReadWorkbook::new();
        workbook.set_excel_type(crate::support::ExcelTypeEnum::Xls);
        workbook.set_input_stream(Some(vec![0x50, 0x4B, 0x03, 0x04]));
        let result = ExcelTypeEnum::value_of(&workbook).unwrap();
        assert_eq!(result, ExcelTypeEnum::Xls);
    }
}
