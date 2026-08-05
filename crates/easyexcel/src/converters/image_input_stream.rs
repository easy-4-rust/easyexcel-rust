//! 对应 Java：`com.alibaba.excel.converters.inputstream.InputStreamImageConverter`.

use std::cell::RefCell;
use std::fmt;
use std::io::Read;

use crate::core::cell_value::CellValue;
use crate::core::convert_context::ConvertContext;
use crate::core::excel_error::ExcelError;
use crate::core::from_excel_cell::FromExcelCell;
use crate::core::into_excel_cell::IntoExcelCell;

/// Java `InputStreamImageConverter` equivalent for a stateful Rust [`Read`] source.
///
/// The first conversion consumes and caches the bytes remaining in the reader;
/// repeated conversion passes reuse that cache. The reader is deliberately not
/// closed or replaced, matching Java `EasyExcel`'s ownership contract for a
/// caller-supplied `InputStream`.
pub struct ImageInputStream<R = Box<dyn Read + Send>> {
    reader: RefCell<R>,
    cached_bytes: RefCell<Option<Vec<u8>>>,
}

impl<R> fmt::Debug for ImageInputStream<R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ImageInputStream")
            .finish_non_exhaustive()
    }
}

impl<R> ImageInputStream<R> {
    /// Wraps a reader whose remaining bytes represent one image.
    #[must_use]
    pub const fn new(reader: R) -> Self {
        Self {
            reader: RefCell::new(reader),
            cached_bytes: RefCell::new(None),
        }
    }

    /// Returns the wrapped reader, preserving its position after conversion.
    #[must_use]
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }
}

impl ImageInputStream {
    /// Type-erases a reader so the default converter registry can use one stable `TypeId`.
    ///
    /// This is the Rust counterpart of declaring a Java model field as
    /// `InputStream` rather than as a concrete `ByteArrayInputStream` subtype.
    #[must_use]
    pub fn boxed<R>(reader: R) -> Self
    where
        R: Read + Send + 'static,
    {
        Self::new(Box::new(reader))
    }
}

impl<R> From<R> for ImageInputStream<R> {
    fn from(reader: R) -> Self {
        Self::new(reader)
    }
}

impl<R> FromExcelCell for ImageInputStream<R> {
    fn from_excel_cell(
        _value: Option<&CellValue>,
        _context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        Err(ExcelError::Unsupported(
            "InputStreamImageConverter does not support reading image cells".to_owned(),
        ))
    }
}

impl<R: Read> IntoExcelCell for ImageInputStream<R> {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        if let Some(bytes) = self.cached_bytes.borrow().as_ref() {
            return Ok(CellValue::Image(bytes.clone()));
        }
        let bytes = read_image_bytes(&mut *self.reader.borrow_mut())?;
        *self.cached_bytes.borrow_mut() = Some(bytes.clone());
        Ok(CellValue::Image(bytes))
    }
}

fn read_image_bytes(reader: &mut dyn Read) -> Result<Vec<u8>, ExcelError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn debug_output_is_non_exhaustive() {
        // 对应 Java：调试输出不暴露内部读取器
        let stream = ImageInputStream::new(std::io::Cursor::new(vec![1_u8]));
        let text = format!("{stream:?}");
        assert!(text.contains("ImageInputStream"));
    }

    #[test]
    fn from_excel_cell_is_unsupported() {
        // 对应 Java：InputStreamImageConverter 不支持读取图片单元格
        let error = ImageInputStream::<std::io::Cursor<Vec<u8>>>::from_excel_cell(
            Some(&CellValue::Image(vec![1])),
            &ConvertContext {
                sheet_name: "Sheet1".to_owned(),
                row_index: 0,
                column_index: Some(0),
                field: "image",
                format: None,
                date_time_format: None,
                number_format: None,
                use_1904_windowing: false,
            },
        )
        .expect_err("unsupported");
        assert!(error.to_string().contains("does not support reading"));
    }

    #[test]
    fn to_excel_cell_reads_and_caches_bytes() {
        // 对应 Java：首次读取消耗读取器并缓存，重复转换复用缓存
        let stream = ImageInputStream::new(std::io::Cursor::new(vec![1_u8, 2_u8, 3_u8]));
        let context = ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 0,
            column_index: Some(0),
            field: "image",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        };
        assert_eq!(
            stream.to_excel_cell(&context).expect("reads"),
            CellValue::Image(vec![1, 2, 3])
        );
        // 第二次转换命中缓存，不再读取
        assert_eq!(
            stream.to_excel_cell(&context).expect("cached"),
            CellValue::Image(vec![1, 2, 3])
        );
    }

    #[test]
    fn boxed_and_from_and_into_inner_preserve_reader() {
        // 对应 Java：类型擦除与读取器取回
        let boxed = ImageInputStream::boxed(std::io::Cursor::new(vec![9_u8]));
        let _reader: Box<dyn Read + Send> = boxed.into_inner();

        let from: ImageInputStream<std::io::Cursor<Vec<u8>>> =
            std::io::Cursor::new(vec![5_u8]).into();
        let mut cursor = from.into_inner();
        let mut bytes = Vec::new();
        cursor.read_to_end(&mut bytes).expect("read");
        assert_eq!(bytes, vec![5_u8]);
    }
}
