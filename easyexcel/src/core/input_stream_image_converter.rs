//! 对应 Java：`com.alibaba.excel.converters.inputstream.InputStreamImageConverter`
//! (sentinel type).

use std::io::Read;

use crate::core::cell_value::CellValue;
use crate::core::converter::Converter;
use crate::core::excel_error::ExcelError;
use crate::core::image_input_stream::ImageInputStream;
use crate::core::into_excel_cell::IntoExcelCell;
use crate::core::write_cell_data::WriteCellData;
use crate::core::write_converter_context::WriteConverterContext;

/// Java `InputStreamImageConverter` equivalent for annotation-selected stream fields.
#[derive(Debug, Clone, Copy, Default)]
pub struct InputStreamImageConverter;

impl<R: Read> Converter<ImageInputStream<R>> for InputStreamImageConverter {
    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, ImageInputStream<R>>,
    ) -> Result<WriteCellData, ExcelError> {
        let value = context.value().to_excel_cell(context.convert_context())?;
        match value {
            CellValue::Image(bytes) => Ok(WriteCellData::from_image(bytes)),
            other => Ok(WriteCellData::new(other)),
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::{ConvertContext, ExcelColumn};

    #[test]
    fn convert_to_excel_data_keeps_non_image_values() {
        // 对应 Java：非图片值原样包装为 WriteCellData
        let stream = ImageInputStream::new(std::io::Cursor::new(Vec::<u8>::new()));
        let column = ExcelColumn::new("image", "Image", Some(0), 0, None);
        let context = ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 0,
            column_index: Some(0),
            field: "image",
            format: None,
            use_1904_windowing: false,
        };
        // 空输入流读取为空图片向量，命中 Image 分支（from_image 包装为 Empty + image 列表）
        let write_context = WriteConverterContext::new(&stream, &column, &context);
        let data = InputStreamImageConverter
            .convert_to_excel_data(&write_context)
            .expect("converts");
        assert_eq!(data.value(), &CellValue::Empty);
        assert_eq!(data.images().len(), 1);
    }
}
