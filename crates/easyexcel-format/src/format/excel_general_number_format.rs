//! 对应 Java：`com.alibaba.excel.metadata.format.ExcelGeneralNumberFormat`.
//!
//! Java's class formats numbers in Excel's "General" format and permits
//! callers to disable scientific notation.

use crate::format::data_formatter::{
    excel_display_number, is_scientific_magnitude, java_plain_extreme_format,
    java_scientific_format,
};
use crate::{ExcelLocale, NumberFormatError};

/// Excel `General` 数字格式器。
///
/// 对应 Java：`com.alibaba.excel.metadata.format.ExcelGeneralNumberFormat`。
/// locale 与科学计数策略属于格式引擎状态，不由 EasyExcel 门面重复保存。
#[derive(Debug, Clone)]
pub struct ExcelGeneralNumberFormat {
    locale: ExcelLocale,
    use_scientific_format: bool,
}

impl ExcelGeneralNumberFormat {
    /// 创建 General 格式器。
    #[must_use]
    pub const fn new(locale: ExcelLocale, use_scientific_format: bool) -> Self {
        Self {
            locale,
            use_scientific_format,
        }
    }

    /// 格式化一个数字。
    #[must_use]
    pub fn format(&self, number: f64) -> String {
        format_general_with_options(
            number,
            self.use_scientific_format,
            self.locale.formatter().decimal_separator,
        )
    }

    /// Java `Format#parseObject` 明确不支持反向解析。
    pub fn parse_object(&self, _source: &str) -> Result<f64, NumberFormatError> {
        Err(NumberFormatError::new(
            "ExcelGeneralNumberFormat does not support parsing".to_owned(),
        ))
    }

    /// 返回区域设置。
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

/// Formats a number in Excel "General" format. (Java
/// `ExcelGeneralNumberFormat.format(Object, StringBuffer, FieldPosition)`)
#[must_use]
/// 对应 Java：com.alibaba.excel.metadata.format.ExcelGeneralNumberFormat。
pub fn format_general(value: f64) -> String {
    format_general_with_options(value, true, '.')
}

/// 使用 Java `ExcelGeneralNumberFormat` 的科学计数与 locale 选项格式化。
#[must_use]
pub fn format_general_with_options(
    value: f64,
    use_scientific_format: bool,
    decimal_separator: char,
) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let value = excel_display_number(value);
    let absolute = value.abs();
    if is_scientific_magnitude(value) {
        return if use_scientific_format {
            java_scientific_format(value, decimal_separator)
        } else {
            java_plain_extreme_format(value)
        };
    }
    if value.floor() == value || absolute >= 1E10 {
        return format!("{value:.0}");
    }

    // Java 先舍入到十位有效数字，再交给 `#.##########`。
    let integer_digits = if absolute < 1.0 {
        0
    } else {
        absolute.log10().floor().max(0.0) as usize + 1
    };
    let fraction_digits = 10_usize.saturating_sub(integer_digits).min(10);
    let mut rendered = format!("{value:.fraction_digits$}");
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
    }
    if decimal_separator != '.' {
        rendered = rendered.replace('.', &decimal_separator.to_string());
    }
    rendered
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn format_general_renders_numbers() {
        // 对应 Java：ExcelGeneralNumberFormat 常规格式
        assert_eq!(format_general(1.5), "1.5");
        assert_eq!(format_general(0.0), "0");
        assert_eq!(format_general(-42.0), "-42");
    }

    #[test]
    fn format_general_with_options_non_dot_separator() {
        let result = format_general_with_options(1.5, true, ',');
        assert!(result.contains(','));
    }

    #[test]
    fn format_general_with_options_no_scientific() {
        let result = format_general_with_options(1E11, false, '.');
        // Should use plain extreme format (no scientific notation)
        assert!(!result.contains('E'));
        assert!(!result.contains('e'));
    }

    #[test]
    fn format_general_with_options_scientific() {
        let result = format_general_with_options(1E11, true, '.');
        assert!(result.contains('E'));
    }

    #[test]
    fn format_general_with_options_infinity_and_nan() {
        assert_eq!(format_general_with_options(f64::INFINITY, true, '.'), "inf");
        assert_eq!(
            format_general_with_options(f64::NEG_INFINITY, true, '.'),
            "-inf"
        );
        assert_eq!(format_general_with_options(f64::NAN, true, '.'), "NaN");
    }

    #[test]
    fn format_general_with_options_integer() {
        assert_eq!(format_general_with_options(42.0, true, '.'), "42");
    }

    #[test]
    fn format_general_with_options_fractional() {
        let result = format_general_with_options(3.14159, true, '.');
        assert!(result.starts_with("3.14"));
    }

    #[test]
    fn format_general_with_options_trailing_zeros_stripped() {
        let result = format_general_with_options(1.10000, true, '.');
        assert_eq!(result, "1.1");
    }

    #[test]
    fn excel_general_number_format_new_and_accessors() {
        let fmt = ExcelGeneralNumberFormat::new(ExcelLocale::default(), true);
        assert!(fmt.use_scientific_format());
        assert_eq!(fmt.locale().formatter().decimal_separator, '.');
    }

    #[test]
    fn excel_general_number_format_format_delegates() {
        let fmt = ExcelGeneralNumberFormat::new(ExcelLocale::default(), true);
        assert_eq!(fmt.format(0.0), "0");
        assert_eq!(fmt.format(1.5), "1.5");
    }

    #[test]
    fn excel_general_number_format_parse_object_returns_error() {
        let fmt = ExcelGeneralNumberFormat::new(ExcelLocale::default(), true);
        assert!(fmt.parse_object("anything").is_err());
    }
}
