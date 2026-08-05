//! Caller-owned XLSX output stream builder.
//!
//! 对应 Java：`ExcelWriterBuilder` 的 `to_writer(OutputStream)` 路径
//! （借用调用方 stream，等价于 Java 的 `autoCloseStream(false)`）。

use std::io::Write;

use crate::core::{ExcelError, ExcelRow, Result};
use crate::write::builder::excel_writer_builder::ExcelWriterBuilder;
use crate::write::{
    DefaultWriteHandlerLoader, write_csv_to_buffer, write_xls_to_writer, write_xlsx_to_writer,
};
use crate::write_type_helpers::{effective_write_type, is_csv_write, is_xls_write};

/// Caller-owned XLSX output stream builder.
pub struct ExcelOutputStreamBuilder<'a, T, W> {
    pub(crate) builder: ExcelWriterBuilder<T>,
    pub(crate) output: &'a mut W,
}

impl<T, W> ExcelOutputStreamBuilder<'_, T, W>
where
    T: ExcelRow,
    W: Write + Send,
{
    /// Writes a complete OOXML package to the borrowed stream and flushes it.
    ///
    /// # Errors
    ///
    /// Returns a conversion, handler, workbook, encryption, or stream I/O error.
    pub fn do_write<I>(mut self, rows: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
    {
        let has_template = self.builder.options.template_file.is_some()
            || self.builder.options.template_bytes.is_some();
        let excel_type = effective_write_type(&self.builder.path, &self.builder.options);
        self.builder
            .handlers
            .extend(DefaultWriteHandlerLoader::load_default_handler_for(
                self.builder.options.use_default_style,
                excel_type,
            ));
        if is_csv_write(&self.builder.path, &self.builder.options) {
            if has_template {
                return Err(ExcelError::Unsupported(
                    "csv cannot use template.".to_owned(),
                ));
            }
            let bytes = write_csv_to_buffer::<T, I>(
                &self.builder.path,
                &self.builder.options,
                rows,
                &mut self.builder.handlers,
            )?;
            self.output.write_all(&bytes)?;
            self.output.flush()?;
            return Ok(());
        }
        if is_xls_write(&self.builder.path, &self.builder.options) {
            // Java stream write with ExcelTypeEnum.XLS — BIFF8 (+ optional template).
            return write_xls_to_writer::<T, I, _>(
                &self.builder.path,
                &mut *self.output,
                &self.builder.options,
                rows,
                &mut self.builder.handlers,
            );
        }
        write_xlsx_to_writer::<T, I, _>(
            &self.builder.path,
            self.output,
            &self.builder.options,
            rows,
            &mut self.builder.handlers,
        )
    }
}
