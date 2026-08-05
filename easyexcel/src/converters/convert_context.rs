//! 对应 Java：`com.alibaba.excel.metadata.property.ExcelContentProperty` and
//! `com.alibaba.excel.metadata.GlobalConfiguration` (subset).

use crate::core::excel_error::ExcelError;

/// Location and formatting information supplied to cell converters.
///
/// Java's `ReadConverterContext` and `WriteConverterContext` carry
/// `contentProperty` (resolved annotation) plus `analysisContext` or
/// `writeContext`. Rust collapses them into a single `Copy` value so each
/// cell conversion can pass it by reference without ownership fuss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertContext {
    /// Sheet name. (Java `AnalysisContext.readSheetHolder().getSheetName()`)
    pub sheet_name: String,
    /// Zero-based row index. (Java `AnalysisContext.readRowHolder().getRowIndex()`)
    pub row_index: u32,
    /// Zero-based column index when it can be resolved.
    pub column_index: Option<usize>,
    /// Rust field name. (Java `ExcelContentProperty.getField().getName()`)
    pub field: &'static str,
    /// 已废弃的 `ExcelProperty.format` 兼容值。
    pub format: Option<&'static str>,
    /// Java `DateTimeFormatProperty.format`。
    pub date_time_format: Option<&'static str>,
    /// Java `NumberFormatProperty.format`。
    pub number_format: Option<&'static str>,
    /// Whether numeric dates use Excel's 1904 date system.
    /// (Java `GlobalConfiguration.getUse1904windowing()`)
    pub use_1904_windowing: bool,
}

impl ConvertContext {
    /// 返回日期转换应使用的格式。
    #[must_use]
    pub const fn effective_date_time_format(&self) -> Option<&'static str> {
        match self.date_time_format {
            Some(format) => Some(format),
            None => self.format,
        }
    }

    /// 返回数字转换应使用的格式。
    #[must_use]
    pub const fn effective_number_format(&self) -> Option<&'static str> {
        match self.number_format {
            Some(format) => Some(format),
            None => self.format,
        }
    }

    /// Builds a typed conversion error matching Java `ExcelDataConvertException`.
    pub(crate) fn invalid(
        &self,
        value: &crate::core::cell_value::CellValue,
        target: &'static str,
    ) -> ExcelError {
        ExcelError::Data {
            sheet: self.sheet_name.clone(),
            row: self.row_index,
            column: self.column_index,
            field: self.field,
            value: value.as_text(),
            message: format!("cannot convert cell to {target}"),
        }
    }

    /// Attaches this field's conversion location to an arbitrary converter error.
    ///
    /// Java wraps every exception raised by `Converter.convertToExcelData` in
    /// `ExcelWriteDataConvertException`, so even converters that return a
    /// generic error retain the field, row, and column from the active cell
    /// handler context. Derive-generated Rust writers use this helper before
    /// the backend replaces the provisional row/column with their physical
    /// worksheet coordinates.
    #[must_use]
    pub fn write_error(&self, error: ExcelError) -> ExcelError {
        let (value, message) = match error {
            ExcelError::Data { value, message, .. } => (value, message),
            other => (String::new(), other.to_string()),
        };
        ExcelError::Data {
            sheet: self.sheet_name.clone(),
            row: self.row_index,
            column: self.column_index,
            field: self.field,
            value,
            message,
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    fn sample_context() -> ConvertContext {
        ConvertContext {
            sheet_name: "Data".to_owned(),
            row_index: 3,
            column_index: Some(1),
            field: "value",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        }
    }

    #[test]
    fn write_error_wraps_generic_error_with_location() {
        // 对应 Java：ExcelWriteDataConvertException 包装通用错误
        let context = sample_context();
        let error = context.write_error(ExcelError::Format("boom".to_owned()));
        // 守卫断言替代 match 兜底 panic 臂（write_error 恒构造 Data 变体，other 臂数学不可达；
        // 解构失败时 assert! 仍然失败，测试不会静默放行）。
        assert!(
            matches!(
                &error,
                ExcelError::Data {
                    sheet,
                    row,
                    column,
                    field,
                    value,
                    message,
                } if sheet == "Data"
                    && *row == 3
                    && *column == Some(1)
                    && *field == "value"
                    && value.is_empty()
                    && message.contains("boom")
            ),
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn write_error_keeps_data_error_fields() {
        // 对应 Java：Data 错误保留原值与消息
        let context = sample_context();
        let original = ExcelError::Data {
            sheet: "ignored".to_owned(),
            row: 9,
            column: None,
            field: "other",
            value: "abc".to_owned(),
            message: "convert failed".to_owned(),
        };
        let error = context.write_error(original);
        // 守卫断言替代 match 兜底 panic 臂（同 write_error_wraps_generic_error_with_location）。
        assert!(
            matches!(
                &error,
                ExcelError::Data {
                    sheet,
                    row,
                    column,
                    field,
                    value,
                    message,
                } if sheet == "Data"
                    && *row == 3
                    && *column == Some(1)
                    && *field == "value"
                    && value == "abc"
                    && message == "convert failed"
            ),
            "unexpected error: {error:?}"
        );
    }
}
