//! 对应 Java：`com.alibaba.excel.converters.byteconverter.ByteStringConverter`.
//!
/// 对应 Java：`ByteStringConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ByteStringConverter;

impl crate::Converter<i8> for ByteStringConverter {
    fn support_excel_type(&self) -> crate::CellDataType {
        crate::CellDataType::String
    }
    fn convert_to_rust_data(&self, context: &crate::ReadConverterContext<'_>) -> crate::Result<i8> {
        crate::converters::number_support::read_string_number(context)
    }
    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, i8>,
    ) -> crate::Result<crate::WriteCellData> {
        crate::converters::number_support::write_number_string(context)
    }
}
