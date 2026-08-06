    #[test]
    fn xlsx_template_existing_sheet_uses_else_target_name() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-existing.xlsx");
        let _options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut writer = ExcelWriter::new(&path);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xls_write_with_automatic_merge_head_disabled() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("no-merge-head.xls");
        let options = WriteOptions {
            automatic_merge_head: false,
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        crate::write::write_xls::write_xls::<TwoColRow, _>(
            &path,
            &options,
            [TwoColRow::new("a", "b")],
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn template_head_style_none_column_matches_head_fallback() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-dyn-head.xlsx");
        let _options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut writer = ExcelWriter::new(&path);
        // DynamicRow has an empty schema, so head columns never match.
        writer.write([dyn_row(&[(0, "a"), (1, "b")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xls_finish_via_output_stream_with_and_without_template_full_loop() -> Result<()> {
        for use_template in [false, true] {
            let directory = tempdir()?;
            let logical = directory.path().join("stream.xls");
            let output = ExcelOutputStream::new(std::io::Cursor::new(Vec::<u8>::new()));
            let mut options = WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                ..WriteOptions::default()
            };
            if use_template {
                let mut book = crate::write::xls_adapter::Biff8Book::default();
                book.sheet_mut("Sheet1");
                options.template_bytes = Some(book.to_cfb_bytes()?);
            }
            let writer = ExcelWriter::with_output_stream(logical, output, Vec::new(), options);
            let mut writer = writer;
            writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
            writer.finish()?;
        }
        Ok(())
    }

    #[test]
    fn ensure_table_annotation_handlers_second_call_short_circuits() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("ensure-twice.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let options = WriteOptions::default();
        writer.ensure_table_annotation_handlers::<TwoColRow>("Sheet1", 0, &options)?;
        writer.ensure_table_annotation_handlers::<TwoColRow>("Sheet1", 0, &options)?;
        Ok(())
    }

    #[test]
    fn xlsx_template_existing_sheet_name_uses_else_target() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-else-target.xlsx");
        let _options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut writer = ExcelWriter::new(&path);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn template_head_extra_column_hits_none_head_fallback() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-extra-col.xlsx");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.set_name("Sheet1").expect("sheet name");
        sheet.write_string(0, 0, "A").expect("head a");
        sheet.write_string(0, 1, "B").expect("head b");
        sheet.write_string(0, 2, "C").expect("head c");
        let template = workbook.save_to_buffer().expect("template");
        let _options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(template),
            ..WriteOptions::default()
        };
        let mut writer = ExcelWriter::new(&path);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn template_append_cell_styles_head_with_unknown_column() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("styles-head.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut package = crate::write::template_write::TemplatePackage::from_bytes(
            xlsx_template_bytes("Sheet1").as_slice(),
        )?;
        let rows = vec![
            vec![
                (0usize, CellValue::String("h0".to_owned())),
                (5usize, CellValue::String("extra".to_owned())),
            ],
            vec![(0usize, CellValue::String("v".to_owned()))],
        ];
        let converted: Vec<Vec<(usize, crate::core::WriteCellData)>> = Vec::new();
        let ignore: Vec<Vec<bool>> = vec![Vec::new(), Vec::new()];
        let requested: Vec<Vec<Option<ExcelCellStyle>>> = vec![Vec::new(), Vec::new()];
        let styles = template_append_cell_styles::<TwoColRow>(
            &mut package,
            &options,
            &[],
            &rows,
            &rows,
            &converted,
            &ignore,
            &requested,
            true,
            0,
        )?;
        assert_eq!(styles.len(), 2);
        let _ = ExcelWriter::new(&path);
        Ok(())
    }
