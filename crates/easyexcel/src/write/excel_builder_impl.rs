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
use crate::write::metadata::{WriteTable, WriteWorkbook};
use crate::{ExcelWriter, MergeRange, WriteOptions, WriteSheet};

/// Concrete builder implementation delegating to [`ExcelWriter`].
///
/// 对应 Java：`com.alibaba.excel.write.ExcelBuilderImpl`.
pub struct ExcelBuilderImpl {
    writer: ExcelWriter,
    logical_path: PathBuf,
    context: WriteContextImpl,
    fill_executor: Option<Box<dyn WriteFillExecutor>>,
    finished_via_fill: bool,
    fill_session_active: bool,
}

impl std::fmt::Debug for ExcelBuilderImpl {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExcelBuilderImpl")
            .field("logical_path", &self.logical_path)
            .field("has_fill_executor", &self.fill_executor.is_some())
            .field("finished_via_fill", &self.finished_via_fill)
            .field("fill_session_active", &self.fill_session_active)
            .finish_non_exhaustive()
    }
}

impl ExcelBuilderImpl {
    /// 对应 Java：com.alibaba.excel.write.ExcelBuilderImpl。 Creates a builder from a stateful writer. (Java `new ExcelBuilderImpl(WriteWorkbook)`)
    #[must_use]
    pub fn new(writer: ExcelWriter, logical_path: impl Into<PathBuf>) -> Self {
        let logical_path = logical_path.into();
        Self {
            context: WriteContextImpl::new(&logical_path),
            writer,
            logical_path,
            fill_executor: None,
            finished_via_fill: false,
            fill_session_active: false,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.ExcelBuilderImpl。 Creates a builder from path and options via [`ExcelWriter::with_handlers_and_options`].
    #[must_use]
    pub fn from_options(path: impl Into<PathBuf>, options: WriteOptions) -> Self {
        let logical_path = path.into();
        Self::new(
            ExcelWriter::with_handlers_and_options(&logical_path, Vec::new(), options),
            logical_path,
        )
    }

    /// 使用 Java `ExcelWriter(WriteWorkbook)` 的配置创建写入器。
    ///
    /// 模板输入会在这里接入真实的 XLSX/XLS 填充执行器，因此随后可以直接调用
    /// [`Self::fill`]，无需调用方再执行额外的 wiring 步骤。
    ///
    /// 对应 Java：`com.alibaba.excel.ExcelWriter#ExcelWriter(WriteWorkbook)`。
    ///
    /// # Errors
    ///
    /// 未配置输出文件，或模板文件/字节无法加载时返回错误。
    pub fn from_write_workbook(write_workbook: WriteWorkbook) -> Result<Self> {
        let path = write_workbook.output_file.clone().ok_or_else(|| {
            ExcelError::Format(
                "WriteWorkbook.file must be set before constructing ExcelWriter".to_owned(),
            )
        })?;
        let mut options = write_workbook.options;
        options.excel_type = Some(write_workbook.excel_type);
        let writer = ExcelWriter::with_handlers_and_options(&path, Vec::new(), options);
        crate::excel_builder::fill_builder_from_writer(writer)
    }

    /// 向工作表写入一批数据，并返回当前写入器以支持链式调用。
    ///
    /// 对应 Java：`ExcelWriter#write(Collection, WriteSheet)`。
    ///
    /// # Errors
    ///
    /// 写入器已结束、Handler 执行失败或数据无法编码时返回错误。
    pub fn write<T, I>(&mut self, data: I, write_sheet: &WriteSheet<T>) -> Result<&mut Self>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        <Self as ExcelBuilder>::add_content(self, data, write_sheet)?;
        Ok(self)
    }

    /// 惰性获取一批数据后写入工作表。
    ///
    /// `supplier` 恰好调用一次；其错误与 panic 不会被重复求值。
    /// 对应 Java：`ExcelWriter#write(Supplier, WriteSheet)`。
    ///
    /// # Errors
    ///
    /// 写入器已结束、Handler 执行失败或数据无法编码时返回错误。
    pub fn write_with_supplier<T, I, F>(
        &mut self,
        supplier: F,
        write_sheet: &WriteSheet<T>,
    ) -> Result<&mut Self>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
        F: FnOnce() -> I,
    {
        self.write(supplier(), write_sheet)
    }

    /// 使用独立的 Table Holder 配置写入一批数据。
    ///
    /// 对应 Java：`ExcelWriter#write(Collection, WriteSheet, WriteTable)`。
    ///
    /// # Errors
    ///
    /// 写入器已结束、Handler 执行失败或数据无法编码时返回错误。
    pub fn write_with_table<T, I>(
        &mut self,
        data: I,
        write_sheet: &WriteSheet<T>,
        write_table: &WriteTable,
    ) -> Result<&mut Self>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        <Self as ExcelBuilder>::add_content_with_table(self, data, write_sheet, write_table)?;
        Ok(self)
    }

