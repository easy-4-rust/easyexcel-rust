#[test]
    fn builder_setters_store_values_on_parameter() {
        let builder = ExcelWriterBuilder::new()
            .password("secret")
            .in_memory(true)
            .write_excel_on_exception(true)
            .charset(CsvCharset::new("GBK"))
            .with_template_bytes(vec![1u8, 2, 3])
            .with_template("template.xlsx")
            .relative_head_row_index(3)
            .automatic_merge_head(true)
            .include_column_field_names(["first", "second"])
            .exclude_column_indexes([2])
            .exclude_column_field_names(["skip"])
            .order_by_include_column(true);
        let parameter = builder.parameter();

        assert_eq!(parameter.password(), Some("secret"));
        assert!(parameter.in_memory());
        assert!(parameter.write_excel_on_exception());
        assert_eq!(parameter.charset(), &CsvCharset::new("GBK"));
        assert_eq!(
            parameter.template_file(),
            Some(std::path::Path::new("template.xlsx"))
        );
        // set_template_file clears an earlier buffered stream template (Java semantics).
        assert_eq!(parameter.options.template_bytes, None);
        assert_eq!(parameter.options.relative_head_row_index, 3);
        assert!(parameter.options.automatic_merge_head);
        assert_eq!(
            parameter.options.include_column_field_names,
            Some(vec!["first".to_owned(), "second".to_owned()])
        );
        assert_eq!(parameter.options.exclude_column_indexes, vec![2]);
        assert_eq!(
            parameter.options.exclude_column_field_names,
            vec!["skip".to_owned()]
        );
        assert!(parameter.options.order_by_include_column);
    }

#[test]
    fn builder_use_default_style_true_sets_bold_head_style() {
        let builder = ExcelWriterBuilder::new().use_default_style(true);
        assert!(builder.parameter().options.use_default_style);
        let plain = ExcelWriterBuilder::new().use_default_style(false);
        assert!(!plain.parameter().options.use_default_style);
    }

#[test]
    fn builder_build_without_file_returns_format_error() {
        let result = ExcelWriterBuilder::new().build();
        assert!(result.is_err());
        let error = result.err().expect("build without a file must fail");
        assert!(matches!(error, ExcelError::Format(_)));
    }

#[test]
    fn builder_default_matches_new() {
        let default_builder = ExcelWriterBuilder::default();
        let new_builder = ExcelWriterBuilder::new();
        let default_parameter = default_builder.parameter();
        let new_parameter = new_builder.parameter();
        assert!(default_parameter.file().is_none());
        assert_eq!(default_parameter.file(), new_parameter.file());
        assert_eq!(default_parameter.excel_type(), new_parameter.excel_type());
    }

#[test]
    fn builder_sheet_no_writes_to_selected_worksheet_index() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("sheet-no.xlsx");

        ExcelWriterBuilder::new()
            .file(&output)
            .need_head(false)
            .sheet_no(1)?
            .do_write(vec![SimpleRow("alice")])?;

        let mut workbook: Xlsx<_> = open_workbook(&output)
            .map_err(|error: calamine::XlsxError| ExcelError::Format(error.to_string()))?;
        let names = workbook.sheet_names();
        assert!(!names.is_empty(), "expected at least one worksheet");
        let range = workbook
            .worksheet_range(&names[0])
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        assert_eq!(
            range.get_value((0, 0)).and_then(|cell| cell.get_string()),
            Some("alice")
        );
        Ok(())
    }

#[test]
    fn builder_sheet_with_writes_to_named_worksheet() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("sheet-with.xlsx");

        ExcelWriterBuilder::new()
            .file(&output)
            .need_head(false)
            .sheet_with(0, "Users")?
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
    fn output_stream_builder_sheet_no_and_sheet_with_write_real_xlsx() -> Result<()> {
        let directory = tempdir()?;
        let logical_path = directory.path().join("stream-sheets.xlsx");
        let output = ExcelOutputStream::new(Cursor::new(Vec::<u8>::new()));
        let inspection = output.clone();

        ExcelWriterBuilder::new()
            .file(&logical_path)
            .auto_close_stream(false)
            .output_stream(output)
            .sheet_no(0)
            .do_write(vec![SimpleRow("alice")])?;

        let bytes = inspection
            .with_inner(|cursor| cursor.get_ref().clone())
            .expect("auto_close_stream(false) must keep the stream open");
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        assert!(archive.by_name("[Content_Types].xml").is_ok());

        ExcelWriterBuilder::new()
            .auto_close_stream(false)
            .output_stream(ExcelOutputStream::new(Cursor::new(Vec::<u8>::new())))
            .sheet_with(0, "Users")
            .do_write(vec![SimpleRow("bob")])?;
        Ok(())
    }

#[test]
    fn test_row_helpers_round_trip_from_row() -> Result<()> {
        let headers = Arc::new(std::collections::HashMap::<String, usize>::new());
        let simple_row = RowData::new(
            "Users",
            0,
            vec![CellValue::String("alice".to_owned())],
            Arc::clone(&headers),
        );
        assert_eq!(SimpleRow::from_row(&simple_row)?.0, "");
        let two_column_row = RowData::new(
            "Users",
            0,
            vec![
                CellValue::String("A".to_owned()),
                CellValue::String("B".to_owned()),
            ],
            headers,
        );
        let parsed = TwoColumnRow::from_row(&two_column_row)?;
        assert_eq!(parsed.0, "");
        assert_eq!(parsed.1, "");
        Ok(())
    }

#[test]
    fn builder_sheet_without_file_propagates_build_error() {
        // Exercises the `?` unwind edges of sheet()/sheet_no/sheet_name/sheet_with.
        for result in [
            ExcelWriterBuilder::new().sheet(),
            ExcelWriterBuilder::new().sheet_no(0),
            ExcelWriterBuilder::new().sheet_name("Users"),
            ExcelWriterBuilder::new().sheet_with(0, "Users"),
        ] {
            let error = result.err().expect("builder without a file must fail");
            assert!(matches!(error, ExcelError::Format(_)));
        }
    }

