//! Mirrors the `convertToExcelData` half of Java `Converter<T>`.

use crate::core::cell_value::CellValue;
use crate::core::convert_context::ConvertContext;
use crate::core::excel_error::ExcelError;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Converts a Rust value into a backend-neutral cell.
///
/// Java-side counterpart: `Converter<T>.convertToExcelData(...)`.
pub trait IntoExcelCell {
    /// Performs the conversion.
    ///
    /// # Errors
    ///
    /// Returns an error when the Rust value cannot be represented as an Excel cell.
    fn to_excel_cell(&self, context: &ConvertContext) -> Result<CellValue, ExcelError>;
}
