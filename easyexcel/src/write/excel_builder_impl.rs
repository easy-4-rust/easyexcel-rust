//! 对应 Java：`com.alibaba.excel.write.ExcelBuilderImpl`.

use std::any::Any;
use std::path::PathBuf;

use crate::core::{
    DynamicRow, ExcelError, ExcelRow, Result, WriteContext, WriteContextImpl,
    WriteContextLifecycle, WriteFillConfig, WriteFillExecutor, WriteFillSheet,
    csv_fill_unsupported_error, fill_requires_template_error, finish_write_context,
};
use crate::write::builder::excel_writer_table_builder::merge_table_options;
use crate::write::excel_builder::{ExcelBuilder, FillConfig};
use crate::write::executor::excel_write_fill_executor::ExcelWriteFillExecutor;
use crate::write::metadata::WriteTable;
use crate::{ExcelWriter, MergeRange, WriteOptions, WriteSheet};

/// Concrete builder implementation delegating to [`ExcelWriter`].
///
/// 对应 Java：`com.alibaba.excel.write.ExcelBuilderImpl`.
pub struct ExcelBuilderImpl {
    writer: ExcelWriter,
    logical_path: PathBuf,
    pending_merges: Vec<MergeRange>,
    context: WriteContextImpl,
    fill_executor: Option<Box<dyn WriteFillExecutor>>,
    finished_via_fill: bool,
    fill_session_active: bool,
}
impl ExcelBuilderImpl {
    /// Creates a builder from a stateful writer. (Java `new ExcelBuilderImpl(WriteWorkbook)`)
    #[must_use]
    pub fn new(writer: ExcelWriter, logical_path: impl Into<PathBuf>) -> Self {
        let logical_path = logical_path.into();
        Self {
            context: WriteContextImpl::new(&logical_path),
            writer,
            logical_path,
            pending_merges: Vec::new(),
            fill_executor: None,
            finished_via_fill: false,
            fill_session_active: false,
        }
    }

    /// Creates a builder from path and options via [`ExcelWriter::with_handlers_and_options`].
    #[must_use]
    pub fn from_options(path: impl Into<PathBuf>, options: WriteOptions) -> Self {
        let logical_path = path.into();
        Self::new(
            ExcelWriter::with_handlers_and_options(&logical_path, Vec::new(), options),
            logical_path,
        )
    }

    /// Returns the underlying writer for Java-style `ExcelWriter` facades.
    #[must_use]
    pub fn into_writer(self) -> ExcelWriter {
        self.writer
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn writer_mut(&mut self) -> &mut ExcelWriter {
        &mut self.writer
    }

    /// Returns the logical output path carried by this builder.
    #[must_use]
    pub fn logical_path(&self) -> &std::path::Path {
        &self.logical_path
    }

    /// Installs a template fill executor wired by the `easyexcel` facade.
    ///
    /// 对应 Java： lazy `ExcelWriteFillExecutor` creation inside
    /// `ExcelBuilderImpl.fill`.
    pub fn set_fill_executor(&mut self, executor: Box<dyn WriteFillExecutor>) {
        self.fill_executor = Some(executor);
    }

    /// Returns whether a template fill executor has been installed.
    #[must_use]
    pub fn has_fill_executor(&self) -> bool {
        self.fill_executor.is_some()
    }

    /// Returns whether [`Self::finish`] already persisted fill output.
    #[must_use]
    pub const fn finished_via_fill(&self) -> bool {
        self.finished_via_fill
    }

    fn update_current_holder<T>(
        &mut self,
        options: &WriteOptions,
        table_no: Option<i32>,
    ) -> Result<()>
    where
        T: ExcelRow,
    {
        self.context.set_sheet_context(&options.sheet_name);
        self.context.set_table_no(table_no);
        self.context
            .set_current_holder_state(crate::write::resolved_write_context_holder_state::<T>(
                options, table_no,
            )?);
        Ok(())
    }

    fn write_rows<T, I>(
        &mut self,
        data: I,
        write_sheet: &WriteSheet<T>,
        write_table: Option<&WriteTable>,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        let mut options = if let Some(table) = write_table {
            merge_table_options(write_sheet.options(), table)
        } else {
            write_sheet.options().clone()
        };
        options.merge_ranges.append(&mut self.pending_merges);
        let sheet_name = if options.auto_trim {
            options.sheet_name.trim().to_owned()
        } else {
            options.sheet_name.clone()
        };
        options.sheet_name.clone_from(&sheet_name);
        self.update_current_holder::<T>(&options, write_table.map(WriteTable::table_no))?;
        let sheet = WriteSheet::from_options(options);
        self.writer.write(data, &sheet).map(|_| ())
    }

    fn finish_resources(&mut self, on_exception: bool) -> Result<()> {
        if self.fill_session_active
            && let Some(delegate) = self.fill_executor.as_mut()
        {
            let mut executor =
                ExcelWriteFillExecutor::with_delegate(&self.context, delegate.as_mut());
            executor.finish(on_exception)?;
            self.writer.mark_finished();
            self.finished_via_fill = true;
            return Ok(());
        }
        if on_exception {
            self.writer.finish_on_exception()
        } else {
            self.writer.finish()
        }
    }
}
impl WriteContext for ExcelBuilderImpl {
    fn current_write_holder(&self) -> &dyn crate::core::WriteContextHolder {
        self.context.current_write_holder()
    }
}
impl WriteContextLifecycle for ExcelBuilderImpl {
    fn finish_context(&mut self, on_exception: bool) -> Result<()> {
        self.finish_resources(on_exception)
    }
}
impl ExcelBuilder for ExcelBuilderImpl {
    fn add_content<T, I>(&mut self, data: I, write_sheet: &WriteSheet<T>) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        self.write_rows(data, write_sheet, None)
    }

