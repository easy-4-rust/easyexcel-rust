#[test]
fn output_stream_exposes_real_ownership_close_and_poison_failures() {
    let mut stream = ExcelOutputStream::new(Vec::new());
    stream.write_all(b"bytes").expect("write open stream");
    stream.flush().expect("flush open stream");
    assert_eq!(stream.with_inner(Clone::clone), Some(b"bytes".to_vec()));

    let clone = stream.clone();
    let stream = stream.into_inner().expect_err("clone retains ownership");
    drop(clone);
    let Ok(inner) = stream.into_inner() else {
        panic!("unique stream must be recoverable");
    };
    assert_eq!(inner, b"bytes");

    let mut closed = ExcelOutputStream::new(Vec::new());
    closed.close().expect("close stream");
    closed.close().expect("repeat close");
    assert!(closed.is_closed());
    assert!(closed.write_all(b"rejected").is_err());
    assert!(closed.flush().is_err());
    assert!(closed.into_inner().is_err());

    let failed_close = ExcelOutputStream::new(FaultyWrite::flushing());
    assert!(failed_close.close().is_err());
    assert!(failed_close.is_closed());

    let mut poisoned = ExcelOutputStream::new(PanicWrite);
    let lock_holder = poisoned.clone();
    assert!(
        std::thread::spawn(move || {
            let mut lock_holder = lock_holder;
            let _ = lock_holder.write(b"poison");
        })
        .join()
        .is_err()
    );
    assert!(poisoned.is_closed());
    assert!(poisoned.with_inner(|_| ()).is_none());
    assert!(poisoned.write_all(b"rejected").is_err());
    assert!(poisoned.flush().is_err());
    assert!(poisoned.close().is_err());
    assert!(poisoned.into_inner().is_err());

    let mut capture = CapturedOutput::default();
    capture.write_all(b"capture").expect("capture bytes");
    capture.flush().expect("capture flush");
    assert_eq!(
        take_captured_output(&capture).expect("take capture"),
        b"capture"
    );
}

#[test]
fn stateful_output_stream_propagates_csv_commit_start_and_close_failures() -> Result<()> {
    let sheet = WriteSheet::<EveryCell>::new("Values");

    for (write_excel_on_exception, should_emit) in [(false, false), (true, true)] {
        let output = ExcelOutputStream::new(Vec::new());
        let inspect = output.clone();
        let mut writer = ExcelWriter::with_output_stream(
            "response.xlsx",
            output,
            Vec::new(),
            WriteOptions {
                auto_close_stream: false,
                write_excel_on_exception,
                ..WriteOptions::default()
            },
        );
        writer.write([every_cell()], &sheet)?;
        writer.finish_on_exception()?;
        let bytes = inspect
            .with_inner(Clone::clone)
            .expect("open output stream");
        assert_eq!(bytes.starts_with(b"PK"), should_emit);
    }

    let mut invalid_charset = ExcelWriter::with_output_stream(
        "response.csv",
        ExcelOutputStream::new(Vec::new()),
        Vec::new(),
        WriteOptions {
            charset: CsvCharset::new("not-a-charset"),
            ..WriteOptions::default()
        },
    );
    assert!(matches!(
        invalid_charset.finish(),
        Err(ExcelError::Unsupported(_))
    ));

    for output in [FaultyWrite::writing(0), FaultyWrite::flushing()] {
        let mut writer = ExcelWriter::with_output_stream(
            "response.csv",
            ExcelOutputStream::new(output),
            Vec::new(),
            WriteOptions {
                with_bom: false,
                auto_close_stream: false,
                ..WriteOptions::default()
            },
        );
        writer.write(Vec::new(), &sheet)?;
        assert!(writer.finish().is_err());
    }

    let mut failed_csv_finish = ExcelWriter::with_output_stream(
        "response.csv",
        ExcelOutputStream::new(Vec::new()),
        Vec::new(),
        WriteOptions {
            with_bom: false,
            auto_close_stream: false,
            ..WriteOptions::default()
        },
    );
    failed_csv_finish.write(Vec::new(), &sheet)?;
    failed_csv_finish.csv_writer = Some(create_csv_record_writer(
        Box::new(FaultyWrite::flushing()),
        &CsvCharset::default(),
        false,
    )?);
    assert!(failed_csv_finish.finish().is_err());

    let fail_close = Arc::new(AtomicBool::new(false));
    let handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(EnableFlushFailure(Arc::clone(&fail_close)))];
    let mut failed_close = ExcelWriter::with_output_stream(
        "response.xlsx",
        ExcelOutputStream::new(ToggleFlushFailure { fail: fail_close }),
        handlers,
        WriteOptions::default(),
    );
    failed_close.write(Vec::new(), &sheet)?;
    assert!(failed_close.finish().is_err());
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn xlsx_output_stream_writes_real_packages_and_propagates_io_failures() -> Result<()> {
    let logical_path = Path::new("http-response.xlsx");
    let mut output = StreamProbe::default();
    write_xlsx_to_writer::<EveryCell, _, _>(
        logical_path,
        &mut output,
        &WriteOptions::default(),
        vec![every_cell()],
        &mut [],
    )?;
    assert!(output.bytes.starts_with(b"PK"));
    let mut workbook: Xlsx<_> =
        Xlsx::new(Cursor::new(std::mem::take(&mut output.bytes))).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get_value((1, 1)),
        Some(&Data::String("text".to_owned()))
    );

    write_xlsx_to_writer::<EveryCell, _, _>(
        logical_path,
        &mut output,
        &WriteOptions {
            password: Some("123456".to_owned()),
            ..WriteOptions::default()
        },
        Vec::new(),
        &mut [],
    )?;
    assert!(output.bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]));
    output.bytes.clear();

    output.fail_write = true;
    assert!(
        write_xlsx_to_writer::<EveryCell, _, _>(
            logical_path,
            &mut output,
            &WriteOptions::default(),
            Vec::new(),
            &mut [],
        )
        .is_err()
    );
    output.fail_write = false;
    let mut before_handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(FailingHandler(FailureStage::BeforeWorkbook))];
    assert!(
        write_xlsx_to_writer::<EveryCell, _, _>(
            logical_path,
            &mut output,
            &WriteOptions::default(),
            Vec::new(),
            &mut before_handlers,
        )
        .is_err()
    );
    let mut broken = every_cell();
    broken.fail = true;
    assert!(
        write_xlsx_to_writer::<EveryCell, _, _>(
            logical_path,
            &mut output,
            &WriteOptions::default(),
            vec![broken],
            &mut [],
        )
        .is_err()
    );

    let mut duplicate = Workbook::new();
    duplicate
        .add_worksheet()
        .set_name("Duplicate")
        .map_err(test_error)?;
    duplicate
        .add_worksheet()
        .set_name("Duplicate")
        .map_err(test_error)?;
    assert!(save_workbook_to_writer(&mut duplicate, &mut output, Some("123456")).is_err());
    output.fail_write = true;
    assert!(
        write_xlsx_to_writer::<EveryCell, _, _>(
            logical_path,
            &mut output,
            &WriteOptions {
                password: Some("123456".to_owned()),
                ..WriteOptions::default()
            },
            Vec::new(),
            &mut [],
        )
        .is_err()
    );
    output.fail_write = false;
    output.fail_flush = true;
    assert!(
        write_xlsx_to_writer::<EveryCell, _, _>(
            logical_path,
            &mut output,
            &WriteOptions::default(),
            Vec::new(),
            &mut [],
        )
        .is_err()
    );
    Ok(())
}

