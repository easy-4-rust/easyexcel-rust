//! 对应 Java：`com.alibaba.excel.converters.date.DateDateConverter`.
//!
//! Rust maps Java `java.util.Date` to [`crate::JavaDate`].

/// 对应 Java：`DateDateConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DateDateConverter;

impl crate::Converter<crate::JavaDate> for DateDateConverter {
    fn support_excel_type(&self) -> crate::CellDataType {
        crate::CellDataType::Date
    }

    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, crate::JavaDate>,
    ) -> Result<crate::WriteCellData, crate::ExcelError> {
        Ok(crate::core::converter::date_support::write_datetime_value(
            context.value().naive_local(),
            context,
        ))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::Converter;
    use crate::{
        CellDataType, CellValue, ConvertContext, ExcelColumn, JavaDate, WriteConverterContext,
    };

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
        // 对应 Java：`DateDateConverter` 写出 `java.util.Date` 等价日期时间单元格
        let converter = DateDateConverter;
        assert_eq!(converter.support_excel_type(), CellDataType::Date);
        let column = ExcelColumn::new("value", "Value", Some(0), 0, None);
        let context = context();
        let value = JavaDate::new(
            chrono::NaiveDate::from_ymd_opt(2025, 1, 2)
                .unwrap()
                .and_hms_opt(3, 4, 5)
                .unwrap(),
        );
        let cell = converter
            .convert_to_excel_data(&WriteConverterContext::new(&value, &column, &context))
            .expect("datetime cell");
        assert_eq!(cell.value(), &CellValue::DateTime(value.naive_local()));
    }
}
