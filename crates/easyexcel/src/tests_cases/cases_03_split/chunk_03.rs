#[test]
fn facade_builds_stateful_gbk_csv_and_appends_without_repeating_head() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("stateful.csv");
    let sheet = EasyExcel::writer_sheet::<Value>("Values");
    let mut writer = EasyExcel::write::<Value>(&path)
        .charset("GBK")
        .with_bom(false)
        .build();
    writer
        .write(vec![Value("第一批".to_owned())], &sheet)?
        .write(vec![Value("第二批".to_owned())], &sheet)?;
    writer.finish()?;
    writer.finish()?;
    let mut empty_writer = EasyExcel::write::<Value>(directory.path().join("empty.csv")).build();
    empty_writer.finish()?;

    assert_eq!(
        EasyExcel::read_sync::<Value>(&path)
            .charset("gbk")
            .do_read_sync()?,
        vec![Value("第一批".to_owned()), Value("第二批".to_owned())]
    );
    Ok(())
}

#[test]
fn facade_csv_stream_writer_propagates_validation_and_io_failures() {
    let mut stream_options = WriteOptions {
        with_bom: false,
        ..WriteOptions::default()
    };
    assert!(
        write_csv_to_writer::<Value, _, _>(
            Path::new("stream.csv"),
            Cursor::new(Vec::new()),
            &stream_options,
            [Value("streamed".to_owned())],
            &mut [],
        )
        .is_ok()
    );
    assert!(matches!(
        write_csv_to_writer::<Value, _, _>(
            Path::new("stream.csv"),
            Cursor::new(Vec::new().into_boxed_slice()),
            &stream_options,
            [Value("output failure".to_owned())],
            &mut [],
        ),
        Err(ExcelError::Io(_) | ExcelError::Format(_))
    ));
    stream_options.charset = CsvCharset::new("not-a-real-charset");
    assert!(matches!(
        write_csv_to_writer::<Value, _, _>(
            Path::new("stream.csv"),
            Cursor::new(Vec::new()),
            &stream_options,
            [Value("ignored".to_owned())],
            &mut [],
        ),
        Err(ExcelError::Unsupported(_))
    ));
    assert!(
        write_csv_to_writer::<Value, _, _>(
            Path::new("stream.csv"),
            FacadeProbeWrite {
                fail_write: true,
                ..FacadeProbeWrite::default()
            },
            &WriteOptions::default(),
            [Value("bom failure".to_owned())],
            &mut [],
        )
        .is_err()
    );
    for fail_flush_at in [0, 1] {
        assert!(matches!(
            write_csv_to_writer::<Value, _, _>(
                Path::new("stream.csv"),
                FacadeProbeWrite {
                    fail_flush_at: Some(fail_flush_at),
                    ..FacadeProbeWrite::default()
                },
                &WriteOptions {
                    with_bom: false,
                    ..WriteOptions::default()
                },
                [Value("flush failure".to_owned())],
                &mut [],
            ),
            Err(ExcelError::Io(_) | ExcelError::Format(_))
        ));
    }

    let mut incomplete =
        CsvEncodingWriter::with_charset(FacadeProbeWrite::default(), &CsvCharset::new("UTF-8"))
            .expect("UTF-8 transcoder");
    assert!(matches!(
        CsvEncodingWriter::with_charset(
            FacadeProbeWrite::default(),
            &CsvCharset::new("not-a-real-charset"),
        ),
        Err(ExcelError::Unsupported(_))
    ));
    incomplete.write_all(&[0xE2]).expect("partial UTF-8 chunk");
    assert_eq!(
        incomplete
            .finish()
            .expect_err("incomplete UTF-8 fails")
            .kind(),
        io::ErrorKind::InvalidData
    );

    let fail = Arc::new(AtomicBool::new(false));
    let mut finalizing = CsvEncodingWriter::with_charset(
        ToggleFacadeWrite {
            fail: Arc::clone(&fail),
        },
        &CsvCharset::new("ISO-2022-JP"),
    )
    .expect("ISO-2022-JP transcoder");
    finalizing
        .write_all("日本".as_bytes())
        .expect("initial encoded bytes");
    fail.store(true, Ordering::SeqCst);
    assert!(finalizing.finish().is_err());
}

