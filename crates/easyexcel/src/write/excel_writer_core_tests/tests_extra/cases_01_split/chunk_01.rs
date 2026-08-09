#[test]
    fn xls_stateful_double_write_appends_rows_and_finish_saves() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("stateful.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a"), (1, "b")])], &sheet)?;
        writer.write([dyn_row(&[(0, "c"), (1, "d")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((0, 0)), Some(&Data::String("a".to_owned())));
        assert_eq!(range.get_value((1, 1)), Some(&Data::String("d".to_owned())));
        Ok(())
    }

#[test]
    fn xls_stateful_finish_on_exception_discards_unless_configured() -> Result<()> {
        let directory = tempdir()?;
        for (on_exception, expected_exists) in [(false, false), (true, true)] {
            let path = directory.path().join(format!("exc-{on_exception}.xls"));
            let mut writer = ExcelWriter::with_handlers_and_options(
                &path,
                Vec::new(),
                WriteOptions {
                    write_excel_on_exception: on_exception,
                    ..WriteOptions::default()
                },
            );
            writer.write([dyn_row(&[(0, "boom")])], &WriteSheet::new("Sheet1"))?;
            writer.finish_on_exception()?;
            assert_eq!(path.exists(), expected_exists);
            if expected_exists {
                let bytes = std::fs::read(&path)?;
                assert!(bytes.starts_with(CFB_MAGIC));
            }
        }
        Ok(())
    }

#[test]
    fn xls_sheet_handlers_registration_rules() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("handlers.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "first")])], &sheet)?;
        // Handlers cannot be attached to an already-initialized sheet.
        let result = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "late")])],
            &sheet,
            vec![Box::new(HeightRequestingHandler)],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));

        // A fresh sheet accepts handlers; a second registration is rejected.
        let fresh = WriteSheet::<DynamicRow>::new("Fresh");
        writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "early")])],
            &fresh,
            vec![Box::new(HeightRequestingHandler)],
        )?;
        let duplicate = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "again")])],
            &fresh,
            vec![Box::new(HeightRequestingHandler)],
        );
        assert!(matches!(duplicate, Err(ExcelError::Unsupported(_))));
        writer.finish()?;
        Ok(())
    }

#[test]
    fn xls_template_stateful_append_and_finish_preserves_seed() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("template.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a")])], &sheet)?;
        writer.write([dyn_row(&[(0, "b")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        // Seed row, then the two appended rows.
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("seed".to_owned()))
        );
        assert_eq!(range.get_value((1, 0)), Some(&Data::String("a".to_owned())));
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("b".to_owned())));
        Ok(())
    }

#[test]
    fn xls_template_rejects_non_xls_bytes() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("bad-template.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn csv_with_template_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("template.csv");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

#[test]
    fn xlsx_template_stateful_append_and_finish_preserves_seed() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("template.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a")])], &sheet)?;
        writer.write([dyn_row(&[(0, "b")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("seed".to_owned()))
        );
        assert_eq!(range.get_value((1, 0)), Some(&Data::String("a".to_owned())));
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("b".to_owned())));
        Ok(())
    }

#[test]
    fn xlsx_template_legacy_seed_path_writes_values() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        writer.write([dyn_row(&[(0, "legacy")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("legacy".to_owned()))
        );
        Ok(())
    }

#[test]
    fn xlsx_template_creates_sheet_absent_from_template() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("new-sheet.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("TemplateOnly")),
                ..WriteOptions::default()
            },
        );
        writer.write([dyn_row(&[(0, "fresh")])], &WriteSheet::new("NewSheet"))?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let names = workbook.sheet_names();
        assert!(names.contains(&"NewSheet".to_owned()));
        let range = workbook.worksheet_range("NewSheet").map_err(format_error)?;
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("fresh".to_owned()))
        );
        Ok(())
    }

#[test]
    fn csv_stateful_append_and_finish_writes_file() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("stateful.csv");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a"), (1, "b")])], &sheet)?;
        writer.write([dyn_row(&[(0, "c")])], &sheet)?;
        writer.finish()?;
        let content = std::fs::read_to_string(&path)?;
        let lines = content.lines().collect::<Vec<_>>();
        assert!(
            lines
                .iter()
                .any(|line| line.contains('a') && line.contains('b'))
        );
        assert!(lines.iter().any(|line| line.contains('c')));
        Ok(())
    }

#[test]
    fn csv_output_stream_finish_on_exception_emits_capture() -> Result<()> {
        for (on_exception, should_emit) in [(false, false), (true, true)] {
            let output = ExcelOutputStream::new(Vec::new());
            let inspect = output.clone();
            let mut writer = ExcelWriter::with_output_stream(
                "response.csv",
                output,
                Vec::new(),
                WriteOptions {
                    auto_close_stream: false,
                    write_excel_on_exception: on_exception,
                    ..WriteOptions::default()
                },
            );
            writer.write([dyn_row(&[(0, "captured")])], &WriteSheet::new("Sheet1"))?;
            writer.finish_on_exception()?;
            let bytes = inspect.with_inner(Clone::clone).expect("open stream");
            let content = String::from_utf8(bytes).map_err(format_error)?;
            assert_eq!(content.contains("captured"), should_emit);
            assert!(!content.is_empty() || !should_emit);
        }
        Ok(())
    }

#[test]
    fn csv_second_sheet_name_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("two-sheets.csv");
        let mut writer = ExcelWriter::new(&path);
        writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("first"))?;
        let result = writer.write([dyn_row(&[(0, "b")])], &WriteSheet::new("second"));
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

#[test]
    fn workbook_mut_exposes_inner_workbook() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("inner.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer
            .workbook_mut()
            .add_worksheet()
            .write_string(0, 0, "manual")
            .map_err(format_error)?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(b"PK"));
        Ok(())
    }

#[test]
    fn xls_formula_cells_emit_formula_records() -> Result<()> {
        // 对应 Java：POI HSSF setCellFormula → FORMULA 记录（rgce Ptg 编码）
        let directory = tempdir()?;
        let path = directory.path().join("formula.xls");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row_values(&[
                (0, CellValue::Int(2)),
                (1, CellValue::Int(3)),
                (2, CellValue::Formula("A1+B1".to_owned())),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        writer.finish()?;
        assert!(path.exists());
        // calamine 回读：普通单元格为原值；公式单元格回读写入时的缓存值
        // （xls 求值引擎当场计算：A1+B1 = 5，而非 0）
        let mut workbook = calamine::Xls::<std::io::BufReader<std::fs::File>>::new(
            std::io::BufReader::new(std::fs::File::open(&path)?),
        )
        .map_err(|e| crate::core::ExcelError::Format(e.to_string()))?;
        let range = workbook
            .worksheet_range("Sheet1")
            .map_err(|e| crate::core::ExcelError::Format(e.to_string()))?;
        assert_eq!(range.get_value((0, 0)), Some(&calamine::Data::Int(2)));
        assert_eq!(range.get_value((0, 1)), Some(&calamine::Data::Int(3)));
        assert_eq!(range.get_value((0, 2)), Some(&calamine::Data::Float(5.0)));
        Ok(())
    }

#[test]
    fn xlsx_password_protected_stateful_output_is_ole() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("secret.xlsx");
        let mut writer =
            ExcelWriter::with_handlers_and_password(&path, Vec::new(), Some("pw".to_owned()));
        writer.write([dyn_row(&[(0, "hidden")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }

#[test]
    fn xlsx_template_password_protected_output_is_ole() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("secret-template.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                password: Some("pw".to_owned()),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write([dyn_row(&[(0, "hidden")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }
