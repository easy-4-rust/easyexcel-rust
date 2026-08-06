#[test]
    fn compatibility_builder_writes_xlsx_and_runs_registered_handlers() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("users.xlsx");
        let calls = Arc::new(AtomicUsize::new(0));

        ExcelWriterBuilder::new()
            .file(&output)
            .need_head(false)
            .register_write_handler(WorkbookProbe(Arc::clone(&calls)))
            .sheet_name("Users")?
            .register_write_handler(WorkbookProbe(Arc::clone(&calls)))
            .do_write(vec![SimpleRow("alice")])?;

        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let mut workbook: Xlsx<_> = open_workbook(&output)
            .map_err(|error: calamine::XlsxError| ExcelError::Format(error.to_string()))?;
        let range = workbook
            .worksheet_range("Users")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        assert_eq!(
            range.get_value((0, 0)).and_then(|cell| cell.get_string()),
            Some("alice")
        );
        Ok(())
    }

#[test]
    fn exact_java_handler_lifecycle_runs_in_creation_and_dispose_order() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("exact-handler-lifecycle.xlsx");
        let events = Arc::new(Mutex::new(Vec::new()));

        ExcelWriterBuilder::new()
            .file(&output)
            .need_head(false)
            .register_write_handler(ExactLifecycleProbe(Arc::clone(&events)))
            .sheet_name("Users")?
            .do_write(vec![SimpleRow("alice")])?;

        assert_eq!(
            *events.lock().expect("event log mutex poisoned"),
            vec![
                "before_workbook_create",
                "after_workbook_create",
                "before_sheet_create",
                "after_sheet_create",
                "before_row_create",
                "after_row_create",
                "before_cell_create",
                "after_cell_create",
                "after_cell_data_converted",
                "after_cell_dispose",
                "after_row_dispose",
                "after_sheet_dispose",
                "after_workbook_dispose",
            ]
        );
        Ok(())
    }

#[test]
    fn duplicate_handlers_execute_only_the_lowest_order_instance() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("deduplicated-handlers.xlsx");
        let first = Arc::new(AtomicUsize::new(0));
        let duplicate = Arc::new(AtomicUsize::new(0));
        let repeatable = Arc::new(AtomicUsize::new(0));

        ExcelWriterBuilder::new()
            .file(&output)
            .register_write_handler(UniqueWorkbookProbe {
                calls: Arc::clone(&duplicate),
                order: 20,
                unique_value: "workbook-probe",
            })
            .register_write_handler(WorkbookProbe(Arc::clone(&repeatable)))
            .register_write_handler(UniqueWorkbookProbe {
                calls: Arc::clone(&first),
                order: -20,
                unique_value: "workbook-probe",
            })
            .register_write_handler(WorkbookProbe(Arc::clone(&repeatable)))
            .sheet_name("Users")?
            .do_write(vec![SimpleRow("alice")])?;

        assert_eq!(first.load(Ordering::SeqCst), 1);
        assert_eq!(duplicate.load(Ordering::SeqCst), 0);
        assert_eq!(repeatable.load(Ordering::SeqCst), 2);
        Ok(())
    }

#[test]
    fn sheet_own_workbook_callback_is_supplementary_to_initialized_parent() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("sheet-handler-precedence.xlsx");
        let workbook_calls = Arc::new(AtomicUsize::new(0));
        let sheet_calls = Arc::new(AtomicUsize::new(0));

        ExcelWriterBuilder::new()
            .file(&output)
            .register_write_handler(UniqueWorkbookProbe {
                calls: Arc::clone(&workbook_calls),
                order: 0,
                unique_value: "same-handler",
            })
            .sheet_name("Users")?
            .register_write_handler(UniqueWorkbookProbe {
                calls: Arc::clone(&sheet_calls),
                order: 0,
                unique_value: "same-handler",
            })
            .do_write(vec![SimpleRow("alice")])?;

        assert_eq!(sheet_calls.load(Ordering::SeqCst), 1);
        assert_eq!(workbook_calls.load(Ordering::SeqCst), 1);
        Ok(())
    }

