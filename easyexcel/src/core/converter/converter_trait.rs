//! Mirrors `com.alibaba.excel.converters.Converter<T>` public surface.
//!
//! Java exposes four default methods:
//! * `convertToJavaData(ReadCellData, ExcelContentProperty, GlobalConfiguration)`
//! * `convertToJavaData(ReadConverterContext)`
//! * `convertToExcelData(T, ExcelContentProperty, GlobalConfiguration)`
//! * `convertToExcelData(WriteConverterContext)`
//!
//! plus `supportJavaTypeKey` / `supportExcelTypeKey`.
//!
//! Rust keeps `support_excel_type` (used as the read dispatch key together
//! with `TypeId`) and the two conversion methods. `supportJavaTypeKey` is
//! implicit in the generic parameter `T`.

use crate::core::enum_cell_data_type::CellDataType;
use crate::core::excel_error::ExcelError;
use crate::core::read_converter_context::ReadConverterContext;
use crate::core::write_cell_data::WriteCellData;
use crate::core::write_converter_context::WriteConverterContext;

/// Custom bidirectional converter selected by `#[excel(converter = Type)]`.
///
/// The Java counterpart exposes six default methods (`supportJavaTypeKey`,
/// `supportExcelTypeKey`, two `convertToJavaData` overloads, two
/// `convertToExcelData` overloads). Rust's idiomatic trait surface keeps
/// `support_excel_type` (read dispatch key) plus the two conversion methods;
/// `supportJavaTypeKey` is encoded by the generic parameter `T` and the
/// `ConverterRegistry::register::<T, _>` call.
#[allow(clippy::missing_errors_doc)]
pub trait Converter<T> {
    /// Returns the source cell type supported when this converter is registered globally.
    ///
    /// Java `EasyExcel` requires global read converters to expose this key.
    /// A string default keeps field-only converters concise while matching
    /// the most common custom converter contract.
    fn support_excel_type(&self) -> CellDataType {
        CellDataType::String
    }

    /// Converts an Excel cell into a Rust field value. (Java `convertToJavaData(ReadConverterContext)`)
    fn convert_to_rust_data(&self, _context: &ReadConverterContext<'_>) -> Result<T, ExcelError> {
        Err(ExcelError::Unsupported(
            "custom converter does not support reading".to_owned(),
        ))
    }

    /// Converts a Rust field value into an Excel cell. (Java `convertToExcelData(WriteConverterContext)`)
    fn convert_to_excel_data(
        &self,
        _context: &WriteConverterContext<'_, T>,
    ) -> Result<WriteCellData, ExcelError> {
        Err(ExcelError::Unsupported(
            "custom converter does not support writing".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::{
        CellValue, ConvertContext, ExcelColumn, ReadConverterContext, WriteConverterContext,
    };

    /// 测试辅助：不实现任何方法，全部走 trait 默认实现。
    struct MinimalConverter;

    impl Converter<i32> for MinimalConverter {}

    fn context() -> ConvertContext {
        ConvertContext {
            sheet_name: "Data".to_owned(),
            row_index: 1,
            column_index: Some(0),
            field: "value",
            format: None,
            use_1904_windowing: false,
        }
    }

    #[test]
    fn default_methods_use_string_key_and_report_unsupported() {
        // 对应 Java：`Converter` 默认 `supportExcelTypeKey` 为 STRING，读写默认抛异常
        let converter = MinimalConverter;
        assert_eq!(converter.support_excel_type(), CellDataType::String);
        let column = ExcelColumn::new("value", "Value", Some(0), 0, None);
        let context = context();
        let cell = CellValue::Int(1);
        let read_error = converter
            .convert_to_rust_data(&ReadConverterContext::new(Some(&cell), &column, &context))
            .expect_err("default read is unsupported");
        assert!(matches!(read_error, ExcelError::Unsupported(_)));
        let write_error = converter
            .convert_to_excel_data(&WriteConverterContext::new(&1_i32, &column, &context))
            .expect_err("default write is unsupported");
        assert!(matches!(write_error, ExcelError::Unsupported(_)));
    }
}
