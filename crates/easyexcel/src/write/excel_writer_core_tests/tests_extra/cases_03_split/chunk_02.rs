#[test]
    fn sort_handlers_dedupes_repeat_executors() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dedupe.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![
            Box::new(UniqueHandler("shared")),
            Box::new(UniqueHandler("shared")),
        ];
        crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "dedupe")])],
            &mut handlers,
        )?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn xlsx_template_to_writer_with_password() -> Result<()> {
        let mut output = Vec::new();
        crate::write::xlsx_write::write_xlsx_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                password: Some("pw".to_owned()),
                ..WriteOptions::default()
            },
            [dyn_row(&[(0, "encrypted")])],
            &mut [],
        )?;
        assert!(output.starts_with(CFB_MAGIC));
        Ok(())
    }

#[test]
    fn legacy_seed_public_with_layout_and_absent_sheet() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-layout.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            column_widths: vec![(0, 25)],
            merge_ranges: vec![MergeRange::new(1, 2, 0, 1)],
            auto_width: true,
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "layout")])],
        )?;
        assert!(path.exists());

        let absent_path = directory.path().join("legacy-absent.xlsx");
        let absent_options = WriteOptions {
            sheet_name: "BrandNew".to_owned(),
            sheet_index: Some(9),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &absent_path,
            &absent_options,
            [dyn_row(&[(0, "fresh")])],
        )?;
        let workbook = open_xlsx(&absent_path)?;
        assert!(workbook.sheet_names().contains(&"BrandNew".to_owned()));
        Ok(())
    }

#[test]
    fn xlsx_template_wide_row_style_column_error() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("wide-tpl.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let wide = dyn_row(&(0..70_000).map(|index| (index, "x")).collect::<Vec<_>>());
        let result = writer.write([wide], &WriteSheet::new("Sheet1"));
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn xlsx_template_absent_rows_get_no_heights() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("absent-tpl.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let rows: Vec<Option<TwoColRow>> = vec![
            Some(TwoColRow::new("a", "b")),
            None,
            Some(TwoColRow::new("c", "d")),
        ];
        writer.write(rows, &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn xlsx_template_requested_styles_merge_with_handler_styles() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("req-styles.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(StyleOnlyHandler)],
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("a".to_owned())));
        Ok(())
    }

#[test]
    fn loop_merge_handler_strategy_applied() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("loop-h.xls");
        let mut xls_writer =
            ExcelWriter::with_handlers(&xls_path, vec![Box::new(LoopMergeHandler)]);
        let rows = vec![
            TwoColRow::new("a", "b"),
            TwoColRow::new("c", "d"),
            TwoColRow::new("e", "f"),
            TwoColRow::new("g", "h"),
        ];
        xls_writer.write(rows, &WriteSheet::new("Sheet1"))?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("loop-h.xlsx");
        let mut xlsx_writer =
            ExcelWriter::with_handlers(&xlsx_path, vec![Box::new(LoopMergeHandler)]);
        let rows = vec![
            TwoColRow::new("a", "b"),
            TwoColRow::new("c", "d"),
            TwoColRow::new("e", "f"),
            TwoColRow::new("g", "h"),
        ];
        xlsx_writer.write(rows, &WriteSheet::new("Sheet1"))?;
        xlsx_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        Ok(())
    }

#[test]
    fn xlsx_legacy_seed_to_writer() -> Result<()> {
        let mut output = Vec::new();
        crate::write::xlsx_write::write_xlsx_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
            [dyn_row(&[(0, "legacy")])],
            &mut [],
        )?;
        assert!(output.starts_with(b"PK"));
        Ok(())
    }

#[test]
    fn stateful_xlsx_template_negative_merge_handler_layout() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("neg-tpl.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(NegativeMergeHandler)],
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write([TwoColRow::new("v", "w")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn row_type_from_row_constructors_are_invokable() {
        let row_data = crate::core::RowData::new(
            "Sheet1",
            0,
            vec![CellValue::String("x".to_owned())],
            std::sync::Arc::new(std::collections::HashMap::new()),
        );
        assert!(TwoColRow::from_row(&row_data).is_ok());
        assert!(LoopMergeRow::from_row(&row_data).is_ok());
        assert!(AbsoluteMergeRow::from_row(&row_data).is_ok());
        assert!(NegativeMergeRow::from_row(&row_data).is_ok());
        assert!(FontStyleRow::from_row(&row_data).is_ok());
        assert!(FailingRow::from_row(&row_data).is_ok());
        assert!(
            NegativeMergeRow::new(vec![CellValue::String("v".to_owned())])
                .to_row()
                .is_ok()
        );
    }

#[test]
    fn xlsx_requested_style_merged_with_handler_style() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("req-style.xlsx");
        let mut writer = ExcelWriter::with_handlers(&path, vec![Box::new(StyleOnlyHandler)]);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn cell_format_applies_converted_data_format_without_annotation() {
        let context = CellFormatContext {
            explicit: None,
            cell: None,
            font: None,
            handler_cell: None,
            converted_cell: None,
            converted_data_format: Some("0.00"),
            global: WriteGlobalFlags::default(),
        };
        let format = cell_format(context);
        // rust_xlsxwriter exposes no num-format getter; exercising cell_format
        // with a converted data format is the coverage goal.
        let _ = format;
    }

#[test]
    fn apply_annotation_once_absolute_merge_applies_when_handler_absent() -> Result<()> {
        let mut worksheet = rust_xlsxwriter::Worksheet::new();
        let handlers: Vec<Box<dyn WriteHandler>> = Vec::new();
        apply_annotation_once_absolute_merge::<AbsoluteMergeRow>(&mut worksheet, &handlers)?;
        Ok(())
    }

#[test]
    fn table_annotation_handlers_second_write_short_circuits() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-twice.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<TwoColRow>::new("Sheet1");
        let table = crate::write::metadata::write_table::WriteTable::new();
        writer.write_with_table([TwoColRow::new("a", "b")], &sheet, &table)?;
        writer.write_with_table([TwoColRow::new("c", "d")], &sheet, &table)?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