#[test]
    fn table_and_sheet_own_workbook_callbacks_are_each_supplementary() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("table-handler-precedence.xlsx");
        let workbook_calls = Arc::new(AtomicUsize::new(0));
        let sheet_calls = Arc::new(AtomicUsize::new(0));
        let table_calls = Arc::new(AtomicUsize::new(0));

        ExcelWriterBuilder::new()
            .file(&output)
            .register_write_handler(UniqueWorkbookProbe {
                calls: Arc::clone(&workbook_calls),
                order: 0,
                unique_value: "same-handler",
            })
            .sheet_name("Users")?
            .register_write_handler(UniqueWorkbookProbe {
                calls: Arc::clone(&sheet_calls),
                order: 0,
                unique_value: "same-handler",
            })
            .table()
            .register_write_handler(Box::new(UniqueWorkbookProbe {
                calls: Arc::clone(&table_calls),
                order: 0,
                unique_value: "same-handler",
            }))
            .do_write(vec![SimpleRow("alice")])?;

        assert_eq!(table_calls.load(Ordering::SeqCst), 1);
        assert_eq!(sheet_calls.load(Ordering::SeqCst), 1);
        assert_eq!(workbook_calls.load(Ordering::SeqCst), 1);
        Ok(())
    }

#[test]
    fn sheet_explicit_need_head_overrides_inherited_workbook_value() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("sheet-override.xlsx");

        ExcelWriterBuilder::new()
            .file(&output)
            .need_head(false)
            .sheet_name("Users")?
            .need_head(true)
            .do_write(vec![SimpleRow("alice")])?;

        let mut workbook: Xlsx<_> = open_workbook(&output)
            .map_err(|error: calamine::XlsxError| ExcelError::Format(error.to_string()))?;
        let range = workbook
            .worksheet_range("Users")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        assert_eq!(
            range.get_value((0, 0)).and_then(|cell| cell.get_string()),
            Some("Value")
        );
        assert_eq!(
            range.get_value((1, 0)).and_then(|cell| cell.get_string()),
            Some("alice")
        );
        Ok(())
    }

#[test]
    fn sheet_default_style_inherits_and_can_override_workbook_value() -> Result<()> {
        let directory = tempdir()?;
        let inherited_output = directory.path().join("style-inherited.xlsx");
        let overridden_output = directory.path().join("style-overridden.xlsx");

        ExcelWriterBuilder::new()
            .file(&inherited_output)
            .use_default_style(false)
            .sheet_name("Users")?
            .do_write(vec![SimpleRow("alice")])?;
        ExcelWriterBuilder::new()
            .file(&overridden_output)
            .use_default_style(false)
            .sheet_name("Users")?
            .use_default_style(true)
            .do_write(vec![SimpleRow("alice")])?;

        let inherited_styles = zip_entry(&inherited_output, "xl/styles.xml")?;
        let overridden_styles = zip_entry(&overridden_output, "xl/styles.xml")?;
        assert!(!inherited_styles.contains("<b/>"));
        assert!(overridden_styles.contains("<b/>"));
        Ok(())
    }

#[test]
    fn explicit_excel_type_overrides_the_output_file_extension() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("users.data");

        ExcelWriterBuilder::new()
            .file(&output)
            .excel_type(ExcelTypeEnum::Csv)
            .with_bom(false)
            .sheet()?
            .do_write(vec![SimpleRow("alice")])?;

        let csv = std::fs::read_to_string(output)?;
        assert_eq!(csv, "Value\nalice\n");
        Ok(())
    }

#[test]
    fn output_stream_builder_writes_real_xlsx_without_creating_a_file() -> Result<()> {
        let directory = tempdir()?;
        let logical_path = directory.path().join("stream.xlsx");
        let output = ExcelOutputStream::new(Cursor::new(Vec::<u8>::new()));
        let inspection = output.clone();

        ExcelWriterBuilder::new()
            .file(&logical_path)
            .auto_close_stream(false)
            .output_stream(output)
            .sheet_name("Users")
            .need_head(false)
            .do_write(vec![SimpleRow("alice")])?;

        let bytes = inspection
            .with_inner(|cursor| cursor.get_ref().clone())
            .expect("auto_close_stream(false) must keep the stream open");
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        assert!(archive.by_name("[Content_Types].xml").is_ok());
        assert!(!logical_path.exists());
        Ok(())
    }

