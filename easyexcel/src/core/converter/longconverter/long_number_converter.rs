//! 对应 Java：`com.alibaba.excel.converters.longconverter.LongNumberConverter`.
//!
/// 对应 Java：`LongNumberConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct LongNumberConverter;

impl crate::Converter<i64> for LongNumberConverter {
    fn support_excel_type(&self) -> crate::CellDataType {
        crate::CellDataType::Number
    }

    fn convert_to_rust_data(
        &self,
        context: &crate::ReadConverterContext<'_>,
    ) -> Result<i64, crate::ExcelError> {
        crate::core::converter::number_support::read_number(context)
    }

    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, i64>,
    ) -> Result<crate::WriteCellData, crate::ExcelError> {
        crate::core::converter::number_support::write_number(context)
    }
}
