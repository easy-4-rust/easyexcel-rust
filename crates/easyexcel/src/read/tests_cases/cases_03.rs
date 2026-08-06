#[test]
#[allow(clippy::too_many_lines)]
fn csv_read_uses_typed_lifecycle_single_sheet_selection_and_flexible_rows() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("fixture.csv");
    fs::write(&path, b"\xEF\xBB\xBFValue,Extra\r\none,1\r\ntwo\r\n")?;
    let mut probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_csv::<TestRow, _>(&path, &options(), &mut probe)?;
    assert_eq!(
        probe.rows,
        vec![TestRow("one".to_owned()), TestRow("two".to_owned())]
    );
    assert_eq!(probe.heads[0].get("Value"), Some(&0));
    assert_eq!(probe.after, vec![("Sheet1".to_owned(), 0, 2)]);

    assert_eq!(csv_sheet_name(&SheetSelector::First)?, "Sheet1");
    assert_eq!(csv_sheet_name(&SheetSelector::Index(0))?, "Sheet1");
    assert_eq!(csv_sheet_name(&SheetSelector::All)?, "Sheet1");
    assert_eq!(
        csv_sheet_name(&SheetSelector::Name("Custom".to_owned()))?,
        "Custom"
    );
    assert!(csv_sheet_name(&SheetSelector::Index(1)).is_err());
    assert_eq!(csv_row_index(0)?, 0);
    if usize::BITS > 32 {
        assert!(csv_row_index(usize::try_from(u64::from(u32::MAX) + 1).unwrap()).is_err());
    }

    let malformed_utf8 = directory.path().join("malformed-utf8.csv");
    fs::write(&malformed_utf8, [0xff])?;
    let mut replacement_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_csv::<TestRow, _>(
        &malformed_utf8,
        &ReadOptions {
            head_row_number: 0,
            ..options()
        },
        &mut replacement_probe,
    )?;
    assert_eq!(replacement_probe.rows, vec![TestRow("�".to_owned())]);
    assert!(
        read_csv::<TestRow, _>(
            &path,
            &ReadOptions {
                sheet: SheetSelector::Index(1),
                ..options()
            },
            &mut probe
        )
        .is_err()
    );
    let mut failing_head = Probe {
        continue_reading: true,
        fail_head: true,
        ..Probe::default()
    };
    assert!(read_csv::<TestRow, _>(&path, &options(), &mut failing_head).is_err());
    let record = vec!["value".to_owned()];
    let mut record_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_csv_records::<TestRow, _>(
        &mut [Ok::<_, easyexcel_io::Error>(record.clone())].into_iter(),
        0,
        "Sheet1",
        &ReadOptions {
            head_row_number: 0,
            ..options()
        },
        &mut record_probe,
    )?;
    assert_eq!(record_probe.rows, vec![TestRow("value".to_owned())]);
    read_csv_records::<TestRow, _>(
        &mut [
            Ok::<_, easyexcel_io::Error>(record.clone()),
            Ok(record.clone()),
        ]
        .into_iter(),
        0,
        "Sheet1",
        &ReadOptions {
            head_row_number: 0,
            ..options()
        },
        &mut record_probe,
    )?;
    assert_eq!(record_probe.rows.len(), 3);
    let mut stopped = Probe::default();
    read_csv_records::<TestRow, _>(
        &mut [
            Ok::<_, easyexcel_io::Error>(record.clone()),
            Ok(record.clone()),
        ]
        .into_iter(),
        0,
        "Sheet1",
        &ReadOptions {
            head_row_number: 0,
            ..options()
        },
        &mut stopped,
    )?;
    assert_eq!(stopped.rows, vec![TestRow("value".to_owned())]);
    assert!(stopped.after.is_empty());
    assert!(
        read_csv_records::<TestRow, _>(
            &mut [Err(easyexcel_io::Error::Csv(
                "invalid UTF-8 record".to_owned(),
            ))]
            .into_iter(),
            0,
            "Sheet1",
            &ReadOptions {
                head_row_number: 0,
                ..options()
            },
            &mut record_probe,
        )
        .is_err()
    );
    if usize::BITS > 32 {
        assert!(
            read_csv_records::<TestRow, _>(
                &mut [Ok::<_, easyexcel_io::Error>(record.clone())].into_iter(),
                usize::try_from(u64::from(u32::MAX) + 1).unwrap(),
                "Sheet1",
                &ReadOptions {
                    head_row_number: 0,
                    ..options()
                },
                &mut probe
            )
            .is_err()
        );
    }
    assert!(
        read_csv_records::<TestRow, _>(
            &mut [Ok::<_, easyexcel_io::Error>(record.clone()), Ok(record)].into_iter(),
            usize::MAX,
            "Sheet1",
            &ReadOptions {
                head_row_number: 0,
                ..options()
            },
            &mut probe
        )
        .is_err()
    );
    assert!(
        read_csv::<TestRow, _>(
            &directory.path().join("missing.csv"),
            &options(),
            &mut probe
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn csv_read_decodes_java_charset_names_and_strips_matching_bom() -> Result<()> {
    let directory = tempdir()?;
    for (name, encoding, bom) in [
        ("utf-8", encoding_rs::UTF_8, b"\xEF\xBB\xBF".as_slice()),
        ("GBK", encoding_rs::GBK, b"".as_slice()),
        ("UTF-16BE", encoding_rs::UTF_16BE, b"\xFE\xFF".as_slice()),
        ("UTF-16LE", encoding_rs::UTF_16LE, b"\xFF\xFE".as_slice()),
    ] {
        let path = directory
            .path()
            .join(format!("{}.csv", name.to_lowercase()));
        let mut bytes = bom.to_vec();
        bytes.extend_from_slice(&encode_csv_fixture(encoding, "Value\r\n姓名\r\n"));
        fs::write(&path, bytes)?;

        let mut probe = Probe {
            continue_reading: true,
            ..Probe::default()
        };
        read_csv::<TestRow, _>(
            &path,
            &ReadOptions {
                charset: CsvCharset::new(name),
                ..options()
            },
            &mut probe,
        )?;
        assert_eq!(
            probe.rows,
            vec![TestRow("姓名".to_owned())],
            "charset {name}"
        );
    }

    let mut probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    let error = read_csv::<TestRow, _>(
        &directory.path().join("utf-8.csv"),
        &ReadOptions {
            charset: CsvCharset::new("not-a-charset"),
            ..options()
        },
        &mut probe,
    )
    .expect_err("unknown charset must be rejected");
    assert!(matches!(error, ExcelError::Unsupported(_)));
    Ok(())
}

#[test]
fn reads_java_easyexcel_official_csv_bom_fixtures() -> Result<()> {
    let directory = tempdir()?;
    for (name, fixture) in [
        (
            "no-bom.csv",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-bom-no-bom.csv.b64"
            )),
        ),
        (
            "office-bom.csv",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-bom-office-bom.csv.b64"
            )),
        ),
    ] {
        let path = directory.path().join(name);
        fs::write(
            &path,
            base64::engine::general_purpose::STANDARD
                .decode(fixture.trim())
                .map_err(test_error)?,
        )?;
        let mut probe = Probe {
            continue_reading: true,
            ..Probe::default()
        };
        read_csv::<TestRow, _>(&path, &options(), &mut probe)?;
        assert_eq!(probe.rows.len(), 10);
        assert_eq!(probe.rows[0], TestRow("姓名0".to_owned()));
        assert_eq!(probe.rows[9], TestRow("姓名9".to_owned()));
    }
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn row_processing_handles_headers_skips_data_and_listener_failures() -> Result<()> {
    let mut headers = Arc::new(HashMap::new());
    let mut probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    process_row::<TestRow>(
        0,
        "First",
        0,
        vec![CellValue::String(" Value ".to_owned()), CellValue::Empty],
        &options(),
        &mut headers,
        &mut probe,
    )?;
    assert_eq!(probe.heads[0].get("Value"), Some(&0));

    process_row::<TestRow>(
        0,
        "First",
        1,
        vec![CellValue::String("one".to_owned())],
        &options(),
        &mut headers,
        &mut probe,
    )?;
    assert_eq!(probe.rows, vec![TestRow("one".to_owned())]);

    process_row::<TestRow>(
        0,
        "First",
        2,
        vec![CellValue::Empty],
        &options(),
        &mut headers,
        &mut probe,
    )?;
    assert_eq!(probe.rows.len(), 1);

    let two_header_rows = ReadOptions {
        head_row_number: 2,
        ..options()
    };
    assert_eq!(
        process_row::<TestRow>(
            0,
            "First",
            0,
            vec![CellValue::String("ignored".to_owned())],
            &two_header_rows,
            &mut headers,
            &mut probe,
        )?,
        ReadFlow::Continue
    );
    assert_eq!(probe.heads.len(), 2);
    process_row::<TestRow>(
        0,
        "First",
        1,
        vec![CellValue::String("Final".to_owned())],
        &two_header_rows,
        &mut headers,
        &mut probe,
    )?;
    assert_eq!(probe.heads.len(), 3);
    assert_eq!(headers.get("Final"), Some(&0));
    assert_eq!(probe.rows.len(), 1);

    probe.continue_reading = false;
    let include_empty = ReadOptions {
        ignore_empty_row: false,
        ..options()
    };
    assert_eq!(
        process_row::<TestRow>(
            0,
            "First",
            3,
            vec![CellValue::Empty],
            &include_empty,
            &mut headers,
            &mut probe,
        )?,
        ReadFlow::Stop
    );
    assert_eq!(probe.rows.len(), 2);

    let mut failing_head = Probe {
        continue_reading: true,
        fail_head: true,
        ..Probe::default()
    };
    assert!(
        process_row::<TestRow>(
            0,
            "First",
            0,
            vec![CellValue::String("Value".to_owned())],
            &options(),
            &mut headers,
            &mut failing_head
        )
        .is_err()
    );
    assert_eq!(failing_head.errors, 1);

    let no_head = ReadOptions {
        head_row_number: 0,
        ..options()
    };
    let mut tolerated_invoke = Probe {
        continue_reading: true,
        fail_invoke: true,
        error_action: Some(ErrorAction::Continue),
        ..Probe::default()
    };
    assert_eq!(
        process_row::<TestRow>(
            0,
            "First",
            0,
            vec![CellValue::String("value".to_owned())],
            &no_head,
            &mut headers,
            &mut tolerated_invoke,
        )?,
        ReadFlow::Continue
    );
    assert_eq!(tolerated_invoke.errors, 1);

    let mut trimming_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    process_row::<TestRow>(
        0,
        "First",
        0,
        vec![CellValue::String("  trimmed  ".to_owned())],
        &no_head,
        &mut headers,
        &mut trimming_probe,
    )?;
    assert_eq!(trimming_probe.rows, vec![TestRow("trimmed".to_owned())]);
    process_row::<TestRow>(
        0,
        "First",
        1,
        vec![CellValue::String("   ".to_owned())],
        &no_head,
        &mut headers,
        &mut trimming_probe,
    )?;
    assert_eq!(trimming_probe.rows.len(), 1);

    let mut untrimmed_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    process_row::<TestRow>(
        0,
        "First",
        0,
        vec![CellValue::String("  preserved  ".to_owned())],
        &ReadOptions {
            head_row_number: 0,
            auto_trim: false,
            ..options()
        },
        &mut headers,
        &mut untrimmed_probe,
    )?;
    assert_eq!(
        untrimmed_probe.rows,
        vec![TestRow("  preserved  ".to_owned())]
    );
    Ok(())
}

#[test]
fn conversion_error_actions_continue_skip_or_stop() -> Result<()> {
    let mut headers = Arc::new(HashMap::new());
    let read_options = ReadOptions {
        head_row_number: 0,
        ignore_empty_row: false,
        ..options()
    };
    for action in [ErrorAction::Continue, ErrorAction::SkipRow] {
        let mut listener = ErrorProbe { action, errors: 0 };
        process_row::<TestRow>(
            0,
            "First",
            0,
            vec![CellValue::String("conversion-error".to_owned())],
            &read_options,
            &mut headers,
            &mut listener,
        )?;
        assert_eq!(listener.errors, 1);
    }
    let mut listener = ErrorProbe {
        action: ErrorAction::Stop,
        errors: 0,
    };
    assert!(
        process_row::<TestRow>(
            0,
            "First",
            0,
            vec![CellValue::String("conversion-error".to_owned())],
            &read_options,
            &mut headers,
            &mut listener
        )
        .is_err()
    );
    assert_eq!(listener.errors, 1);
    Ok(())
}

