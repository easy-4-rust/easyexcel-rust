//! 对应 Java：`com.alibaba.excel.write.metadata.WriteSheet`.

use crate::WriteOptions;
use crate::write::metadata::WriteBasicParameter;

/// 对应 Java：`WriteSheet extends WriteBasicParameter`.
///
/// Java stores `sheetNo` and `sheetName`. Rust reuses [`WriteOptions`] and
/// extends the type with the two fields so 1:1 naming is preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSheet {
    /// Mirrors `WriteSheet.sheetNo`.
    pub sheet_no: i32,
    /// Mirrors `WriteSheet.sheetName`.
    pub sheet_name: String,
    /// Mirrors the remaining `WriteBasicParameter` fields.
    pub options: WriteOptions,
    /// Nullable sheet-level overrides before workbook inheritance.
    pub parameter: WriteBasicParameter,
}

impl WriteSheet {
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。 Creates a `WriteSheet` matching Java `new WriteSheet()`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sheet_no: 0,
            sheet_name: String::new(),
            options: WriteOptions::default(),
            parameter: WriteBasicParameter::default(),
        }
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。 Creates a `WriteSheet` with the given sheet no. (Java `WriteSheet(sheetNo)`)
    #[must_use]
    pub fn with_sheet_no(sheet_no: i32) -> Self {
        Self {
            sheet_no,
            sheet_name: String::new(),
            options: WriteOptions::default(),
            parameter: WriteBasicParameter::default(),
        }
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。 Creates a `WriteSheet` with the given sheet no and name. (Java `WriteSheet(sheetNo, sheetName)`)
    #[must_use]
    pub fn with_sheet(sheet_no: i32, sheet_name: impl Into<String>) -> Self {
        Self {
            sheet_no,
            sheet_name: sheet_name.into(),
            options: WriteOptions::default(),
            parameter: WriteBasicParameter::default(),
        }
    }

    /// Returns the zero-based sheet index. (Java `getSheetNo()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。
    pub const fn sheet_no(&self) -> i32 {
        self.sheet_no
    }
    /// Java `getSheetNo` 别名。
    #[must_use] pub const fn get_sheet_no(&self) -> i32 { self.sheet_no }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。 Sets the zero-based sheet index. (Java `setSheetNo(Integer)`)
    pub fn set_sheet_no(&mut self, sheet_no: i32) -> &mut Self {
        self.sheet_no = sheet_no;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。 Returns the sheet name. (Java `getSheetName()`)
    #[must_use]
    pub fn sheet_name(&self) -> &str {
        &self.sheet_name
    }
    /// Java `getSheetName` 别名。
    #[must_use] pub fn get_sheet_name(&self) -> &str { &self.sheet_name }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。 Sets the sheet name. (Java `setSheetName(String)`)
    pub fn set_sheet_name(&mut self, sheet_name: impl Into<String>) -> &mut Self {
        self.sheet_name = sheet_name.into();
        self
    }

    /// Returns the shared write options.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。
    pub const fn options(&self) -> &WriteOptions {
        &self.options
    }
    /// 替换共享写入选项。
    pub fn set_options(&mut self, value: WriteOptions) { self.options = value; }
    /// 返回可变共享写入选项。
    pub const fn options_mut(&mut self) -> &mut WriteOptions { &mut self.options }

    /// Returns nullable sheet-level overrides before workbook inheritance.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteSheet。
    pub const fn parameter(&self) -> &WriteBasicParameter {
        &self.parameter
    }
    /// 替换 Java 父类参数。
    pub fn set_parameter(&mut self, value: WriteBasicParameter) { self.parameter = value; }
    /// 返回可变 Java 父类参数。
    pub const fn parameter_mut(&mut self) -> &mut WriteBasicParameter { &mut self.parameter }
}

impl Default for WriteSheet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_sheet_new_defaults() {
        let sheet = WriteSheet::new();
        assert_eq!(sheet.sheet_no(), 0);
        assert_eq!(sheet.sheet_name(), "");
    }

    #[test]
    fn write_sheet_default_impl() {
        let sheet = WriteSheet::default();
        assert_eq!(sheet.sheet_no(), 0);
    }

    #[test]
    fn write_sheet_with_sheet_no() {
        let sheet = WriteSheet::with_sheet_no(3);
        assert_eq!(sheet.sheet_no(), 3);
        assert_eq!(sheet.sheet_name(), "");
    }

    #[test]
    fn write_sheet_with_sheet() {
        let sheet = WriteSheet::with_sheet(2, "MySheet");
        assert_eq!(sheet.sheet_no(), 2);
        assert_eq!(sheet.sheet_name(), "MySheet");
    }

    #[test]
    fn write_sheet_set_sheet_no() {
        let mut sheet = WriteSheet::new();
        sheet.set_sheet_no(5);
        assert_eq!(sheet.sheet_no(), 5);
    }

    #[test]
    fn write_sheet_set_sheet_name() {
        let mut sheet = WriteSheet::new();
        sheet.set_sheet_name("NewSheet");
        assert_eq!(sheet.sheet_name(), "NewSheet");
    }

    #[test]
    fn write_sheet_options_accessor() {
        let sheet = WriteSheet::new();
        let _opts = sheet.options();
    }

    #[test]
    fn write_sheet_parameter_accessor() {
        let sheet = WriteSheet::new();
        let _param = sheet.parameter();
    }

    #[test]
    fn write_sheet_equality() {
        let a = WriteSheet::new();
        let b = WriteSheet::new();
        assert_eq!(a, b);
    }

    #[test]
    fn write_sheet_clone() {
        let original = WriteSheet::with_sheet(1, "Clone");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
