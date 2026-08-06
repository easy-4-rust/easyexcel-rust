#[test]
fn calamine_values_map_to_every_core_cell_variant() {
    let datetime = ExcelDateTime::new(46_120.5, ExcelDateTimeType::DateTime, false);
    let invalid_datetime = ExcelDateTime::new(f64::MAX, ExcelDateTimeType::DateTime, false);
    let duration = ExcelDateTime::new(1.5, ExcelDateTimeType::TimeDelta, false);
    let cases = [
        (DataRef::Empty, CellValue::Empty),
        (
            DataRef::String("owned".to_owned()),
            CellValue::String("owned".to_owned()),
        ),
        (
            DataRef::SharedString("shared"),
            CellValue::String("shared".to_owned()),
        ),
        (
            DataRef::DateTimeIso("2026-01-01".to_owned()),
            CellValue::String("2026-01-01".to_owned()),
        ),
        (
            DataRef::DurationIso("PT1H".to_owned()),
            CellValue::String("PT1H".to_owned()),
        ),
        (DataRef::Bool(true), CellValue::Bool(true)),
        (DataRef::Int(7), CellValue::Int(7)),
        (DataRef::Float(1.25), CellValue::Float(1.25)),
        (DataRef::DateTime(duration), CellValue::Float(1.5)),
        (
            DataRef::DateTime(invalid_datetime),
            CellValue::Float(f64::MAX),
        ),
        (
            DataRef::Error(CellErrorType::Div0),
            CellValue::Error("#DIV/0!".to_owned()),
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(from_calamine(&input, false), expected);
        assert_eq!(from_data(&Data::from(input), false), expected);
    }
    assert!(matches!(
        from_calamine(&DataRef::DateTime(datetime), false),
        CellValue::DateTime(_)
    ));
    assert!(matches!(
        from_data(&Data::DateTime(datetime), false),
        CellValue::DateTime(_)
    ));
    let serial_one = ExcelDateTime::new(1.0, ExcelDateTimeType::DateTime, true);
    assert_eq!(
        from_calamine(&DataRef::DateTime(serial_one), false).as_text(),
        "1900-01-01 00:00:00"
    );
    assert_eq!(
        from_data(&Data::DateTime(serial_one), true).as_text(),
        "1904-01-02 00:00:00"
    );
}

#[test]
fn helpers_preserve_diagnostics_and_xlsx_column_limits() {
    assert_eq!(ReadOptions::default(), options());
    assert_eq!(SheetSelector::default(), SheetSelector::First);
    assert_eq!(to_column_index(0).expect("column"), 0);
    assert_eq!(
        to_column_index(u32::from(u16::MAX)).expect("column"),
        usize::from(u16::MAX)
    );
    assert!(to_column_index(u32::from(u16::MAX) + 1).is_err());
    assert_eq!(
        ExcelError::Format("broken".to_owned()).to_string(),
        "excel format error: broken"
    );
    assert!(!is_compound_document(&mut FaultyBufRead));
    assert_eq!(
        easyexcel_utils::string_utils::java_trim("\0\t value \r\n"),
        "value"
    );
    assert_eq!(
        easyexcel_utils::string_utils::java_trim("\u{a0}value\u{a0}"),
        "\u{a0}value\u{a0}"
    );
    assert!(
        easyexcel_utils::string_utils::equals_with_optional_java_trim(" Sheet ", "Sheet", true)
    );
    assert!(
        !easyexcel_utils::string_utils::equals_with_optional_java_trim(" Sheet ", "Sheet", false)
    );
}

#[test]
fn header_aliases_and_inclusive_row_ranges_apply_before_typed_mapping() -> Result<()> {
    let mut range = Range::new((0, 0), (3, 0));
    range.set_value((0, 0), Data::String("Source".to_owned()));
    range.set_value((1, 0), Data::String("one".to_owned()));
    range.set_value((2, 0), Data::String("two".to_owned()));
    range.set_value((3, 0), Data::String("three".to_owned()));
    let mut aliases = HashMap::new();
    aliases.insert("Source".to_owned(), "Canonical".to_owned());
    let options = ReadOptions {
        start_row: Some(2),
        end_row: Some(2),
        header_aliases: aliases,
        ..ReadOptions::default()
    };
    let mut probe = NamedProbe::default();

    assert_eq!(
        read_range(
            &range,
            0,
            "Aliased",
            &options,
            &HashMap::new(),
            &mut TypedRowConsumer::<NamedRow> {
                listener: &mut probe,
            },
        )?,
        ReadFlow::Continue
    );
    assert_eq!(probe.heads[0].get("Canonical"), Some(&0));
    assert_eq!(probe.rows, vec![NamedRow("two".to_owned())]);
    Ok(())
}

#[test]
fn read_row_range_validation_rejects_only_reversed_bounds() {
    assert!(validate_read_options(&ReadOptions::default()).is_ok());
    assert!(
        validate_read_options(&ReadOptions {
            start_row: Some(2),
            ..ReadOptions::default()
        })
        .is_ok()
    );
    assert!(
        validate_read_options(&ReadOptions {
            end_row: Some(2),
            ..ReadOptions::default()
        })
        .is_ok()
    );
    assert!(
        validate_read_options(&ReadOptions {
            start_row: Some(2),
            end_row: Some(2),
            ..ReadOptions::default()
        })
        .is_ok()
    );
    assert_eq!(
        validate_read_options(&ReadOptions {
            start_row: Some(3),
            end_row: Some(2),
            ..ReadOptions::default()
        })
        .expect_err("reversed row range")
        .to_string(),
        "excel format error: read row range start 3 exceeds end 2"
    );

    let reversed = ReadOptions {
        start_row: Some(3),
        end_row: Some(2),
        ..ReadOptions::default()
    };
    let mut modern_workbook_probe = Probe::default();
    let mut legacy_workbook_probe = Probe::default();
    let mut delimited_text_probe = Probe::default();
    assert!(
        read_xlsx::<TestRow, _>(
            Path::new("missing.xlsx"),
            &reversed,
            &mut modern_workbook_probe,
        )
        .is_err()
    );
    assert!(
        read_xls::<TestRow, _>(
            Path::new("missing.xls"),
            &reversed,
            &mut legacy_workbook_probe,
        )
        .is_err()
    );
    assert!(
        read_csv::<TestRow, _>(
            Path::new("missing.csv"),
            &reversed,
            &mut delimited_text_probe,
        )
        .is_err()
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn legacy_range_read_preserves_coordinates_headers_and_empty_sheets() -> Result<()> {
    let mut range = Range::new((2, 1), (3, 1));
    range.set_value((2, 1), Data::String("Value".to_owned()));
    range.set_value((3, 1), Data::String("one".to_owned()));
    let mut probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_range(
        &range,
        2,
        "Legacy",
        &ReadOptions {
            head_row_number: 3,
            ..options()
        },
        &HashMap::new(),
        &mut TypedRowConsumer::<TestRow> {
            listener: &mut probe,
        },
    )?;
    assert_eq!(probe.heads[0].get("Value"), Some(&1));
    assert_eq!(probe.rows, vec![TestRow(String::new())]);
    assert_eq!(probe.after, vec![("Legacy".to_owned(), 2, 3)]);

    let mut stopped = Probe::default();
    assert_eq!(
        read_range(
            &range,
            2,
            "Legacy",
            &ReadOptions {
                head_row_number: 3,
                ..options()
            },
            &HashMap::new(),
            &mut TypedRowConsumer::<TestRow> {
                listener: &mut stopped,
            },
        )?,
        ReadFlow::Stop
    );
    assert_eq!(stopped.heads.len(), 1);
    assert!(stopped.rows.is_empty());
    assert!(stopped.after.is_empty());

    read_range(
        &Range::empty(),
        3,
        "Empty",
        &options(),
        &std::collections::HashMap::new(),
        &mut TypedRowConsumer::<TestRow> {
            listener: &mut probe,
        },
    )?;
    assert_eq!(probe.after.last(), Some(&("Empty".to_owned(), 3, 0)));

    let mut failing_empty_after = Probe {
        continue_reading: true,
        fail_after: true,
        ..Probe::default()
    };
    assert!(
        read_range(
            &Range::empty(),
            3,
            "Empty",
            &options(),
            &std::collections::HashMap::new(),
            &mut TypedRowConsumer::<TestRow> {
                listener: &mut failing_empty_after,
            },
        )
        .is_err()
    );

    let mut failing_range_after = Probe {
        continue_reading: true,
        fail_after: true,
        ..Probe::default()
    };
    assert!(
        read_range(
            &range,
            2,
            "Legacy",
            &ReadOptions {
                head_row_number: 3,
                ..options()
            },
            &HashMap::new(),
            &mut TypedRowConsumer::<TestRow> {
                listener: &mut failing_range_after,
            },
        )
        .is_err()
    );

    let invalid_column = Range::new((0, u32::from(u16::MAX) + 1), (0, u32::from(u16::MAX) + 1));
    assert!(
        read_range(
            &invalid_column,
            0,
            "Invalid",
            &options(),
            &std::collections::HashMap::new(),
            &mut TypedRowConsumer::<TestRow> {
                listener: &mut probe,
            },
        )
        .is_err()
    );

    let mut failing_head = Probe {
        continue_reading: true,
        fail_head: true,
        ..Probe::default()
    };
    assert!(
        read_range(
            &range,
            0,
            "Legacy",
            &ReadOptions {
                head_row_number: 3,
                ..options()
            },
            &HashMap::new(),
            &mut TypedRowConsumer::<TestRow> {
                listener: &mut failing_head,
            },
        )
        .is_err()
    );
    Ok(())
}

