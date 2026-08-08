//! Java `ExcelGeneralNumberFormat` 兼容对象。

use crate::read::ExcelLocale;
use crate::ExcelError;

pub use easyexcel_format::{format_general, format_general_with_options};

/// Excel `General` 数字格式器。
///
/// 对应 Java：`com.alibaba.excel.metadata.format.ExcelGeneralNumberFormat`。
#[derive(Debug, Clone)]
pub struct ExcelGeneralNumberFormat {
    locale: ExcelLocale,
    use_scientific_format: bool,
}

impl ExcelGeneralNumberFormat {
    /// 对应 Java 构造器 `ExcelGeneralNumberFormat(Locale, boolean)`。
    #[must_use]
    pub const fn new(locale: ExcelLocale, use_scientific_format: bool) -> Self {
        Self {
            locale,
            use_scientific_format,
        }
    }

    /// 格式化数值。对应 Java `format(Object, StringBuffer, FieldPosition)` 的
    /// 数字分支；返回值替代可变 `StringBuffer`。
    #[must_use]
    pub fn format(&self, number: f64) -> String {
        format_general_with_options(
            number,
            self.use_scientific_format,
            self.locale.formatter().decimal_separator,
        )
    }

    /// Java `Format#parseObject` 明确不支持反向解析。
    pub fn parse_object(&self, _source: &str) -> Result<f64, ExcelError> {
        Err(ExcelError::Unsupported(
            "ExcelGeneralNumberFormat does not support parsing".to_owned(),
        ))
    }

    /// 返回配置的 locale。
    #[must_use]
    pub const fn locale(&self) -> &ExcelLocale {
        &self.locale
    }

    /// 返回是否启用科学计数法。
    #[must_use]
    pub const fn use_scientific_format(&self) -> bool {
        self.use_scientific_format
    }
}
