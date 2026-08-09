//! 对应 Java：`com.alibaba.excel.exception.ExcelWriteDataConvertException`。

use super::ExcelDataConvertException;
use std::hash::{Hash, Hasher};

/// 写入转换异常，额外保留发生错误时的完整 Cell Handler 上下文。
#[derive(Debug, Clone)]
pub struct ExcelWriteDataConvertException {
    inner: ExcelDataConvertException,
    cell_write_handler_context: crate::WriteCellContext,
}

// Java Lombok 只比较本类声明的 `cellWriteHandlerContext`，不比较父异常。
impl PartialEq for ExcelWriteDataConvertException {
    fn eq(&self, other: &Self) -> bool {
        self.cell_write_handler_context == other.cell_write_handler_context
    }
}

// 相等上下文必然具有相同物理位置与生命周期标志；这些稳定字段足以满足 Hash 契约，
// 而无需把动态 Holder/handler 对象强制变成可哈希对象。
impl Hash for ExcelWriteDataConvertException {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let context = &self.cell_write_handler_context;
        context.sheet_name.hash(state);
        context.row_index.hash(state);
        context.column_index.hash(state);
        context.is_head.hash(state);
        context.relative_row_index.hash(state);
        context.original_field_type.hash(state);
        context.ignore_fill_style.hash(state);
        context.skip.hash(state);
    }
}
impl ExcelWriteDataConvertException {
    /// Java 双参数构造器。
    #[must_use]
    pub fn new(context: crate::WriteCellContext, message: impl Into<String>) -> Self {
        let inner = data_convert_exception(&context, message, None::<String>);
        Self { inner, cell_write_handler_context: context }
    }
    /// Java 带 cause 构造器。
    #[must_use]
    pub fn with_cause(context: crate::WriteCellContext, message: impl Into<String>, cause: impl ToString) -> Self {
        let inner = data_convert_exception(&context, message, Some(cause.to_string()));
        Self { inner, cell_write_handler_context: context }
    }
    #[must_use] pub const fn get_cell_write_handler_context(&self) -> &crate::WriteCellContext { &self.cell_write_handler_context }
    pub fn set_cell_write_handler_context(&mut self, value: crate::WriteCellContext) { self.cell_write_handler_context = value; }
    #[must_use] pub const fn data_convert_exception(&self) -> &ExcelDataConvertException { &self.inner }
}

fn data_convert_exception(context: &crate::WriteCellContext, message: impl Into<String>, cause: Option<String>) -> ExcelDataConvertException {
    let first = context.get_first_cell_data().cloned().unwrap_or(crate::CellValue::Empty);
    let mut cell_data = crate::CellData::new();
    cell_data.set_type(Some(first.data_type()));
    cell_data.set_data(Some(first));
    match cause {
        Some(cause) => ExcelDataConvertException::with_cause(usize::try_from(context.get_row_index()).unwrap_or(usize::MAX), usize::from(context.get_column_index()), cell_data, context.get_excel_content_property().cloned(), message, cause),
        None => ExcelDataConvertException::new(usize::try_from(context.get_row_index()).unwrap_or(usize::MAX), usize::from(context.get_column_index()), cell_data, context.get_excel_content_property().cloned(), message),
    }
}
impl std::fmt::Display for ExcelWriteDataConvertException { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(&self.inner, f) } }
impl std::error::Error for ExcelWriteDataConvertException {}
impl From<ExcelWriteDataConvertException> for crate::ExcelError { fn from(value: ExcelWriteDataConvertException) -> Self { value.inner.into() } }
