#[test]
    fn write_with_sheet_handlers_new_sheet_callback_error_propagates() -> Result<()> {
        // 对应 Java：新 sheet 首次注册 sheet handler 时运行 workbook 回调，
        // 回调失败必须向上传播（`runOwnWorkbookCallbacks`）。
        let directory = tempdir()?;
        let path = directory.path().join("sheet-cb-err.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Fresh");
        let result = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "x")])],
            &sheet,
            vec![Box::new(StageFailingHandler3(
                FailStage3::BeforeWorkbookCreate,
            ))],
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn write_with_table_handlers_new_sheet_callback_error_propagates() -> Result<()> {
        // 对应 Java：表写入路径首次注册 sheet handler 时 workbook 回调失败。
        let directory = tempdir()?;
        let path = directory.path().join("table-cb-err.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [dyn_row(&[(0, "x")])],
            &WriteSheet::<DynamicRow>::new("Fresh"),
            &MirroredWriteTable::new(),
            vec![Box::new(StageFailingHandler3(
                FailStage3::BeforeWorkbookCreate,
            ))],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn xlsx_write_after_sheet_create_error_propagates() -> Result<()> {
        // 对应 Java：`afterSheetCreate` 回调失败 → `ExcelWriteExecutor` 报错。
        let directory = tempdir()?;
        let path = directory.path().join("sheet-create-err.xlsx");
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler3(FailStage3::AfterSheetCreate))];
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "x")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn xlsx_template_new_sheet_missing_workbook_rels_errors() -> Result<()> {
        // 对应 Java：withTemplate 后写入模板中不存在的 sheet，POI 需要创建，
        // 缺少 workbook 关系表时 `createSheet` 抛异常。
        let directory = tempdir()?;
        let path = directory.path().join("tpl-no-rels.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_missing_workbook_rels()),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "x")])],
            &WriteSheet::<DynamicRow>::new("BrandNew"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn save_xls_book_creates_nested_parent_directory() -> Result<()> {
        // 对应 Java：写入 `a/b/out.xls` 时自动 `mkdirs`。
        let directory = tempdir()?;
        let path = directory.path().join("nested").join("plain.xls");
        crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "x")])],
        )?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn save_xls_book_parent_is_regular_file_errors() -> Result<()> {
        // 对应 Java：父路径被普通文件占位时 `mkdirs` 抛 IOException。
        let directory = tempdir()?;
        let blocker = directory.path().join("blocker");
        std::fs::write(&blocker, b"not a directory")?;
        let path = blocker.join("out.xls");
        let result = crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "x")])],
        );
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn xls_write_head_cell_failure_propagates() -> Result<()> {
        // 对应 Java：写表头时 `beforeCellCreate` 抛异常 → 整次写入失败。
        // 使用 schema 非空的行（非 dynamic 表头分支）；DynamicRow 无表头时
        // head 回调不会被调用（见 public_xls_head_cell_handler_not_invoked_without_head）。
        let directory = tempdir()?;
        let path = directory.path().join("head-cell-err.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler3(FailStage3::HeadCell))];
        let result = crate::write::write_xls::write_xls_with_handlers::<SingleColRow3, _>(
            &path,
            &WriteOptions::default(),
            [SingleColRow3 {
                cells: vec![CellValue::String("x".to_owned())],
            }],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn write_sheet_onto_template_rejects_csv_template_bytes() {
        // 对应 Java：`validateTemplateSource` 拒绝 CSV 模板。
        let mut workbook = Workbook::new();
        let options = WriteOptions {
            template_bytes: Some(b"a,b\n1,2".to_vec()),
            ..WriteOptions::default()
        };
        let result = write_sheet_onto_template::<DynamicRow, _>(
            &mut workbook,
            &options,
            [dyn_row(&[(0, "x")])],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
    }

#[test]
    fn write_sheet_onto_template_missing_template_file_errors() -> Result<()> {
        // 对应 Java：`withTemplate(file)` 指向不存在的文件 → IOException。
        let directory = tempdir()?;
        let missing = directory.path().join("missing.xlsx");
        let mut workbook = Workbook::new();
        let options = WriteOptions {
            template_file: Some(missing),
            ..WriteOptions::default()
        };
        let result = write_sheet_onto_template::<DynamicRow, _>(
            &mut workbook,
            &options,
            [dyn_row(&[(0, "x")])],
            &mut [],
        );
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn template_package_from_bytes_rejects_garbage() {
        let result =
            crate::write::template_write::TemplatePackage::from_bytes(b"not a zip package");
        assert!(matches!(result, Err(ExcelError::Format(_))));
    }

#[test]
    fn sheet_handlers_failing_row_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("sheet-handlers-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_sheet_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Fresh"),
            vec![Box::new(NoopHandler3)],
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn table_handlers_xlsx_new_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-fail.xlsx");
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
    fn table_handlers_xls_existing_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-fail.xls");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row(&[(0, "first")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::with_table_no(7),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn table_handlers_with_sheet_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-handlers-fail.xlsx");
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
    fn xls_to_writer_template_failing_row() {
        let mut output = Vec::new();
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let result = crate::write::write_xls::write_xls_to_writer::<FailingRow3, _, _>(
            std::path::Path::new("logical.xls"),
            &mut output,
            &options,
            [FailingRow3],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
    }

#[test]
    fn xlsx_stateful_table_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("incoming-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row(&[(0, "one")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

#[test]
    fn xls_absolute_merge_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("merge-fail.xls");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write([FailingRow3], &WriteSheet::<FailingRow3>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn xlsx_absolute_merge_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("merge-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write([FailingRow3], &WriteSheet::<FailingRow3>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn xls_font_style_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("fonts-fail.xls");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write([FailingRow3], &WriteSheet::<FailingRow3>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn xls_template_dynamic_head_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-table-fail.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<FailingRow3>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
                vec!["Meta".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &sheet,
            &MirroredWriteTable::with_table_no(2),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

