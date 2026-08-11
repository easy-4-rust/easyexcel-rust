//! 对应 Java：`com.alibaba.excel.read.metadata.holder.csv.CsvReadWorkbookHolder`.

use crate::read::holder::read_workbook_holder::ReadWorkbookHolder;
use crate::read::metadata::holder::read_holder::delegate_read_holder_contract;
use std::ops::{Deref, DerefMut};

/// 对应 Java：`CsvReadWorkbookHolder extends ReadWorkbookHolder`.
#[derive(Debug, Clone)]
pub struct CsvReadWorkbookHolder {
    inner: ReadWorkbookHolder,
    csv_format: String,
    csv_parser_initialized: bool,
}

impl CsvReadWorkbookHolder {
    /// 对应 Java： constructor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ReadWorkbookHolder::default(),
            csv_format: "DEFAULT".to_owned(),
            csv_parser_initialized: false,
        }
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.holder.csv.CsvReadWorkbookHolder。 Creates the format-specific holder from resolved workbook options.
    #[must_use]
    pub fn from_options(options: &crate::ReadOptions) -> Self {
        let mut value = Self {
            inner: ReadWorkbookHolder::from_options(options),
            csv_format: "DEFAULT".to_owned(),
            csv_parser_initialized: false,
        };
        value
            .inner
            .set_excel_type(Some(crate::support::ExcelTypeEnum::Csv));
        value
    }

    /// Java `CsvReadWorkbookHolder(ReadWorkbook)`。
    #[must_use]
    pub fn from_read_workbook(read_workbook: crate::ReadWorkbook) -> Self {
        let mut value = Self::new();
        value.inner = ReadWorkbookHolder::from_read_workbook(read_workbook);
        value
            .inner
            .set_excel_type(Some(crate::support::ExcelTypeEnum::Csv));
        value
    }

    /// 返回后端中立 CSV 格式描述。
    #[must_use]
    pub fn get_csv_format(&self) -> &str {
        &self.csv_format
    }
    /// 设置后端中立 CSV 格式描述。
    pub fn set_csv_format(&mut self, value: impl Into<String>) {
        self.csv_format = value.into();
    }
    /// 返回 CSV parser 是否已建立。
    #[must_use]
    pub const fn get_csv_parser(&self) -> bool {
        self.csv_parser_initialized
    }
    /// 设置 CSV parser 生命周期状态。
    pub const fn set_csv_parser(&mut self, value: bool) {
        self.csv_parser_initialized = value;
    }

    /// Returns the inner holder.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.csv.CsvReadWorkbookHolder。
    pub const fn inner(&self) -> &ReadWorkbookHolder {
        &self.inner
    }
}

impl Deref for CsvReadWorkbookHolder {
    type Target = ReadWorkbookHolder;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for CsvReadWorkbookHolder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

delegate_read_holder_contract!(CsvReadWorkbookHolder, inner);

impl Default for CsvReadWorkbookHolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_holder_constructors_and_inner_access() {
        // 对应 Java：CsvReadWorkbookHolder 构造与 inner 访问器
        let holder = CsvReadWorkbookHolder::new();
        assert!(
            !holder.inner().ignore_empty_row,
            "derive Default 初始为 false"
        );

        let options = crate::ReadOptions {
            ignore_empty_row: false,
            ..crate::ReadOptions::default()
        };
        let from_options = CsvReadWorkbookHolder::from_options(&options);
        assert!(!from_options.inner().ignore_empty_row);
        assert_eq!(from_options.inner().charset, options.charset);
        assert_eq!(from_options.inner().password, options.password);
        let default_from_options =
            CsvReadWorkbookHolder::from_options(&crate::ReadOptions::default());
        assert!(default_from_options.inner().ignore_empty_row);

        let defaulted = CsvReadWorkbookHolder::default();
        assert!(defaulted.inner().auto_close_stream);
    }
}
