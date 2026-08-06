#[test]
fn csv_writer_emits_bom_all_cell_values_and_handler_lifecycle() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("values.csv");
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![
        Box::new(RecordingHandler {
            order: 10,
            events: Rc::clone(&events),
        }),
        Box::new(RecordingHandler {
            order: -1,
            events: Rc::clone(&events),
        }),
    ];
    write_csv_with_handlers::<EveryCell, _>(
        &path,
        &WriteOptions::default(),
        [every_cell()],
        &mut handlers,
    )?;
    let bytes = std::fs::read(&path)?;
    assert!(bytes.starts_with(b"\xEF\xBB\xBF"));
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(bytes.as_slice());
    let records = reader
        .records()
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(test_error)?;
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].get(0), Some(""));
    assert_eq!(records[0].get(1), Some("true"));
    assert_eq!(records[0].get(2), Some("header-error"));
    assert_eq!(records[1].get(1), Some("transformed"));
    assert_eq!(records[1].get(2), Some(""));
    assert_eq!(records[1].get(3), Some("true"));
    assert_eq!(records[1].get(13), Some(""));
    assert!(
        events
            .borrow()
            .iter()
            .any(|event| event == "10:after_workbook")
    );
    Ok(())
}

#[test]
fn csv_writer_encodes_java_charsets_and_configurable_bom() -> Result<()> {
    let directory = tempdir()?;
    let mut row = every_cell();
    row.cells[1] = CellValue::String("姓名".to_owned());

    for (name, encoding, expected_bom) in [
        ("utf-8", encoding_rs::UTF_8, b"\xEF\xBB\xBF".as_slice()),
        ("GBK", encoding_rs::GBK, b"".as_slice()),
        ("UTF-16BE", encoding_rs::UTF_16BE, b"\xFE\xFF".as_slice()),
        ("UTF-16LE", encoding_rs::UTF_16LE, b"\xFF\xFE".as_slice()),
    ] {
        let path = directory
            .path()
            .join(format!("{}.csv", name.to_lowercase()));
        write_csv_with_handlers::<EveryCell, _>(
            &path,
            &WriteOptions {
                charset: CsvCharset::new(name),
                ..WriteOptions::default()
            },
            [row.clone()],
            &mut [],
        )?;
        let bytes = std::fs::read(path)?;
        assert!(bytes.starts_with(expected_bom));
        let (decoded, actual_encoding, had_errors) = encoding.decode(&bytes);
        assert_eq!(actual_encoding, encoding);
        assert!(!had_errors);
        assert!(decoded.contains("姓名"));
    }

    let no_bom = directory.path().join("no-bom.csv");
    write_csv_with_handlers::<EveryCell, _>(
        &no_bom,
        &WriteOptions {
            with_bom: false,
            ..WriteOptions::default()
        },
        [row],
        &mut [],
    )?;
    assert!(!std::fs::read(no_bom)?.starts_with(b"\xEF\xBB\xBF"));

    let unsupported = directory.path().join("unsupported.csv");
    let error = write_csv_with_handlers::<EveryCell, _>(
        &unsupported,
        &WriteOptions {
            charset: CsvCharset::new("not-a-charset"),
            ..WriteOptions::default()
        },
        Vec::new(),
        &mut [],
    )
    .expect_err("unknown charset must be rejected");
    assert!(matches!(error, ExcelError::Unsupported(_)));
    assert!(!unsupported.exists());
    Ok(())
}

