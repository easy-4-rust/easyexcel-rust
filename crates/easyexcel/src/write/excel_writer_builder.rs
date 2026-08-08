//! Rust implementation of Java
//! `com.alibaba.excel.write.builder.ExcelWriterBuilder`.

use std::path::PathBuf;

use crate::core::{CsvCharset, ExcelError, Result, WriteHandler};
use crate::support::ExcelTypeEnum;

use crate::write::builder::excel_writer_sheet_builder::ExcelWriterSheetBuilder;
use crate::write::metadata::write_workbook::WriteWorkbook;
use crate::{ExcelOutputStream, ExcelWriter};

/// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Java-compatible workbook writer builder backed by the real Rust writer.
///
/// Unlike the former documentation-only placeholder, every supported option
/// is stored on [`WriteWorkbook`] and is passed into [`ExcelWriter`] by
/// [`Self::build`].
pub struct ExcelWriterBuilder {
    write_workbook: WriteWorkbook,
    handlers: Vec<Box<dyn WriteHandler>>,
    memory_selection: Option<bool>,
}

impl ExcelWriterBuilder {
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Creates an empty builder. (Java `ExcelWriterBuilder()`)
    #[must_use]
    pub fn new() -> Self {
        Self {
            write_workbook: WriteWorkbook::new(),
            handlers: Vec::new(),
            memory_selection: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets the final output file. (Java `file(File/String)`)
    #[must_use]
    pub fn file(mut self, file: impl Into<PathBuf>) -> Self {
        self.write_workbook.set_file(file);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets the requested workbook type. (Java `excelType(ExcelTypeEnum)`)
    #[must_use]
    pub fn excel_type(mut self, excel_type: ExcelTypeEnum) -> Self {
        self.write_workbook.set_excel_type(excel_type);
        self
    }

    /// 设置 BIFF8 模板的 VBA 项目策略；默认原样保留且从不执行宏。
    #[must_use]
    pub fn biff8_macro_policy(mut self, policy: crate::Biff8MacroPolicy) -> Self {
        self.write_workbook.options.biff8_macro_policy = policy;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Enables or disables Java's default bold header style.
    #[must_use]
    pub fn use_default_style(mut self, enabled: bool) -> Self {
        self.write_workbook.options.use_default_style = enabled;
        self.write_workbook.options.head_style = if enabled {
            crate::CellStyle::new().bold(true)
        } else {
            crate::CellStyle::new()
        };
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Controls whether an owned output stream closes on finish.
    #[must_use]
    pub fn auto_close_stream(mut self, enabled: bool) -> Self {
        self.write_workbook.set_auto_close_stream(enabled);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Encrypts XLSX output.
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.write_workbook.set_password(password);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Selects in-memory instead of constant-memory writing.
    #[must_use]
    pub fn in_memory(mut self, enabled: bool) -> Self {
        self.write_workbook.set_in_memory(enabled);
        self.memory_selection = Some(enabled);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Controls whether partial output is emitted on an exception.
    #[must_use]
    pub fn write_excel_on_exception(mut self, enabled: bool) -> Self {
        self.write_workbook.set_write_excel_on_exception(enabled);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets CSV output encoding.
    #[must_use]
    pub fn charset(mut self, charset: impl Into<CsvCharset>) -> Self {
        self.write_workbook.options.charset = charset.into();
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Controls whether CSV starts with a byte-order mark.
    #[must_use]
    pub fn with_bom(mut self, enabled: bool) -> Self {
        self.write_workbook.set_with_bom(enabled);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets a template file. (Java `withTemplate(File/String)`)
    #[must_use]
    pub fn with_template(mut self, template_file: impl Into<PathBuf>) -> Self {
        self.write_workbook.set_template_file(template_file);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets a buffered template stream. (Java `withTemplate(InputStream)`)
    #[must_use]
    pub fn with_template_bytes(mut self, template_bytes: impl Into<Vec<u8>>) -> Self {
        self.write_workbook.set_template_bytes(template_bytes);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets the number of rows before the header.
    #[must_use]
    pub fn relative_head_row_index(mut self, index: i32) -> Self {
        self.write_workbook.options.relative_head_row_index = index;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Controls header output.
    #[must_use]
    pub fn need_head(mut self, enabled: bool) -> Self {
        self.write_workbook.options.need_head = enabled;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Controls automatic merging of equal multi-level header cells.
    #[must_use]
    pub fn automatic_merge_head(mut self, enabled: bool) -> Self {
        self.write_workbook.options.automatic_merge_head = enabled;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Includes only the supplied physical columns.
    #[must_use]
    pub fn include_column_indexes(mut self, indexes: impl IntoIterator<Item = usize>) -> Self {
        self.write_workbook.options.include_column_indexes = Some(indexes.into_iter().collect());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Includes only the supplied Rust field names.
    #[must_use]
    pub fn include_column_field_names<S>(mut self, names: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        self.write_workbook.options.include_column_field_names =
            Some(names.into_iter().map(Into::into).collect());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Excludes physical columns.
    #[must_use]
    pub fn exclude_column_indexes(mut self, indexes: impl IntoIterator<Item = usize>) -> Self {
        self.write_workbook.options.exclude_column_indexes = indexes.into_iter().collect();
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Excludes Rust field names.
    #[must_use]
    pub fn exclude_column_field_names<S>(mut self, names: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        self.write_workbook.options.exclude_column_field_names =
            names.into_iter().map(Into::into).collect();
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Orders output by the include-list order.
    #[must_use]
    pub fn order_by_include_column(mut self, enabled: bool) -> Self {
        self.write_workbook.options.order_by_include_column = enabled;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Appends a write handler in registration order.
    #[must_use]
    pub fn register_write_handler(mut self, handler: impl WriteHandler + 'static) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Selects an owned output stream instead of a file.
    ///
    /// This is Rust's typed equivalent of Java `file(OutputStream)`. Configure
    /// workbook options before this call, then call `build` or `sheet` on the
    /// returned stream builder.
    #[must_use]
    pub fn output_stream<W>(self, output: ExcelOutputStream<W>) -> ExcelWriterOutputStreamBuilder<W>
    where
        W: std::io::Write + Send + 'static,
    {
        ExcelWriterOutputStreamBuilder {
            builder: self,
            output,
        }
    }

    /// Returns the accumulated Java-style metadata.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn parameter(&self) -> &WriteWorkbook {
        &self.write_workbook
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Builds a stateful writer. (Java `build()`)
    ///
    /// # Errors
    ///
    /// Returns a format error when no output file was configured via
    /// `file(...)` before `build()`.
    pub fn build(self) -> Result<ExcelWriter> {
        let path = self.write_workbook.output_file.ok_or_else(|| {
            ExcelError::Format("ExcelWriterBuilder.file must be set before build()".to_owned())
        })?;
        let selection = match self.memory_selection {
            None => crate::WriteBackendSelection::AutoUndecided,
            Some(true) => crate::WriteBackendSelection::ExplicitInMemory,
            Some(false) => crate::WriteBackendSelection::ExplicitStreaming,
        };
        Ok(ExcelWriter::with_handlers_and_options_and_selection(
            path,
            self.handlers,
            self.write_workbook.options,
            selection,
        ))
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Builds a writer-bound default sheet.
    ///
    /// # Errors
    ///
    /// Propagates the error from [`Self::build`].
    pub fn sheet(self) -> Result<ExcelWriterSheetBuilder> {
        let inherited_options = self.write_workbook.options.clone();
        Ok(ExcelWriterSheetBuilder::with_excel_writer_and_options(
            self.build()?,
            inherited_options,
        ))
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Builds a writer-bound sheet selected by number.
    ///
    /// # Errors
    ///
    /// Propagates the error from [`Self::build`].
    pub fn sheet_no(self, sheet_no: i32) -> Result<ExcelWriterSheetBuilder> {
        Ok(self.sheet()?.sheet_no(sheet_no))
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Builds a writer-bound sheet selected by name.
    ///
    /// # Errors
    ///
    /// Propagates the error from [`Self::build`].
    pub fn sheet_name(self, sheet_name: impl Into<String>) -> Result<ExcelWriterSheetBuilder> {
        Ok(self.sheet()?.sheet_name(sheet_name))
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Builds a writer-bound sheet selected by number and name.
    ///
    /// # Errors
    ///
    /// Propagates the error from [`Self::build`].
    pub fn sheet_with(
        self,
        sheet_no: i32,
        sheet_name: impl Into<String>,
    ) -> Result<ExcelWriterSheetBuilder> {
        Ok(self.sheet()?.sheet_no(sheet_no).sheet_name(sheet_name))
    }
}

impl Default for ExcelWriterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

include!("excel_writer_builder/excel_writer_output_stream_builder.rs");

#[cfg(test)]
#[path = "excel_writer_builder_tests/tests.rs"]
mod tests;
