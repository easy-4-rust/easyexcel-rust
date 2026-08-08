#[test]
#[allow(clippy::too_many_lines)]
fn stateful_writer_supports_multiple_sheets_and_idempotent_finish() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("multi.xlsx");
    let events = Rc::new(RefCell::new(Vec::new()));
    let handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(RecordingHandler {
        order: 5,
        events: Rc::clone(&events),
    })];
    let first = WriteSheet::<EveryCell>::new("Users")
        .sheet_index(7)
        .freeze_head(true)
        .merge_cells(MergeRange::new(0, 0, 0, 1))
        .auto_width(true)
        .column_width(0, 20)
        .head_style(CellStyle::new().italic(true))
        .content_style(CellStyle::new().bold(true))
        .content_styles([CellStyle::new().wrap_text(true)])
        .loop_merge(MirroredLoopMergeStrategy::new(2, 1, 0)?);
    let second = WriteSheet::<EveryCell>::new("Archive")
        .sheet_index(9)
        .need_head(false);
    assert_eq!(first.options().sheet_name, "Users");
    assert_eq!(first.options().sheet_index, Some(7));
    assert!(first.options().freeze_head);
    assert!(first.options().auto_width);
    assert_eq!(first.options().column_widths, vec![(0, 20)]);
    assert!(first.options().head_style.italic);
    assert_eq!(first.options().content_styles.len(), 1);
    assert!(first.options().content_styles[0].wrap_text);
    assert_eq!(first.options().loop_merges.len(), 1);
    assert!(!second.options().need_head);
    assert!(!second.options().constant_memory);

    let mut writer = ExcelWriter::with_handlers(&path, handlers);
    assert!(!writer.is_finished());
    writer
        .write(vec![every_cell(), every_cell()], &first)?
        .write(vec![every_cell(), every_cell()], &first)?
        .write(vec![every_cell(), every_cell()], &second)?;
    writer.write(Vec::new(), &WriteSheet::<EveryCell>::new_index(7))?;
    writer.write(Vec::new(), &WriteSheet::<EveryCell>::new_index(9))?;
    writer.finish()?;
    assert!(writer.is_finished());
    writer.finish()?;
    let Err(error) = writer.write(vec![every_cell()], &first) else {
        panic!("finished writer must reject data");
    };
    assert!(error.to_string().contains("already finished"));

    let actual = events.borrow();
    assert_eq!(
        actual
            .iter()
            .filter(|event| event.contains("before_workbook"))
            .count(),
        1
    );
    assert_eq!(
        actual
            .iter()
            .filter(|event| event.contains("before_sheet"))
            .count(),
        2
    );
    assert_eq!(
        actual
            .iter()
            .filter(|event| event.contains("after_sheet"))
            .count(),
        2
    );
    assert_eq!(
        actual
            .iter()
            .filter(|event| event.contains("after_workbook"))
            .count(),
        1
    );
    drop(actual);

    let mut workbook: Xlsx<_> = open_workbook(path).map_err(test_error)?;
    assert_eq!(workbook.sheet_names(), vec!["Users", "Archive"]);
    assert_eq!(
        workbook
            .merge_cells_by_sheet_name("Users")
            .map_err(test_error)?,
        vec![
            Dimensions::new((0, 0), (0, 1)),
            Dimensions::new((1, 0), (2, 0)),
            Dimensions::new((3, 0), (4, 0)),
        ]
    );
    let users = workbook.worksheet_range("Users").map_err(test_error)?;
    assert_eq!(
        users.get_value((1, 1)),
        Some(&Data::String("text".to_owned()))
    );
    assert_eq!(
        users.get_value((4, 1)),
        Some(&Data::String("text".to_owned()))
    );
    let archive = workbook.worksheet_range("Archive").map_err(test_error)?;
    assert_eq!(
        archive.get_value((0, 1)),
        Some(&Data::String("text".to_owned()))
    );
    assert_eq!(
        archive.get_value((1, 1)),
        Some(&Data::String("text".to_owned()))
    );
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn stateful_writer_propagates_start_sheet_and_finish_failures() -> Result<()> {
    let directory = tempdir()?;
    let sheet = WriteSheet::<EveryCell>::new("Values");

    let handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(FailingHandler(FailureStage::BeforeWorkbook))];
    let mut rejected = ExcelWriter::with_handlers(directory.path().join("rejected.xlsx"), handlers);
    assert!(rejected.write(Vec::new(), &sheet).is_err());

    let handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(FailingHandler(FailureStage::BeforeWorkbook))];
    let mut rejected_finish =
        ExcelWriter::with_handlers(directory.path().join("rejected-finish.xlsx"), handlers);
    assert!(rejected_finish.finish().is_err());

    let handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(FailingHandler(FailureStage::AfterWorkbook))];
    let mut rejected_after =
        ExcelWriter::with_handlers(directory.path().join("rejected-after.xlsx"), handlers);
    rejected_after.write(Vec::new(), &sheet)?;
    assert!(rejected_after.finish().is_err());

    let mut schema_change = ExcelWriter::new(directory.path().join("schema-change.xlsx"));
    schema_change.write(Vec::new(), &sheet)?;
    USE_WIDE_SCHEMA.with(|wide| wide.set(true));
    let schema_change_result = schema_change.write(Vec::new(), &sheet);
    USE_WIDE_SCHEMA.with(|wide| wide.set(false));
    assert!(matches!(schema_change_result, Err(ExcelError::Format(_))));

    let invalid = WriteSheet::<EveryCell>::new("bad/name");
    let mut invalid_sheet = ExcelWriter::new(directory.path().join("invalid.xlsx"));
    assert!(invalid_sheet.write(Vec::new(), &invalid).is_err());

    let mut invalid_output = ExcelWriter::new(directory.path());
    assert!(invalid_output.finish().is_err());

    let mut csv = ExcelWriter::with_handlers_and_options(
        directory.path().join("invalid-charset.CSV"),
        Vec::new(),
        WriteOptions {
            charset: CsvCharset::new("not-a-charset"),
            ..WriteOptions::default()
        },
    );
    assert!(matches!(
        csv.write(Vec::new(), &sheet),
        Err(ExcelError::Unsupported(_))
    ));
    let mut protected_csv = ExcelWriter::with_handlers_and_password(
        directory.path().join("protected.csv"),
        Vec::new(),
        Some("secret".to_owned()),
    );
    assert!(matches!(
        protected_csv.finish(),
        Err(ExcelError::Unsupported(_))
    ));
    assert!(matches!(
        validate_csv_options(&WriteOptions {
            password: Some("secret".to_owned()),
            ..WriteOptions::default()
        }),
        Err(ExcelError::Unsupported(_))
    ));
    let mut xls = ExcelWriter::new(directory.path().join("stateful.XLS"));
    // EveryCell includes Image values; BIFF8 rejects images — write headers only.
    xls.write(Vec::<EveryCell>::new(), &sheet)?;
    xls.finish()?;
    let xls_path = directory.path().join("stateful.XLS");
    assert!(xls_path.exists());
    let mut xls_book: Xls<_> = open_workbook(&xls_path).map_err(test_error)?;
    let range = xls_book
        .worksheet_range(sheet.options().sheet_name.as_str())
        .map_err(test_error)?;
    assert!(!range.is_empty());

    let mut failed_xlsx_append = ExcelWriter::new(directory.path().join("failed-xlsx-append.xlsx"));
    failed_xlsx_append.write(vec![every_cell()], &sheet)?;
    let mut broken = every_cell();
    broken.fail = true;
    assert!(matches!(
        failed_xlsx_append.write(vec![broken.clone()], &sheet),
        Err(ExcelError::Format(_))
    ));

    let mut missing_cached_sheet =
        ExcelWriter::new(directory.path().join("missing-cached-sheet.xlsx"));
    missing_cached_sheet.write(Vec::new(), &sheet)?;
    missing_cached_sheet.workbook = Workbook::new();
    assert!(missing_cached_sheet.write(Vec::new(), &sheet).is_err());

    let mut no_autofit = ExcelWriter::new(directory.path().join("no-autofit.xlsx"));
    no_autofit
        .write(Vec::new(), &sheet)?
        .write(Vec::new(), &sheet)?;

    let mut failed_csv_append = ExcelWriter::new(directory.path().join("failed-csv-append.csv"));
    failed_csv_append.write(vec![every_cell()], &sheet)?;
    assert!(matches!(
        failed_csv_append.write(vec![broken], &sheet),
        Err(ExcelError::Format(_))
    ));

    let missing_parent = directory.path().join("missing").join("stateful.csv");
    let mut missing_csv_output = ExcelWriter::new(missing_parent);
    assert!(missing_csv_output.finish().is_err());

    for stage in [FailureStage::BeforeSheet, FailureStage::AfterSheet] {
        let handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(FailingHandler(stage))];
        let mut failed_sheet = ExcelWriter::with_handlers(
            directory
                .path()
                .join(format!("stateful-csv-handler-{}.csv", stage as u8)),
            handlers,
        );
        assert!(failed_sheet.write(Vec::new(), &sheet).is_err());
    }

    let mut failed_csv_finish = ExcelWriter::new(directory.path().join("failed-csv-finish.csv"));
    failed_csv_finish.start()?;
    failed_csv_finish.csv_writer = Some(
        easyexcel_csv::CsvRecordWriter::new(
            Box::new(FaultyWrite::flushing()),
            &CsvCharset::default(),
            false,
        )
        .map_err(ExcelError::from)?,
    );
    assert!(failed_csv_finish.finish().is_err());

    Ok(())
}

