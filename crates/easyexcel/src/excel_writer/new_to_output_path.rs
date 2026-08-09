impl ExcelWriter {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a multi-sheet writer without handlers.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::with_handlers(path, Vec::new())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a multi-sheet writer with owned lifecycle handlers.
    #[must_use]
    pub fn with_handlers(path: impl Into<PathBuf>, handlers: Vec<Box<dyn WriteHandler>>) -> Self {
        Self::with_handlers_and_password(path, handlers, None)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a multi-sheet writer with handlers and optional XLSX encryption.
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a stateful writer with workbook-level builder options.
    #[must_use]
    pub fn with_handlers_and_options(
        path: impl Into<PathBuf>,
        handlers: Vec<Box<dyn WriteHandler>>,
        options: WriteOptions,
    ) -> Self {
        let selection = if options.constant_memory || options.compress_temp_files {
            crate::write::WriteBackendSelection::ExplicitStreaming
        } else {
            crate::write::WriteBackendSelection::AutoUndecided
        };
        Self::with_handlers_and_options_and_selection(path, handlers, options, selection)
    }

    /// 使用显式状态机创建 Stateful writer。
    pub(crate) fn with_handlers_and_options_and_selection(
        path: impl Into<PathBuf>,
        mut handlers: Vec<Box<dyn WriteHandler>>,
        options: WriteOptions,
        backend_selection: crate::write::WriteBackendSelection,
    ) -> Self {
        let path = path.into();
        let excel_type = effective_write_type(&path, &options);
        handlers.extend(DefaultWriteHandlerLoader::load_default_handler_for(
            options.use_default_style,
            excel_type,
        ));
        let converters =
            crate::converters::default_converter_loader::load_default_write_converter()
                .merged_with(&options.converters);
        let has_custom_converters = !options.converters.is_empty();
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
            biff8_macro_policy: options.biff8_macro_policy,
            converters,
            has_custom_converters,
            compress_temp_files: options.compress_temp_files,
            default_constant_memory: options.constant_memory || options.compress_temp_files,
            backend_selection,
            template_file: options.template_file,
            template_bytes: options.template_bytes,
            template_pending_rows: HashMap::new(),
            template_package: None,
            xls_template: None,
            use_legacy_template_seed: options.use_legacy_template_seed,
            gzip_spills: HashMap::new(),
            last_gzip_spill: None,
            mutation_plan: crate::context::write_mutation_plan::WriteMutationPlan::default(),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a stateful writer backed by a cloneable output stream.
    #[must_use]
    pub fn with_output_stream<W>(
        logical_path: impl Into<PathBuf>,
        output: ExcelOutputStream<W>,
        handlers: Vec<Box<dyn WriteHandler>>,
        options: WriteOptions,
    ) -> Self
    where
        W: Write + Send + 'static,
    {
        let selection = if options.constant_memory || options.compress_temp_files {
            crate::write::WriteBackendSelection::ExplicitStreaming
        } else {
            crate::write::WriteBackendSelection::AutoUndecided
        };
        Self::with_output_stream_and_selection(
            logical_path,
            output,
            handlers,
            options,
            selection,
        )
    }

    pub(crate) fn with_output_stream_and_selection<W>(
        logical_path: impl Into<PathBuf>,
        output: ExcelOutputStream<W>,
        mut handlers: Vec<Box<dyn WriteHandler>>,
        options: WriteOptions,
        backend_selection: crate::write::WriteBackendSelection,
    ) -> Self
    where
        W: Write + Send + 'static,
    {
        let path = logical_path.into();
        let excel_type = effective_write_type(&path, &options);
        handlers.extend(DefaultWriteHandlerLoader::load_default_handler_for(
            options.use_default_style,
            excel_type,
        ));
        let converters =
            crate::converters::default_converter_loader::load_default_write_converter()
                .merged_with(&options.converters);
        let has_custom_converters = !options.converters.is_empty();
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
            biff8_macro_policy: options.biff8_macro_policy,
            converters,
            has_custom_converters,
            compress_temp_files: options.compress_temp_files,
            default_constant_memory: options.constant_memory || options.compress_temp_files,
            backend_selection,
            template_file: options.template_file,
            template_bytes: options.template_bytes,
            template_pending_rows: HashMap::new(),
            template_package: None,
            xls_template: None,
            use_legacy_template_seed: options.use_legacy_template_seed,
            gzip_spills: HashMap::new(),
            last_gzip_spill: None,
            mutation_plan: crate::context::write_mutation_plan::WriteMutationPlan::default(),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Registers an additional handler before the first write starts.
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Prepends handlers owned by a more specific Java write holder.
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes a batch to a worksheet, appending when the sheet was used before.
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
        let sheet_name = self
            .resolve_sheet_name(sheet.options())
            .unwrap_or_else(|| sheet.options().sheet_name.clone());
        self.ensure_sheet_annotation_handlers::<T>(&sheet_name, sheet.options())?;
        let handler_scope = self.sheet_handler_scope(&sheet_name);
        let mut handlers = handler_scope.effective_boxed();
        self.ensure_backend_for_write::<T>(sheet.options(), &handlers)?;
        self.start()?;
        // `start()` 执行 workbook Handler；回调可能通过共享 mutation plan 新增
        // 随机访问修改，因此首批数据落盘前必须按回调后的真实状态再判定一次。
        self.ensure_backend_for_write::<T>(sheet.options(), &handlers)?;
        let write_result = if self.is_csv() {
            self.write_csv_batch::<T, I>(rows, sheet, &mut handlers, false, false, false, None)?;
            Ok(())
        } else if self.is_xls() {
            self.write_xls_batch::<T, I>(rows, sheet, &mut handlers, false, false, false, None)?;
            Ok(())
        } else {
            self.write_xlsx_batch::<T, I>(rows, sheet, &mut handlers, false, false, false, None)
        };
        self.record_streaming_write_result(write_result)?;
        self.current_effective_handlers = handler_scope.effective;
        debug_assert!(self.resolve_sheet_name(sheet.options()).is_some());
        Ok(self)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes with handlers owned by this Sheet holder.
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
        let sheet_name = self
            .resolve_sheet_name(sheet.options())
            .unwrap_or_else(|| sheet.options().sheet_name.clone());
        let is_initialized = self.sheets.contains_key(&sheet_name);
        let had_handlers = !handlers.is_empty();
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
            self.sheet_handlers.insert(sheet_name.clone(), own_handlers);
        }
        self.ensure_sheet_annotation_handlers::<T>(&sheet_name, sheet.options())?;
        let scope = self.sheet_handler_scope(&sheet_name);
        let effective = scope.effective_boxed();
        self.ensure_backend_for_write::<T>(sheet.options(), &effective)?;
        self.start()?;
        if !is_initialized && had_handlers {
            run_own_workbook_callbacks(&scope, &self.path)?;
        }
        self.write(rows, sheet)
    }

    /// 返回 Stateful writer 当前的后端状态。
    #[must_use]
    pub const fn backend_selection(&self) -> crate::WriteBackendSelection {
        self.backend_selection
    }

    /// 将 Java `ExcelBuilder.merge(...)` 产生的区域加入当前工作簿修改计划。
    ///
    /// 自动流式 writer 会先用已保留的 journal 晋升到内存后端，以保证保存
    /// 阶段仍能执行随机访问修改；显式流式模式继续 fail-closed。
    pub(crate) fn add_deferred_merge(
        &mut self,
        sheet_name: impl Into<String>,
        range: crate::MergeRange,
    ) -> Result<()> {
        if self.is_csv() {
            return Err(ExcelError::Unsupported(
                "csv does not support merged regions".to_owned(),
            ));
        }
        match self.backend_selection {
            crate::WriteBackendSelection::AutoStreaming => {
                self.promote_auto_streaming_to_memory()?;
            }
            crate::WriteBackendSelection::ExplicitStreaming => {
                return Err(ExcelError::Unsupported(
                    "explicit constant-memory writer cannot add a merged region after rows were written"
                        .to_owned(),
                ));
            }
            crate::WriteBackendSelection::Promoting => {
                return Err(ExcelError::Unsupported(
                    "automatic streaming backend promotion is already in progress".to_owned(),
                ));
            }
            crate::WriteBackendSelection::Failed => {
                return Err(ExcelError::Format(
                    "stateful streaming backend previously failed".to_owned(),
                ));
            }
            _ => {}
        }
        self.mutation_plan.add_merge(sheet_name, range)
    }

    fn ensure_backend_for_write<T>(
        &mut self,
        options: &WriteOptions,
        handlers: &[Box<dyn WriteHandler>],
    ) -> Result<()>
    where
        T: ExcelRow,
    {
        if self.is_csv() || self.is_xls() {
            if self.backend_selection == crate::WriteBackendSelection::AutoUndecided {
                self.backend_selection = crate::WriteBackendSelection::InMemory;
            }
            return Ok(());
        }
        let has_template = crate::write::template_write::has_template(
            self.template_file.as_deref(),
            self.template_bytes.as_deref(),
        );
        let has_deferred_mutations = !self.mutation_plan.is_empty()?;
        let configuration_safe = !has_deferred_mutations
            && !self.has_custom_converters
            && options.converters.is_empty()
            && crate::write::builder::excel_writer_builder::stateful_streaming_configuration_is_safe::<T>(
                options,
                handlers,
                has_template,
            );
        let safe = configuration_safe
            && crate::write::builder::excel_writer_builder::stateful_constant_memory_is_safe::<T>(
                options,
                handlers,
                has_template,
            );
        match self.backend_selection {
            crate::WriteBackendSelection::AutoUndecided if safe => {
                self.backend_selection = crate::WriteBackendSelection::AutoStreaming;
                self.default_constant_memory = true;
                // AutoStreaming 始终保留磁盘 journal，以便后续能力冲突时晋升。
                self.compress_temp_files = true;
                Ok(())
            }
            crate::WriteBackendSelection::AutoUndecided => {
                self.backend_selection = crate::WriteBackendSelection::InMemory;
                Ok(())
            }
            crate::WriteBackendSelection::ExplicitStreaming if !configuration_safe => {
                Err(ExcelError::Unsupported(
                    "explicit constant-memory writer encountered a handler or sheet option that requires random access"
                        .to_owned(),
                ))
            }
            crate::WriteBackendSelection::AutoStreaming if !safe && self.sheets.is_empty() => {
                self.backend_selection = crate::WriteBackendSelection::InMemory;
                self.default_constant_memory = false;
                self.compress_temp_files = false;
                Ok(())
            }
            crate::WriteBackendSelection::AutoStreaming if !safe => {
                self.promote_auto_streaming_to_memory()
            }
            crate::WriteBackendSelection::Promoting => Err(ExcelError::Unsupported(
                "automatic streaming backend promotion is already in progress".to_owned(),
            )),
            crate::WriteBackendSelection::Failed => Err(ExcelError::Format(
                "stateful streaming backend previously failed".to_owned(),
            )),
            _ => Ok(()),
        }
    }

    fn promote_auto_streaming_to_memory(&mut self) -> Result<()> {
        self.backend_selection = crate::WriteBackendSelection::Promoting;
        let mut spills = std::mem::take(&mut self.gzip_spills);
        let promoted = (|| {
            let mut workbook = easyexcel_xlsx::xlsx::generation::new_workbook();
            let mut ordered_sheets = self.sheet_indexes.iter().collect::<Vec<_>>();
            ordered_sheets.sort_by_key(|(index, _)| **index);
            for (_, sheet_name) in ordered_sheets {
                let state = self.sheets.get(sheet_name).ok_or_else(|| {
                    ExcelError::Format(format!(
                        "stateful journal is missing sheet state for '{sheet_name}'"
                    ))
                })?;
                let mut reader = spills
                    .remove(sheet_name)
                    .map(crate::write::gzip_spill::GzipSheetDataWriter::finish)
                    .transpose()?;
                replay_stateful_sheet_journal(&mut workbook, state, reader.as_mut())?;
            }
            if !spills.is_empty() {
                return Err(ExcelError::Format(
                    "stateful journal contains sheets absent from the workbook index".to_owned(),
                ));
            }
            Ok(workbook)
        })();
        let workbook = match promoted {
            Ok(workbook) => workbook,
            Err(error) => {
                // finish() 已消费部分 journal，无法诚实回滚；进入终止失败态，
                // 禁止后续保存一个不完整的内存工作簿。
                self.backend_selection = crate::WriteBackendSelection::Failed;
                return Err(error);
            }
        };
        self.workbook = workbook;
        for state in self.sheets.values_mut() {
            state.options.constant_memory = false;
            state.options.compress_temp_files = false;
        }
        self.default_constant_memory = false;
        self.compress_temp_files = false;
        self.backend_selection = crate::WriteBackendSelection::InMemory;
        Ok(())
    }

    /// 将可能已经部分消费输入、执行 Handler 或追加 journal 的流式批次错误
    /// 转换为终止失败态，禁止调用方在未知物理行位置上继续写入。
    fn record_streaming_write_result(&mut self, result: Result<()>) -> Result<()> {
        if result.is_err()
            && matches!(
                self.backend_selection,
                crate::WriteBackendSelection::AutoStreaming
                    | crate::WriteBackendSelection::ExplicitStreaming
                    | crate::WriteBackendSelection::Promoting
            )
        {
            self.backend_selection = crate::WriteBackendSelection::Failed;
        }
        result
    }

    fn workbook_handler_scope(&self) -> HandlerExecutionScope {
        HandlerExecutionScope::root(&self.workbook_handlers)
    }
/// 对应 Java：无直接对应对象；Rust 架构扩展。
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
/// 对应 Java：无直接对应对象；Rust 架构扩展。
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Three-arg write with an explicit `WriteTable`, mirroring Java
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes through independent Sheet and Table holder handler chains.
    ///
    /// # Errors
    ///
    /// Returns an error when the writer is finished, a handler fails, or
    /// data cannot be written.
    // 语义敏感：该函数端到端对应 Java `ExcelWriter.writeWithTableHandlers`
    // 的完整流程，拆分会割裂上下文，故豁免 too_many_lines。
    #[allow(clippy::too_many_lines)]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
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
        self.ensure_backend_for_write::<T>(sheet_with_table.options(), &handlers)?;
        let write_result = if self.is_csv() {
            self.write_csv_batch::<T, I>(
                rows,
                &sheet_with_table,
                &mut handlers,
                sheet_is_new,
                true,
                table_is_new,
                Some(table_no),
            )
        } else if self.is_xls() {
            self.write_xls_batch::<T, I>(
                rows,
                &sheet_with_table,
                &mut handlers,
                sheet_is_new,
                true,
                table_is_new,
                Some(table_no),
            )
        } else {
            self.write_xlsx_batch::<T, I>(
                rows,
                &sheet_with_table,
                &mut handlers,
                sheet_is_new,
                true,
                table_is_new,
                Some(table_no),
            )
        };
        self.record_streaming_write_result(write_result)?;
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the logical output path used by Java-style builder facades.
    #[must_use]
    pub fn output_path(&self) -> &std::path::Path {
        &self.path
    }

    /// 返回该 writer 是否直接拥有文件路径输出，而非调用方提供的流。
    #[must_use]
    pub(crate) fn uses_output_path(&self) -> bool {
        self.output_stream.is_none()
    }

}
