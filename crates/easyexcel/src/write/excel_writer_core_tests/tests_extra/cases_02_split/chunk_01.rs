#[test]
    fn xlsx_height_requesting_handler_applies_row_heights() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("heights.xlsx");
        let mut writer = ExcelWriter::with_handlers(&path, vec![Box::new(HeightRequestingHandler)]);
        writer.write([TwoColRow::new("h", "c")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(b"PK"));
        Ok(())
    }

#[test]
    fn xlsx_stateful_double_write_with_incoming_table_options() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("incoming.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "one")])], &sheet)?;
        let table = MirroredWriteTable::new();
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
            range.get_value((1, 0)),
            Some(&Data::String("two".to_owned()))
        );
        Ok(())
    }

#[test]
    fn xls_cell_value_variant_branches() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("values.xls");
        let mut writer = ExcelWriter::new(&path);
        let date = NaiveDate::from_ymd_opt(2024, 1, 2).expect("date");
        let row = dyn_row_values(&[
            (0, CellValue::Bool(true)),
            (1, CellValue::Int(-7)),
            (2, CellValue::Float(1.5)),
            (3, CellValue::Error("boom".to_owned())),
            (4, CellValue::Formula("SUM(1,2)".to_owned())),
            (
                5,
                CellValue::Hyperlink {
                    url: "https://example.test".to_owned(),
                    text: "link".to_owned(),
                },
            ),
            (6, CellValue::Date(date)),
            (
                7,
                CellValue::DateTime(date.and_hms_opt(3, 4, 5).expect("time")),
            ),
            (8, CellValue::RichText(RichTextStringData::new("rich"))),
            (
                9,
                CellValue::Decimal(BigDecimal::from_str("12.34").expect("dec")),
            ),
            (
                10,
                CellValue::Decimal(BigDecimal::from_str("9007199254740992").expect("dec")),
            ),
            (11, CellValue::Float(1e12)),
            (12, CellValue::Empty),
        ]);
        writer.write([row], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
    Ok(())
}

#[test]
fn xls_image_cell_values_return_typed_unsupported() -> Result<()> {
    let directory = tempdir()?;
    let cases = [
        (
            "image.xls",
            CellValue::Image(vec![4, 5, 6]),
            "CellValue::Image",
        ),
        (
            "images.xls",
            CellValue::Images {
                value: Box::new(CellValue::String("img".to_owned())),
                images: vec![ImageData::new(vec![1, 2, 3])],
            },
            "CellValue::Images",
        ),
    ];
    for (file_name, value, expected) in cases {
        let path = directory.path().join(file_name);
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write(
            [dyn_row_values(&[(0, value)])],
            &WriteSheet::new("Sheet1"),
        );
        assert!(matches!(
            result,
            Err(ExcelError::Unsupported(message)) if message.contains(expected)
        ));
    }
    Ok(())
}

#[test]
    fn xlsx_cell_value_variant_branches() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("values.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            use_scientific_format: true,
            use_1904_windowing: true,
            ..WriteOptions::default()
        });
        let date = NaiveDate::from_ymd_opt(2024, 1, 2).expect("date");
        let row = dyn_row_values(&[
            (0, CellValue::Float(1e12)),
            (
                1,
                CellValue::Decimal(BigDecimal::from_str("9007199254740992").expect("dec")),
            ),
            (
                2,
                CellValue::Decimal(BigDecimal::from_str("1000000000000").expect("dec")),
            ),
            (3, CellValue::Date(date)),
            (
                4,
                CellValue::DateTime(date.and_hms_opt(1, 2, 3).expect("time")),
            ),
            (
                5,
                CellValue::Comment {
                    value: Box::new(CellValue::Bool(true)),
                    text: "note text".to_owned(),
                },
            ),
            (6, CellValue::Bool(false)),
            (7, CellValue::Error("boom".to_owned())),
            (8, CellValue::Formula("A1+B1".to_owned())),
            (
                9,
                CellValue::Hyperlink {
                    url: "https://example.test".to_owned(),
                    text: "go".to_owned(),
                },
            ),
        ]);
        writer.write([row], &sheet)?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(b"PK"));
        Ok(())
    }