#[test]
fn stateful_csv_appends_batches_with_one_head_and_one_sheet_lifecycle() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("stateful.csv");
    let events = Rc::new(RefCell::new(Vec::new()));
    let handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(RecordingHandler {
        order: 1,
        events: Rc::clone(&events),
    })];
    let options = WriteOptions {
        charset: CsvCharset::new("GBK"),
        with_bom: false,
        ..WriteOptions::default()
    };
    let sheet = WriteSheet::<EveryCell>::new("Values").sheet_index(3);
    let indexed_alias = WriteSheet::<EveryCell>::new_index(3);
    let mut writer = ExcelWriter::with_handlers_and_options(&path, handlers, options);
    writer
        .write(vec![every_cell()], &sheet)?
        .write(vec![every_cell()], &indexed_alias)?;
    USE_WIDE_SCHEMA.with(|wide| wide.set(true));
    let schema_change_result = writer.write(Vec::new(), &sheet);
    USE_WIDE_SCHEMA.with(|wide| wide.set(false));
    assert!(matches!(schema_change_result, Err(ExcelError::Format(_))));
    let other = WriteSheet::<EveryCell>::new("Other");
    assert!(matches!(
        writer.write(Vec::new(), &other),
        Err(ExcelError::Unsupported(_))
    ));
    writer.finish()?;
    writer.finish()?;

    let bytes = std::fs::read(path)?;
    assert!(!bytes.starts_with(b"\xEF\xBB\xBF"));
    let (decoded, actual, had_errors) = encoding_rs::GBK.decode(&bytes);
    assert_eq!(actual, encoding_rs::GBK);
    assert!(!had_errors);
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(decoded.as_bytes());
    let records = csv
        .records()
        .collect::<csv::Result<Vec<_>>>()
        .map_err(test_error)?;
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].get(1), Some("String"));
    assert_eq!(records[1].get(1), Some("text"));
    assert_eq!(records[2].get(1), Some("text"));

    let events = events.borrow();
    for event in [
        "before_workbook",
        "after_workbook",
        "before_sheet",
        "after_sheet",
    ] {
        assert_eq!(
            events.iter().filter(|value| value.contains(event)).count(),
            1,
            "event {event}"
        );
    }
    Ok(())
}
