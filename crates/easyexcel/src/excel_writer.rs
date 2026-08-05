//! 有状态 Excel 写入器。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter`
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/ExcelWriter.java

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use crate::core::{
    ConverterRegistry, CsvCharset, ExcelColumn, ExcelError, ExcelRow, Result, WriteHandler,
    WriteSheetContext, WriteWorkbookContext,
};
use crate::util::work_book_util::create_sheet;
use easyexcel_xlsx::xlsx::generation::{self, Workbook};

use crate::write::append_rows::append_rows_to_worksheet_with_gzip_and_context;
use crate::write::biff8::Biff8Book;
use easyexcel_csv::CsvRecordWriter;
use crate::write::excel_output_stream::ExcelOutputStream;
use crate::write::excel_writer_core::{
    CapturedOutput, HandlerHolderScope, after_sheet, after_sheet_create, after_workbook,
    after_workbook_create, append_csv_rows, append_rows_to_biff8_sheet,
    apply_annotation_column_widths, apply_biff8_column_widths,
    apply_biff8_once_absolute_merge_property, apply_handler_column_widths,
    apply_once_absolute_merge_property, apply_template_holder_layout,
    automatic_dynamic_head_merge_ranges, before_sheet, before_workbook,
    collect_handler_once_absolute_merges, collect_once_absolute_merges,
    collect_template_append_rows, create_csv_record_writer, create_stateful_csv_writer,
    finish_csv_record_writer, format_error, handlers_request_auto_width, head_rows_for_schema,
    is_csv_path, is_xls_path, merge_range_to_biff8, relative_head_start_row, resolve_excel_type,
    run_own_workbook_callbacks, run_template_handler_callbacks, save_template_package,
    save_workbook, save_workbook_to_writer, save_xls_book, set_xlsx_column_width_chars,
    sort_handlers, take_captured_output, template_append_cell_styles, template_append_row_heights,
    validate_excel_row_schema, validate_stateful_backend, validate_stateful_schema,
    write_sheet_to_workbook_with_gzip,
};
use crate::write::handler::default_write_handler_loader::DefaultWriteHandlerLoader;
use crate::write::handler_execution_scope::{
    HandlerExecutionScope, ensure_gzip_spill, load_annotation_handlers,
};
use crate::write::metadata::write_table::WriteTable as MirroredWriteTable;
use crate::write::shared_write_handler::{
    SharedWriteHandler, StatefulSheetState, boxed_handlers, share_handlers,
};
use crate::write::write_options::WriteOptions;
use crate::write::write_progress::WriteProgress;
use crate::write::write_sheet::WriteSheet;

/// Stateful XLSX or single-sheet CSV writer matching Java `ExcelWriter`'s lifecycle.
#[allow(clippy::struct_excessive_bools)]
pub struct ExcelWriter {
    path: PathBuf,
    excel_type: Option<crate::support::ExcelTypeEnum>,
    output_stream: Option<Box<dyn Write + Send>>,
    close_stream: Option<Box<dyn FnOnce() -> std::io::Result<()> + Send>>,
    pub(crate) workbook: Workbook,
    xls_book: Biff8Book,
    pub(crate) workbook_handlers: Vec<SharedWriteHandler>,
    pub(crate) sheet_annotation_handlers: HashMap<String, Vec<SharedWriteHandler>>,
    sheet_handlers: HashMap<String, Vec<SharedWriteHandler>>,
    table_annotation_handlers: HashMap<(String, i32), Vec<SharedWriteHandler>>,
    table_handlers: HashMap<(String, i32), Vec<SharedWriteHandler>>,
    table_schemas: HashMap<(String, i32), &'static [ExcelColumn]>,
    current_effective_handlers: Vec<SharedWriteHandler>,
    sheets: HashMap<String, StatefulSheetState>,
    sheet_indexes: HashMap<usize, String>,
    pub(crate) csv_writer: Option<CsvRecordWriter>,
    csv_capture: Option<CapturedOutput>,
    csv_charset: CsvCharset,
    csv_with_bom: bool,
    started: bool,
    finished: bool,
    auto_close_stream: bool,
    write_excel_on_exception: bool,
    password: Option<String>,
    converters: ConverterRegistry,
    /// Workbook-level spill preference from the builder. (Java SXSSF `setCompressTempFiles`)
    compress_temp_files: bool,
    /// Workbook-level constant-memory default from the builder.
    default_constant_memory: bool,
    template_file: Option<PathBuf>,
    template_bytes: Option<Vec<u8>>,
    /// First-write markers for sheets present in a `withTemplate` package.
    template_pending_rows: HashMap<String, u32>,
    /// ZIP/OOXML package used when preserving template styles and merges.
    template_package: Option<crate::write::template_write::TemplatePackage>,
    /// OLE/BIFF8 package used when `with_template` targets a `.xls` workbook.
    ///
    /// Java mapping: `HSSFWorkbook(template)` + append cells; unmodified BIFF
    /// records are copied verbatim ([`crate::write::biff8::Biff8TemplatePackage`]).
    xls_template: Option<crate::write::biff8::Biff8TemplatePackage>,
    /// Explicit legacy value-replay for `with_template` (styles/merges discarded).
    use_legacy_template_seed: bool,
    /// Active gzip spill writers keyed by sheet name (when `compress_temp_files`).
    pub(crate) gzip_spills: HashMap<String, crate::write::gzip_spill::GzipSheetDataWriter>,
    /// Last finished gzip spill snapshot (for tests / observability).
    last_gzip_spill: Option<crate::write::gzip_spill::GzipSpillSnapshot>,
}

