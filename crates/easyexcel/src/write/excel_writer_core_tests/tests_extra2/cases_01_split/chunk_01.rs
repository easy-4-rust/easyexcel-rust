#[test]
    fn loop_merge_bad_annotation_handlers_rejected() -> Result<()> {
        // 对应 Java：`@ContentLoopMerge(eachRow=1, columnExtend=1) → IllegalArgumentException`。
        let directory = tempdir()?;
        let path = directory.path().join("bad-loop.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write(
            [LoopMergeBadRow { cells: Vec::new() }],
            &WriteSheet::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn loop_merge_bad_table_annotation_handlers_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("bad-loop-table.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let table = MirroredWriteTable::new();
        let result = writer.write_with_table_handlers(
            [LoopMergeBadRow { cells: Vec::new() }],
            &WriteSheet::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn sheet_handlers_workbook_callback_error_propagates() -> Result<()> {
        // 对应 Java：新 sheet 首次注册 sheet handler 时运行 workbook 回调。
        let directory = tempdir()?;
        let path = directory.path().join("sheet-cb.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "first")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
            vec![Box::new(StageFailingHandler(
                FailStage::BeforeWorkbookCreate,
            ))],
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn table_handlers_new_sheet_workbook_callback_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-cb.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let table = MirroredWriteTable::new();
        let result = writer.write_with_table_handlers(
            [dyn_row(&[(0, "first")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
            &table,
            vec![Box::new(StageFailingHandler(
                FailStage::BeforeWorkbookCreate,
            ))],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn existing_sheet_table_template_layout_error_propagates() -> Result<()> {
        // 对应 Java：已有 sheet 上建表时按模板布局（列宽/合并），列号超限必须报错。
        let directory = tempdir()?;
        let path = directory.path().join("table-layout.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write(
            [dyn_row(&[(0, "seed")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let table = MirroredWriteTable::new();
        let result = writer.write_with_table_handlers(
            [WideIndexRow { cells: Vec::new() }],
            &WriteSheet::<WideIndexRow>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn table_batch_row_conversion_errors_by_backend() -> Result<()> {
        // 对应 Java：doWrite 期间行转换失败 → 各后端（csv/xls/xlsx）批量写入报错。
        let directory = tempdir()?;

        let csv_path = directory.path().join("table.csv");
        let mut csv_writer = ExcelWriter::new(&csv_path);
        let table = MirroredWriteTable::new();
        let csv_result = csv_writer.write_with_table_handlers(
            [FailingRow2],
            &WriteSheet::<FailingRow2>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(csv_result, Err(ExcelError::Data { .. })));

        let xls_path = directory.path().join("table.xls");
        let mut xls_writer = ExcelWriter::new(&xls_path);
        let xls_result = xls_writer.write_with_table_handlers(
            [FailingRow2],
            &WriteSheet::<FailingRow2>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(xls_result, Err(ExcelError::Data { .. })));

        let xlsx_path = directory.path().join("table.xlsx");
        let mut xlsx_writer = ExcelWriter::new(&xlsx_path);
        let xlsx_outcome = xlsx_writer.write_with_table_handlers(
            [FailingRow2],
            &WriteSheet::<FailingRow2>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(xlsx_outcome, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn stateful_xls_start_rejects_missing_template_file() -> Result<()> {
        // 对应 Java：withTemplate(file) 指向不存在的文件 → 打开失败。
        let directory = tempdir()?;
        let path = directory.path().join("missing-template.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_file: Some(directory.path().join("absent.xls")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

#[test]
    fn stateful_xlsx_start_rejects_csv_template_source() -> Result<()> {
        // 对应 Java：xlsx 不允许用 csv 模板。
        let directory = tempdir()?;
        let path = directory.path().join("csv-template.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_file: Some(directory.path().join("template.csv")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

#[test]
    fn stateful_xlsx_start_rejects_missing_template_file() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("missing-template.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_file: Some(directory.path().join("absent.xlsx")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

#[test]
    fn stateful_xlsx_legacy_seed_rejects_invalid_sheet_name() -> Result<()> {
        // 对应 Java：模板 sheet 名含非法字符（`[`）时 seed 到工作簿必须失败。
        let bytes = zip_template(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES_XML),
            ("_rels/.rels", MINIMAL_PACKAGE_RELS_XML),
            (
                "xl/workbook.xml",
                minimal_workbook_xml("bad[name").as_bytes(),
            ),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", MINIMAL_SHEET_XML),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("legacy-bad-name.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "bad[name".to_owned(),
                template_bytes: Some(bytes),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("bad[name"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn stateful_xls_template_handler_callback_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("xls-tpl-cb.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(StageFailingHandler(FailStage::DataCell))],
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn stateful_xlsx_legacy_seed_after_sheet_create_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-cb.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(StageFailingHandler(FailStage::AfterSheetCreate))],
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn stateful_xlsx_legacy_seed_row_conversion_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-bad-row.xlsx");
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
        let result = writer.write([FailingRow2], &WriteSheet::<FailingRow2>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

#[test]
    fn stateful_xlsx_template_absent_sheet_missing_content_types() -> Result<()> {
        // 对应 Java：模板缺少 [Content_Types].xml 时创建新 sheet 必须报错。
        let bytes = zip_template(&[
            ("xl/workbook.xml", minimal_workbook_xml("Sheet1").as_bytes()),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", MINIMAL_SHEET_XML),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("no-ct.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Absent".to_owned(),
                template_bytes: Some(bytes),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Absent"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn stateful_xlsx_template_missing_styles_xml() -> Result<()> {
        // 对应 Java：模板缺少 styles.xml 且单元格请求样式 → 导入样式必须报错。
        let bytes = zip_template(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES_XML),
            ("xl/workbook.xml", minimal_workbook_xml("Sheet1").as_bytes()),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", MINIMAL_SHEET_XML),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("no-styles.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(StyleRequestingHandler)],
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(bytes),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