    /// 惰性获取一批数据后通过独立的 Table Holder 写入。
    ///
    /// `supplier` 恰好调用一次。
    /// 对应 Java：`ExcelWriter#write(Supplier, WriteSheet, WriteTable)`。
    ///
    /// # Errors
    ///
    /// 写入器已结束、Handler 执行失败或数据无法编码时返回错误。
    pub fn write_with_table_supplier<T, I, F>(
        &mut self,
        supplier: F,
        write_sheet: &WriteSheet<T>,
        write_table: &WriteTable,
    ) -> Result<&mut Self>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
        F: FnOnce() -> I,
    {
        self.write_with_table(supplier(), write_sheet, write_table)
    }

    /// 使用 Java 默认 `FillConfig` 填充模板并返回当前写入器。
    ///
    /// 对应 Java：`ExcelWriter#fill(Object, WriteSheet)`。
    ///
    /// # Errors
    ///
    /// 未配置模板、CSV 不支持填充或模板处理失败时返回错误。
    pub fn fill_default(
        &mut self,
        data: &dyn Any,
        write_sheet: &WriteSheet<DynamicRow>,
    ) -> Result<&mut Self> {
        self.fill(data, FillConfig::default(), write_sheet)
    }

    /// 使用显式配置填充模板并返回当前写入器。
    ///
    /// 对应 Java：`ExcelWriter#fill(Object, FillConfig, WriteSheet)`。
    ///
    /// # Errors
    ///
    /// 未配置模板、CSV 不支持填充或模板处理失败时返回错误。
    pub fn fill(
        &mut self,
        data: &dyn Any,
        fill_config: FillConfig,
        write_sheet: &WriteSheet<DynamicRow>,
    ) -> Result<&mut Self> {
        <Self as ExcelBuilder>::fill(self, data, fill_config, write_sheet)?;
        Ok(self)
    }

    /// 惰性获取对象后使用默认配置填充模板。
    ///
    /// `supplier` 恰好调用一次。
    /// 对应 Java：`ExcelWriter#fill(Supplier, WriteSheet)`。
    ///
    /// # Errors
    ///
    /// 未配置模板、CSV 不支持填充或模板处理失败时返回错误。
    pub fn fill_with_supplier<F>(
        &mut self,
        supplier: F,
        write_sheet: &WriteSheet<DynamicRow>,
    ) -> Result<&mut Self>
    where
        F: FnOnce() -> Box<dyn Any>,
    {
        let data = supplier();
        self.fill_default(data.as_ref(), write_sheet)
    }

    /// 惰性获取对象后使用显式配置填充模板。
    ///
    /// `supplier` 恰好调用一次。
    /// 对应 Java：`ExcelWriter#fill(Supplier, FillConfig, WriteSheet)`。
    ///
    /// # Errors
    ///
    /// 未配置模板、CSV 不支持填充或模板处理失败时返回错误。
    pub fn fill_with_config_supplier<F>(
        &mut self,
        supplier: F,
        fill_config: FillConfig,
        write_sheet: &WriteSheet<DynamicRow>,
    ) -> Result<&mut Self>
    where
        F: FnOnce() -> Box<dyn Any>,
    {
        let data = supplier();
        self.fill(data.as_ref(), fill_config, write_sheet)
    }

    /// 返回整个写入生命周期中稳定的上下文对象。
    ///
    /// 后续 `write`/`fill` 会原位更新其当前 Sheet/Table Holder；重复调用返回
    /// 同一实例。
    /// 对应 Java：`ExcelWriter#writeContext()`。
    #[must_use]
    pub fn write_context(&self) -> &dyn WriteContext {
        &self.context
    }

    /// 正常结束写入；重复调用保持幂等。
    ///
    /// 对应 Java：`ExcelWriter#finish()`。
    ///
    /// # Errors
    ///
    /// 输出、关闭流或 Handler 收尾失败时返回错误。
    pub fn finish(&mut self) -> Result<()> {
        <Self as ExcelBuilder>::finish(self, false)
    }

    /// 异常路径结束写入，遵循 `writeExcelOnException` 配置。
    ///
    /// 对应 Java：`ExcelBuilderImpl#finishOnException()`。
    ///
    /// # Errors
    ///
    /// 输出、关闭流或 Handler 收尾失败时返回错误。
    pub fn finish_on_exception(&mut self) -> Result<()> {
        <Self as ExcelBuilder>::finish(self, true)
    }

    /// `Closeable.close()` 的幂等别名。
    ///
    /// 对应 Java：`ExcelWriter#close()`。
    ///
    /// # Errors
    ///
    /// 与 [`Self::finish`] 相同。
    pub fn close(&mut self) -> Result<()> {
        self.finish()
    }

    /// 对应 Java：com.alibaba.excel.write.ExcelBuilderImpl。 Returns the underlying writer for Java-style `ExcelWriter` facades.
    #[must_use]
    pub fn into_writer(self) -> ExcelWriter {
        self.writer
    }

    /// 对应 Java：com.alibaba.excel.write.ExcelBuilderImpl。 Returns a mutable reference to the underlying writer.
    pub fn writer_mut(&mut self) -> &mut ExcelWriter {
        &mut self.writer
    }

