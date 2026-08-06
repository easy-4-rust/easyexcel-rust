//! 对应 Java：`com.alibaba.excel.write.ExcelBuilder` and `ExcelBuilderImpl`.
use std::any::Any;

pub use crate::write::excel_builder_impl::ExcelBuilderImpl;

#[cfg(test)]
use crate::WriteOptions;
use crate::WriteSheet;
#[cfg(test)]
use crate::core::Holder;
use crate::core::{DynamicRow, ExcelRow, Result, WriteContext, fill_requires_template_error};
#[cfg(test)]
use crate::core::{
    ExcelError, WriteContextImpl, WriteFillConfig, WriteFillExecutor, WriteFillSheet,
    finish_write_context,
};
use crate::write::metadata::WriteTable;
include!("excel_builder/fill_config.rs");

/// Workbook builder contract matching Java `ExcelBuilder`.
///
/// 对应 Java：`com.alibaba.excel.write.ExcelBuilder`.
pub trait ExcelBuilder {
    /// Appends rows to a worksheet. (Java `addContent(Collection, WriteSheet)`)
    ///
    /// # Errors
    ///
    /// Returns a conversion, handler, or I/O error from the underlying writer.
    fn add_content<T, I>(&mut self, data: I, write_sheet: &WriteSheet<T>) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>;

    /// Appends rows to a worksheet table. (Java `addContent(Collection, WriteSheet, WriteTable)`)
    ///
    /// # Errors
    ///
    /// Returns a conversion, handler, or I/O error from the underlying writer.
    fn add_content_with_table<T, I>(
        &mut self,
        data: I,
        write_sheet: &WriteSheet<T>,
        write_table: &WriteTable,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>;

    /// Fills template placeholders on a worksheet. (Java `fill(Object, FillConfig, WriteSheet)`)
    ///
    /// `data` must be a supported fill payload (`TemplateData`, `FillWrapper`, …)
    /// wired through `WriteFillExecutor` by the `easyexcel` facade when a
    /// template is configured.
    ///
    /// # Errors
    ///
    /// Returns `ExcelError::Unsupported` when no template stream is configured.
    fn fill(
        &mut self,
        _data: &dyn Any,
        _fill_config: FillConfig,
        _write_sheet: &WriteSheet<DynamicRow>,
    ) -> Result<()> {
        Err(fill_requires_template_error())
    }

    /// Creates a merged region using zero-based inclusive coordinates.
    ///
    /// Mirrors deprecated Java `merge(int, int, int, int)`.
    ///
    /// # Errors
    ///
    /// Returns a format error when the coordinates are out of range or the
    /// writer backend cannot merge the region.
    fn merge(&mut self, first_row: u32, last_row: u32, first_col: u16, last_col: u16)
    -> Result<()>;

    /// Returns the active write context. (Java `writeContext()`)
    fn write_context(&self) -> &dyn WriteContext;

    /// Completes the workbook lifecycle. (Java `finish(boolean onException)`)
    ///
    /// # Errors
    ///
    /// Returns an output, close, or handler error.
    fn finish(&mut self, on_exception: bool) -> Result<()>;
}
#[cfg(test)]
#[path = "excel_builder_tests/tests.rs"]
mod tests;
