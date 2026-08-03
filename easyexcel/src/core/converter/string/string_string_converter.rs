//! 对应 Java：`com.alibaba.excel.converters.string.StringStringConverter`.
//!
/// 对应 Java：`StringStringConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StringStringConverter;

impl crate::Converter<String> for StringStringConverter {
    fn support_excel_type(&self) -> crate::CellDataType {
        crate::CellDataType::String
    }

    fn convert_to_rust_data(
        &self,
        context: &crate::ReadConverterContext<'_>,
    ) -> Result<String, crate::ExcelError> {
        match context.cell().unwrap_or(&crate::CellValue::Empty) {
            crate::CellValue::String(value) => Ok(value.clone()),
            value => Err(context.convert_context().invalid(value, "String")),
        }
    }

    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, String>,
    ) -> Result<crate::WriteCellData, crate::ExcelError> {
        Ok(crate::WriteCellData::from_string(context.value().clone()))
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
            use_1904_windowing: false,
        }
    }

    #[test]
    fn reads_string_cells_and_rejects_other_cells() {
        // 对应 Java：`StringStringConverter.convertToJavaData` 仅接受字符串单元格
        let converter = StringStringConverter;
        let text = CellValue::String("exact".to_owned());
        let context = context();
        let read = ReadConverterContext::new(Some(&text), &COLUMN, &context);
        assert_eq!(converter.convert_to_rust_data(&read).unwrap(), "exact");
        let number = CellValue::Int(1);
        let read = ReadConverterContext::new(Some(&number), &COLUMN, &context);
        assert!(converter.convert_to_rust_data(&read).is_err());
        let read = ReadConverterContext::new(None, &COLUMN, &context);
        assert!(converter.convert_to_rust_data(&read).is_err());
    }

    #[test]
    fn writes_string_cells() {
        // 对应 Java：`StringStringConverter.convertToExcelData` 写出字符串单元格
        let value = "text".to_owned();
        let cell = StringStringConverter
            .convert_to_excel_data(&WriteConverterContext::new(&value, &COLUMN, &context()))
            .expect("string cell");
        assert_eq!(cell.value(), &CellValue::String("text".to_owned()));
    }
}
