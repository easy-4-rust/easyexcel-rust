//! New-workbook writer builder.
//!
//! 对应 Java：`com.alibaba.excel.write.builder.ExcelWriterBuilder`
//! （typed `write(path)` 路径；Java 兼容的无类型入口见
//! [`crate::write::CompatibleExcelWriterBuilder`]）。

use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::core::{
    Converter, CsvCharset, DynamicRow, ExcelError, ExcelRow, NullableObjectConverter, Result,
    WriteDirection, WriteHandler,
};
use crate::excel_builder::do_fill_template_with_compiled_styles;
use crate::excel_output_stream_builder::ExcelOutputStreamBuilder;
use crate::excel_owned_output_stream_builder::ExcelOwnedOutputStreamBuilder;
use crate::template::{FillConfig, FillDirection};
use crate::write::{
    BuilderFillConfig, CellStyle, DefaultWriteHandlerLoader, ExcelWriter, MergeRange,
    MirroredLoopMergeStrategy as LoopMergeStrategy, WriteOptions, WriteSheet,
    write_csv_with_handlers, write_xls_with_handlers, write_xlsx_with_handlers,
};
use crate::write_type_helpers::{effective_write_type, is_csv_write, is_xls_write};

/// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 New-workbook writer builder.
pub struct ExcelWriterBuilder<T> {
    pub(crate) path: PathBuf,
    pub(crate) options: WriteOptions,
    pub(crate) handlers: Vec<Box<dyn WriteHandler>>,
    pub(crate) marker: PhantomData<T>,
    pub(crate) memory_selection: WriteMemorySelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteMemorySelection {
    Auto,
    Explicit,
}

impl<T> ExcelWriterBuilder<T>
where
    T: ExcelRow,
{
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 从路径构造一个默认配置的写入 builder。
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            path,
            options: WriteOptions::default(),
            handlers: Vec::new(),
            marker: PhantomData,
            memory_selection: WriteMemorySelection::Auto,
        }
    }

    /// Sets an explicit output type, overriding the file extension.
    /// (Java `ExcelWriterBuilder.excelType`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn excel_type(mut self, excel_type: crate::support::ExcelTypeEnum) -> Self {
        self.options.excel_type = Some(excel_type);
        self
    }

    /// 设置 BIFF8 模板的 VBA 项目策略；默认原样保留且从不执行宏。
    #[must_use]
    pub fn biff8_macro_policy(mut self, policy: crate::Biff8MacroPolicy) -> Self {
        self.options.biff8_macro_policy = policy;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets the worksheet name.
    #[must_use]
    pub fn sheet(mut self, name: impl Into<String>) -> Self {
        self.options.sheet_name = name.into();
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets the Java-style zero-based logical worksheet number.
    #[must_use]
    pub fn sheet_index(mut self, index: usize) -> Self {
        self.options.sheet_index = Some(index);
        self.options.sheet_name = index.to_string();
        self
    }

    /// Enables or disables the header row.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn need_head(mut self, need_head: bool) -> Self {
        self.options.need_head = need_head;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Enables or disables Java's default header style.
    #[must_use]
    pub fn use_default_style(mut self, enabled: bool) -> Self {
        self.options.use_default_style = enabled;
        self.options.head_style = if enabled {
            CellStyle::new().bold(true)
        } else {
            CellStyle::new()
        };
        self
    }

    /// Controls automatic merging of equal multi-level headers.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn automatic_merge_head(mut self, enabled: bool) -> Self {
        self.options.automatic_merge_head = enabled;
        self
    }

    /// Sets the relative head row index. (Java `ExcelWriterBuilder.relativeHeadRowIndex`)
    ///
    /// When `index > 0`, the header (and subsequent data rows) start at that
    /// zero-based row, leaving the rows above blank — matching Java
    /// `WriteBasicParameter.relativeHeadRowIndex`.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn relative_head_row_index(mut self, index: i32) -> Self {
        self.options.relative_head_row_index = index;
        self
    }

    /// Freezes the header row.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn freeze_head(mut self, freeze: bool) -> Self {
        self.options.freeze_head = freeze;
        self
    }

    /// Freezes rows and columns above and to the left of the position.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn freeze_panes(mut self, row: u32, column: u16) -> Self {
        self.options.freeze_panes = Some((row, column));
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Includes only the supplied physical column indexes.
    #[must_use]
    pub fn include_column_indexes(mut self, indexes: impl IntoIterator<Item = usize>) -> Self {
        self.options.include_column_indexes = Some(indexes.into_iter().collect());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Includes only the supplied Rust field names.
    #[must_use]
    pub fn include_column_field_names<S>(mut self, names: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        self.options.include_column_field_names = Some(names.into_iter().map(Into::into).collect());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Excludes physical column indexes.
    #[must_use]
    pub fn exclude_column_indexes(mut self, indexes: impl IntoIterator<Item = usize>) -> Self {
        self.options.exclude_column_indexes = indexes.into_iter().collect();
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Excludes Rust field names.
    #[must_use]
    pub fn exclude_column_field_names<S>(mut self, names: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        self.options.exclude_column_field_names = names.into_iter().map(Into::into).collect();
        self
    }

    /// Orders selected columns by the corresponding include list.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn order_by_include_column(mut self, enabled: bool) -> Self {
        self.options.order_by_include_column = enabled;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Adds an absolute merged-cell range using zero-based inclusive coordinates.
    #[must_use]
    pub fn merge_cells(mut self, range: MergeRange) -> Self {
        self.options.merge_ranges.push(range);
        self
    }

    /// Enables automatic width calculation for used columns.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn auto_width(mut self) -> Self {
        self.options.auto_width = true;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets an explicit width for a zero-based physical column.
    #[must_use]
    pub fn column_width(mut self, column: u16, width: u16) -> Self {
        self.options.column_widths.push((column, width));
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Replaces the default bold header style.
    #[must_use]
    pub fn head_style(mut self, style: CellStyle) -> Self {
        self.options.head_style = style;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Applies one style to every content row.
    #[must_use]
    pub fn content_style(mut self, style: CellStyle) -> Self {
        self.options.content_styles = vec![style];
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Cycles the supplied styles across content rows.
    #[must_use]
    pub fn content_styles(mut self, styles: impl IntoIterator<Item = CellStyle>) -> Self {
        self.options.content_styles = styles.into_iter().collect();
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Registers a Java-style global converter for this workbook.
    #[must_use]
    pub fn register_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: Converter<V> + Send + Sync + 'static,
    {
        self.options.converters.register::<V, C>(converter);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Registers a nullable converter for this workbook.
    #[must_use]
    pub fn register_nullable_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: NullableObjectConverter<V> + Send + Sync + 'static,
    {
        self.options.converters.register_nullable::<V, C>(converter);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Registers a repeating data-row merge strategy.
    #[must_use]
    pub fn loop_merge(mut self, strategy: LoopMergeStrategy) -> Self {
        self.options.loop_merges.push(strategy);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Replaces derived headers with dynamic multi-level head paths.
    #[must_use]
    pub fn head<S, P>(mut self, paths: impl IntoIterator<Item = P>) -> Self
    where
        S: Into<String>,
        P: IntoIterator<Item = S>,
    {
        self.options.dynamic_head = Some(
            paths
                .into_iter()
                .map(|path| path.into_iter().map(Into::into).collect())
                .collect(),
        );
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Registers a write lifecycle handler. Handlers execute by ascending order.
    #[must_use]
    pub fn register_write_handler(mut self, handler: impl WriteHandler + 'static) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Encrypts XLSX output using ECMA-376 Agile Encryption.
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.options.password = Some(password.into());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets the character encoding used for CSV output.
    #[must_use]
    pub fn charset(mut self, charset: impl Into<CsvCharset>) -> Self {
        self.options.charset = charset.into();
        self
    }

    /// Enables or disables the CSV byte-order mark. Java `EasyExcel` defaults to enabled.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn with_bom(mut self, enabled: bool) -> Self {
        self.options.with_bom = enabled;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets a template workbook file. (Java `ExcelWriterBuilder.withTemplate(String/File)`)
    ///
    /// The template is loaded fully into memory (Java warns this can OOM for large
    /// files). Typed `do_write` / stateful `write` appends after existing template
    /// rows on the selected sheet and keeps other template sheets.
    ///
    /// # Notes
    ///
    /// - CSV templates are rejected (`csv cannot use template.`), matching Java.
    /// - **XLS templates:** record-preserving BIFF8 overlay via
    ///   `easyexcel-xls::biff8::Biff8TemplatePackage` (unmodified records kept;
    ///   new cells appended as LABEL/NUMBER). Sheets absent from the template
    ///   are created as record-valid BIFF8 worksheet substreams.
    /// - **Default (XLSX):** styles and merges are preserved via ZIP/OOXML append
    ///   (`styles.xml` + `mergeCells` kept; new rows appended to `sheetData`).
    ///   Creating a sheet absent from the template adds an empty worksheet part
    ///   without rewriting existing sheets (styles/merges stay intact).
    /// - Images / comments / drawings / column widths from the template remain in
    ///   the package on the ZIP (XLSX) path.
    /// - Opt into value-only replay for XLSX (styles/merges discarded) with
    ///   [`Self::use_legacy_template_seed`].
    #[must_use]
    pub fn with_template(mut self, path: impl Into<PathBuf>) -> Self {
        self.options.template_file = Some(path.into());
        self.options.template_bytes = None;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Sets a template from owned bytes. (Java `ExcelWriterBuilder.withTemplate(InputStream)`)
    ///
    /// Same semantics as [`Self::with_template`]; the stream/file is fully buffered.
    #[must_use]
    pub fn with_template_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.options.template_bytes = Some(bytes.into());
        self.options.template_file = None;
        self
    }

    /// Explicitly enables the legacy calamine → `rust_xlsxwriter` template seed.
    ///
    /// When enabled, `with_template` replays cell **values** only — styles, merges,
    /// images, comments, and drawings are not preserved. Default is `false` (ZIP
    /// preserve). Prefer the default unless you need the legacy seed for debugging.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn use_legacy_template_seed(mut self, enabled: bool) -> Self {
        self.options.use_legacy_template_seed = enabled;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Redirects this write from its logical path to a caller-owned XLSX stream.
    ///
    /// The path remains available to handler contexts but no file is created.
    /// Borrowing the stream makes ownership explicit and corresponds to Java
    /// `EasyExcel`'s `autoCloseStream(false)` behavior: the caller retains and
    /// may continue using the stream after [`ExcelOutputStreamBuilder::do_write`].
    #[must_use]
    pub fn to_writer<W>(self, output: &mut W) -> ExcelOutputStreamBuilder<'_, T, W>
    where
        W: std::io::Write + Send,
    {
        ExcelOutputStreamBuilder {
            builder: self,
            output,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Redirects this builder to a cloneable, explicitly closeable stream.
    ///
    /// This form supports both one-shot writes and stateful multi-batch writes,
    /// including Java-compatible `autoCloseStream` behavior.
    #[must_use]
    pub fn to_output_stream<W>(
        self,
        output: crate::write::ExcelOutputStream<W>,
    ) -> ExcelOwnedOutputStreamBuilder<T, W>
    where
        W: std::io::Write + Send + 'static,
    {
        ExcelOwnedOutputStreamBuilder {
            builder: self,
            output,
        }
    }

    /// Enables or disables closing an owned output stream during finish.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn auto_close_stream(mut self, enabled: bool) -> Self {
        self.options.auto_close_stream = enabled;
        self
    }

    /// Controls whether accumulated rows are emitted by `finish_on_exception`.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn write_excel_on_exception(mut self, enabled: bool) -> Self {
        self.options.write_excel_on_exception = enabled;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Builds a stateful writer for multiple `.write(rows, &sheet)` calls.
    #[must_use]
    pub fn build(self) -> ExcelWriter {
        let backend_selection = self.stateful_backend_selection();
        ExcelWriter::with_handlers_and_options_and_selection(
            self.path,
            self.handlers,
            self.options,
            backend_selection,
        )
    }

    pub(crate) const fn stateful_backend_selection(&self) -> crate::WriteBackendSelection {
        match self.memory_selection {
            WriteMemorySelection::Auto => crate::WriteBackendSelection::AutoUndecided,
            WriteMemorySelection::Explicit if self.options.constant_memory => {
                crate::WriteBackendSelection::ExplicitStreaming
            }
            WriteMemorySelection::Explicit => crate::WriteBackendSelection::ExplicitInMemory,
        }
    }

    fn resolve_automatic_memory_selection(
        &mut self,
        excel_type: crate::support::ExcelTypeEnum,
        has_template: bool,
    ) {
        if self.memory_selection == WriteMemorySelection::Auto
            && excel_type == crate::support::ExcelTypeEnum::Xlsx
            && stateful_constant_memory_is_safe::<T>(&self.options, &self.handlers, has_template)
        {
            self.options.constant_memory = true;
        }
    }

    /// Selects constant-memory output.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn constant_memory(mut self, enabled: bool) -> Self {
        self.options.constant_memory = enabled;
        self.memory_selection = WriteMemorySelection::Explicit;
        self
    }

    /// Selects Java's `inMemory` behavior explicitly.
    ///
    /// `true` keeps the full workbook available for random-access handlers;
    /// `false` forces SXSSF-style constant-memory output. Without either this
    /// method or [`Self::constant_memory`], safe one-shot scalar XLSX writes use
    /// constant memory from the first batch, while advanced/random-access writes
    /// stay in memory.
    #[must_use]
    pub const fn in_memory(mut self, enabled: bool) -> Self {
        self.options.constant_memory = !enabled;
        self.memory_selection = WriteMemorySelection::Explicit;
        self
    }

    /// Enables SXSSF-style compressed / disk-spill temporary files for bulk writes.
    ///
    /// Java mapping: `SXSSFWorkbook.setCompressTempFiles(true)` (commonly set in
    /// `WorkbookWriteHandler.afterWorkbookCreate`). Forces constant-memory row
    /// spill so large multi-batch writes do not keep the full sheet in RAM.
    ///
    /// See [`WriteOptions::compress_temp_files`] for the POI vs `rust_xlsxwriter`
    /// gzip difference.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。
    pub const fn compress_temp_files(mut self, enabled: bool) -> Self {
        self.options.compress_temp_files = enabled;
        if enabled {
            self.options.constant_memory = true;
        }
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Writes any owned row iterator.
    ///
    /// When [`Self::with_template`] is set, rows are appended onto the template
    /// workbook (Java `withTemplate(...).sheet().doWrite(data)`).
    ///
    /// # Errors
    ///
    /// Returns a conversion, worksheet-configuration, XLSX-format, template, or I/O error.
    pub fn do_write<I>(mut self, rows: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
    {
        let has_template =
            self.options.template_file.is_some() || self.options.template_bytes.is_some();
        let excel_type = effective_write_type(&self.path, &self.options);
        self.resolve_automatic_memory_selection(excel_type, has_template);
        self.handlers
            .extend(DefaultWriteHandlerLoader::load_default_handler_for(
                self.options.use_default_style,
                excel_type,
            ));
        if is_csv_write(&self.path, &self.options) {
            if has_template {
                return Err(ExcelError::Unsupported(
                    "csv cannot use template.".to_owned(),
                ));
            }
            write_csv_with_handlers::<T, _>(
                Path::new(&self.path),
                &self.options,
                rows,
                &mut self.handlers,
            )
        } else if is_xls_write(&self.path, &self.options) {
            // Java: EasyExcel.write(...).excelType(ExcelTypeEnum.XLS).sheet().doWrite(...)
            // Minimal BIFF8; with_template uses the easyexcel-xls record-preserving engine.
            write_xls_with_handlers::<T, _>(
                Path::new(&self.path),
                &self.options,
                rows,
                &mut self.handlers,
            )
        } else {
            write_xlsx_with_handlers::<T, _>(
                Path::new(&self.path),
                &self.options,
                rows,
                &mut self.handlers,
            )
        }
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterBuilder。 Alias emphasizing that the input is consumed incrementally.
    ///
    /// # Errors
    ///
    /// Returns a conversion, worksheet-configuration, XLSX-format, or I/O error.
    pub fn do_write_iter<I>(self, rows: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
    {
        self.do_write(rows)
    }

    /// Fills scalar `{key}` placeholders through `ExcelBuilderImpl::fill`.
    ///
    /// 对应 Java：`EasyExcel.write(file).withTemplate(template).sheet().doFill(data)`.
    ///
    /// # Errors
    ///
    /// Returns template, fill, CSV/XLS unsupported, or output errors.
    pub fn do_fill(self, data: &dyn std::any::Any) -> Result<()> {
        let sheet = WriteSheet::<DynamicRow>::from_options(self.options.clone());
        let mut handlers = self.handlers;
        handlers.extend(
            crate::write::handler_execution_scope::load_annotation_handlers::<T>(&self.options)?,
        );
        let styles = crate::write::excel_writer_core::compile_template_fill_styles::<T>(
            &self.options,
            &mut handlers,
        )?;
        let writer = ExcelWriter::with_handlers_and_options(self.path, handlers, self.options);
        do_fill_template_with_compiled_styles(
            writer,
            data,
            BuilderFillConfig::default(),
            &sheet,
            styles,
        )
    }

    /// Fills scalar or collection data with Java-compatible `FillConfig`.
    ///
    /// 对应 Java：`ExcelWriterSheetBuilder.doFill(Object, FillConfig)`.
    ///
    /// # Errors
    ///
    /// Returns template, fill, CSV/XLS unsupported, or output errors.
    pub fn do_fill_with_config(
        self,
        data: &dyn std::any::Any,
        fill_config: FillConfig,
    ) -> Result<()> {
        let builder_config = BuilderFillConfig::new()
            .direction(match fill_config.get_direction() {
                FillDirection::Vertical => WriteDirection::Vertical,
                FillDirection::Horizontal => WriteDirection::Horizontal,
            })
            .force_new_row(fill_config.get_force_new_row())
            .auto_style(fill_config.get_auto_style());
        let sheet = WriteSheet::<DynamicRow>::from_options(self.options.clone());
        let mut handlers = self.handlers;
        handlers.extend(
            crate::write::handler_execution_scope::load_annotation_handlers::<T>(&self.options)?,
        );
        let styles = crate::write::excel_writer_core::compile_template_fill_styles::<T>(
            &self.options,
            &mut handlers,
        )?;
        let writer = ExcelWriter::with_handlers_and_options(self.path, handlers, self.options);
        do_fill_template_with_compiled_styles(writer, data, builder_config, &sheet, styles)
    }

    /// Resolves fill data lazily, then delegates to [`Self::do_fill`].
    ///
    /// 对应 Java：`doFill(Supplier<Object>)`.
    ///
    /// # Errors
    ///
    /// Returns template, fill, CSV/XLS unsupported, or output errors.
    pub fn do_fill_with<D, F>(self, supplier: F) -> Result<()>
    where
        D: std::any::Any,
        F: FnOnce() -> D,
    {
        let data = supplier();
        self.do_fill(&data)
    }

    /// Resolves fill data lazily and applies an explicit fill configuration.
    ///
    /// 对应 Java：`doFill(Supplier<Object>, FillConfig)`.
    ///
    /// # Errors
    ///
    /// Returns template, fill, CSV/XLS unsupported, or output errors.
    pub fn do_fill_with_config_supplier<D, F>(
        self,
        supplier: F,
        fill_config: FillConfig,
    ) -> Result<()>
    where
        D: std::any::Any,
        F: FnOnce() -> D,
    {
        let data = supplier();
        self.do_fill_with_config(&data, fill_config)
    }
}

pub(crate) fn stateful_constant_memory_is_safe<T>(
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
    has_template: bool,
) -> bool
where
    T: ExcelRow,
{
    if !stateful_streaming_configuration_is_safe::<T>(options, handlers, has_template) {
        return false;
    }

    T::schema().iter().all(|column| {
        column.field_type.is_some_and(is_streaming_scalar_type)
            && column.loop_merge.is_none()
            && column.image_path.is_none()
            && column.comment.is_none()
            && column.hyperlink.is_none()
            && column.formula.is_none()
            && column.data_validation.is_none()
            && column.conditional_format.is_none()
            && !column.auto_filter
    })
}

pub(crate) fn stateful_streaming_configuration_is_safe<T>(
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
    has_template: bool,
) -> bool
where
    T: ExcelRow,
{
    !(has_template
        || handlers
            .iter()
            .any(|handler| !handler.backend_capability().is_streaming_safe())
        || options.auto_width
        || !options.merge_ranges.is_empty()
        || !options.loop_merges.is_empty()
        || T::write_metadata().once_absolute_merge.is_some())
}

fn is_streaming_scalar_type(field_type: &str) -> bool {
    let compact = field_type.replace(' ', "");
    if let Some(inner) = compact
        .strip_prefix("Option<")
        .and_then(|value| value.strip_suffix('>'))
    {
        return is_streaming_scalar_type(inner);
    }
    matches!(
        compact.as_str(),
        "String"
            | "bool"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "NaiveDate"
            | "NaiveDateTime"
            | "BigDecimal"
            | "BigInt"
    ) || compact.ends_with("::NaiveDate")
        || compact.ends_with("::NaiveDateTime")
        || compact.ends_with("::BigDecimal")
        || compact.ends_with("::BigInt")
}

#[cfg(test)]
mod automatic_memory_tests {
    use std::io::Read;

    use super::*;

    #[derive(crate::ExcelRow)]
    struct ScalarRow {
        id: i64,
        name: String,
    }

    struct ProbeHandler;
    impl WriteHandler for ProbeHandler {}

    #[test]
    fn plain_xlsx_automatically_selects_constant_memory() {
        let mut builder = ExcelWriterBuilder::<ScalarRow>::new("auto.xlsx".into());
        builder.resolve_automatic_memory_selection(crate::support::ExcelTypeEnum::Xlsx, false);
        assert!(builder.options.constant_memory);
    }

    #[test]
    fn advanced_or_explicit_writes_do_not_get_overridden_by_auto_selection() {
        let mut with_handler = ExcelWriterBuilder::<ScalarRow>::new("handler.xlsx".into())
            .register_write_handler(ProbeHandler);
        with_handler.resolve_automatic_memory_selection(crate::support::ExcelTypeEnum::Xlsx, false);
        assert!(!with_handler.options.constant_memory);

        let mut explicit_in_memory =
            ExcelWriterBuilder::<ScalarRow>::new("memory.xlsx".into()).in_memory(true);
        explicit_in_memory
            .resolve_automatic_memory_selection(crate::support::ExcelTypeEnum::Xlsx, false);
        assert!(!explicit_in_memory.options.constant_memory);
    }

    #[test]
    fn streaming_scalar_type_filter_rejects_image_like_fields() {
        assert!(is_streaming_scalar_type("Option<chrono::NaiveDate>"));
        assert!(is_streaming_scalar_type("String"));
        assert!(!is_streaming_scalar_type("Vec<u8>"));
        assert!(!is_streaming_scalar_type("DynamicValue"));
    }

    #[test]
    fn default_one_shot_write_uses_streaming_xlsx_package()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("automatic-streaming.xlsx");
        ExcelWriterBuilder::<ScalarRow>::new(path.clone()).do_write((0..100).map(|id| {
            ScalarRow {
                id: i64::from(id),
                name: format!("row-{id}"),
            }
        }))?;

        let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
        assert!(archive.by_name("xl/sharedStrings.xml").is_err());
        let mut worksheet = String::new();
        archive
            .by_name("xl/worksheets/sheet1.xml")?
            .read_to_string(&mut worksheet)?;
        assert!(worksheet.contains("t=\"inlineStr\""));
        assert!(worksheet.contains("row-99"));
        Ok(())
    }
}
