#[test]
    fn stateful_xlsx_template_missing_sheet_data() -> Result<()> {
        // 对应 Java：模板 worksheet 缺少 sheetData → 追加行必须报错。
        let sheet_xml = br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><dimension ref="A1"/></worksheet>"#;
        let bytes = zip_template(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES_XML),
            ("xl/workbook.xml", minimal_workbook_xml("Sheet1").as_bytes()),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", sheet_xml),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("no-sheetdata.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
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

#[test]
    fn public_xls_template_missing_file_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("out.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(directory.path().join("absent.xls")),
            ..WriteOptions::default()
        };
        let result = crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

#[test]
    fn public_xls_template_handler_callback_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("out.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::DataCell))];
        let result = crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn public_xls_save_under_regular_file_rejected() -> Result<()> {
        // 对应 Java：父路径是普通文件时无法创建目录。
        let directory = tempdir()?;
        let blocker = directory.path().join("blocker");
        std::fs::write(&blocker, b"not a directory")?;
        let path = blocker.join("out.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let result = crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

#[test]
    fn public_xls_head_cell_handler_not_invoked_without_head() -> Result<()> {
        // 对应 Java：无表头（空 schema 且未配置 dynamic_head）时 head cell
        // handler 不会被调用，写入正常完成（handler 错误仅在表头真实创建时传播）。
        let directory = tempdir()?;
        let path = directory.path().join("head-nohead.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::HeadCell))];
        let result = crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(result.is_ok(), "{result:?}");
        Ok(())
    }

#[test]
    fn public_xls_dynamic_head_cell_handler_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("head-dyn-err.xls");
        let options = WriteOptions {
            dynamic_head: Some(vec![vec!["Level".to_owned()], vec!["Field".to_owned()]]),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::HeadCell))];
        let result = crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn public_xls_loop_merge_column_overflow_rejected() -> Result<()> {
        // 对应 Java：BIFF8 合并列号超过 255 → 报错。
        let directory = tempdir()?;
        let path = directory.path().join("loop-overflow.xls");
        let loop_merges = vec![MirroredLoopMergeStrategy::new(2, 1, 300)?];
        let options = WriteOptions {
            loop_merges,
            ..WriteOptions::default()
        };
        let result = crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v"), (1, "w")])],
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn public_xls_head_cells_skipped_by_handler() -> Result<()> {
        // 对应 Java：handler 跳过单元格后表头不落盘（ExcelWriter 空 sheet 仍可保存）。
        let directory = tempdir()?;
        let path = directory.path().join("skipped.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(SkipCellHandler)];
        crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        )?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn public_xls_handler_loop_merge_invalid_property_rejected() -> Result<()> {
        // 对应 Java：handler 返回 eachRow=1/columnExtend=1 的 loop merge → 校验失败。
        let directory = tempdir()?;
        let path = directory.path().join("bad-handler-loop.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(LoopMergeBadHandler)];
        let result = crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn public_xlsx_template_missing_file_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("out.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(directory.path().join("absent.xlsx")),
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

#[test]
    fn public_xlsx_template_handler_callback_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("out.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::DataCell))];
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn public_xlsx_template_missing_styles_xml() -> Result<()> {
        let bytes = zip_template(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES_XML),
            ("xl/workbook.xml", minimal_workbook_xml("Sheet1").as_bytes()),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", MINIMAL_SHEET_XML),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("no-styles.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(bytes),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(StyleRequestingHandler)];
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn template_append_cell_styles_column_overflow_rejected() -> Result<()> {
        // 对应 Java：模板列号超过 XLSX 上限时样式编译必须报错。
        let mut package = crate::write::template_write::TemplatePackage::from_bytes(
            xlsx_template_bytes("Sheet1").as_slice(),
        )?;
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let rows = vec![vec![(70_000usize, CellValue::String("wide".to_owned()))]];
        let empty_converted: Vec<Vec<(usize, WriteCellData)>> = Vec::new();
        let empty_ignore: Vec<Vec<bool>> = vec![Vec::new()];
        let empty_requested: Vec<Vec<Option<ExcelCellStyle>>> = vec![Vec::new()];
        let result = template_append_cell_styles::<PlainRow>(
            &mut package,
            &options,
            &[],
            &rows,
            &rows,
            &empty_converted,
            &empty_ignore,
            &empty_requested,
            true,
            0,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

