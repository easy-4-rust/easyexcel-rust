#[test]
    fn xls_template_with_table_handlers_and_dynamic_head() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-table.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
                vec!["Meta".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        let table = MirroredWriteTable::with_table_no(2);
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "n"), (1, "a"), (2, "m")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        let table2 = MirroredWriteTable::with_table_no(3);
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "n2"), (1, "a2"), (2, "m2")])],
            &sheet,
            &table2,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert!(range.get_value((4, 0)).is_some());
        Ok(())
    }

#[test]
    fn xlsx_template_with_table_handlers_existing_state() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-table.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "one")])], &sheet)?;
        let table = MirroredWriteTable::with_table_no(0);
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "two")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((2, 0)),
            Some(&Data::String("two".to_owned()))
        );
        Ok(())
    }

#[test]
    fn xlsx_template_height_handler_styles_and_zero_rows() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-styles.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(HeightRequestingHandler)],
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            content_styles: vec![CellStyle {
                bold: true,
                font_color: Some(0x00_FF00),
                background_color: Some(0x00_00FF),
                ..CellStyle::new()
            }],
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "styled")])], &sheet)?;
        writer.finish()?;
        assert!(path.exists());

        let empty_path = directory.path().join("tpl-empty.xlsx");
        let mut empty_writer = ExcelWriter::with_handlers_and_options(
            &empty_path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        empty_writer.write(Vec::<DynamicRow>::new(), &WriteSheet::new("Sheet1"))?;
        empty_writer.finish()?;
        assert!(empty_path.exists());
        Ok(())
    }

#[test]
    fn xlsx_template_public_legacy_seed_with_spill() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-public.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "legacy")])],
        )?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("legacy".to_owned()))
        );
        Ok(())
    }

#[test]
    fn xlsx_template_public_rejects_xls_template_file() -> Result<()> {
        let directory = tempdir()?;
        let template_path = directory.path().join("seed.xls");
        std::fs::write(&template_path, xls_template_bytes("Sheet1"))?;
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(template_path),
            ..WriteOptions::default()
        };
        let path = directory.path().join("dual.xlsx");
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "x")])],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

#[test]
    fn xlsx_template_public_creates_absent_sheet() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("new-sheet-public.xlsx");
        let options = WriteOptions {
            sheet_name: "NewSheet".to_owned(),
            template_bytes: Some(xlsx_template_bytes("TemplateOnly")),
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "fresh")])],
        )?;
        let workbook = open_xlsx(&path)?;
        assert!(workbook.sheet_names().contains(&"NewSheet".to_owned()));
        Ok(())
    }

#[test]
    fn xls_public_template_bad_bytes_and_absent_sheet() -> Result<()> {
        let directory = tempdir()?;
        let bad_path = directory.path().join("bad.xls");
        let bad_options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        assert!(
            crate::write::write_xls::write_xls::<DynamicRow, _>(
                &bad_path,
                &bad_options,
                [dyn_row(&[(0, "x")])],
            )
            .is_err()
        );
        // absent sheet index 9 + XLS template bytes → resolve_package_target
        // 会 create_new=true，ensure_sheet 自动创建新 sheet，写入成功
        let absent_path = directory.path().join("absent.xls");
        let absent_options = WriteOptions {
            sheet_index: Some(9),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        assert!(
            crate::write::write_xls::write_xls::<DynamicRow, _>(
                &absent_path,
                &absent_options,
                [dyn_row(&[(0, "x")])],
            )
            .is_ok()
        );
        Ok(())
    }

#[test]
    fn xls_public_template_with_handlers_to_subdirectory() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("nested").join("out.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(HeightRequestingHandler)];
        crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "nested")])],
            &mut handlers,
        )?;
        assert!(path.exists());

        let plain_path = directory.path().join("plain.xls");
        let plain_options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &plain_path,
            &plain_options,
            [dyn_row(&[(0, "plain")])],
            &mut handlers,
        )?;
        assert!(plain_path.exists());
        Ok(())
    }

#[test]
    fn xlsx_public_compress_temp_files() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("spill-public.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "spill")])],
        )?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn csv_write_with_table_handlers() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table.csv");
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
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "again")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains("tabled"));
        assert!(content.contains("again"));
        Ok(())
    }

