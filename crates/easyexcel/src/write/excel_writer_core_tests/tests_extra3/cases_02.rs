    #[test]
    fn xls_table_merges_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-fail.xls");
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

    #[test]
    fn xlsx_template_layout_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-tpl-fail.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
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

    #[test]
    fn xlsx_column_widths_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-widths-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            column_widths: vec![(0, 30)],
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "a")])], &sheet)?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::from_options(WriteOptions {
                column_widths: vec![(0, 30)],
                ..WriteOptions::default()
            }),
            &MirroredWriteTable::with_table_no(5),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xlsx_font_style_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("fonts-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write([FailingRow3], &WriteSheet::<FailingRow3>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn dedupe_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dedupe-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![
            Box::new(UniqueHandler3("shared")),
            Box::new(UniqueHandler3("shared")),
        ];
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
    fn xlsx_template_password_failing_row() {
        let mut output = Vec::new();
        let options = WriteOptions {
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            password: Some("pw".to_owned()),
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx_to_writer::<FailingRow3, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            [FailingRow3],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
    }

    #[test]
    fn legacy_seed_layout_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-layout-fail.xlsx");
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
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn legacy_seed_absent_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-absent-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "BrandNew".to_owned(),
            sheet_index: Some(9),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xlsx_legacy_seed_writer_failing_row() {
        let mut output = Vec::new();
        let options = WriteOptions {
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx_to_writer::<FailingRow3, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            [FailingRow3],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
    }

    // ========================================================================
    // 模板样式编译：列号超限（对应 Java `@ExcelProperty.index` 越界）。
    // ========================================================================

    #[test]
    fn template_append_cell_styles_wide_column_direct() -> Result<()> {
        // 对应 Java：模板样式编译时列号超过 XLSX 上限 → 报错。
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
        let result = template_append_cell_styles::<FailingRow3>(
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

    // ========================================================================
    // 跳过表头单元格 + 行转换失败（对应 Java handler 跳过单元格后仍转换行）。
    // ========================================================================

    #[test]
    fn xls_skipped_head_cells_failing_row() -> Result<()> {
        // 对应 Java：handler 跳过全部单元格时，行转换（toRow）失败仍须上报。
        let directory = tempdir()?;
        let path = directory.path().join("skipped-fail.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(SkipCellHandler3)];
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
    fn xlsx_explicit_widths_failing_row() -> Result<()> {
        // 对应 Java：显式列宽 + 行转换失败 → 错误传播。
        let directory = tempdir()?;
        let path = directory.path().join("explicit-width-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            column_widths: vec![(0, 30)],
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    // ========================================================================
    // trait 方法直接调用补充：from_row / to_row / before_cell_create Ok 分支
    // ========================================================================

    #[test]
    fn failing_row3_from_row_is_constructible() -> Result<()> {
        // 对应 Java：ExcelRow.fromRow 只在读取侧被调用，写入侧直接调用验证。
        let row = FailingRow3::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(matches!(
            row.to_row(),
            Err(ExcelError::Data { message, .. })
                if message == "round-2 injected conversion failure"
        ));
        Ok(())
    }

    #[test]
    fn single_col_row3_from_and_to_row_round_trip() -> Result<()> {
        // 对应 Java：SingleColRow3 的 fromRow/toRow 往返一致（空单元格行）。
        let row = SingleColRow3::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(row.to_row()?.is_empty());
        Ok(())
    }

    #[test]
    fn stage_failing_handler3_non_matching_stage_passes_cells() {
        // 对应 Java：失败阶段不匹配时 beforeCellCreate 放行（Ok 分支）。
        let mut context = WriteCellContext::new("Sheet1", 0, 0, CellValue::String("v".to_owned()));
        let mut handler = StageFailingHandler3(FailStage3::AfterSheetCreate);
        assert!(handler.before_cell_create(&mut context).is_ok());
    }
