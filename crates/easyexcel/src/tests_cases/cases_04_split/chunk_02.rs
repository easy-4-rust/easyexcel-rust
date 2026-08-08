#[test]
fn facade_propagates_read_sync_and_write_failures() {
    let missing = PathBuf::from("target/does-not-exist/easyexcel.xlsx");
    assert!(
        EasyExcel::read::<Value, _>(&missing, Listener::default())
            .do_read()
            .is_err()
    );
    assert!(
        EasyExcel::read_sync::<Value>(&missing)
            .do_read_sync()
            .is_err()
    );
    assert!(
        EasyExcel::read_sync::<Value>("target/does-not-exist/easyexcel.csv")
            .do_read_sync()
            .is_err()
    );
    assert!(
        EasyExcel::read::<Value, _>("target/does-not-exist/easyexcel.xls", Listener::default())
            .do_read()
            .is_err()
    );
    assert!(
        EasyExcel::read_sync::<Value>("target/does-not-exist/easyexcel.xls")
            .do_read_sync()
            .is_err()
    );
    assert!(
        EasyExcel::write::<Value>("target/does-not-exist/output.xlsx")
            .do_write(Vec::new())
            .is_err()
    );
    assert!(
        EasyExcel::write::<Value>("target/does-not-exist/output.csv")
            .do_write(Vec::new())
            .is_err()
    );
    assert!(
        EasyExcel::write::<Value>("target/does-not-exist/encrypted.xlsx")
            .password("123456")
            .do_write(Vec::new())
            .is_err()
    );

    let directory = tempdir().expect("temporary directory");
    // Minimal BIFF8 and CryptoAPI-encrypted .xls writes both succeed.
    let xls_empty = directory.path().join("empty.xls");
    EasyExcel::write::<Value>(&xls_empty)
        .do_write(Vec::<Value>::new())
        .expect("empty BIFF8 write");
    assert!(xls_empty.exists());
    let encrypted_xls = directory.path().join("encrypted.xls");
    EasyExcel::write::<Value>(&encrypted_xls)
        .password("123456")
        .do_write(Vec::new())
        .expect("CryptoAPI BIFF8 write");
    assert!(
        std::fs::read(&encrypted_xls)
            .expect("encrypted XLS bytes")
            .starts_with(&[0xD0, 0xCF, 0x11, 0xE0])
    );

    let date = NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date");
    for (index, value) in [
        CellValue::Empty,
        CellValue::String("text".to_owned()),
        CellValue::Error("#DIV/0!".to_owned()),
        CellValue::Bool(true),
        CellValue::Int(1),
        CellValue::Int(i64::MAX),
        CellValue::Float(1.25),
        CellValue::Date(date),
        CellValue::DateTime(date.and_hms_opt(12, 34, 56).expect("valid time")),
        CellValue::Formula("1+1".to_owned()),
        CellValue::Hyperlink {
            url: "https://www.rust-lang.org".to_owned(),
            text: "Rust".to_owned(),
        },
        CellValue::Comment {
            value: Box::new(CellValue::String("annotated".to_owned())),
            text: "cell note".to_owned(),
        },
        CellValue::Image(vec![1, 2, 3]),
        CellValue::Image(tiny_png()),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            EasyExcel::write::<WideCell>(directory.path().join(format!("wide-cell-{index}.xlsx")))
                .need_head(false)
                .do_write([WideCell(value)])
                .is_err()
        );
    }
    assert!(
        EasyExcel::write::<SingleCell>(directory.path().join("oversized-comment.xlsx"))
            .need_head(false)
            .do_write([SingleCell(CellValue::Comment {
                value: Box::new(CellValue::String("annotated".to_owned())),
                text: "x".repeat(32_768),
            })])
            .is_err()
    );
}

#[test]
fn collecting_listener_appends_rows() -> Result<()> {
    let mut listener = CollectListener(Vec::new());
    listener.invoke(
        Value("value".to_owned()),
        &AnalysisContext::new("Sheet1", 0, 1),
    )?;
    assert_eq!(listener.0, vec![Value("value".to_owned())]);
    Ok(())
}

#[test]
fn registered_converter_runs_in_sync_and_event_read_paths() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("registered-read.xlsx");
    EasyExcel::write::<ConverterRow>(&path).do_write([ConverterRow {
        value: "source".to_owned(),
    }])?;

    let rows = EasyExcel::read_sync::<ConverterRow>(&path)
        .register_converter::<String, _>(PrefixConverter::string("sync"))
        .do_read_sync()?;
    assert_eq!(rows[0].value, "sync:source");

    let probe = ConverterListener::default();
    let observed = Arc::clone(&probe.0);
    EasyExcel::read::<ConverterRow, _>(&path, probe)
        .register_converter::<String, _>(PrefixConverter::string("event"))
        .do_read()?;
    assert_eq!(
        observed.lock().expect("converter listener lock")[0].value,
        "event:source"
    );

    let fallback = EasyExcel::read_sync::<ConverterRow>(&path)
        .register_converter::<String, _>(PrefixConverter {
            prefix: "wrong-cell-type",
            cell_type: CellDataType::Boolean,
        })
        .do_read_sync()?;
    assert_eq!(fallback[0].value, "source");
    Ok(())
}

