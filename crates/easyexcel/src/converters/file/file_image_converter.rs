//! 对应 Java：`com.alibaba.excel.converters.file.FileImageConverter`.
//!
/// 对应 Java：`FileImageConverter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileImageConverter;

impl crate::Converter<std::path::PathBuf> for FileImageConverter {
    fn convert_to_excel_data(
        &self,
        context: &crate::WriteConverterContext<'_, std::path::PathBuf>,
    ) -> Result<crate::WriteCellData, crate::ExcelError> {
        easyexcel_io::io::file_utils::read_file(context.value())
            .map(crate::WriteCellData::from_image)
            .map_err(crate::ExcelError::from)
    }
}
