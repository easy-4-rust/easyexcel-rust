#[test]
    fn xls_template_dynamic_head_second_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-table2-fail.xls");
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
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "n"), (1, "a"), (2, "m")])],
            &sheet,
            &MirroredWriteTable::with_table_no(2),
            Vec::new(),
            Vec::new(),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::from_options(WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                dynamic_head: Some(vec![
                    vec!["User".to_owned(), "Name".to_owned()],
                    vec!["User".to_owned(), "Age".to_owned()],
                    vec!["Meta".to_owned()],
                ]),
                ..WriteOptions::default()
            }),
            &MirroredWriteTable::with_table_no(3),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn xlsx_template_existing_state_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-state-fail.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write(
            [dyn_row(&[(0, "one")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::with_table_no(0),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn xlsx_legacy_seed_spill_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn xlsx_absent_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("absent-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "NewSheet".to_owned(),
            template_bytes: Some(xlsx_template_bytes("TemplateOnly")),
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn xls_nested_dir_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("nested").join("out-fail.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoopHandler3)];
        let result = crate::write::write_xls::write_xls_with_handlers::<FailingRow3, _>(
            &path,
            &WriteOptions::default(),
            [FailingRow3],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn xls_plain_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("plain-fail.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoopHandler3)];
        let result = crate::write::write_xls::write_xls_with_handlers::<FailingRow3, _>(
            &path,
            &WriteOptions::default(),
            [FailingRow3],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn xlsx_compress_temp_files_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("spill-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn csv_table_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-fail.csv");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn csv_table_handlers_second_write_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-fail2.csv");
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
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn table_handlers_first_write_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-schema-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn table_handlers_new_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("new-sheet-handlers-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            vec![Box::new(NoopHandler3)],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn xlsx_template_annotation_merge_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-ann-fail.xlsx");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(
            MirroredOnceAbsoluteMerge::from_property(crate::core::OnceAbsoluteMergeProperty::new(
                0, 0, 0, 1,
            ))
            .expect("merge strategy"),
        )];
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<FailingRow3, _>(
            &path,
            &options,
            [FailingRow3],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn csv_early_return_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-early-fail.csv");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row(&[(0, "a")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::with_table_no(5),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