#[test]
fn registered_write_converter_uses_latest_registration_and_field_precedence() -> Result<()> {
    // 探针记录 (原始值, 字段类型, 转换后值) 三元组。
    type ProbeEntry = (Option<CellValue>, Option<&'static str>, CellValue);
    struct OriginalValueProbe(Arc<Mutex<Vec<ProbeEntry>>>);

    impl WriteHandler for OriginalValueProbe {
        fn after_cell_data_converted(&mut self, context: &WriteCellContext) -> Result<()> {
            if !context.is_head {
                self.0
                    .lock()
                    .map_err(|_| ExcelError::Format("converter probe poisoned".to_owned()))?
                    .push((
                        context.original_value.clone(),
                        context.original_field_type,
                        context.value.clone(),
                    ));
            }
            Ok(())
        }
    }

    let directory = tempdir()?;
    let global_path = directory.path().join("registered-write.xlsx");
    EasyExcel::write::<ConverterRow>(&global_path)
        .register_converter::<String, _>(PrefixConverter::string("first"))
        .register_converter::<String, _>(PrefixConverter::string("latest"))
        .do_write([ConverterRow {
            value: "source".to_owned(),
        }])?;
    let global = EasyExcel::read_sync::<ConverterRow>(&global_path).do_read_sync()?;
    assert_eq!(global[0].value, "latest:source");

    let field_path = directory.path().join("field-precedence.xlsx");
    let observed = Arc::new(Mutex::new(Vec::new()));
    EasyExcel::write::<FieldConverterRow>(&field_path)
        .register_converter::<String, _>(PrefixConverter::string("global"))
        .register_write_handler(OriginalValueProbe(Arc::clone(&observed)))
        .do_write([FieldConverterRow {
            value: "source".to_owned(),
        }])?;
    assert_eq!(
        observed
            .lock()
            .map_err(|_| ExcelError::Format("converter probe poisoned".to_owned()))?
            .as_slice(),
        [(
            Some(CellValue::String("source".to_owned())),
            Some("String"),
            CellValue::String("field:source".to_owned()),
        )]
    );
    let written = EasyExcel::read_sync::<ConverterRow>(&field_path).do_read_sync()?;
    assert_eq!(written[0].value, "field:source");

    let read = EasyExcel::read_sync::<FieldConverterRow>(&global_path)
        .register_converter::<String, _>(PrefixConverter::string("global"))
        .do_read_sync()?;
    assert_eq!(read[0].value, "field:latest:source");
    Ok(())
}

#[test]
fn write_converter_errors_report_physical_sheet_row_column_and_field() -> Result<()> {
    fn row(failing: &str) -> LocatedWriteFailureRow {
        LocatedWriteFailureRow {
            forced: "forced".to_owned(),
            late: "late".to_owned(),
            failing: failing.to_owned(),
        }
    }

    fn assert_location(error: &ExcelError, expected_row: u32) {
        // 守卫断言替代 match 兜底 panic 臂（写入转换错误恒为 Data 变体）。
        assert!(
            matches!(
                &error,
                ExcelError::Data {
                    sheet,
                    row,
                    column,
                    field,
                    value,
                    message,
                } if sheet == "Diagnostics"
                    && *row == expected_row
                    && *column == Some(0)
                    && *field == "failing"
                    && value.is_empty()
                    && message.contains("converter rejected value")
            ),
            "expected location-aware conversion error, got {error:?}"
        );
    }

    let directory = tempdir()?;
    for extension in ["xlsx", "xls", "csv"] {
        let path = directory
            .path()
            .join(format!("located-write-error.{extension}"));
        let error = EasyExcel::write::<LocatedWriteFailureRow>(&path)
            .sheet("Diagnostics")
            .with_bom(false)
            .do_write([row("ok"), row("fail")])
            .expect_err("the second data row must fail conversion");
        assert_location(&error, 2);
    }

    let template = directory.path().join("located-write-template.xlsx");
    EasyExcel::write::<Value>(&template)
        .sheet("Diagnostics")
        .need_head(false)
        .do_write([Value("existing".to_owned())])?;
    let template_output = directory.path().join("located-write-template-output.xlsx");
    let template_error = EasyExcel::write::<LocatedWriteFailureRow>(&template_output)
        .with_template(&template)
        .sheet("Diagnostics")
        .need_head(false)
        .do_write([row("ok"), row("fail")])
        .expect_err("template conversion must use the appended physical row");
    assert_location(&template_error, 2);

    for extension in ["xlsx", "xls", "csv"] {
        EasyExcel::write::<LocatedWriteFailureRow>(
            directory
                .path()
                .join(format!("excluded-converter.{extension}")),
        )
        .sheet("Diagnostics")
        .with_bom(false)
        .exclude_column_field_names(["failing"])
        .do_write([row("fail")])?;
    }
    Ok(())
}

#[test]
fn sheet_converter_overrides_stateful_workbook_converter() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("stateful-converters.xlsx");
    let mut writer = EasyExcel::write::<ConverterRow>(&path)
        .register_converter::<String, _>(PrefixConverter::string("workbook"))
        .build();
    let workbook_sheet = EasyExcel::writer_sheet::<ConverterRow>("Workbook");
    let override_sheet = EasyExcel::writer_sheet::<ConverterRow>("Override")
        .register_converter::<String, _>(PrefixConverter::string("sheet"));
    writer.write(
        [ConverterRow {
            value: "one".to_owned(),
        }],
        &workbook_sheet,
    )?;
    writer.write(
        [ConverterRow {
            value: "two".to_owned(),
        }],
        &override_sheet,
    )?;
    writer.finish()?;

    let rows = EasyExcel::read_sync::<ConverterRow>(&path)
        .all_sheets()
        .do_read_sync()?;
    assert_eq!(rows[0].value, "workbook:one");
    assert_eq!(rows[1].value, "sheet:two");
    Ok(())
}