impl ExcelWriter {
    /// Creates a multi-sheet writer without handlers.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::with_handlers(path, Vec::new())
    }

    /// Creates a multi-sheet writer with owned lifecycle handlers.
    #[must_use]
    pub fn with_handlers(path: impl Into<PathBuf>, handlers: Vec<Box<dyn WriteHandler>>) -> Self {
        Self::with_handlers_and_password(path, handlers, None)
    }

    /// Creates a multi-sheet writer with handlers and optional XLSX encryption.
    #[must_use]
    pub fn with_handlers_and_password(
        path: impl Into<PathBuf>,
        handlers: Vec<Box<dyn WriteHandler>>,
        password: Option<String>,
    ) -> Self {
        Self::with_handlers_and_options(
            path,
            handlers,
            WriteOptions {
                password,
                ..WriteOptions::default()
            },
        )
    }

    /// Creates a stateful writer with workbook-level builder options.
    #[must_use]
    pub fn with_handlers_and_options(
        path: impl Into<PathBuf>,
        mut handlers: Vec<Box<dyn WriteHandler>>,
        options: WriteOptions,
    ) -> Self {
        let path = path.into();
        let excel_type = resolve_excel_type(&path, &options);
        handlers.extend(DefaultWriteHandlerLoader::load_default_handler_for(
            options.use_default_style,
            excel_type,
        ));
        let converters =
            crate::converters::default_converter_loader::load_default_write_converter()
                .merged_with(&options.converters);
        let workbook_handlers = share_handlers(handlers);
        let current_effective_handlers = HandlerExecutionScope::root(&workbook_handlers).effective;
        Self {
            path,
            excel_type: options.excel_type,
            output_stream: None,
            close_stream: None,
            workbook: easyexcel_xlsx::xlsx::generation::new_workbook(),
            xls_book: Biff8Book::default(),
            workbook_handlers: workbook_handlers.clone(),
            sheet_annotation_handlers: HashMap::new(),
            sheet_handlers: HashMap::new(),
            table_annotation_handlers: HashMap::new(),
            table_handlers: HashMap::new(),
            table_schemas: HashMap::new(),
            current_effective_handlers,
            sheets: HashMap::new(),
            sheet_indexes: HashMap::new(),
            csv_writer: None,
            csv_capture: None,
            csv_charset: options.charset,
            csv_with_bom: options.with_bom,
            started: false,
            finished: false,
            auto_close_stream: options.auto_close_stream,
            write_excel_on_exception: options.write_excel_on_exception,
            password: options.password,
            converters,
            compress_temp_files: options.compress_temp_files,
            default_constant_memory: options.constant_memory || options.compress_temp_files,
            template_file: options.template_file,
            template_bytes: options.template_bytes,
            template_pending_rows: HashMap::new(),
            template_package: None,
            xls_template: None,
            use_legacy_template_seed: options.use_legacy_template_seed,
            gzip_spills: HashMap::new(),
            last_gzip_spill: None,
        }
    }

    /// Creates a stateful writer backed by a cloneable output stream.
    #[must_use]
    pub fn with_output_stream<W>(
        logical_path: impl Into<PathBuf>,
        output: ExcelOutputStream<W>,
        mut handlers: Vec<Box<dyn WriteHandler>>,
        options: WriteOptions,
    ) -> Self
    where
        W: Write + Send + 'static,
    {
        let path = logical_path.into();
        let excel_type = resolve_excel_type(&path, &options);
        handlers.extend(DefaultWriteHandlerLoader::load_default_handler_for(
            options.use_default_style,
            excel_type,
        ));
        let converters =
            crate::converters::default_converter_loader::load_default_write_converter()
                .merged_with(&options.converters);
        let workbook_handlers = share_handlers(handlers);
        let current_effective_handlers = HandlerExecutionScope::root(&workbook_handlers).effective;
        let write_output = output.clone();
        let close_stream = Box::new(move || output.close());
        Self {
            path,
            excel_type: options.excel_type,
            output_stream: Some(Box::new(write_output)),
            close_stream: Some(close_stream),
            workbook: easyexcel_xlsx::xlsx::generation::new_workbook(),
            xls_book: Biff8Book::default(),
            workbook_handlers: workbook_handlers.clone(),
            sheet_annotation_handlers: HashMap::new(),
            sheet_handlers: HashMap::new(),
            table_annotation_handlers: HashMap::new(),
            table_handlers: HashMap::new(),
            table_schemas: HashMap::new(),
            current_effective_handlers,
            sheets: HashMap::new(),
            sheet_indexes: HashMap::new(),
            csv_writer: None,
            csv_capture: None,
            csv_charset: options.charset,
            csv_with_bom: options.with_bom,
            started: false,
            finished: false,
            auto_close_stream: options.auto_close_stream,
            write_excel_on_exception: options.write_excel_on_exception,
            password: options.password,
            converters,
            compress_temp_files: options.compress_temp_files,
            default_constant_memory: options.constant_memory || options.compress_temp_files,
            template_file: options.template_file,
            template_bytes: options.template_bytes,
            template_pending_rows: HashMap::new(),
            template_package: None,
            xls_template: None,
            use_legacy_template_seed: options.use_legacy_template_seed,
            gzip_spills: HashMap::new(),
            last_gzip_spill: None,
        }
    }

    /// Registers an additional handler before the first write starts.
    ///
    /// This is used by Java-compatible sheet builders, where handlers are
    /// attached after the workbook writer has been constructed but before
    /// `doWrite` begins.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Unsupported`] when the writer has already
    /// started writing.
    pub fn register_write_handler(&mut self, handler: Box<dyn WriteHandler>) -> Result<&mut Self> {
        if self.started {
            return Err(ExcelError::Unsupported(
                "write handlers must be registered before the first write".to_owned(),
            ));
        }
        self.workbook_handlers
            .push(SharedWriteHandler::new(handler));
        self.current_effective_handlers = self.workbook_handler_scope().effective;
        Ok(self)
    }

    /// Prepends handlers owned by a more specific Java write holder.
    ///
    /// Java builds each effective handler list as `own handlers + parent
    /// handlers` before the stable `order()` sort. Consequently an own
    /// handler wins `NotRepeatExecutor` de-duplication when both handlers have
    /// the same order and unique value. Sheet and table builders use this
    /// method to preserve that precedence.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Unsupported`] when the writer has already
    /// started writing.
    pub fn prepend_write_handlers(
        &mut self,
        handlers: Vec<Box<dyn WriteHandler>>,
    ) -> Result<&mut Self> {
        if self.started {
            return Err(ExcelError::Unsupported(
                "write handlers must be registered before the first write".to_owned(),
            ));
        }
        let mut handlers = share_handlers(handlers);
        handlers.append(&mut self.workbook_handlers);
        self.workbook_handlers = handlers;
        self.current_effective_handlers = self.workbook_handler_scope().effective;
        Ok(self)
    }

    /// Writes a batch to a worksheet, appending when the sheet was used before.
    ///
    /// XLSX and BIFF8 (`.xls`) permit multiple sheets. CSV permits repeated writes
    /// to one logical sheet, matching Java `EasyExcel`'s stateful writer.
    ///
    /// # Errors
    ///
    /// Returns an error when the writer is finished, a handler fails, or data cannot be written.
    pub fn write<T, I>(&mut self, rows: I, sheet: &WriteSheet<T>) -> Result<&mut Self>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        if self.finished {
            return Err(ExcelError::Unsupported(
                "writer already finished".to_owned(),
            ));
        }
        validate_excel_row_schema::<T>()?;
        self.start()?;
        let sheet_name = self
            .resolve_sheet_name(sheet.options())
            .unwrap_or_else(|| sheet.options().sheet_name.clone());
        self.ensure_sheet_annotation_handlers::<T>(&sheet_name, sheet.options())?;
        let handler_scope = self.sheet_handler_scope(&sheet_name);
        let mut handlers = handler_scope.effective_boxed();
        if self.is_csv() {
            self.write_csv_batch::<T, I>(rows, sheet, &mut handlers, false, false, false, None)?;
        } else if self.is_xls() {
            self.write_xls_batch::<T, I>(rows, sheet, &mut handlers, false, false, false, None)?;
        } else {
            self.write_xlsx_batch::<T, I>(rows, sheet, &mut handlers, false, false, false, None)?;
        }
        self.current_effective_handlers = handler_scope.effective;
        debug_assert!(self.resolve_sheet_name(sheet.options()).is_some());
        Ok(self)
    }

    /// Writes with handlers owned by this Sheet holder.
    ///
    /// Java stores these handlers on `WriteSheetHolder`, runs only their
    /// workbook hooks as supplementary callbacks when the holder is first
    /// initialized, then executes `sheet own + workbook parent` for sheet,
    /// row, and cell events.
    ///
    /// # Errors
    ///
    /// Returns an error when the writer is finished, a handler fails, or
    /// data cannot be written.
    pub fn write_with_sheet_handlers<T, I>(
        &mut self,
        rows: I,
        sheet: &WriteSheet<T>,
        handlers: Vec<Box<dyn WriteHandler>>,
    ) -> Result<&mut Self>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        if self.finished {
            return Err(ExcelError::Unsupported(
                "writer already finished".to_owned(),
            ));
        }
        validate_excel_row_schema::<T>()?;
        self.start()?;
        let sheet_name = self
            .resolve_sheet_name(sheet.options())
            .unwrap_or_else(|| sheet.options().sheet_name.clone());
        let is_initialized = self.sheets.contains_key(&sheet_name);
        if is_initialized && !handlers.is_empty() && !self.sheet_handlers.contains_key(&sheet_name)
        {
            return Err(ExcelError::Unsupported(format!(
                "sheet handlers must be registered before sheet '{sheet_name}' is initialized"
            )));
        }
        if !handlers.is_empty() {
            if self.sheet_handlers.contains_key(&sheet_name) {
                return Err(ExcelError::Unsupported(format!(
                    "sheet handlers for '{sheet_name}' are already registered"
                )));
            }
            let own_handlers = share_handlers(handlers);
            if !is_initialized {
                let parent = self.workbook_handler_scope();
                let scope = HandlerExecutionScope::child(&own_handlers, &parent);
                run_own_workbook_callbacks(&scope, &self.path)?;
            }
            self.sheet_handlers.insert(sheet_name.clone(), own_handlers);
        }
        self.write(rows, sheet)
    }

    fn workbook_handler_scope(&self) -> HandlerExecutionScope {
        HandlerExecutionScope::root(&self.workbook_handlers)
    }

    pub(crate) fn sheet_handler_scope(&self, sheet_name: &str) -> HandlerExecutionScope {
        let mut own_handlers = self
            .sheet_annotation_handlers
            .get(sheet_name)
            .cloned()
            .unwrap_or_default();
        own_handlers.extend(
            self.sheet_handlers
                .get(sheet_name)
                .cloned()
                .unwrap_or_default(),
        );
        HandlerExecutionScope::child(&own_handlers, &self.workbook_handler_scope())
    }

    fn table_handler_scope(&self, sheet_name: &str, table_no: i32) -> HandlerExecutionScope {
        let table_key = (sheet_name.to_owned(), table_no);
        let mut own_handlers = self
            .table_annotation_handlers
            .get(&table_key)
            .cloned()
            .unwrap_or_default();
        own_handlers.extend(
            self.table_handlers
                .get(&table_key)
                .cloned()
                .unwrap_or_default(),
        );
        HandlerExecutionScope::child(&own_handlers, &self.sheet_handler_scope(sheet_name))
    }

    fn ensure_sheet_annotation_handlers<T>(
        &mut self,
        sheet_name: &str,
        options: &WriteOptions,
    ) -> Result<()>
    where
        T: ExcelRow,
    {
        if self.sheet_annotation_handlers.contains_key(sheet_name) {
            return Ok(());
        }
        self.sheet_annotation_handlers.insert(
            sheet_name.to_owned(),
            share_handlers(load_annotation_handlers::<T>(options)?),
        );
        Ok(())
    }

    pub(crate) fn ensure_table_annotation_handlers<T>(
        &mut self,
        sheet_name: &str,
        table_no: i32,
        options: &WriteOptions,
    ) -> Result<()>
    where
        T: ExcelRow,
    {
        let table_key = (sheet_name.to_owned(), table_no);
        if self.table_annotation_handlers.contains_key(&table_key) {
            return Ok(());
        }
        self.table_annotation_handlers.insert(
            table_key,
            share_handlers(load_annotation_handlers::<T>(options)?),
        );
        Ok(())
    }

    fn initialize_existing_table_holder<T>(
        &mut self,
        sheet_name: &str,
        table_no: i32,
        options: &WriteOptions,
    ) -> Result<()>
    where
        T: ExcelRow,
    {
        let own_handlers = self.table_handler_scope(sheet_name, table_no).own_boxed();
        let parent_handlers = self.sheet_handler_scope(sheet_name).effective_boxed();
        // The parent list must only describe merges that were actually installed
        // by the parent holder. `T` is the table row type here; treating its
        // metadata as parent metadata would suppress a table-only absolute merge.
        let parent_merges = collect_handler_once_absolute_merges(&parent_handlers);
        let table_merges = collect_once_absolute_merges::<T>(&own_handlers)
            .into_iter()
            .filter(|merge| !parent_merges.contains(merge))
            .collect::<Vec<_>>();
        if self.is_csv() {
            return Ok(());
        }
        if self.is_xls() {
            if self.xls_template.is_none() {
                let sheet = self.xls_book.sheet_mut(sheet_name);
                apply_biff8_column_widths::<T>(sheet, options, &own_handlers)?;
                for merge in table_merges {
                    apply_biff8_once_absolute_merge_property(sheet, merge)?;
                }
            }
            return Ok(());
        }
        if let Some(package) = self.template_package.as_mut() {
            apply_template_holder_layout::<T>(
                package,
                sheet_name,
                options,
                &own_handlers,
                &parent_merges,
            )?;
        } else {
            let worksheet = generation::worksheet_by_name(&mut self.workbook, sheet_name)
                .map_err(format_error)?;
            for (column, width) in &options.column_widths {
                set_xlsx_column_width_chars(worksheet, *column, *width)?;
            }
            apply_annotation_column_widths::<T>(worksheet, options)?;
            apply_handler_column_widths::<T>(worksheet, options, &own_handlers)?;
            for merge in table_merges {
                apply_once_absolute_merge_property(worksheet, merge)?;
            }
        }
        Ok(())
    }

    /// Three-arg write with an explicit `WriteTable`, mirroring Java
    /// `ExcelWriter.write(Collection, WriteSheet, WriteTable)`.
    ///
    /// Phase 4 addition: this overload is the canonical entry point used
    /// when a single sheet contains multiple tables (e.g. one row block
    /// followed by a second typed block). The table options
    /// (`table_no`, `need_head`, `head_style`) override the parent
    /// sheet's options via [`crate::write::builder::excel_writer_table_builder::merge_table_options`].
    ///
    /// For backward compatibility this overload currently delegates to
    /// the two-arg `write` path. The merged options are applied to the
    /// sheet for the duration of this batch.
    ///
    /// # Errors
    ///
    /// Same as `write(rows, sheet)`. In addition, returns an error when
    /// the writer is finished.
    pub fn write_with_table<T, I>(
        &mut self,
        rows: I,
        sheet: &WriteSheet<T>,
        table: &MirroredWriteTable,
    ) -> Result<&mut Self>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        self.write_with_table_handlers(rows, sheet, table, Vec::new(), Vec::new())
    }

    /// Writes through independent Sheet and Table holder handler chains.
    ///
    /// # Errors
    ///
    /// Returns an error when the writer is finished, a handler fails, or
    /// data cannot be written.
    // 语义敏感：该函数端到端对应 Java `ExcelWriter.writeWithTableHandlers`
    // 的完整流程，拆分会割裂上下文，故豁免 too_many_lines。
    #[allow(clippy::too_many_lines)]
    pub fn write_with_table_handlers<T, I>(
        &mut self,
        rows: I,
        sheet: &WriteSheet<T>,
        table: &MirroredWriteTable,
        sheet_handlers: Vec<Box<dyn WriteHandler>>,
        table_handlers: Vec<Box<dyn WriteHandler>>,
    ) -> Result<&mut Self>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        if self.finished {
            return Err(ExcelError::Unsupported(
                "writer already finished".to_owned(),
            ));
        }
        validate_excel_row_schema::<T>()?;
        self.start()?;
        let merged = crate::write::builder::excel_writer_table_builder::merge_table_options(
            sheet.options(),
            table,
        );
        let sheet_with_table: WriteSheet<T> = WriteSheet::from_options(merged);
        let sheet_name = self
            .resolve_sheet_name(sheet_with_table.options())
            .unwrap_or_else(|| sheet_with_table.options().sheet_name.clone());
        let sheet_is_new = !self.sheets.contains_key(&sheet_name);

        if !sheet_handlers.is_empty() {
            if !sheet_is_new && !self.sheet_handlers.contains_key(&sheet_name) {
                return Err(ExcelError::Unsupported(format!(
                    "sheet handlers must be registered before sheet '{sheet_name}' is initialized"
                )));
            }
            if self.sheet_handlers.contains_key(&sheet_name) {
                return Err(ExcelError::Unsupported(format!(
                    "sheet handlers for '{sheet_name}' are already registered"
                )));
            }
            let own = share_handlers(sheet_handlers);
            if sheet_is_new {
                let scope = HandlerExecutionScope::child(&own, &self.workbook_handler_scope());
                run_own_workbook_callbacks(&scope, &self.path)?;
            }
            self.sheet_handlers.insert(sheet_name.clone(), own);
        }

        if sheet_is_new {
            let holder_scope =
                self.handler_holder_scope::<T>(sheet_with_table.options(), &sheet_name, None)?;
            let sheet_context = holder_scope.sheet(WriteSheetContext::new(&sheet_name));
            let mut sheet_chain = self.sheet_handler_scope(&sheet_name).effective_boxed();
            before_sheet(&mut sheet_chain, &sheet_context)?;
            after_sheet_create(&mut sheet_chain, &sheet_context)?;
        }

        let table_no = table.table_no.max(0);
        let table_key = (sheet_name.clone(), table_no);
        let table_is_new = !self.table_handlers.contains_key(&table_key);
        if let Some(schema) = self.table_schemas.get(&table_key) {
            if *schema != T::schema() {
                return Err(ExcelError::Unsupported(format!(
                    "table {table_no} on sheet '{sheet_name}' was initialized with a different schema"
                )));
            }
        } else {
            self.table_schemas.insert(table_key.clone(), T::schema());
        }
        if table_is_new {
            self.ensure_table_annotation_handlers::<T>(
                &sheet_name,
                table_no,
                sheet_with_table.options(),
            )?;
            let own = share_handlers(table_handlers);
            let mut all_own = self
                .table_annotation_handlers
                .get(&table_key)
                .cloned()
                .unwrap_or_default();
            all_own.extend(own.iter().cloned());
            let execution_scope =
                HandlerExecutionScope::child(&all_own, &self.sheet_handler_scope(&sheet_name));
            run_own_workbook_callbacks(&execution_scope, &self.path)?;
            let mut supplementary = execution_scope.own_boxed();
            let holder_scope =
                self.handler_holder_scope::<T>(sheet_with_table.options(), &sheet_name, None)?;
            let sheet_context = holder_scope.sheet(WriteSheetContext::new(&sheet_name));
            before_sheet(&mut supplementary, &sheet_context)?;
            after_sheet_create(&mut supplementary, &sheet_context)?;
            self.table_handlers.insert(table_key, own);
        } else if !table_handlers.is_empty() {
            return Err(ExcelError::Unsupported(format!(
                "table handlers for '{sheet_name}' table {table_no} are already registered"
            )));
        }
        if table_is_new && !sheet_is_new {
            self.initialize_existing_table_holder::<T>(
                &sheet_name,
                table_no,
                sheet_with_table.options(),
            )?;
        }

        let handler_scope = self.table_handler_scope(&sheet_name, table_no);
        let mut handlers = handler_scope.effective_boxed();
        if self.is_csv() {
            self.write_csv_batch::<T, I>(
                rows,
                &sheet_with_table,
                &mut handlers,
                sheet_is_new,
                true,
                table_is_new,
                Some(table_no),
            )?;
        } else if self.is_xls() {
            self.write_xls_batch::<T, I>(
                rows,
                &sheet_with_table,
                &mut handlers,
                sheet_is_new,
                true,
                table_is_new,
                Some(table_no),
            )?;
        } else {
            self.write_xlsx_batch::<T, I>(
                rows,
                &sheet_with_table,
                &mut handlers,
                sheet_is_new,
                true,
                table_is_new,
                Some(table_no),
            )?;
        }
        if let Some(state) = self.sheets.get_mut(&sheet_name) {
            let mut options = sheet.options().clone();
            options.sheet_name.clone_from(&sheet_name);
            options.converters = self.converters.merged_with(&options.converters);
            options.compress_temp_files |= self.compress_temp_files;
            options.constant_memory |= self.default_constant_memory;
            state.options = options;
        }
        self.current_effective_handlers = handler_scope.effective;
        Ok(self)
    }

    /// Returns the logical output path used by Java-style builder facades.
    #[must_use]
    pub fn output_path(&self) -> &std::path::Path {
        &self.path
    }

    /// Appends raw bytes to the BIFF8 output stream. These bytes are
    /// written as an "Images" OLE stream in the CFB container when
    /// the file is serialized. Used for embedding image data in XLS.
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.xls_book.write_raw_bytes(bytes);
        self
    }

    /// Encodes image bytes as BIFF8 Obj + `MSODrawing` + Escher BSE
    /// records (POI HSSF compatible) and embeds them in the output.
    pub fn write_image(&mut self, image_data: &[u8], col: u8, row: u32) -> &mut Self {
        self.xls_book.write_image(image_data, col, row);
        self
    }

    /// Returns whether [`WriteOptions::template_file`] / `template_bytes` is set.
    ///
    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`.
    #[must_use]
    pub fn has_template_configured(&self) -> bool {
        crate::write::template_write::has_template(
            self.template_file.as_deref(),
            self.template_bytes.as_deref(),
        )
    }

    /// Returns the configured template file, if any.
    #[must_use]
    pub fn template_file(&self) -> Option<&std::path::Path> {
        self.template_file.as_deref()
    }

    /// Returns the configured in-memory template bytes, if any.
    #[must_use]
    pub fn template_bytes(&self) -> Option<&[u8]> {
        self.template_bytes.as_deref()
    }

    /// Marks the writer finished without persisting workbook output.
    ///
    /// Used when a [`WriteFillExecutor`] already wrote the filled package.
    pub(crate) fn mark_finished(&mut self) {
        self.finished = true;
    }

    /// Saves and closes the writer. Repeated calls are no-ops.
    ///
    /// # Errors
    ///
    /// Returns an output or handler error.
    pub fn finish(&mut self) -> Result<()> {
        self.finish_with_exception(false)
    }

    /// Finishes after a write-side exception.
    ///
    /// By default accumulated workbook data is discarded. Set
    /// [`WriteOptions::write_excel_on_exception`] to emit it, matching Java
    /// `EasyExcel`'s `writeExcelOnException` switch.
    ///
    /// # Errors
    ///
    /// Returns an output, close, or handler error.
    pub fn finish_on_exception(&mut self) -> Result<()> {
        self.finish_with_exception(true)
    }

    fn finish_with_exception(&mut self, on_exception: bool) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        self.start()?;
        if let Err(error) = self.finish_gzip_spills() {
            self.finished = true;
            return Err(error);
        }
        self.finished = true;
        let write_excel = !on_exception || self.write_excel_on_exception;
        let mut result = Ok(());
        if self.is_csv() {
            let writer = self
                .csv_writer
                .take()
                .expect("a successfully started CSV writer must own its record writer");
            if let Err(error) = finish_csv_record_writer(writer) {
                result = Err(error);
            }
            if write_excel && let Some(capture) = self.csv_capture.take() {
                match take_captured_output(&capture).and_then(|bytes| {
                    let output = self
                        .output_stream
                        .as_mut()
                        .expect("CSV capture requires an output stream");
                    easyexcel_io::write_all_and_flush(output.as_mut(), &bytes)?;
                    Ok(())
                }) {
                    Ok(()) => {}
                    Err(error) => result = Err(error),
                }
            }
        } else if write_excel && self.is_xls() {
            let save_result = if let Some(package) = self.xls_template.take() {
                if let Some(output) = self.output_stream.as_mut() {
                    package.save_to_writer(output.as_mut())
                } else {
                    package.save_to_path(&self.path)
                }
            } else if let Some(output) = self.output_stream.as_mut() {
                self.xls_book
                    .write_to(output.as_mut())
                    .map_err(ExcelError::from)
            } else {
                save_xls_book(&self.xls_book, &self.path)
            };
            if let Err(error) = save_result {
                result = Err(error);
            }
        } else if write_excel {
            let save_result = if let Some(package) = self.template_package.take() {
                save_template_package(
                    &package,
                    &self.path,
                    self.output_stream
                        .as_mut()
                        .map(|output| output.as_mut() as &mut (dyn Write + Send)),
                    self.password.as_deref(),
                )
            } else if let Some(output) = self.output_stream.as_mut() {
                save_workbook_to_writer(
                    &mut self.workbook,
                    output.as_mut(),
                    self.password.as_deref(),
                )
            } else {
                save_workbook(&mut self.workbook, &self.path, self.password.as_deref())
            };
            if let Err(error) = save_result {
                result = Err(error);
            }
        }
        let context = WriteWorkbookContext::new(&self.path);
        let mut handlers = boxed_handlers(&self.current_effective_handlers);
        sort_handlers(&mut handlers);
        if let Err(error) = after_workbook(&mut handlers, &context) {
            result = Err(error);
        }
        if self.auto_close_stream
            && let Some(close) = self.close_stream.take()
            && let Err(error) = close()
        {
            result = Err(ExcelError::Io(error));
        }
        result
    }

    /// Returns whether [`Self::finish`] completed successfully.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    /// Returns the underlying `rust_xlsxwriter` workbook for advanced XLSX customization.
    ///
    /// Callers are responsible for preserving valid worksheet names and
    /// workbook invariants. CSV writers do not use this workbook.
    #[must_use]
    pub fn workbook_mut(&mut self) -> &mut Workbook {
        &mut self.workbook
    }

    /// Enables SXSSF-style compressed / disk-spill temp files for later sheets.
    ///
    /// Java mapping: `SXSSFWorkbook.setCompressTempFiles(true)`, typically called from
    /// `WorkbookWriteHandler.afterWorkbookCreate`. Call this before the first
    /// `write` that creates a worksheet. Already-created sheets keep their mode.
    pub fn set_compress_temp_files(&mut self, enabled: bool) -> &mut Self {
        self.compress_temp_files = enabled;
        if enabled {
            self.default_constant_memory = true;
        }
        self
    }

    /// Returns whether workbook-level temp-file compression / spill is enabled.
    #[must_use]
    pub const fn compress_temp_files_enabled(&self) -> bool {
        self.compress_temp_files
    }

    /// Last finished gzip spill snapshot (Java SXSSF compressed temp observability).
    ///
    /// Populated when [`Self::finish`] closes active [`crate::write::gzip_spill::GzipSheetDataWriter`]s.
    #[must_use]
    pub const fn last_gzip_spill_snapshot(
        &self,
    ) -> Option<&crate::write::gzip_spill::GzipSpillSnapshot> {
        self.last_gzip_spill.as_ref()
    }

    /// Finishes active gzip spill writers and retains the last snapshot.
    fn finish_gzip_spills(&mut self) -> Result<()> {
        let spills = std::mem::take(&mut self.gzip_spills);
        for (_, spill) in spills {
            let reader = spill.finish()?;
            self.last_gzip_spill = Some(reader.snapshot());
        }
        Ok(())
    }

    /// Applies workbook-level spill defaults onto a sheet's write options.
    fn apply_workbook_spill_defaults(&self, options: &mut WriteOptions) {
        if self.compress_temp_files {
            options.compress_temp_files = true;
        }
        if self.default_constant_memory || options.compress_temp_files {
            options.constant_memory = true;
        }
    }

    pub(crate) fn start(&mut self) -> Result<()> {
        if self.started {
            return Ok(());
        }
        validate_stateful_backend(self.is_csv(), self.password.as_deref())?;
        if crate::write::template_write::has_template(
            self.template_file.as_deref(),
            self.template_bytes.as_deref(),
        ) {
            if self.is_csv() {
                return Err(ExcelError::Unsupported(
                    "csv cannot use template.".to_owned(),
                ));
            }
            if self.is_xls() {
                // Java: withTemplate(.xls) → HSSFWorkbook(template) + append.
                let bytes = crate::write::template_write::load_template_bytes(
                    self.template_file.as_deref(),
                    self.template_bytes.as_deref(),
                )?;
                if !crate::write::biff8::looks_like_xls(&bytes) {
                    return Err(ExcelError::Format(
                        "xls with_template requires an OLE .xls workbook".to_owned(),
                    ));
                }
                let package = crate::write::biff8::Biff8TemplatePackage::from_bytes(&bytes)?;
                for (index, name) in package.sheet_names().into_iter().enumerate() {
                    let next_row = package.next_row_for_sheet(&name)?;
                    self.sheet_indexes.insert(index, name.clone());
                    self.template_pending_rows.insert(name, next_row);
                }
                self.xls_template = Some(package);
            } else {
                crate::write::template_write::validate_template_source(
                    self.template_file.as_deref(),
                    self.template_bytes.as_deref(),
                )?;
                let bytes = crate::write::template_write::load_template_bytes(
                    self.template_file.as_deref(),
                    self.template_bytes.as_deref(),
                )?;
                if self.use_legacy_template_seed {
                    // Explicit legacy fallback: value replay without styles/merges.
                    let sheets = easyexcel_xlsx::load_legacy_template_sheets(&bytes)?;
                    easyexcel_xlsx::seed_legacy_template_workbook(&mut self.workbook, &sheets)?;
                    for (index, sheet) in sheets.into_iter().enumerate() {
                        self.sheet_indexes.insert(index, sheet.name.clone());
                        self.template_pending_rows
                            .insert(sheet.name, sheet.next_row);
                    }
                } else {
                    // Default ZIP preserve path: keep styles.xml / mergeCells, append sheetData.
                    let package =
                        crate::write::template_write::TemplatePackage::from_bytes(&bytes)?;
                    for (index, name) in package.sheet_names()?.into_iter().enumerate() {
                        let next_row = package.next_row_for_sheet(&name)?;
                        self.sheet_indexes.insert(index, name.clone());
                        self.template_pending_rows.insert(name, next_row);
                    }
                    self.template_package = Some(package);
                }
            }
        }
        let mut handlers = self.workbook_handler_scope().effective_boxed();
        let context = WriteWorkbookContext::new(&self.path);
        before_workbook(&mut handlers, &context)?;
        after_workbook_create(&mut handlers, &context)?;
        if self.is_csv() {
            if self.output_stream.is_some() {
                let capture = CapturedOutput::default();
                self.csv_writer = Some(create_csv_record_writer(
                    Box::new(capture.clone()),
                    &self.csv_charset,
                    self.csv_with_bom,
                )?);
                self.csv_capture = Some(capture);
            } else {
                self.csv_writer = Some(create_stateful_csv_writer(
                    &self.path,
                    &self.csv_charset,
                    self.csv_with_bom,
                )?);
            }
        }
        self.started = true;
        Ok(())
    }

    pub(crate) fn is_csv(&self) -> bool {
        match self.excel_type {
            Some(excel_type) => excel_type == crate::support::ExcelTypeEnum::Csv,
            None => is_csv_path(&self.path),
        }
    }

    pub(crate) fn is_xls(&self) -> bool {
        match self.excel_type {
            Some(excel_type) => excel_type == crate::support::ExcelTypeEnum::Xls,
            None => is_xls_path(&self.path),
        }
    }

    // 语义敏感：参数与 Java `ExcelWriter.writeXlsBatch` 的写入路径参数一一对应，
    // 拆分结构体会破坏 1:1 可追溯性；函数体端到端覆盖完整写入流程，
    // 拆分会割裂上下文，故豁免 too_many_arguments / too_many_lines。
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn write_xls_batch<T, I>(
        &mut self,
        rows: I,
        sheet: &WriteSheet<T>,
        handlers: &mut [Box<dyn WriteHandler>],
        skip_sheet_create_callbacks: bool,
        use_incoming_options: bool,
        initialize_holder_head: bool,
        active_table_no: Option<i32>,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        if self.xls_template.is_some() {
            return self.write_xls_batch_onto_template::<T, I>(
                rows,
                sheet,
                handlers,
                skip_sheet_create_callbacks,
                use_incoming_options,
                initialize_holder_head,
                active_table_no,
            );
        }
        let requested_name = sheet.options().sheet_name.clone();
        let existing_name = self.resolve_sheet_name(sheet.options());
        let sheet_name = existing_name.unwrap_or_else(|| requested_name.clone());
        let (state, is_new) = if let Some(state) = self.sheets.get(&sheet_name).cloned() {
            if !use_incoming_options {
                validate_stateful_schema(&sheet_name, &state, T::schema())?;
            }
            (state, false)
        } else {
            let mut options = sheet.options().clone();
            options.converters = self.converters.merged_with(&options.converters);
            (
                StatefulSheetState {
                    schema: T::schema(),
                    metadata: *T::write_metadata(),
                    options,
                    next_row: 0,
                    next_data_index: 0,
                },
                true,
            )
        };
        let mut batch_options = if use_incoming_options {
            let mut options = sheet.options().clone();
            options.converters = self.converters.merged_with(&options.converters);
            options
        } else {
            state.options.clone()
        };
        batch_options.sheet_name.clone_from(&sheet_name);

        let holder_scope =
            self.handler_holder_scope::<T>(sheet.options(), &sheet_name, active_table_no)?;
        let sheet_context = holder_scope.sheet(WriteSheetContext::new(&sheet_name));
        if is_new && !skip_sheet_create_callbacks {
            before_sheet(handlers, &sheet_context)?;
            after_sheet_create(handlers, &sheet_context)?;
        }
        let progress = {
            let next_row = if is_new {
                relative_head_start_row(&batch_options)
            } else {
                state.next_row
            };
            if is_new {
                let biff_sheet = create_sheet(&mut self.xls_book, &sheet_name)?;
                biff_sheet.next_row = next_row;
                biff_sheet.next_data_index = state.next_data_index;
            }
            append_rows_to_biff8_sheet::<T, I>(
                &mut self.xls_book,
                &sheet_name,
                &batch_options,
                rows,
                handlers,
                WriteProgress {
                    next_row,
                    next_data_index: state.next_data_index,
                },
                is_new || initialize_holder_head,
                Some(&holder_scope),
            )?
        };
        if is_new {
            after_sheet(handlers, &sheet_context)?;
        }
        self.sheets.insert(
            sheet_name.clone(),
            StatefulSheetState {
                next_row: progress.next_row,
                next_data_index: progress.next_data_index,
                ..state
            },
        );
        self.remember_sheet_index(sheet.options().sheet_index, &sheet_name);
        Ok(())
    }

    /// Appends typed rows onto a record-preserving `.xls` template package.
    ///
    /// Mirrors [`Self::write_xlsx_batch_onto_template_package`] for HSSF/BIFF8.
    /// Creating sheets absent from the template remains unsupported (MVP).
    // 语义敏感：参数与 Java 对应写入路径一一对应，拆分结构体会破坏
    // 1:1 可追溯性；函数体端到端覆盖完整写入流程，故豁免
    // too_many_arguments / too_many_lines。
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn write_xls_batch_onto_template<T, I>(
        &mut self,
        rows: I,
        sheet: &WriteSheet<T>,
        handlers: &mut [Box<dyn WriteHandler>],
        skip_sheet_create_callbacks: bool,
        use_incoming_options: bool,
        initialize_holder_head: bool,
        active_table_no: Option<i32>,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        let sheet_names = {
            let package = self
                .xls_template
                .as_ref()
                .expect("xls template must exist for BIFF preserve path");
            package.sheet_names()
        };
        let (_target_index, target_name, create_new) =
            crate::write::template_write::resolve_package_target(
                &sheet_names,
                sheet.options().sheet_index,
                &sheet.options().sheet_name,
            );
        if create_new {
            return Err(ExcelError::Unsupported(
                "xls template cannot create sheets absent from the template".to_owned(),
            ));
        }
        let sheet_name = target_name;
        let (state, is_new) = if let Some(state) = self.sheets.get(&sheet_name).cloned() {
            if !use_incoming_options {
                validate_stateful_schema(&sheet_name, &state, T::schema())?;
            }
            (state, false)
        } else {
            let mut options = sheet.options().clone();
            options.sheet_name.clone_from(&sheet_name);
            options.converters = self.converters.merged_with(&options.converters);
            let next_row = self
                .template_pending_rows
                .get(&sheet_name)
                .copied()
                .unwrap_or(0);
            (
                StatefulSheetState {
                    schema: T::schema(),
                    metadata: *T::write_metadata(),
                    options,
                    next_row,
                    next_data_index: 0,
                },
                true,
            )
        };
        let mut batch_options = if use_incoming_options {
            let mut options = sheet.options().clone();
            options.converters = self.converters.merged_with(&options.converters);
            options
        } else {
            state.options.clone()
        };
        batch_options.sheet_name.clone_from(&sheet_name);
        let holder_scope =
            self.handler_holder_scope::<T>(sheet.options(), &sheet_name, active_table_no)?;
        let sheet_context = holder_scope.sheet(WriteSheetContext::new(&sheet_name));
        if is_new && !skip_sheet_create_callbacks {
            before_sheet(handlers, &sheet_context)?;
            after_sheet_create(handlers, &sheet_context)?;
        }
        let first_write = self.template_pending_rows.remove(&sheet_name).is_some() || is_new;
        let write_head = first_write || initialize_holder_head;
        let start_row = self
            .xls_template
            .as_ref()
            .expect("xls template must exist for BIFF preserve path")
            .next_row_for_sheet(&sheet_name)?;
        if first_write {
            let head_merges =
                automatic_dynamic_head_merge_ranges::<T>(&batch_options, start_row, write_head)?;
            let package = self
                .xls_template
                .as_mut()
                .expect("xls template must exist for BIFF preserve path");
            for range in head_merges {
                package.add_merge_range(&sheet_name, merge_range_to_biff8(range)?)?;
            }
        }
        let (mut append_rows, original_rows, _converted_rows, absent_rows) =
            collect_template_append_rows::<T, I>(&batch_options, rows, write_head, start_row)?;
        let _ignore_styles = run_template_handler_callbacks::<T>(
            &batch_options,
            handlers,
            &mut append_rows,
            &original_rows,
            &absent_rows,
            write_head,
            state.next_data_index,
            start_row,
            Some(&holder_scope),
        )?;
        let next_row = {
            let package = self
                .xls_template
                .as_mut()
                .expect("xls template must exist for BIFF preserve path");
            package.append_rows(&sheet_name, &append_rows)?
        };
        let head_rows = if write_head {
            usize::try_from(head_rows_for_schema(T::schema(), &batch_options)?).unwrap_or(0)
        } else {
            0
        };
        let data_added = append_rows.len().saturating_sub(head_rows).saturating_sub(
            usize::try_from(relative_head_start_row(&batch_options)).unwrap_or(usize::MAX),
        );
        if is_new {
            after_sheet(handlers, &sheet_context)?;
        }
        self.sheets.insert(
            sheet_name.clone(),
            StatefulSheetState {
                next_row,
                next_data_index: state.next_data_index.saturating_add(data_added),
                ..state
            },
        );
        self.remember_sheet_index(sheet.options().sheet_index, &sheet_name);
        Ok(())
    }

    // 语义敏感：参数与 Java `ExcelWriter.writeXlsxBatch` 的写入路径参数一一对应，
    // 拆分结构体会破坏 1:1 可追溯性；函数体端到端覆盖完整写入流程，
    // 拆分会割裂上下文，故豁免 too_many_arguments / too_many_lines。
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn write_xlsx_batch<T, I>(
        &mut self,
        rows: I,
        sheet: &WriteSheet<T>,
        handlers: &mut [Box<dyn WriteHandler>],
        skip_sheet_create_callbacks: bool,
        use_incoming_options: bool,
        initialize_holder_head: bool,
        active_table_no: Option<i32>,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        let requested_name = sheet.options().sheet_name.clone();
        if self.template_package.is_some() {
            return self.write_xlsx_batch_onto_template_package::<T, I>(
                rows,
                sheet,
                handlers,
                skip_sheet_create_callbacks,
                use_incoming_options,
                initialize_holder_head,
                active_table_no,
            );
        }
        if let Some(sheet_name) = self.resolve_sheet_name(sheet.options()) {
            if let Some(start_row) = self.template_pending_rows.remove(&sheet_name) {
                let mut options = sheet.options().clone();
                options.converters = self.converters.merged_with(&options.converters);
                self.apply_workbook_spill_defaults(&mut options);
                // Preserve the real template sheet name (index-based Java `.sheet()`).
                options.sheet_name.clone_from(&sheet_name);
                let holder_scope =
                    self.handler_holder_scope::<T>(sheet.options(), &sheet_name, active_table_no)?;
                let worksheet = generation::worksheet_by_name(&mut self.workbook, &sheet_name)
                    .map_err(format_error)?;
                let sheet_context = holder_scope.sheet(WriteSheetContext::new(&sheet_name));
                if !skip_sheet_create_callbacks {
                    before_sheet(handlers, &sheet_context)?;
                    after_sheet_create(handlers, &sheet_context)?;
                }
                let compress = options.compress_temp_files;
                let progress = {
                    let spill = ensure_gzip_spill(&mut self.gzip_spills, &sheet_name, compress)?;
                    append_rows_to_worksheet_with_gzip_and_context::<T, I>(
                        worksheet,
                        &options,
                        rows,
                        handlers,
                        WriteProgress {
                            next_row: start_row,
                            next_data_index: 0,
                        },
                        true,
                        T::write_metadata(),
                        spill,
                        Some(&holder_scope),
                    )?
                };
                after_sheet(handlers, &sheet_context)?;
                // Java LongestMatchColumnWidthStyleStrategy setColumnWidth after cells
                apply_handler_column_widths::<T>(worksheet, &options, handlers)?;
                if options.auto_width || handlers_request_auto_width(handlers) {
                    generation::autofit(worksheet);
                }
                self.sheets.insert(
                    sheet_name.clone(),
                    StatefulSheetState {
                        schema: T::schema(),
                        metadata: *T::write_metadata(),
                        options,
                        next_row: progress.next_row,
                        next_data_index: progress.next_data_index,
                    },
                );
                self.remember_sheet_index(sheet.options().sheet_index, &sheet_name);
                return Ok(());
            }
            let state = self
                .sheets
                .get(&sheet_name)
                .cloned()
                .expect("resolved worksheet must exist");
            if !use_incoming_options {
                validate_stateful_schema(&sheet_name, &state, T::schema())?;
            }
            let mut batch_options = if use_incoming_options {
                let mut options = sheet.options().clone();
                options.converters = self.converters.merged_with(&options.converters);
                self.apply_workbook_spill_defaults(&mut options);
                options
            } else {
                state.options.clone()
            };
            batch_options.sheet_name.clone_from(&sheet_name);
            let holder_scope =
                self.handler_holder_scope::<T>(sheet.options(), &sheet_name, active_table_no)?;
            let worksheet = generation::worksheet_by_name(&mut self.workbook, &sheet_name)
                .map_err(format_error)?;
            let compress = batch_options.compress_temp_files;
            let metadata = if use_incoming_options {
                T::write_metadata()
            } else {
                &state.metadata
            };
            let progress = {
                let spill = ensure_gzip_spill(&mut self.gzip_spills, &sheet_name, compress)?;
                append_rows_to_worksheet_with_gzip_and_context::<T, I>(
                    worksheet,
                    &batch_options,
                    rows,
                    handlers,
                    WriteProgress {
                        next_row: state.next_row,
                        next_data_index: state.next_data_index,
                    },
                    initialize_holder_head,
                    metadata,
                    spill,
                    Some(&holder_scope),
                )?
            };
            if batch_options.auto_width || handlers_request_auto_width(handlers) {
                generation::autofit(worksheet);
            }
            // Re-apply measured LongestMatch widths after incremental append.
            apply_handler_column_widths::<T>(worksheet, &batch_options, handlers)?;
            let current = self
                .sheets
                .get_mut(&sheet_name)
                .expect("stateful worksheet must exist");
            current.next_row = progress.next_row;
            current.next_data_index = progress.next_data_index;
            return Ok(());
        }

        let mut options = sheet.options().clone();
        options.converters = self.converters.merged_with(&options.converters);
        self.apply_workbook_spill_defaults(&mut options);
        let sheet_name = options.sheet_name.clone();
        let compress = options.compress_temp_files;
        let holder_scope =
            self.handler_holder_scope::<T>(sheet.options(), &sheet_name, active_table_no)?;
        let progress = {
            let spill = ensure_gzip_spill(&mut self.gzip_spills, &sheet_name, compress)?;
            write_sheet_to_workbook_with_gzip::<T, I>(
                &mut self.workbook,
                &options,
                rows,
                handlers,
                spill,
                skip_sheet_create_callbacks,
                Some(&holder_scope),
            )?
        };
        self.sheets.insert(
            requested_name.clone(),
            StatefulSheetState {
                schema: T::schema(),
                metadata: *T::write_metadata(),
                options,
                next_row: progress.next_row,
                next_data_index: progress.next_data_index,
            },
        );
        self.remember_sheet_index(sheet.options().sheet_index, &requested_name);
        Ok(())
    }

    /// Appends typed rows onto a ZIP-preserved template package.
    ///
    /// Keeps `styles.xml` and `mergeCells` from the template; only `sheetData`
    /// grows. When the requested sheet is absent, a new empty worksheet part is
    /// created without rewriting existing sheets.
    // 语义敏感：参数与 Java 对应写入路径一一对应，拆分结构体会破坏
    // 1:1 可追溯性；函数体端到端覆盖完整写入流程，故豁免
    // too_many_arguments / too_many_lines。
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn write_xlsx_batch_onto_template_package<T, I>(
        &mut self,
        rows: I,
        sheet: &WriteSheet<T>,
        handlers: &mut [Box<dyn WriteHandler>],
        skip_sheet_create_callbacks: bool,
        use_incoming_options: bool,
        initialize_holder_head: bool,
        active_table_no: Option<i32>,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        let sheet_names = {
            let package = self
                .template_package
                .as_ref()
                .expect("template package must exist for ZIP preserve path");
            package.sheet_names()?
        };
        let (_target_index, target_name, create_new) =
            crate::write::template_write::resolve_package_target(
                &sheet_names,
                sheet.options().sheet_index,
                &sheet.options().sheet_name,
            );
        let sheet_name = if let Some(resolved) = self.resolve_sheet_name(sheet.options()) {
            resolved
        } else {
            if create_new {
                let package = self
                    .template_package
                    .as_mut()
                    .expect("template package must exist for ZIP preserve path");
                package.ensure_sheet(&target_name)?;
                self.template_pending_rows.insert(target_name.clone(), 0);
            }
            target_name
        };
        let first_write = self.template_pending_rows.remove(&sheet_name).is_some()
            || !self.sheets.contains_key(&sheet_name);
        let mut options = if let Some(state) = self.sheets.get(&sheet_name) {
            if !use_incoming_options {
                validate_stateful_schema(&sheet_name, state, T::schema())?;
            }
            if use_incoming_options {
                let mut options = sheet.options().clone();
                options.converters = self.converters.merged_with(&options.converters);
                self.apply_workbook_spill_defaults(&mut options);
                options
            } else {
                state.options.clone()
            }
        } else {
            let mut options = sheet.options().clone();
            options.converters = self.converters.merged_with(&options.converters);
            self.apply_workbook_spill_defaults(&mut options);
            options.sheet_name.clone_from(&sheet_name);
            options
        };
        options.sheet_name.clone_from(&sheet_name);

        let write_head = first_write || initialize_holder_head;
        let next_data_index = self
            .sheets
            .get(&sheet_name)
            .map_or(0, |state| state.next_data_index);
        let start_row = self
            .template_package
            .as_ref()
            .expect("template package must exist for ZIP preserve path")
            .next_row_for_sheet(&sheet_name)?
            .saturating_sub(1);
        let (mut append_rows, original_rows, converted_rows, absent_rows) =
            collect_template_append_rows::<T, I>(&options, rows, write_head, start_row)?;
        let mut row_heights = template_append_row_heights::<T>(
            &options,
            handlers,
            write_head,
            append_rows.len(),
            &absent_rows,
        )?;
        let holder_scope =
            self.handler_holder_scope::<T>(sheet.options(), &sheet_name, active_table_no)?;
        let sheet_context = holder_scope.sheet(WriteSheetContext::new(&sheet_name));
        if first_write && !skip_sheet_create_callbacks {
            before_sheet(handlers, &sheet_context)?;
            after_sheet_create(handlers, &sheet_context)?;
        }
        let effects = run_template_handler_callbacks::<T>(
            &options,
            handlers,
            &mut append_rows,
            &original_rows,
            &absent_rows,
            write_head,
            next_data_index,
            start_row,
            Some(&holder_scope),
        )?;
        if row_heights.is_empty() && effects.requested_row_heights.iter().any(Option::is_some) {
            row_heights.resize(effects.requested_row_heights.len(), None);
        }
        for (height, requested) in row_heights.iter_mut().zip(&effects.requested_row_heights) {
            if requested.is_some() {
                *height = *requested;
            }
        }
        let next_row = {
            let package = self
                .template_package
                .as_mut()
                .expect("template package must exist for ZIP preserve path");
            if first_write {
                apply_template_holder_layout::<T>(package, &sheet_name, &options, handlers, &[])?;
                let head_merges =
                    automatic_dynamic_head_merge_ranges::<T>(&options, start_row, write_head)?;
                package.apply_sheet_layout(&sheet_name, &[], &head_merges)?;
            }
            let cell_styles = template_append_cell_styles::<T>(
                package,
                &options,
                handlers,
                &append_rows,
                &original_rows,
                &converted_rows,
                &effects.ignore_styles,
                &effects.requested_styles,
                write_head,
                next_data_index,
            )?;
            package.append_rows_with_layout_and_absent(
                &sheet_name,
                &append_rows,
                &row_heights,
                &cell_styles,
                &absent_rows,
            )?
        };
        if first_write {
            after_sheet(handlers, &sheet_context)?;
        }
        let added = append_rows.len();
        let head_rows = if write_head {
            usize::try_from(head_rows_for_schema(T::schema(), &options)?).unwrap_or(0)
        } else {
            0
        };
        let data_added = added.saturating_sub(head_rows).saturating_sub(
            usize::try_from(relative_head_start_row(&options)).unwrap_or(usize::MAX),
        );
        self.sheets.insert(
            sheet_name.clone(),
            StatefulSheetState {
                schema: T::schema(),
                metadata: *T::write_metadata(),
                options,
                next_row,
                next_data_index: next_data_index.saturating_add(data_added),
            },
        );
        self.remember_sheet_index(sheet.options().sheet_index, &sheet_name);
        Ok(())
    }

    // 语义敏感：参数与 Java `ExcelWriter.writeCsvBatch` 的写入路径参数一一对应，
    // 拆分结构体会破坏 1:1 可追溯性；函数体端到端覆盖完整写入流程，
    // 拆分会割裂上下文，故豁免 too_many_arguments / too_many_lines。
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn write_csv_batch<T, I>(
        &mut self,
        rows: I,
        sheet: &WriteSheet<T>,
        handlers: &mut [Box<dyn WriteHandler>],
        skip_sheet_create_callbacks: bool,
        use_incoming_options: bool,
        initialize_holder_head: bool,
        active_table_no: Option<i32>,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        let requested_name = sheet.options().sheet_name.clone();
        let existing_name = self.resolve_sheet_name(sheet.options());
        if existing_name.is_none() && !self.sheets.is_empty() {
            return Err(ExcelError::Unsupported(
                "CSV supports only one worksheet".to_owned(),
            ));
        }
        let sheet_name = existing_name.unwrap_or(requested_name);

        let (state, is_new) = if let Some(state) = self.sheets.get(&sheet_name).cloned() {
            if !use_incoming_options {
                validate_stateful_schema(&sheet_name, &state, T::schema())?;
            }
            (state, false)
        } else {
            let mut options = sheet.options().clone();
            options.charset = self.csv_charset.clone();
            options.with_bom = self.csv_with_bom;
            options.converters = self.converters.merged_with(&options.converters);
            (
                StatefulSheetState {
                    schema: T::schema(),
                    metadata: *T::write_metadata(),
                    options,
                    next_row: 0,
                    next_data_index: 0,
                },
                true,
            )
        };

        let mut batch_options = if use_incoming_options {
            let mut options = sheet.options().clone();
            options.charset = self.csv_charset.clone();
            options.with_bom = self.csv_with_bom;
            options.converters = self.converters.merged_with(&options.converters);
            options
        } else {
            state.options.clone()
        };
        batch_options.sheet_name.clone_from(&sheet_name);
        let holder_scope =
            self.handler_holder_scope::<T>(sheet.options(), &sheet_name, active_table_no)?;
        let sheet_context = holder_scope.sheet(WriteSheetContext::new(&sheet_name));
        if is_new && !skip_sheet_create_callbacks {
            before_sheet(handlers, &sheet_context)?;
            after_sheet_create(handlers, &sheet_context)?;
        }
        let writer = self
            .csv_writer
            .as_mut()
            .expect("stateful CSV writer must be initialized");
        let progress = append_csv_rows::<T, I>(
            writer,
            &batch_options,
            rows,
            handlers,
            state.next_row,
            state.next_data_index,
            is_new || initialize_holder_head,
            Some(&holder_scope),
        )?;
        if is_new {
            after_sheet(handlers, &sheet_context)?;
        }
        self.sheets.insert(
            sheet_name.clone(),
            StatefulSheetState {
                next_row: progress.next_row,
                next_data_index: progress.next_data_index,
                ..state
            },
        );
        if is_new {
            self.remember_sheet_index(sheet.options().sheet_index, &sheet_name);
        }
        Ok(())
    }

    fn resolve_sheet_name(&self, options: &WriteOptions) -> Option<String> {
        options
            .sheet_index
            .and_then(|index| self.sheet_indexes.get(&index).cloned())
            .or_else(|| {
                self.sheets
                    .contains_key(&options.sheet_name)
                    .then(|| options.sheet_name.clone())
            })
            .or_else(|| {
                self.template_pending_rows
                    .contains_key(&options.sheet_name)
                    .then(|| options.sheet_name.clone())
            })
    }

    // 语义敏感：(0..).find 查找首个空闲 sheet 序号，find 命中即终止；
    // sheet 数量受工作簿规模约束，不存在真正的无限迭代。
    #[allow(clippy::maybe_infinite_iter)]
    fn handler_holder_scope<T>(
        &self,
        options: &WriteOptions,
        sheet_name: &str,
        table_no: Option<i32>,
    ) -> Result<HandlerHolderScope>
    where
        T: ExcelRow,
    {
        let sheet_no = options
            .sheet_index
            .or_else(|| {
                self.sheet_indexes
                    .iter()
                    .find_map(|(index, name)| (name == sheet_name).then_some(*index))
            })
            .unwrap_or_else(|| {
                (0..)
                    .find(|index| !self.sheet_indexes.contains_key(index))
                    .unwrap_or(self.sheet_indexes.len())
            });
        let mut effective_options = options.clone();
        effective_options.converters = self.converters.merged_with(&options.converters);
        HandlerHolderScope::new_resolved::<T>(
            &self.path,
            i32::try_from(sheet_no).unwrap_or(i32::MAX),
            table_no,
            &effective_options,
        )
    }

    // 语义敏感：同上，(0..).find 查找空闲序号，命中即终止。
    #[allow(clippy::maybe_infinite_iter)]
    fn remember_sheet_index(&mut self, index: Option<usize>, sheet_name: &str) {
        if self.sheet_indexes.values().any(|name| name == sheet_name) {
            return;
        }
        let index = index.unwrap_or_else(|| {
            (0..)
                .find(|candidate| !self.sheet_indexes.contains_key(candidate))
                .unwrap_or(self.sheet_indexes.len())
        });
        self.sheet_indexes.insert(index, sheet_name.to_owned());
    }
}