#[test]
    fn compatibility_sheet_table_chain_writes_and_finishes() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("table-chain.xlsx");

        ExcelWriterBuilder::new()
            .file(&output)
            .sheet_name("Users")?
            .need_head(false)
            .table_no(2)
            .do_write(vec![SimpleRow("alice")])?;

        let mut workbook: Xlsx<_> = open_workbook(&output)
            .map_err(|error: calamine::XlsxError| ExcelError::Format(error.to_string()))?;
        let range = workbook
            .worksheet_range("Users")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        assert_eq!(
            range.get_value((0, 0)).and_then(|cell| cell.get_string()),
            Some("alice")
        );
        Ok(())
    }

#[test]
    fn table_explicit_need_head_overrides_inherited_sheet_value() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("table-override.xlsx");

        ExcelWriterBuilder::new()
            .file(&output)
            .need_head(false)
            .sheet_name("Users")?
            .table_no(2)
            .need_head(true)
            .do_write(vec![SimpleRow("alice")])?;

        let mut workbook: Xlsx<_> = open_workbook(&output)
            .map_err(|error: calamine::XlsxError| ExcelError::Format(error.to_string()))?;
        let range = workbook
            .worksheet_range("Users")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        assert_eq!(
            range.get_value((0, 0)).and_then(|cell| cell.get_string()),
            Some("Value")
        );
        assert_eq!(
            range.get_value((1, 0)).and_then(|cell| cell.get_string()),
            Some("alice")
        );
        Ok(())
    }

#[test]
    fn table_column_selection_inherits_and_can_override_parent() -> Result<()> {
        let directory = tempdir()?;
        let inherited_output = directory.path().join("table-inherited-columns.xlsx");
        let overridden_output = directory.path().join("table-overridden-columns.xlsx");

        ExcelWriterBuilder::new()
            .file(&inherited_output)
            .need_head(false)
            .include_column_indexes([1])
            .sheet_name("Users")?
            .table_no(0)
            .do_write(vec![TwoColumnRow("A", "B")])?;
        ExcelWriterBuilder::new()
            .file(&overridden_output)
            .need_head(false)
            .include_column_indexes([1])
            .sheet_name("Users")?
            .table_no(0)
            .include_column_indexes([0])
            .do_write(vec![TwoColumnRow("A", "B")])?;

        for (path, expected) in [(&inherited_output, "B"), (&overridden_output, "A")] {
            let mut workbook: Xlsx<_> = open_workbook(path)
                .map_err(|error: calamine::XlsxError| ExcelError::Format(error.to_string()))?;
            let range = workbook
                .worksheet_range("Users")
                .map_err(|error| ExcelError::Format(error.to_string()))?;
            let values = range
                .rows()
                .flat_map(|row| row.iter())
                .filter_map(|cell| cell.get_string())
                .collect::<Vec<_>>();
            assert_eq!(values, vec![expected]);
        }
        Ok(())
    }

#[test]
    fn write_workbook_file_and_template_setters_store_real_paths() {
        let mut workbook = WriteWorkbook::new();
        workbook.set_file("result.xlsx");
        workbook.set_template_file("template.xlsx");

        assert_eq!(workbook.file(), Some(std::path::Path::new("result.xlsx")));
        assert_eq!(
            workbook.template_file(),
            Some(std::path::Path::new("template.xlsx"))
        );

        let workbook = WriteWorkbook::from(crate::WriteOptions {
            excel_type: Some(ExcelTypeEnum::Csv),
            ..crate::WriteOptions::default()
        });
        assert_eq!(workbook.excel_type(), ExcelTypeEnum::Csv);
    }

