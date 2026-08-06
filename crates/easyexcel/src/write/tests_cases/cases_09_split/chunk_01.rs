#[test]
fn every_handler_failure_stage_is_propagated() -> Result<()> {
    let directory = tempdir()?;
    for (index, stage) in [
        FailureStage::BeforeWorkbook,
        FailureStage::BeforeSheet,
        FailureStage::BeforeHeadRow,
        FailureStage::BeforeHeadCell,
        FailureStage::AfterHeadCell,
        FailureStage::AfterHeadRow,
        FailureStage::BeforeDataRow,
        FailureStage::BeforeDataCell,
        FailureStage::AfterDataCell,
        FailureStage::AfterDataRow,
        FailureStage::AfterSheet,
        FailureStage::AfterWorkbook,
    ]
    .into_iter()
    .enumerate()
    {
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(FailingHandler(stage))];
        let error = write_xlsx_with_handlers::<EveryCell, _>(
            &directory.path().join(format!("handler-{index}.xlsx")),
            &WriteOptions::default(),
            vec![every_cell()],
            &mut handlers,
        )
        .expect_err("handler failure must propagate");
        assert_eq!(error.to_string(), "excel format error: handler failed");
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(InvalidHeaderValueHandler)];
    assert!(
        write_headers_with_handlers(
            worksheet,
            &selected_columns(EveryCell::schema(), &WriteOptions::default())?,
            "Sheet1",
            SheetStyleContext::head(
                &CellStyle::default(),
                &ExcelWriteMetadata::new(),
                WriteGlobalFlags::default()
            ),
            &mut handlers,
            &ImageLayout::default(),
            0,
            None,
        )
        .is_err()
    );
    let worksheet = workbook.add_worksheet();
    let columns = selected_columns(EveryCell::schema(), &WriteOptions::default())?;
    let head = columns
        .iter()
        .map(|_| vec!["Head".to_owned()])
        .collect::<Vec<_>>();
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(InvalidHeaderValueHandler)];
    assert!(
        write_dynamic_headers_with_handlers(
            worksheet,
            &columns,
            &head,
            "Sheet2",
            SheetStyleContext::head(
                &CellStyle::default(),
                &ExcelWriteMetadata::new(),
                WriteGlobalFlags::default()
            ),
            &mut handlers,
            &ImageLayout::default(),
            0,
            true,
            None,
        )
        .is_err()
    );
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn conversion_configuration_column_and_save_failures_propagate() -> Result<()> {
    let directory = tempdir()?;
    let mut broken = every_cell();
    broken.fail = true;
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("broken.xlsx"),
            &WriteOptions::default(),
            vec![broken]
        )
        .is_err()
    );

    let wide_column = Box::leak(Box::new(ExcelColumn::new(
        "wide",
        "Wide",
        Some(65_536),
        0,
        None,
    )));
    let columns = vec![(65_536, 0, &*wide_column)];
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    assert!(write_headers(worksheet, &columns).is_err());
    assert!(
        write_data_row(
            worksheet,
            0,
            &columns,
            &[CellValue::String("wide".to_owned())]
        )
        .is_err()
    );

    USE_WIDE_SCHEMA.with(|wide| wide.set(true));
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("wide-head.xlsx"),
            &WriteOptions::default(),
            Vec::new()
        )
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("wide-data.xlsx"),
            &WriteOptions {
                need_head: false,
                ..WriteOptions::default()
            },
            vec![every_cell()]
        )
        .is_err()
    );
    USE_WIDE_SCHEMA.with(|wide| wide.set(false));

    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("bad-freeze.xlsx"),
            &WriteOptions {
                freeze_panes: Some((1_048_576, 0)),
                ..WriteOptions::default()
            },
            Vec::new()
        )
        .is_err()
    );

    let long_name = Box::leak("x".repeat(32_768).into_boxed_str());
    let long_header = Box::leak(Box::new(ExcelColumn::new(
        "long",
        long_name,
        Some(0),
        0,
        None,
    )));
    assert!(write_headers(worksheet, &[(0, 0, &*long_header)]).is_err());

    let date = NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date");
    let invalid_row = 1_048_576;
    for value in [
        CellValue::String("text".to_owned()),
        CellValue::Bool(true),
        CellValue::Int(1),
        CellValue::Int(i64::MAX),
        CellValue::Float(1.0),
        CellValue::Date(date),
        CellValue::DateTime(date.and_hms_opt(1, 2, 3).expect("valid time")),
        CellValue::Formula("1+1".to_owned()),
        CellValue::Hyperlink {
            url: "https://www.rust-lang.org".to_owned(),
            text: "Rust".to_owned(),
        },
        CellValue::Comment {
            value: Box::new(CellValue::String("value".to_owned())),
            text: "note".to_owned(),
        },
        CellValue::Comment {
            value: Box::new(CellValue::Empty),
            text: "note".to_owned(),
        },
        CellValue::Image(tiny_png()),
    ] {
        let metadata = Box::leak(Box::new(ExcelColumn::new(
            "value",
            "Value",
            Some(0),
            0,
            None,
        )));
        assert!(write_data_row(worksheet, invalid_row, &[(0, 0, &*metadata)], &[value]).is_err());
    }
    let metadata = Box::leak(Box::new(ExcelColumn::new(
        "image",
        "Image",
        Some(0),
        0,
        None,
    )));
    for bytes in [vec![1, 2, 3], vec![0; 8]] {
        assert!(
            write_data_row(
                worksheet,
                0,
                &[(0, 0, &*metadata)],
                &[CellValue::Image(bytes)]
            )
            .is_err()
        );
    }
    assert!(
        write_data_row(
            worksheet,
            0,
            &[(0, 0, &*metadata)],
            &[CellValue::Comment {
                value: Box::new(CellValue::String("value".to_owned())),
                text: "x".repeat(32_768),
            }]
        )
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("bad-sheet.xlsx"),
            &WriteOptions {
                sheet_name: "bad/name".to_owned(),
                ..WriteOptions::default()
            },
            Vec::new()
        )
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("bad-merge.xlsx"),
            &WriteOptions {
                merge_ranges: vec![MergeRange::new(0, 0, 0, 0)],
                ..WriteOptions::default()
            },
            Vec::new()
        )
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("bad-width.xlsx"),
            &WriteOptions {
                column_widths: vec![(u16::MAX, 20)],
                ..WriteOptions::default()
            },
            Vec::new()
        )
        .is_err()
    );
    assert!(
        apply_loop_merges(
            worksheet,
            u32::MAX,
            0,
            &[MirroredLoopMergeStrategy::new(2, 1, 0)?]
        )
        .is_err()
    );
    assert!(
        apply_loop_merges(
            worksheet,
            0,
            0,
            &[MirroredLoopMergeStrategy::new(1, 2, u16::MAX)?]
        )
        .is_err()
    );
    assert!(
        apply_loop_merges(
            worksheet,
            0,
            0,
            &[MirroredLoopMergeStrategy::new(1, 2, 16_383)?]
        )
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("bad-loop-merge.xlsx"),
            &WriteOptions {
                loop_merges: vec![MirroredLoopMergeStrategy::new(1, 2, 16_383)?],
                ..WriteOptions::default()
            },
            vec![every_cell()]
        )
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(directory.path(), &WriteOptions::default(), Vec::new()).is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("missing").join("encrypted.xlsx"),
            &WriteOptions {
                password: Some("123456".to_owned()),
                ..WriteOptions::default()
            },
            Vec::new(),
        )
        .is_err()
    );
    let mut invalid_encrypted = Workbook::new();
    invalid_encrypted
        .add_worksheet()
        .set_name("Duplicate")
        .map_err(test_error)?;
    invalid_encrypted
        .add_worksheet()
        .set_name("Duplicate")
        .map_err(test_error)?;
    assert!(
        save_workbook(
            &mut invalid_encrypted,
            &directory.path().join("invalid-encrypted.xlsx"),
            Some("123456"),
        )
        .is_err()
    );
    let mut create_failure = Workbook::new();
    create_failure.add_worksheet();
    let mut create_output = LimitedCursor::new(0);
    assert!(
        save_encrypted_workbook_to(&mut create_failure, "123456", &mut create_output,).is_err()
    );
    let mut finalize_failure = Workbook::new();
    finalize_failure.add_worksheet();
    let mut finalize_output = LimitedCursor::new(4_096);
    assert!(
        save_encrypted_workbook_to(&mut finalize_failure, "123456", &mut finalize_output,).is_err()
    );
    let mut successful = Workbook::new();
    successful.add_worksheet();
    let mut successful_output = LimitedCursor::new(u64::MAX);
    save_encrypted_workbook_to(&mut successful, "123456", &mut successful_output)?;
    Ok(())
}

