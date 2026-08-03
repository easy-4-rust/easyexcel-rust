//! 对应 Java：`com.alibaba.excel.converters.localdate.LocalDateDateConverter`.
//!
/// 对应 Java：`LocalDateDateConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalDateDateConverter;

impl crate::Converter<chrono::NaiveDate> for LocalDateDateConverter {
    fn support_excel_type(&self) -> crate::CellDataType {
        crate::CellDataType::Date
    }

    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, chrono::NaiveDate>,
    ) -> Result<crate::WriteCellData, crate::ExcelError> {
        Ok(crate::converters::date_support::write_date_value(
            *context.value(),
            context,
        ))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::Converter;
    use crate::{CellDataType, CellValue, ConvertContext, ExcelColumn, WriteConverterContext};

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
    fn supports_date_cell_type_and_writes_date_cell() {
        // 对应 Java：`LocalDateDateConverter` 写出日期单元格
        let converter = LocalDateDateConverter;
        assert_eq!(converter.support_excel_type(), CellDataType::Date);
        let column = ExcelColumn::new("value", "Value", Some(0), 0, None);
        let context = context();
        let value = chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let cell = converter
            .convert_to_excel_data(&WriteConverterContext::new(&value, &column, &context))
            .expect("date cell");
        assert_eq!(cell.value(), &CellValue::Date(value));
    }
}