    fn add_content_with_table<T, I>(
        &mut self,
        data: I,
        write_sheet: &WriteSheet<T>,
        write_table: &WriteTable,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        self.write_rows(data, write_sheet, Some(write_table))
    }

    fn merge(
        &mut self,
        first_row: u32,
        last_row: u32,
        first_col: u16,
        last_col: u16,
    ) -> Result<()> {
        self.pending_merges
            .push(MergeRange::new(first_row, last_row, first_col, last_col));
        Ok(())
    }

    fn write_context(&self) -> &dyn WriteContext {
        &self.context
    }

    fn fill(
        &mut self,
        data: &dyn Any,
        mut fill_config: FillConfig,
        write_sheet: &WriteSheet<DynamicRow>,
    ) -> Result<()> {
        fill_config.init();
        if !self.writer.has_template_configured() {
            return Err(fill_requires_template_error());
        }
        if self.writer.is_csv() {
            return Err(csv_fill_unsupported_error());
        }
        if self.writer.is_xls() {
            return Err(ExcelError::Unsupported(
                "legacy XLS template fill is not supported".to_owned(),
            ));
        }
        let mut holder_options = write_sheet.options().clone();
        holder_options.sheet_name = if holder_options.auto_trim {
            holder_options.sheet_name.trim().to_owned()
        } else {
            holder_options.sheet_name.clone()
        };
        self.update_current_holder::<DynamicRow>(&holder_options, None)?;
        let delegate = self.fill_executor.as_mut().ok_or_else(|| {
            ExcelError::Unsupported(
                "template fill executor is not wired; build through easyexcel::builder_from_writer"
                    .to_owned(),
            )
        })?;
        let sheet = WriteFillSheet {
            sheet_name: write_sheet.options().sheet_name.clone(),
            sheet_index: write_sheet.options().sheet_index,
        };
        let mut executor = ExcelWriteFillExecutor::with_delegate(&self.context, delegate.as_mut());
        executor.fill(
            data,
            WriteFillConfig {
                force_new_row: fill_config.force_new_row,
                direction: fill_config.direction,
                auto_style: fill_config.auto_style,
            },
            sheet,
        )?;
        self.fill_session_active = true;
        Ok(())
    }

    fn finish(&mut self, on_exception: bool) -> Result<()> {
        finish_write_context(self, on_exception)
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn finish_resources_skips_missing_fill_executor() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("finish-no-executor.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        builder.fill_session_active = true;
        builder.finish(false)?;
        assert!(path.exists());
        Ok(())
    }
}
