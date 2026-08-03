//! 对应 Java：`com.alibaba.excel.converters.localdatetime.LocalDateTimeDateConverter`.
//!
/// 对应 Java：`LocalDateTimeDateConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalDateTimeDateConverter;

impl crate::Converter<chrono::NaiveDateTime> for LocalDateTimeDateConverter {
    fn support_excel_type(&self) -> crate::CellDataType {
        crate::CellDataType::Date
    }

    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, chrono::NaiveDateTime>,
    ) -> Result<crate::WriteCellData, crate::ExcelError> {
        Ok(crate::core::converter::date_support::write_datetime_value(
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
    fn supports_date_cell_type_and_writes_datetime_cell() {
        // 对应 Java：`LocalDateTimeDateConverter` 写出日期时间单元格
        let converter = LocalDateTimeDateConverter;
        assert_eq!(converter.support_excel_type(), CellDataType::Date);
        let column = ExcelColumn::new("value", "Value", Some(0), 0, None);
        let context = context();
        let value = chrono::NaiveDate::from_ymd_opt(2025, 1, 2)
            .unwrap()
            .and_hms_opt(3, 4, 5)
            .unwrap();
        let cell = converter
            .convert_to_excel_data(&WriteConverterContext::new(&value, &column, &context))
            .expect("datetime cell");
        assert_eq!(cell.value(), &CellValue::DateTime(value));
    }
}
