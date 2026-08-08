/// `has_fill_executor` gate plus the fill-then-finish lifecycle that marks
    /// the builder as finished via the fill delegate.
    #[test]
    fn fill_executor_gates_and_finish_via_fill() -> Result<()> {
        let mut builder = ExcelBuilderImpl::from_options(
            "fill-finish.xlsx",
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        assert!(!builder.has_fill_executor());
        assert!(!builder.finished_via_fill());
        builder.set_fill_executor(Box::new(ContextFillExecutor));
        assert!(builder.has_fill_executor());

        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        builder.fill(&DynamicRow::default(), FillConfig::new(), &sheet)?;
        builder.finish()?;
        assert!(builder.finished_via_fill());
        Ok(())
    }

/// `ExcelBuilderImpl` coerced to `&dyn WriteContext` must dispatch to the
    /// live holder (covers the `WriteContext` trait impl).
    #[test]
    fn builder_dispatches_as_dyn_write_context() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dyn-context.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        builder.add_content([], &sheet)?;

        let context: &dyn WriteContext = &builder;
        assert!(context.current_write_holder().workbook_context().is_some());
        Ok(())
    }

/// `fill` without a configured template stream is rejected.
    #[test]
    fn fill_without_template_is_rejected() {
        let mut builder =
            ExcelBuilderImpl::from_options("no-template.xlsx", WriteOptions::default());
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("fill without a template must fail");
        assert_eq!(
            error.to_string(),
            "unsupported operation: Calling the 'fill' method must use a template."
        );
    }

    /// `fill` on a legacy XLS writer reaches the real executor wiring gate.
    #[test]
    fn fill_legacy_xls_is_not_blanket_rejected() {
        let mut builder = ExcelBuilderImpl::from_options(
            "legacy.xls",
            WriteOptions {
                excel_type: Some(crate::support::ExcelTypeEnum::Xls),
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("an unwired executor must remain visible");
        assert_eq!(
            error.to_string(),
            "unsupported operation: template fill executor is not wired; build through easyexcel::builder_from_writer"
        );
    }

/// `fill` with a template but no installed executor is rejected.
    #[test]
    fn fill_without_executor_wired_is_rejected() {
        let mut builder = ExcelBuilderImpl::from_options(
            "unwired.xlsx",
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("fill without a wired executor must fail");
        assert!(error.to_string().contains("not wired"));
    }

/// A failing fill delegate propagates its error through `ExcelBuilderImpl::fill`.
    #[test]
    fn fill_propagates_delegate_error() {
        struct FailingFillExecutor {
            fail_first: bool,
        }

        impl WriteFillExecutor for FailingFillExecutor {
            fn fill(
                &mut self,
                _data: &dyn Any,
                _fill_config: WriteFillConfig,
                _sheet: WriteFillSheet,
            ) -> Result<()> {
                if self.fail_first {
                    self.fail_first = false;
                    Err(ExcelError::Unsupported("fill failed on purpose".to_owned()))
                } else {
                    Ok(())
                }
            }

            fn finish(&mut self, _on_exception: bool) -> Result<()> {
                Ok(())
            }
        }

        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("fill-error.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(
            &path,
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        builder.set_fill_executor(Box::new(FailingFillExecutor { fail_first: true }));
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("delegate error must propagate");
        assert_eq!(
            error.to_string(),
            "unsupported operation: fill failed on purpose"
        );

        // A later successful fill routes `finish` through the fill delegate.
        builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect("second fill succeeds");
        builder.finish().expect("finish via fill delegate");
        assert!(builder.finished_via_fill());
    }

/// `write_rows` preserves the sheet name when `auto_trim` is disabled.
    #[test]
    fn write_rows_preserves_sheet_name_when_auto_trim_disabled() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("no-trim.xlsx");
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Data2".to_owned(),
            auto_trim: false,
            ..WriteOptions::default()
        });
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let mut cells = std::collections::BTreeMap::new();
        cells.insert(0, crate::core::DynamicValue::String("x".to_owned()));
        builder.add_content([DynamicRow::new(cells)], &sheet)?;
        assert_eq!(
            builder
                .write_context()
                .current_write_holder()
                .sheet_context()
                .map(WriteSheetContext::sheet_name),
            Some("Data2")
        );
        Ok(())
    }

/// `fill` preserves the sheet name when `auto_trim` is disabled.
    #[test]
    fn fill_respects_auto_trim_disabled() -> Result<()> {
        let mut builder = ExcelBuilderImpl::from_options(
            "fill-no-trim.xlsx",
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        builder.set_fill_executor(Box::new(ContextFillExecutor));
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "FillNoTrim".to_owned(),
            auto_trim: false,
            need_head: false,
            ..WriteOptions::default()
        });
        builder.fill(&DynamicRow::default(), FillConfig::new(), &sheet)?;
        assert_eq!(
            builder
                .write_context()
                .current_write_holder()
                .sheet_context()
                .map(WriteSheetContext::sheet_name),
            Some("FillNoTrim")
        );
        Ok(())
    }

/// A schema with duplicate forced indexes surfaces as a holder-state error
    /// from `add_content` (covers the `?` inside `update_current_holder`).
    #[test]
    fn add_content_propagates_holder_resolution_error() {
        struct DuplicateIndexRow;

        impl ExcelRow for DuplicateIndexRow {
            fn schema() -> &'static [ExcelColumn] {
                static SCHEMA: [ExcelColumn; 2] = [
                    ExcelColumn::new("a", "A", Some(0), 0, None),
                    ExcelColumn::new("b", "B", Some(0), 1, None),
                ];
                &SCHEMA
            }

            fn from_row(_row: &RowData) -> Result<Self> {
                Ok(Self)
            }

            fn to_row(&self) -> Result<Vec<CellValue>> {
                Ok(Vec::new())
            }
        }

        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("dup-index.xlsx");
        let sheet = WriteSheet::<DuplicateIndexRow>::new("Sheet1");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let error = builder
            .add_content([DuplicateIndexRow], &sheet)
            .expect_err("duplicate forced index must fail");
        assert!(error.to_string().contains("must be inconsistent"));

        // The conversion surface of the failing-schema row stays callable.
        let row = DuplicateIndexRow::from_row(&RowData::new(
            "Sheet1",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))
        .expect("from_row");
        assert!(row.to_row().expect("to_row").is_empty());
    }

/// Direct conversion-surface check for `ContextRow` (`from_row`/`to_row`).
    #[test]
    fn context_row_conversion_surface() -> Result<()> {
        let row = ContextRow::from_row(&RowData::new(
            "Sheet1",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert_eq!(
            row.to_row()?,
            vec![
                CellValue::String("a".to_owned()),
                CellValue::String("b".to_owned()),
                CellValue::String("c".to_owned()),
            ]
        );
        Ok(())
    }

/// `finish(true)` routes to `finish_on_exception` when no fill session ran.
    #[test]
    fn finish_on_exception_discards_workbook_data() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("finish-on-exception.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        builder.add_content([], &sheet)?;
        builder.finish_on_exception()?;
        Ok(())
    }
