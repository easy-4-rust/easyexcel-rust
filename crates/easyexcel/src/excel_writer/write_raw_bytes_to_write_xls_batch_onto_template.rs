impl ExcelWriter {
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

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Returns the configured template file, if any.
    #[must_use]
    pub fn template_file(&self) -> Option<&std::path::Path> {
        self.template_file.as_deref()
    }

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Returns the configured in-memory template bytes, if any.
    #[must_use]
    pub fn template_bytes(&self) -> Option<&[u8]> {
        self.template_bytes.as_deref()
    }

    /// 返回当前工作簿写密码，供模板填充执行器沿用同一调用级配置。
    ///
    /// 对应 Java：`WriteWorkbookHolder#getWriteWorkbook().getPassword()`。
    #[must_use]
    pub(crate) fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// 返回异常结束时是否仍输出工作簿，供共享模板 executor 保持同一生命周期语义。
    ///
    /// 对应 Java：`WriteWorkbookHolder#getWriteExcelOnException()`。
    #[must_use]
    pub(crate) const fn write_excel_on_exception(&self) -> bool {
        self.write_excel_on_exception
    }

    /// 返回 BIFF8 模板 VBA 保存策略，供共享 fill executor 复用。
    #[must_use]
    pub(crate) const fn biff8_macro_policy(&self) -> &crate::Biff8MacroPolicy {
        &self.biff8_macro_policy
    }

    /// 返回统一 writer 的输出流关闭策略，供模板引擎接管相同生命周期。
    #[must_use]
    pub(crate) const fn auto_close_stream_enabled(&self) -> bool {
        self.auto_close_stream
    }

    /// 将真实输出目标移交给共享模板引擎。
    ///
    /// 路径 writer 仅传递路径；流 writer 同时移交已擦除类型的 writer 与
    /// 原关闭回调，避免模板 fill 误写到逻辑文件名。
    pub(crate) fn take_template_output(&mut self) -> crate::template::TemplateOutput<'static> {
        if let Some(writer) = self.output_stream.take() {
            return crate::template::TemplateOutput::Managed {
                writer,
                close: self.close_stream.take(),
            };
        }
        crate::template::TemplateOutput::Path(self.path.clone())
    }

    /// 模板 executor 尚未安装即初始化失败时，完成原始输出目标的生命周期。
    ///
    /// 此时不能调用普通 `finish_on_exception`，因为它会再次尝试启动并解析同一
    /// 无效模板。路径目标按 Java 已打开 `FileOutputStream` 的可观察语义创建为空；
    /// 受管流仅在 `autoCloseStream(true)` 时调用原关闭动作。
    pub(crate) fn discard_uninitialized_template_output(&mut self) -> Result<()> {
        let uses_output_path = self.output_stream.is_none();
        self.output_stream.take();
        if self.auto_close_stream
            && let Some(close) = self.close_stream.take()
        {
            close().map_err(ExcelError::from)?;
        }
        if uses_output_path {
            if let Some(parent) = self.path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::File::create(&self.path)?;
        }
        self.finished = true;
        Ok(())
    }

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Marks the writer finished without persisting workbook output.
    ///
    /// Used when a [`WriteFillExecutor`] already wrote the filled package.
    pub(crate) fn mark_finished(&mut self) {
        self.finished = true;
    }

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Saves and closes the writer. Repeated calls are no-ops.
    ///
    /// # Errors
    ///
    /// Returns an output or handler error.
    pub fn finish(&mut self) -> Result<()> {
        self.finish_with_exception(false)
    }

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Finishes after a write-side exception.
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
        if self.backend_selection == crate::WriteBackendSelection::Failed {
            self.finished = true;
            if self.auto_close_stream
                && let Some(close) = self.close_stream.take()
                && let Err(error) = close()
            {
                return Err(ExcelError::Format(format!(
                    "stateful streaming backend failed; output close also failed: {error}"
                )));
            }
            return Err(ExcelError::Format(
                "cannot finish after stateful streaming backend failed".to_owned(),
            ));
        }
        if self.finished {
            return Ok(());
        }
        self.start()?;
        self.finished = true;
        let mut write_excel = !on_exception || self.write_excel_on_exception;
        let mut result = Ok(());
        let context = WriteWorkbookContext::new(&self.path)
            .with_mutation_plan(self.mutation_plan.clone());
        let mut handlers = boxed_handlers(&self.current_effective_handlers);
        sort_handlers(&mut handlers);
        if let Err(error) = after_workbook(&mut handlers, &context) {
            result = Err(error);
        }
        // `afterWorkbookDispose` 仍可通过共享 mutation plan 请求随机访问。
        // 必须在消费 journal 前观察这些最终能力；否则自动流式 writer 已无从
        // 晋升，只能把不完整的常量内存 workbook 误当作完整结果保存。
        let has_deferred_mutations = match self.mutation_plan.is_empty() {
            Ok(is_empty) => !is_empty,
            Err(error) => {
                result = Err(error);
                write_excel = false;
                false
            }
        };
        if write_excel && has_deferred_mutations {
            match self.backend_selection {
                crate::WriteBackendSelection::AutoStreaming => {
                    if let Err(error) = self.promote_auto_streaming_to_memory() {
                        result = Err(error);
                        write_excel = false;
                    }
                }
                crate::WriteBackendSelection::ExplicitStreaming => {
                    self.backend_selection = crate::WriteBackendSelection::Failed;
                    result = Err(ExcelError::Unsupported(
                        "explicit constant-memory writer received a deferred random-access mutation"
                            .to_owned(),
                    ));
                    write_excel = false;
                }
                _ => {}
            }
        }
        if let Err(error) = self.finish_gzip_spills() {
            self.backend_selection = crate::WriteBackendSelection::Failed;
            result = Err(error);
            write_excel = false;
        }
        if has_deferred_mutations && self.is_csv() {
            result = Err(ExcelError::Unsupported(
                "workbook handler mutations are not supported for CSV output".to_owned(),
            ));
        }
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
            let mutation_result = if let Some(package) = self.xls_template.as_mut() {
                package.apply_mutations(&self.mutation_plan)
            } else {
                apply_xls_mutations(&mut self.xls_book, &self.mutation_plan)
            };
            if let Err(error) = mutation_result.and_then(|()| self.save_xls_output()) {
                result = Err(error);
            }
        } else if write_excel
            && let Err(error) = self.save_xlsx_output()
        {
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

    fn save_xlsx_output(&mut self) -> Result<()> {
        let deferred_package = if self.template_package.is_none() {
            apply_xlsx_mutations(&mut self.workbook, &self.mutation_plan)?;
            self.build_deferred_package()?
        } else {
            None
        };
        if let Some(package) = self.template_package.take() {
            return save_template_package(
                &package,
                &self.path,
                self.output_stream
                    .as_mut()
                    .map(|output| output.as_mut() as &mut (dyn Write + Send)),
                self.password.as_deref(),
            );
        }
        if let Some(package) = deferred_package.as_ref() {
            return save_template_package(
                package,
                &self.path,
                self.output_stream
                    .as_mut()
                    .map(|output| output.as_mut() as &mut (dyn Write + Send)),
                self.password.as_deref(),
            );
        }
        if let Some(output) = self.output_stream.as_mut() {
            save_workbook_to_writer(
                &mut self.workbook,
                output.as_mut(),
                self.password.as_deref(),
            )
        } else {
            save_workbook(&mut self.workbook, &self.path, self.password.as_deref())
        }
    }

    fn build_deferred_package(
        &mut self,
    ) -> Result<Option<crate::write::template_write::TemplatePackage>> {
        let merges = self.mutation_plan.merge_ranges()?;
        let comment_removals = self.mutation_plan.comment_removals()?;
        if merges.is_empty() && comment_removals.is_empty() {
            return Ok(None);
        }
        let bytes = generation::serialize_workbook(&mut self.workbook).map_err(ExcelError::from)?;
        let mut package = crate::write::template_write::TemplatePackage::from_bytes(&bytes)?;
        crate::write::excel_writer_core::apply_deferred_xlsx_mutations(
            &mut package,
            &self.mutation_plan,
        )?;
        Ok(Some(package))
    }

    fn save_xls_output(&mut self) -> Result<()> {
        if let Some(package) = self.xls_template.take() {
            return if let Some(output) = self.output_stream.as_mut() {
                package.save_to_writer_with_password_and_macro_policy(
                    output.as_mut(),
                    self.password.as_deref(),
                    &self.biff8_macro_policy,
                )
            } else {
                package.save_to_path_with_password_and_macro_policy(
                    &self.path,
                    self.password.as_deref(),
                    &self.biff8_macro_policy,
                )
            };
        }
        if let Some(output) = self.output_stream.as_mut() {
            return if self.password.is_some() {
                self.xls_book
                    .write_to_with_password(output.as_mut(), self.password.as_deref())
                    .map_err(ExcelError::from)
            } else {
                self.xls_book
                    .write_to(output.as_mut())
                    .map_err(ExcelError::from)
            };
        }
        if self.password.is_some() {
            self.xls_book
                .save_to_path_with_password(&self.path, self.password.as_deref())
                .map_err(ExcelError::from)
        } else {
            save_xls_book(&self.xls_book, &self.path)
        }
    }

    /// Returns whether [`Self::finish`] completed successfully.
    #[must_use]
    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。
    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    /// 对应 Java：`WriteWorkbookHolder#getWorkbook()`。返回底层
    /// `rust_xlsxwriter` workbook，供高级 XLSX 定制使用。
    ///
    /// 首次写入前调用会锁定为内存后端。已经进入自动流式后端时，本兼容入口
    /// 会尝试 journal 晋升；晋升失败会明确 panic，而不会返回一个不完整的
    /// workbook。需要处理 I/O/重放错误的调用方应使用 [`Self::try_workbook_mut`]。
    /// CSV/XLS writer 不使用该 XLSX workbook。
    #[must_use]
    pub fn workbook_mut(&mut self) -> &mut Workbook {
        match self.try_workbook_mut() {
            Ok(workbook) => workbook,
            Err(error) => panic!("cannot access the stateful XLSX workbook safely: {error}"),
        }
    }

    /// 安全返回底层 XLSX workbook，并在需要时完成自动流式 journal 晋升。
    ///
    /// 该方法是 Rust 对 Java `WriteWorkbookHolder#getWorkbook()` 的可失败替代：
    /// 显式常量内存模式拒绝随机访问；自动模式在首批写入前锁定为内存，或在
    /// 已写入批次后重放 journal。这样高级定制不会绕过 Stateful 后端状态机。
    ///
    /// # Errors
    ///
    /// 当格式不是 XLSX、显式常量内存禁止随机访问、writer 已结束，或 journal
    /// 晋升失败时返回错误。
    pub fn try_workbook_mut(&mut self) -> Result<&mut Workbook> {
        if self.finished {
            return Err(ExcelError::Unsupported(
                "writer already finished".to_owned(),
            ));
        }
        if self.is_csv() || self.is_xls() {
            return Err(ExcelError::Unsupported(
                "the rust_xlsxwriter workbook is available only for XLSX output".to_owned(),
            ));
        }
        match self.backend_selection {
            crate::WriteBackendSelection::AutoUndecided => {
                self.backend_selection = crate::WriteBackendSelection::InMemory;
                self.default_constant_memory = false;
                self.compress_temp_files = false;
            }
            crate::WriteBackendSelection::AutoStreaming => {
                self.promote_auto_streaming_to_memory()?;
            }
            crate::WriteBackendSelection::ExplicitStreaming => {
                return Err(ExcelError::Unsupported(
                    "explicit constant-memory writer does not expose random-access workbook mutation"
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
            crate::WriteBackendSelection::InMemory
            | crate::WriteBackendSelection::ExplicitInMemory => {}
        }
        Ok(&mut self.workbook)
    }

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Enables SXSSF-style compressed / disk-spill temp files for later sheets.
    ///
    /// Java mapping: `SXSSFWorkbook.setCompressTempFiles(true)`, typically called from
    /// `WorkbookWriteHandler.afterWorkbookCreate`. Call this before the first
    /// `write` that creates a worksheet. Already-created sheets keep their mode.
    ///
    /// # Panics
    ///
    /// See [`Self::try_set_compress_temp_files`] for conflicting state transitions.
    pub fn set_compress_temp_files(&mut self, enabled: bool) -> &mut Self {
        if let Err(error) = self.try_set_compress_temp_files(enabled) {
            panic!("cannot change stateful temp-file compression safely: {error}");
        }
        self
    }

    /// 尝试修改后续常量内存 Sheet 的临时文件压缩策略。
    ///
    /// 自动模式尚未决策或已经进入流式后端时可以修改；已经锁定完整内存后端
    /// 时不能再借此把后续 Sheet 偷换成流式实现。
    ///
    /// # Errors
    ///
    /// writer 已结束/失败，或启用压缩会与已经锁定的内存后端冲突时返回错误。
    pub fn try_set_compress_temp_files(&mut self, enabled: bool) -> Result<&mut Self> {
        if self.backend_selection == crate::WriteBackendSelection::Failed {
            return Err(ExcelError::Format(
                "stateful streaming backend previously failed".to_owned(),
            ));
        }
        if self.finished {
            return Err(ExcelError::Unsupported(
                "writer already finished".to_owned(),
            ));
        }
        if enabled
            && matches!(
                self.backend_selection,
                crate::WriteBackendSelection::InMemory
                    | crate::WriteBackendSelection::ExplicitInMemory
            )
        {
            return Err(ExcelError::Unsupported(
                "stateful writer is already locked to the in-memory backend".to_owned(),
            ));
        }
        self.compress_temp_files = enabled;
        if enabled {
            self.default_constant_memory = true;
        } else if self.backend_selection == crate::WriteBackendSelection::AutoUndecided {
            self.default_constant_memory = false;
        }
        Ok(self)
    }

    /// Returns whether workbook-level temp-file compression / spill is enabled.
    #[must_use]
    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。
    pub const fn compress_temp_files_enabled(&self) -> bool {
        self.compress_temp_files
    }

    /// Last finished gzip spill snapshot (Java SXSSF compressed temp observability).
    ///
    /// Populated when [`Self::finish`] closes active [`crate::write::gzip_spill::GzipSheetDataWriter`]s.
    #[must_use]
    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。
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
/// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。
    pub(crate) fn start(&mut self) -> Result<()> {
        if self.backend_selection == crate::WriteBackendSelection::Failed {
            return Err(ExcelError::Format(
                "cannot start after stateful streaming backend failed".to_owned(),
            ));
        }
        if self.started {
            return Ok(());
        }
        validate_stateful_backend(self.is_csv(), self.is_xls(), self.password.as_deref())?;
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
                if !crate::write::xls_adapter::looks_like_xls(&bytes) {
                    return Err(ExcelError::Format(
                        "xls with_template requires an OLE .xls workbook".to_owned(),
                    ));
                }
                let package =
                    crate::write::xls_adapter::Biff8TemplatePackage::from_bytes_with_password(
                        &bytes,
                        self.password.as_deref(),
                    )?;
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
        let context = WriteWorkbookContext::new(&self.path)
            .with_mutation_plan(self.mutation_plan.clone());
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
/// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。
    pub(crate) fn is_csv(&self) -> bool {
        match self.excel_type {
            Some(excel_type) => excel_type == crate::support::ExcelTypeEnum::Csv,
            None => easyexcel_io::path_has_extension(&self.path, "csv"),
        }
    }
/// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。
    pub(crate) fn is_xls(&self) -> bool {
        match self.excel_type {
            Some(excel_type) => excel_type == crate::support::ExcelTypeEnum::Xls,
            None => easyexcel_io::path_has_extension(&self.path, "xls"),
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
    /// Mirrors [`Self::write_xlsx_batch_onto_template_package`] for HSSF/BIFF8，
    /// including creation of sheets absent from the template.
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
            let package = self
                .xls_template
                .as_mut()
                .expect("xls template must exist for BIFF preserve path");
            package.ensure_sheet(&target_name)?;
            self.template_pending_rows.insert(target_name.clone(), 0);
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

}
