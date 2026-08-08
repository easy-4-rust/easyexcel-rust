impl ExcelWriter {
    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Appends raw bytes to the BIFF8 output stream. These bytes are
    /// written as an "Images" OLE stream in the CFB container when
    /// the file is serialized. Used for embedding image data in XLS.
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.xls_book.write_raw_bytes(bytes);
        self
    }

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Encodes image bytes as BIFF8 Obj + `MSODrawing` + Escher BSE
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
        let context = WriteWorkbookContext::new(&self.path)
            .with_mutation_plan(self.mutation_plan.clone());
        let mut handlers = boxed_handlers(&self.current_effective_handlers);
        sort_handlers(&mut handlers);
        if let Err(error) = after_workbook(&mut handlers, &context) {
            result = Err(error);
        }
        if !self.mutation_plan.is_empty()?
            && (self.is_csv() || self.xls_template.is_some() || self.template_package.is_some())
        {
            result = Err(ExcelError::Unsupported(
                "workbook handler mutations are not supported for CSV or template output"
                    .to_owned(),
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
            if let Err(error) = apply_xls_mutations(&mut self.xls_book, &self.mutation_plan)
                .and_then(|()| self.save_xls_output())
            {
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
        let deferred_merge_package = if self.template_package.is_none() {
            apply_xlsx_mutations(&mut self.workbook, &self.mutation_plan)?;
            self.build_deferred_merge_package()?
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
        if let Some(package) = deferred_merge_package.as_ref() {
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

    fn build_deferred_merge_package(
        &mut self,
    ) -> Result<Option<crate::write::template_write::TemplatePackage>> {
        let merges = self.mutation_plan.merge_ranges()?;
        if merges.is_empty() {
            return Ok(None);
        }
        let bytes = generation::serialize_workbook(&mut self.workbook).map_err(ExcelError::from)?;
        let mut package = crate::write::template_write::TemplatePackage::from_bytes(&bytes)?;
        for (sheet_name, range) in merges {
            package.apply_sheet_layout(&sheet_name, &[], &[range])?;
        }
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

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Returns the underlying `rust_xlsxwriter` workbook for advanced XLSX customization.
    ///
    /// Callers are responsible for preserving valid worksheet names and
    /// workbook invariants. CSV writers do not use this workbook.
    #[must_use]
    pub fn workbook_mut(&mut self) -> &mut Workbook {
        &mut self.workbook
    }

    /// 对应 Java：`WriteWorkbookHolder.getTempTemplateInputStream() != null`。 Enables SXSSF-style compressed / disk-spill temp files for later sheets.
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

}
