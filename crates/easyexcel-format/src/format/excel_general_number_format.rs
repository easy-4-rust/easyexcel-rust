//! 对应 Java：`com.alibaba.excel.metadata.format.ExcelGeneralNumberFormat`.
//!
//! Java's 81-line class formats numbers in Excel's "General" format.
//! Rust delegates to `ssfmt::format` with format code `"General"`.

/// Formats a number in Excel "General" format. (Java
/// `ExcelGeneralNumberFormat.format(Object, StringBuffer, FieldPosition)`)
#[allow(dead_code)]
#[must_use]
/// 对应 Java：com.alibaba.excel.metadata.format.ExcelGeneralNumberFormat。
pub fn format_general(value: f64) -> String {
    format!("{value}")
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
}
