impl ExcelWriter {
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
        HandlerHolderScope::new_resolved_with_plan::<T>(
            &self.path,
            i32::try_from(sheet_no).unwrap_or(i32::MAX),
            table_no,
            &effective_options,
            self.mutation_plan.clone(),
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
