//! Owned, cloneable output-stream builder for one-shot or stateful writes.
//!
//! 对应 Java：`ExcelWriterBuilder` 的 `to_output_stream(OutputStream)` 路径
//! （持有 `ExcelOutputStream`，支持 one-shot 与多批次写入）。

use std::io::Write;

use crate::core::{ExcelRow, Result};
use crate::write::builder::excel_writer_builder::ExcelWriterBuilder;
use crate::write::{ExcelOutputStream, ExcelWriter, WriteSheet};

/// Owned, cloneable output-stream builder for one-shot or stateful writes.
pub struct ExcelOwnedOutputStreamBuilder<T, W> {
    pub(crate) builder: ExcelWriterBuilder<T>,
    pub(crate) output: ExcelOutputStream<W>,
}

impl<T, W> ExcelOwnedOutputStreamBuilder<T, W>
where
    T: ExcelRow,
    W: Write + Send + 'static,
{
    /// Builds a stateful writer for repeated `write` calls.
    #[must_use]
    pub fn build(self) -> ExcelWriter {
        ExcelWriter::with_output_stream(
            self.builder.path,
            self.output,
            self.builder.handlers,
            self.builder.options,
        )
    }

    /// Writes one batch and completes the output-stream lifecycle.
    ///
    /// # Errors
    ///
    /// Returns a conversion, handler, workbook, close, or stream I/O error.
    pub fn do_write<I>(self, rows: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
    {
        let sheet = WriteSheet::from_options(self.builder.options.clone());
        let mut writer = self.build();
        if let Err(error) = writer.write(rows, &sheet) {
            writer.finish_on_exception()?;
            return Err(error);
        }
        writer.finish()
    }
}
