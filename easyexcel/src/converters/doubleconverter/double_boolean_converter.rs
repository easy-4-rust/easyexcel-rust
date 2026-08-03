//! 对应 Java：`com.alibaba.excel.converters.doubleconverter.DoubleBooleanConverter`.
//!
/// 对应 Java：`DoubleBooleanConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DoubleBooleanConverter;

impl crate::Converter<f64> for DoubleBooleanConverter {
    fn support_excel_type(&self) -> crate::CellDataType {
        crate::CellDataType::Boolean
    }

    fn convert_to_rust_data(
        &self,
        context: &crate::ReadConverterContext<'_>,
    ) -> Result<f64, crate::ExcelError> {
        crate::converters::boolean_support::read_boolean_scalar(context)
    }

    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, f64>,
    ) -> Result<crate::WriteCellData, crate::ExcelError> {
        Ok(crate::converters::boolean_support::write_scalar_boolean(
            context,
        ))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::Converter;
    use crate::{CellValue, ConvertContext, ExcelColumn, WriteConverterContext};

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
    fn writes_one_as_true_and_zero_as_false() {
        // 对应 Java：`DoubleBooleanConverter` 按 1.0 / 0.0 写出布尔单元格
        let converter = DoubleBooleanConverter;
        let column = ExcelColumn::new("value", "Value", Some(0), 0, None);
        let context = context();
        let one = 1.0_f64;
        let zero = 0.0_f64;
        assert_eq!(
            converter
                .convert_to_excel_data(&WriteConverterContext::new(&one, &column, &context))
                .unwrap()
                .value(),
            &CellValue::Bool(true)
        );
        assert_eq!(
            converter
                .convert_to_excel_data(&WriteConverterContext::new(&zero, &column, &context))
                .unwrap()
                .value(),
            &CellValue::Bool(false)
        );
    }
}
