#[test]
    fn xlsx_compress_temp_files_populates_gzip_spill_snapshot() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("spill.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                compress_temp_files: true,
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "spill")])], &sheet)?;
        writer.write([dyn_row(&[(0, "again")])], &sheet)?;
        writer.finish()?;
        let snapshot = writer
            .last_gzip_spill_snapshot()
            .expect("snapshot after finish");
        assert_eq!(snapshot.sheet_name, "Sheet1");
        assert!(snapshot.is_gzip);
        assert!(snapshot.uncompressed_len > 0);
        Ok(())
    }

#[test]
    fn finish_gzip_spill_failure_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("spill-fail.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                compress_temp_files: true,
                ..WriteOptions::default()
            },
        );
        let mut spill = crate::write::gzip_spill::GzipSheetDataWriter::create_owned("Sheet1")?;
        let snapshot = spill.snapshot()?;
        std::fs::remove_file(&snapshot.path)?;
        writer.gzip_spills.insert("Sheet1".to_owned(), spill);
        assert!(writer.finish().is_err());
        Ok(())
    }

#[test]
    fn write_with_table_handlers_xlsx_new_sheet_and_table() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "tabled")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("tabled".to_owned()))
        );
        Ok(())
    }

#[test]
    fn write_with_table_handlers_xls_existing_sheet_new_table() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<TwoColRow>::new("Sheet1");
        writer.write([TwoColRow::new("first", "x")], &sheet)?;
        let table = MirroredWriteTable::with_table_no(7);
        writer.write_with_table_handlers(
            [TwoColRow::new("second", "y")],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((3, 0)),
            Some(&Data::String("second".to_owned()))
        );
        Ok(())
    }

#[test]
    fn write_with_table_handlers_registration_errors() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-err.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "first")])],
            &sheet,
            &table,
            vec![Box::new(HeightRequestingHandler)],
            Vec::new(),
        )?;
        // Duplicate sheet-handler registration on an initialized sheet.
        let duplicate_sheet = writer.write_with_table_handlers(
            [dyn_row(&[(0, "second")])],
            &sheet,
            &table,
            vec![Box::new(HeightRequestingHandler)],
            Vec::new(),
        );
        assert!(matches!(duplicate_sheet, Err(ExcelError::Unsupported(_))));
        // Duplicate table-handler registration on an initialized table.
        let duplicate_table = writer.write_with_table_handlers(
            [dyn_row(&[(0, "second")])],
            &sheet,
            &table,
            Vec::new(),
            vec![Box::new(HeightRequestingHandler)],
        );
        assert!(matches!(duplicate_table, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

#[test]
    fn xls_dynamic_head_automatic_merge_applied() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dyn-head.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
                vec!["Meta".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "n"), (1, "a"), (2, "m")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let merges = workbook
            .merge_cells_by_sheet_name("Sheet1")
            .map_err(format_error)?;
        assert!(!merges.is_empty());
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("n".to_owned())));
        Ok(())
    }

#[test]
    fn xls_dynamic_row_with_over_256_columns_errors() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("wide.xls");
        let mut writer = ExcelWriter::new(&path);
        let wide = dyn_row(&(0..300).map(|index| (index, "x")).collect::<Vec<_>>());
        let result = writer.write([wide], &WriteSheet::new("Sheet1"));
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn xls_content_styles_apply_all_attributes() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("styled.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            content_styles: vec![CellStyle {
                bold: true,
                italic: true,
                font_color: Some(0xFF_0000),
                background_color: Some(0x00_FF00),
                horizontal_alignment: Some(HorizontalAlignment::Center),
                vertical_alignment: Some(VerticalAlignment::Center),
                wrap_text: true,
                number_format: Some("0.00".to_owned()),
            }],
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "styled")])], &sheet)?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }

#[test]
    fn xlsx_public_write_with_template_bytes() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("public-template.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "pub")])],
        )?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("pub".to_owned()))
        );
        Ok(())
    }

#[test]
    fn xls_public_write_with_template_bytes() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("public-template.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "pub")])],
        )?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("pub".to_owned()))
        );
        Ok(())
    }

#[test]
    fn xls_public_write_to_writer_with_template() -> Result<()> {
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut output = Vec::new();
        crate::write::write_xls::write_xls_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xls"),
            &mut output,
            &options,
            [dyn_row(&[(0, "streamed")])],
            &mut [],
        )?;
        assert!(output.starts_with(CFB_MAGIC));
        Ok(())
    }

#[test]
    fn finish_twice_is_noop() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("twice.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        writer.finish()?;
        writer.finish_on_exception()?;
        assert!(writer.is_finished());
        Ok(())
    }

#[test]
    fn xls_height_requesting_handler_applies_head_and_content_heights() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("heights.xls");
        let mut writer = ExcelWriter::with_handlers(&path, vec![Box::new(HeightRequestingHandler)]);
        writer.write([TwoColRow::new("h", "c")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((1, 0)), Some(&Data::String("h".to_owned())));
        Ok(())
    }

