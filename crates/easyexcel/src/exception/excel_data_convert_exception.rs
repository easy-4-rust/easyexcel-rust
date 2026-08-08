//! 对应 Java：`com.alibaba.excel.exception.ExcelDataConvertException`。

use super::ExcelRuntimeException;

/// 携带精确单元格位置与字段配置的数据转换异常。
#[derive(Debug, Clone, PartialEq)]
pub struct ExcelDataConvertException {
    inner: ExcelRuntimeException,
    row_index: usize,
    column_index: usize,
    cell_data: crate::CellData<crate::CellValue>,
    excel_content_property: Option<crate::ExcelContentProperty>,
}
impl ExcelDataConvertException {
    /// Java 五参数构造器。
    #[must_use]
    pub fn new(row_index: usize, column_index: usize, cell_data: crate::CellData<crate::CellValue>, excel_content_property: Option<crate::ExcelContentProperty>, message: impl Into<String>) -> Self {
        Self { inner: ExcelRuntimeException::with_message(message), row_index, column_index, cell_data, excel_content_property }
    }
    /// Java 带 cause 构造器。
    #[must_use]
    pub fn with_cause(row_index: usize, column_index: usize, cell_data: crate::CellData<crate::CellValue>, excel_content_property: Option<crate::ExcelContentProperty>, message: impl Into<String>, cause: impl ToString) -> Self {
        Self { inner: ExcelRuntimeException::with_message_and_cause(message, cause), row_index, column_index, cell_data, excel_content_property }
    }
    #[must_use] pub const fn get_row_index(&self) -> usize { self.row_index }
    pub const fn set_row_index(&mut self, value: usize) { self.row_index = value; }
    #[must_use] pub const fn get_column_index(&self) -> usize { self.column_index }
    pub const fn set_column_index(&mut self, value: usize) { self.column_index = value; }
    #[must_use] pub const fn get_cell_data(&self) -> &crate::CellData<crate::CellValue> { &self.cell_data }
    pub fn set_cell_data(&mut self, value: crate::CellData<crate::CellValue>) { self.cell_data = value; }
    #[must_use] pub const fn get_excel_content_property(&self) -> Option<&crate::ExcelContentProperty> { self.excel_content_property.as_ref() }
    pub fn set_excel_content_property(&mut self, value: Option<crate::ExcelContentProperty>) { self.excel_content_property = value; }
    #[must_use] pub const fn runtime_exception(&self) -> &ExcelRuntimeException { &self.inner }
}
impl std::fmt::Display for ExcelDataConvertException { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(&self.inner, f) } }
impl std::error::Error for ExcelDataConvertException {}
impl From<ExcelDataConvertException> for crate::ExcelError {
    fn from(value: ExcelDataConvertException) -> Self {
        crate::ExcelError::Data { sheet: String::new(), row: u32::try_from(value.row_index).unwrap_or(u32::MAX), column: Some(value.column_index), field: "", value: value.cell_data.get_string_value().unwrap_or("").to_owned(), message: value.to_string() }
    }
}
