    #[test]
    fn public_xlsx_legacy_seed_rejects_csv_template_source() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-csv.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(directory.path().join("template.csv")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn public_xlsx_legacy_seed_missing_template_file_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-missing.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(directory.path().join("absent.xlsx")),
            use_legacy_template_seed: true,
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
    fn public_xlsx_legacy_seed_row_conversion_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-bad-row.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow2, _>(&path, &options, [FailingRow2]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn public_xlsx_dynamic_head_handler_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("head-dyn.xlsx");
        let options = WriteOptions {
            dynamic_head: Some(vec![vec!["Level".to_owned()], vec!["Field".to_owned()]]),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::HeadCell))];
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
    fn template_layout_skips_explicit_column_widths() -> Result<()> {
        // 对应 Java：WriteOptions.column_widths 显式列宽优先于注解/策略宽度。
        let directory = tempdir()?;
        let path = directory.path().join("explicit-width.xlsx");
        let mut options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        options.column_widths = vec![(0, 30)];
        crate::write::xlsx_write::write_xlsx::<PlainRow, _>(
            &path,
            &options,
            [PlainRow::new("a", "b")],
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_comment_with_invalid_image_errors() -> Result<()> {
        // 对应 Java：批注内的图片数据损坏时按图片解析错误处理。
        let directory = tempdir()?;
        let path = directory.path().join("comment-img.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let row = dyn_row_values(&[(
            0,
            CellValue::Comment {
                value: Box::new(CellValue::Image(vec![0x89, 0x50, 0x4E])),
                text: "note".to_owned(),
            },
        )]);
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(&path, &options, [row]);
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // trait 方法直接调用补充：from_row / to_row 不经过写入主路径
    // ========================================================================

    #[test]
    fn failing_row2_from_row_is_constructible() -> Result<()> {
        // 对应 Java：ExcelRow.fromRow 只在读取侧被调用，写入侧直接调用验证。
        let row = FailingRow2::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(matches!(
            row.to_row(),
            Err(ExcelError::Data { message, .. })
                if message == "test-only row conversion failure"
        ));
        Ok(())
    }

    #[test]
    fn plain_row_from_and_to_row_round_trip() -> Result<()> {
        // 对应 Java：PlainRow 的 fromRow/toRow 往返一致（空单元格行）。
        let row = PlainRow::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(row.to_row()?.is_empty());
        Ok(())
    }

    #[test]
    fn loop_merge_bad_row_from_and_to_row_round_trip() -> Result<()> {
        // 对应 Java：注解行 fromRow/toRow 直接调用（校验失败发生在写入前）。
        let row = LoopMergeBadRow::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(row.to_row()?.is_empty());
        Ok(())
    }

    #[test]
    fn wide_index_row_from_and_to_row_round_trip() -> Result<()> {
        // 对应 Java：宽列索引行 fromRow/toRow 直接调用（写入前即被列号校验拦截）。
        let row = WideIndexRow::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(row.to_row()?.is_empty());
        Ok(())
    }
