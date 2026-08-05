//! 对应 Java：`com.alibaba.excel.converters.string.StringErrorConverter`.
//!
/// 对应 Java：`StringErrorConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StringErrorConverter;

impl crate::Converter<String> for StringErrorConverter {
    fn support_excel_type(&self) -> crate::CellDataType {
        crate::CellDataType::Error
    }

    fn convert_to_rust_data(
        &self,
        context: &crate::ReadConverterContext<'_>,
    ) -> Result<String, crate::ExcelError> {
        match context.cell().unwrap_or(&crate::CellValue::Empty) {
            crate::CellValue::Error(value) => Ok(value.clone()),
            value => Err(context.convert_context().invalid(value, "String")),
        }
    }

    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, String>,
    ) -> Result<crate::WriteCellData, crate::ExcelError> {
        Ok(crate::WriteCellData::new(crate::CellValue::Error(
            context.value().clone(),
        )))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::Converter;
    use crate::{
        CellValue, ConvertContext, ExcelColumn, ReadConverterContext, WriteConverterContext,
    };

    const COLUMN: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, None);

    fn context() -> ConvertContext {
        ConvertContext {
            sheet_name: "Data".to_owned(),
            row_index: 1,
            column_index: Some(0),
            field: "value",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        }
    }

    #[test]
    fn reads_error_cells_and_rejects_other_cells() {
        // 对应 Java：`StringErrorConverter.convertToJavaData` 仅接受错误单元格
        let converter = StringErrorConverter;
        let error = CellValue::Error("#DIV/0!".to_owned());
        let context = context();
        let read = ReadConverterContext::new(Some(&error), &COLUMN, &context);
        assert_eq!(converter.convert_to_rust_data(&read).unwrap(), "#DIV/0!");
        let text = CellValue::String("x".to_owned());
        let read = ReadConverterContext::new(Some(&text), &COLUMN, &context);
        assert!(converter.convert_to_rust_data(&read).is_err());
        let read = ReadConverterContext::new(None, &COLUMN, &context);
        assert!(converter.convert_to_rust_data(&read).is_err());
    }

    #[test]
    fn writes_error_cells() {
        // 对应 Java：`StringErrorConverter.convertToExcelData` 写出错误单元格
        let value = "#N/A".to_owned();
        let cell = StringErrorConverter
            .convert_to_excel_data(&WriteConverterContext::new(&value, &COLUMN, &context()))
            .expect("error cell");
        assert_eq!(cell.value(), &CellValue::Error("#N/A".to_owned()));
    }
}
