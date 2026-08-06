#[test]
    fn csv_schema_change_between_writes_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("schema.csv");
        let mut writer = ExcelWriter::new(&path);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        let result = writer.write([dyn_row(&[(0, "x")])], &WriteSheet::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn table_schema_mismatch_between_writes_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-schema.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [TwoColRow::new("a", "b")],
            &WriteSheet::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        let result = writer.write_with_table_handlers(
            [dyn_row(&[(0, "x")])],
            &WriteSheet::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

#[test]
    fn sheet_handlers_on_initialized_sheet_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("late-sheet-handlers.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write([dyn_row(&[(0, "first")])], &sheet)?;
        let result = writer.write_with_table_handlers(
            [dyn_row(&[(0, "second")])],
            &sheet,
            &table,
            vec![Box::new(HeightRequestingHandler)],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

#[test]
    fn table_handlers_on_new_sheet_run_workbook_callbacks() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("new-sheet-handlers.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "a")])],
            &sheet,
            &table,
            vec![Box::new(HeightRequestingHandler)],
            Vec::new(),
        )?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn xls_dynamic_head_with_height_handler() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dyn-heights.xls");
        let mut writer = ExcelWriter::with_handlers(&path, vec![Box::new(HeightRequestingHandler)]);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "n"), (1, "a")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("n".to_owned())));
        Ok(())
    }

#[test]
    fn xlsx_legacy_template_autofit() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("autofit.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            auto_width: true,
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "autofit me")])], &sheet)?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn biff8_create_row_overflow_errors() {
        let mut book = Biff8Book::default();
        let mut creator = Biff8RowCreator {
            sheet: book.sheet_mut("Sheet1"),
        };
        let result = create_row(&mut creator, 65_536);
        assert!(matches!(result, Err(ExcelError::Format(_))));
        let result = create_row(&mut creator, 65_535);
        assert!(result.is_ok());
    }

#[test]
    fn effective_sheet_name_keeps_trimmed_when_disabled() {
        let options = WriteOptions {
            auto_trim: false,
            sheet_name: "  padded  ".to_owned(),
            ..WriteOptions::default()
        };
        assert_eq!(effective_sheet_name(&options), "  padded  ");
        let trimmed = WriteOptions {
            auto_trim: true,
            sheet_name: "  padded  ".to_owned(),
            ..WriteOptions::default()
        };
        assert_eq!(effective_sheet_name(&trimmed), "padded");
    }

#[test]
    fn write_with_sheet_handlers_after_finish_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("finished.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let result = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "b")])],
            &WriteSheet::new("Sheet1"),
            vec![Box::new(HeightRequestingHandler)],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

#[test]
    fn handler_ignore_fill_and_requested_style() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("style-h.xls");
        let mut xls_writer =
            ExcelWriter::with_handlers(&xls_path, vec![Box::new(StyleRequestingHandler)]);
        xls_writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("style-h.xlsx");
        let mut xlsx_writer =
            ExcelWriter::with_handlers(&xlsx_path, vec![Box::new(StyleRequestingHandler)]);
        xlsx_writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        xlsx_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        Ok(())
    }

#[test]
    fn dynamic_head_merge_mismatch_errors() -> Result<()> {
        let options = WriteOptions {
            dynamic_head: Some(vec![vec!["A".to_owned()]]),
            ..WriteOptions::default()
        };
        let columns = selected_columns(&[], &options)?;
        assert_eq!(columns.len(), 1);
        let head = vec![vec!["A".to_owned()], vec!["B".to_owned()]];
        let result = dynamic_head_merge_ranges(&columns, &head, 0);
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn xlsx_template_annotation_merge_and_width_handlers() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-ann.xlsx");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(
            MirroredOnceAbsoluteMerge::from_property(crate::core::OnceAbsoluteMergeProperty::new(
                0, 0, 0, 1,
            ))
            .expect("merge strategy"),
        )];
        let mut width_strategy = SimpleColumnWidthStyleStrategy::new();
        width_strategy.set_column_width(0, 42);
        handlers.push(Box::new(width_strategy));
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx_with_handlers::<AbsoluteMergeRow, _>(
            &path,
            &options,
            [AbsoluteMergeRow::new(vec![
                CellValue::String("l".to_owned()),
                CellValue::String("r".to_owned()),
            ])],
            &mut handlers,
        )?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn initialize_existing_table_holder_csv_early_return() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl.csv");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a")])], &sheet)?;
        let table = MirroredWriteTable::with_table_no(5);
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "c")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains('a'));
        assert!(content.contains('c'));
        Ok(())
    }

#[test]
    fn initialize_existing_table_holder_xls_applies_table_merges() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<TwoColRow>::new("Sheet1");
        writer.write([TwoColRow::new("a", "b")], &sheet)?;
        let table = MirroredWriteTable::with_table_no(5);
        writer.write_with_table_handlers(
            [TwoColRow::new("c", "d")],
            &sheet,
            &table,
            Vec::new(),
            vec![Box::new(
                MirroredOnceAbsoluteMerge::from_property(
                    crate::core::OnceAbsoluteMergeProperty::new(10, 10, 0, 1),
                )
                .expect("merge strategy"),
            )],
        )?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert!(range.get_value((3, 0)).is_some());
        Ok(())
    }

#[test]
    fn initialize_existing_table_holder_xlsx_template_layout() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-tpl.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<TwoColRow>::new("Sheet1");
        writer.write([TwoColRow::new("a", "b")], &sheet)?;
        let table = MirroredWriteTable::with_table_no(5);
        writer.write_with_table_handlers(
            [TwoColRow::new("c", "d")],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn initialize_existing_table_holder_xlsx_column_widths() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-widths.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<TwoColRow>::from_options(WriteOptions {
            column_widths: vec![(0, 30)],
            ..WriteOptions::default()
        });
        writer.write([TwoColRow::new("a", "b")], &sheet)?;
        let table = MirroredWriteTable::with_table_no(5);
        writer.write_with_table_handlers(
            [TwoColRow::new("c", "d")],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn xlsx_annotation_font_merge_and_number_format() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("fonts.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [FontStyleRow::new(vec![
                CellValue::String("f".to_owned()),
                CellValue::String("o".to_owned()),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        writer.finish()?;

        let fmt_path = directory.path().join("fmt.xlsx");
        let mut fmt_writer = ExcelWriter::new(&fmt_path);
        let fmt_sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            content_styles: vec![CellStyle {
                number_format: Some("0.00".to_owned()),
                bold: true,
                italic: true,
                font_color: Some(0x00_FF00),
                background_color: Some(0xFF_0000),
                horizontal_alignment: Some(HorizontalAlignment::Right),
                vertical_alignment: Some(VerticalAlignment::Top),
                wrap_text: true,
            }],
            ..WriteOptions::default()
        });
        fmt_writer.write([dyn_row(&[(0, "x")])], &fmt_sheet)?;
        fmt_writer.finish()?;
        assert!(path.exists());
        assert!(fmt_path.exists());
        Ok(())
    }