#[test]
#[allow(clippy::too_many_lines)]
fn facade_borrowed_xlsx_stream_is_real_and_remains_caller_owned() -> Result<()> {
    let mut output = FacadeProbeWrite::default();
    EasyExcel::write::<Value>("response.xlsx")
        .sheet("Values")
        .to_writer(&mut output)
        .do_write([Value("streamed".to_owned())])?;
    assert!(output.bytes.starts_with(b"PK"));
    output.write_all(b"caller-still-owns-stream")?;
    assert!(output.bytes.ends_with(b"caller-still-owns-stream"));

    let mut encrypted = FacadeProbeWrite::default();
    EasyExcel::write::<Value>("response.xlsx")
        .password("123456")
        .to_writer(&mut encrypted)
        .do_write([Value("secret".to_owned())])?;
    assert!(encrypted.bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]));

    let mut csv = FacadeProbeWrite::default();
    EasyExcel::write::<Value>("response.csv")
        .with_bom(false)
        .to_writer(&mut csv)
        .do_write([Value("csv-stream".to_owned())])?;
    assert_eq!(csv.bytes, b"Value\ncsv-stream\n");

    for charset in ["UTF-16LE", "UTF-16BE"] {
        let mut encoded = FacadeProbeWrite::default();
        EasyExcel::write::<Value>("response.csv")
            .charset(charset)
            .to_writer(&mut encoded)
            .do_write([Value("encoded".to_owned())])?;
        assert!(!encoded.bytes.is_empty());
    }

    let mut invalid_csv = FacadeProbeWrite::default();
    assert!(matches!(
        EasyExcel::write::<Value>("response.csv")
            .charset("not-a-charset")
            .to_writer(&mut invalid_csv)
            .do_write([Value("invalid".to_owned())]),
        Err(ExcelError::Unsupported(_))
    ));

    for mut output in [
        FacadeProbeWrite {
            fail_write: true,
            ..FacadeProbeWrite::default()
        },
        FacadeProbeWrite {
            fail_flush: true,
            ..FacadeProbeWrite::default()
        },
    ] {
        assert!(
            EasyExcel::write::<Value>("response.csv")
                .with_bom(false)
                .to_writer(&mut output)
                .do_write([Value("failure".to_owned())])
                .is_err()
        );
    }
    for mut output in [
        FacadeProbeWrite {
            fail_write: true,
            ..FacadeProbeWrite::default()
        },
        FacadeProbeWrite {
            fail_flush: true,
            ..FacadeProbeWrite::default()
        },
    ] {
        assert!(
            EasyExcel::write::<Value>("response.xlsx")
                .to_writer(&mut output)
                .do_write([Value("failure".to_owned())])
                .is_err()
        );
    }
    let mut encrypted_failure = FacadeProbeWrite {
        fail_write: true,
        ..FacadeProbeWrite::default()
    };
    assert!(
        EasyExcel::write::<Value>("response.xlsx")
            .password("123456")
            .to_writer(&mut encrypted_failure)
            .do_write([Value("failure".to_owned())])
            .is_err()
    );
    for (before_workbook, before_cell) in [(true, false), (false, true)] {
        let mut output = FacadeProbeWrite::default();
        assert!(
            EasyExcel::write::<Value>("response.xlsx")
                .register_write_handler(FailingFacadeWriteHandler {
                    before_workbook,
                    before_cell,
                })
                .to_writer(&mut output)
                .do_write([Value("failure".to_owned())])
                .is_err()
        );
    }

    let mut xls_stream = FacadeProbeWrite::default();
    EasyExcel::write::<Value>("response.xls")
        .to_writer(&mut xls_stream)
        .do_write([Value("biff8".to_owned())])?;
    // OLE/CFB compound-document signature (D0 CF 11 E0).
    assert!(xls_stream.bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]));
    // Phase 5.3: XLS password is now supported via BIFF8 RC4
    assert!(
        EasyExcel::write::<Value>("response.xls")
            .password("123456")
            .to_writer(&mut FacadeProbeWrite::default())
            .do_write([Value("encrypted".to_owned())])
            .is_ok()
    );
    Ok(())
}