    /// 对应 Java：com.alibaba.excel.write.ExcelBuilderImpl。 Returns the logical output path carried by this builder.
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

    /// 安装已经接管模板输出目标的 executor，并立即激活其资源生命周期。
    ///
    /// 与公开的 [`Self::set_fill_executor`] 不同，该入口只供 facade wiring
    /// 使用：真实 writer/关闭回调一旦移交，即使首个 fill/write 随后失败，
    /// 异常收尾也必须回到同一个 executor。
    pub(crate) fn set_active_template_fill_executor(
        &mut self,
        executor: Box<dyn WriteFillExecutor>,
    ) {
        self.fill_executor = Some(executor);
        self.fill_session_active = true;
    }

    /// 对应 Java：com.alibaba.excel.write.ExcelBuilderImpl。 Returns whether a template fill executor has been installed.
    #[must_use]
    pub fn has_fill_executor(&self) -> bool {
        self.fill_executor.is_some()
    }

    /// Returns whether [`Self::finish`] already persisted fill output.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.ExcelBuilderImpl。
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
        let sheet_name = if options.auto_trim {
            easyexcel_utils::string_utils::java_trim(&options.sheet_name).to_owned()
        } else {
            options.sheet_name.clone()
        };
        options.sheet_name.clone_from(&sheet_name);
        self.update_current_holder::<T>(&options, write_table.map(WriteTable::table_no))?;
        if self.writer.has_template_configured() {
            // Java 在 ExcelBuilderImpl(WriteWorkbook) 内部惰性创建
            // ExcelWriteFillExecutor。所有公开构造路径都必须获得同样行为，
            // 不能要求调用方知道 facade 私有的 executor wiring。
            crate::excel_builder::wire_template_fill(self)?;
            let delegate = self.fill_executor.as_mut().ok_or_else(|| {
                ExcelError::Unsupported(
                    "template executor could not be initialized for ordinary row writes"
                        .to_owned(),
                )
            })?;
            let effective_options =
                crate::write::excel_writer_core::with_default_write_converters(&options);
            let rows = data
                .into_iter()
                .map(|row| {
                    if row.is_absent_row() {
                        Ok(Vec::new())
                    } else {
                        row.to_row_with_converters(&effective_options.converters)
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            delegate.write_rows(
                rows,
                WriteFillSheet {
                    sheet_name: options.sheet_name.clone(),
                    sheet_index: options.sheet_index,
                },
            )?;
            self.fill_session_active = true;
            return Ok(());
        }
        if let Some(table) = write_table {
            self.writer
                .write_with_table(data, write_sheet, table)
                .map(|_| ())
        } else {
            let sheet = WriteSheet::from_options(options);
            self.writer.write(data, &sheet).map(|_| ())
        }
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
            let uses_output_path = self.writer.uses_output_path();
            self.writer.finish_on_exception()?;
            if uses_output_path {
                // Java `WriteContextImpl` 在构造阶段已经打开目标
                // `FileOutputStream`；异常结束时默认不写工作簿，但文件仍存在且
                // 长度为 0。Rust 延迟打开输出，因此在这里补齐相同可观察语义。
                std::fs::File::create(&self.logical_path)?;
            }
            Ok(())
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
        let sheet_context = self.context.sheet_context().ok_or_else(|| {
            ExcelError::Format(
                "ExcelBuilder.merge requires a current worksheet; call add_content first"
                    .to_owned(),
            )
        })?;
        let sheet_name = sheet_context.sheet_name().to_owned();
        let sheet_index = sheet_context
            .write_sheet_holder()
            .sheet_no()
            .and_then(|value| usize::try_from(value).ok());
        let range = MergeRange::new(first_row, last_row, first_col, last_col);
        if self.writer.has_template_configured() {
            crate::excel_builder::wire_template_fill(self)?;
            let delegate = self.fill_executor.as_mut().ok_or_else(|| {
                ExcelError::Unsupported(
                    "template executor could not be initialized for merged regions".to_owned(),
                )
            })?;
            delegate.add_merge(
                range,
                WriteFillSheet {
                    sheet_name,
                    sheet_index,
                },
            )?;
            self.fill_session_active = true;
            return Ok(());
        }
        self.writer.add_deferred_merge(sheet_name, range)
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
        let mut holder_options = write_sheet.options().clone();
        holder_options.sheet_name = if holder_options.auto_trim {
            easyexcel_utils::string_utils::java_trim(&holder_options.sheet_name).to_owned()
        } else {
            holder_options.sheet_name.clone()
        };
        self.update_current_holder::<DynamicRow>(&holder_options, None)?;
        crate::excel_builder::wire_template_fill(self)?;
        let delegate = self.fill_executor.as_mut().ok_or_else(|| {
            ExcelError::Unsupported(
                "template fill executor could not be initialized".to_owned(),
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
                force_new_row: fill_config.effective_force_new_row(),
                direction: Some(fill_config.effective_direction()),
                auto_style: fill_config.effective_auto_style(),
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
        builder.finish()?;
        assert!(path.exists());
        Ok(())
    }
}