#[test]
fn csv_transcoding_writer_handles_chunk_boundaries_and_invalid_utf8() -> Result<()> {
    let mut split = CsvEncodingWriter::new(
        Box::new(Vec::<u8>::new()),
        CsvEncoding::Standard(encoding_rs::GBK),
    );
    assert_eq!(split.write(&[0xE5])?, 1);
    assert!(split.finish().is_err());

    let mut split_ok = CsvEncodingWriter::new(
        Box::new(Vec::<u8>::new()),
        CsvEncoding::Standard(encoding_rs::GBK),
    );
    assert_eq!(split_ok.write(&[0xE5])?, 1);
    assert_eq!(split_ok.write(&[0xA7, 0x93])?, 2);
    split_ok.finish()?;

    let mut invalid = CsvEncodingWriter::new(
        Box::new(Vec::<u8>::new()),
        CsvEncoding::Standard(encoding_rs::UTF_8),
    );
    assert!(invalid.write(&[0xFF]).is_err());

    let mut long = CsvEncodingWriter::new(Box::new(Vec::<u8>::new()), CsvEncoding::Utf16Be);
    let value = "姓名".repeat(5_000);
    long.write_all(value.as_bytes())?;
    long.finish()?;

    let mut standard_long = CsvEncodingWriter::new(
        Box::new(Vec::<u8>::new()),
        CsvEncoding::Standard(encoding_rs::GBK),
    );
    standard_long.write_all(value.as_bytes())?;
    standard_long.finish()?;

    let mut failing_utf16 =
        CsvEncodingWriter::new(Box::new(FaultyWrite::writing(0)), CsvEncoding::Utf16Le);
    assert!(failing_utf16.write_all(value.as_bytes()).is_err());

    let mut direct_utf16 = Vec::new();
    CsvEncodingWriter::encode_utf16(&mut direct_utf16, "姓名", u16::to_le_bytes)?;
    assert_eq!(direct_utf16, [0xD3, 0x59, 0x0D, 0x54]);

    let mut finish_failure = CsvEncodingWriter::new(
        Box::new(FaultyWrite::writing(1)),
        CsvEncoding::Standard(encoding_rs::ISO_2022_JP),
    );
    finish_failure.write_all("日本".as_bytes())?;
    assert!(finish_failure.finish().is_err());
    Ok(())
}

#[test]
fn csv_writer_supports_dynamic_heads_no_head_and_configuration_failures() -> Result<()> {
    let directory = tempdir()?;
    let mut dynamic = (0..EveryCell::schema().len())
        .map(|index| vec!["Group".to_owned(), format!("Column {index}")])
        .collect::<Vec<_>>();
    dynamic[0].pop();
    write_csv_with_handlers::<EveryCell, _>(
        &directory.path().join("dynamic.csv"),
        &WriteOptions {
            dynamic_head: Some(dynamic),
            ..WriteOptions::default()
        },
        Vec::new(),
        &mut [],
    )?;
    write_csv_with_handlers::<EveryCell, _>(
        &directory.path().join("no-head.csv"),
        &WriteOptions {
            need_head: false,
            ..WriteOptions::default()
        },
        [every_cell()],
        &mut [],
    )?;
    assert!(
        write_csv_with_handlers::<EveryCell, _>(
            &directory.path().join("bad-head.csv"),
            &WriteOptions {
                dynamic_head: Some(vec![vec!["Only one".to_owned()]]),
                ..WriteOptions::default()
            },
            Vec::new(),
            &mut []
        )
        .is_err()
    );
    assert!(
        write_csv_with_handlers::<EveryCell, _>(
            &directory.path().join("empty-head.csv"),
            &WriteOptions {
                dynamic_head: Some(vec![Vec::new(); EveryCell::schema().len()]),
                ..WriteOptions::default()
            },
            Vec::new(),
            &mut []
        )
        .is_err()
    );
    assert!(
        write_csv_with_handlers::<EveryCell, _>(
            &directory.path().join("conversion.csv"),
            &WriteOptions::default(),
            [EveryCell {
                cells: Vec::new(),
                fail: true
            }],
            &mut []
        )
        .is_err()
    );
    assert!(
        write_csv_with_handlers::<EveryCell, _>(
            directory.path(),
            &WriteOptions::default(),
            Vec::new(),
            &mut []
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn csv_writer_propagates_every_handler_failure() -> Result<()> {
    let directory = tempdir()?;
    for stage in [
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
    ] {
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(FailingHandler(stage))];
        assert!(
            write_csv_with_handlers::<EveryCell, _>(
                &directory
                    .path()
                    .join(format!("failure-{}.csv", stage as u8)),
                &WriteOptions::default(),
                [every_cell()],
                &mut handlers
            )
            .is_err()
        );
    }
    let mut dynamic_handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(FailingHandler(FailureStage::BeforeHeadRow))];
    assert!(
        write_csv_with_handlers::<EveryCell, _>(
            &directory.path().join("dynamic-handler-failure.csv"),
            &WriteOptions {
                dynamic_head: Some(vec![vec!["Head".to_owned()]; EveryCell::schema().len()]),
                ..WriteOptions::default()
            },
            Vec::new(),
            &mut dynamic_handlers
        )
        .is_err()
    );
    Ok(())
}

