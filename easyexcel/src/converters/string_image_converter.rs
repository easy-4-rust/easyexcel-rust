//! 对应 Java：`com.alibaba.excel.converters.string.StringImageConverter`.
//!
//! Used with `#[excel(converter = StringImageConverter)]`. The file path is
//! read during row conversion; missing or unreadable files return an I/O
//! error.

use crate::converters::Converter;
use crate::core::excel_error::ExcelError;
use crate::core::write_converter_context::WriteConverterContext;
use crate::write::write_cell_data::WriteCellData;

/// Java `StringImageConverter` equivalent for fields containing an image file path.
///
/// Use it with `#[excel(converter = StringImageConverter)]`. The file is
/// read during row conversion; missing or unreadable files return an I/O
/// error.
#[derive(Debug, Clone, Copy, Default)]
pub struct StringImageConverter;

impl Converter<String> for StringImageConverter {
    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, String>,
    ) -> Result<WriteCellData, ExcelError> {
        std::fs::read(context.value())
            .map(WriteCellData::from_image)
            .map_err(Into::into)
    }
}