#[test]
    fn xls_and_xlsx_loop_merge_annotation_rows() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("loop.xls");
        let mut xls_writer = ExcelWriter::new(&xls_path);
        let rows = vec![
            LoopMergeRow::new(vec![CellValue::String("a".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("b".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("c".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("d".to_owned())]),
        ];
        xls_writer.write(rows, &WriteSheet::new("Sheet1"))?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("loop.xlsx");
        let mut xlsx_writer = ExcelWriter::new(&xlsx_path);
        let rows = vec![
            LoopMergeRow::new(vec![CellValue::String("a".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("b".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("c".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("d".to_owned())]),
        ];
        xlsx_writer.write(rows, &WriteSheet::new("Sheet1"))?;
        xlsx_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        Ok(())
    }

#[test]
    fn xls_and_xlsx_absolute_merge_annotation_rows() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("merge.xls");
        let mut xls_writer = ExcelWriter::new(&xls_path);
        xls_writer.write(
            [AbsoluteMergeRow::new(vec![
                CellValue::String("l".to_owned()),
                CellValue::String("r".to_owned()),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("merge.xlsx");
        let mut xlsx_writer = ExcelWriter::new(&xlsx_path);
        xlsx_writer.write(
            [AbsoluteMergeRow::new(vec![
                CellValue::String("l".to_owned()),
                CellValue::String("r".to_owned()),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        xlsx_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        Ok(())
    }

#[test]
    fn negative_merge_handler_properties_are_skipped() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("neg.xls");
        let mut xls_writer =
            ExcelWriter::with_handlers(&xls_path, vec![Box::new(NegativeMergeHandler)]);
        xls_writer.write([TwoColRow::new("v", "w")], &WriteSheet::new("Sheet1"))?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("neg.xlsx");
        let mut xlsx_writer =
            ExcelWriter::with_handlers(&xlsx_path, vec![Box::new(NegativeMergeHandler)]);
        xlsx_writer.write([TwoColRow::new("v", "w")], &WriteSheet::new("Sheet1"))?;
        xlsx_writer.finish()?;

        // Negative indexes in template layout merges are skipped too.
        let tpl_path = directory.path().join("neg-tpl.xlsx");
        let mut tpl_writer = ExcelWriter::with_handlers_and_options(
            &tpl_path,
            vec![Box::new(NegativeMergeHandler)],
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        tpl_writer.write([TwoColRow::new("v", "w")], &WriteSheet::new("Sheet1"))?;
        tpl_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        assert!(tpl_path.exists());
        Ok(())
    }

#[test]
    fn negative_metadata_merge_is_rejected_at_handler_load() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("neg-meta.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write(
            [NegativeMergeRow::new(vec![CellValue::String(
                "v".to_owned(),
            )])],
            &WriteSheet::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

#[test]
    fn xls_annotation_font_style_merge_and_rgb_remap() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("fonts.xls");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [FontStyleRow::new(vec![
                CellValue::String("f".to_owned()),
                CellValue::String("o".to_owned()),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }

#[test]
    fn convert_row_at_data_error_maps_physical_column() -> Result<()> {
        let columns = selected_columns(FailingRow::schema(), &WriteOptions::default())?;
        let result = convert_row_at(
            &FailingRow,
            &ConverterRegistry::default(),
            "Sheet1",
            3,
            &columns,
        );
        let error = result.expect_err("must fail");
        let text = error.to_string();
        assert!(text.contains("Sheet1"), "{text}");
        assert!(text.contains("row=3"), "{text}");
        assert!(text.contains("column=Some(0)"), "{text}");
        assert!(text.contains("injected"), "{text}");

        let directory = tempdir()?;
        let path = directory.path().join("failing.xlsx");
        let mut writer = ExcelWriter::new(&path);
        assert!(
            writer
                .write([FailingRow], &WriteSheet::new("Sheet1"))
                .is_err()
        );
        Ok(())
    }

#[test]
    fn xls_finish_via_output_stream_with_and_without_template() -> Result<()> {
        for use_template in [false, true] {
            let output = ExcelOutputStream::new(Vec::new());
            let inspect = output.clone();
            let mut writer = ExcelWriter::with_output_stream(
                "logical.xls",
                output,
                Vec::new(),
                WriteOptions {
                    auto_close_stream: false,
                    template_bytes: if use_template {
                        Some(xls_template_bytes("Sheet1"))
                    } else {
                        None
                    },
                    ..WriteOptions::default()
                },
            );
            writer.write([dyn_row(&[(0, "streamed")])], &WriteSheet::new("Sheet1"))?;
            writer.finish()?;
            let bytes = inspect.with_inner(Clone::clone).expect("open stream");
            assert!(bytes.starts_with(CFB_MAGIC));
        }
        Ok(())
    }

#[test]
    fn xls_finish_save_failure_propagates() -> Result<()> {
        let directory = tempdir()?;
        // A directory with an .xls name is not a writable file, so saving fails.
        let path = directory.path().join("out.xls");
        std::fs::create_dir(&path)?;
        let mut writer = ExcelWriter::new(&path);
        writer.write([dyn_row(&[(0, "x")])], &WriteSheet::new("Sheet1"))?;
        assert!(matches!(writer.finish(), Err(ExcelError::Io(_))));
        Ok(())
    }

#[test]
    fn xls_template_write_absent_sheet_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("absent.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write([dyn_row(&[(0, "x")])], &WriteSheet::new("NoSuchSheet"));
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }
