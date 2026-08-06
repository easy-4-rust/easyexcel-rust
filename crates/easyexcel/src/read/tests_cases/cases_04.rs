#[test]
#[allow(clippy::too_many_lines)]
fn public_reader_streams_all_sheets_and_reports_invalid_workbooks() -> Result<()> {
    let (fixture_directory, path) = workbook_fixture()?;
    let mut probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(
        &path,
        &ReadOptions {
            sheet: SheetSelector::All,
            ..options()
        },
        &mut probe,
    )?;
    assert_eq!(
        probe.rows,
        vec![TestRow("one".to_owned()), TestRow("two".to_owned())]
    );
    assert_eq!(
        probe.after,
        vec![("First".to_owned(), 0, 1), ("Second".to_owned(), 1, 1)]
    );

    let mut failing_after = Probe {
        continue_reading: true,
        fail_after: true,
        ..Probe::default()
    };
    assert!(read_xlsx::<TestRow, _>(&path, &options(), &mut failing_after).is_err());

    let mut stopped = Probe::default();
    read_xlsx::<TestRow, _>(
        &path,
        &ReadOptions {
            sheet: SheetSelector::All,
            ..options()
        },
        &mut stopped,
    )?;
    assert_eq!(stopped.heads.len(), 1);
    assert!(stopped.rows.is_empty());
    assert!(stopped.after.is_empty());

    let mut missing = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    assert!(
        read_xlsx::<TestRow, _>(
            &path,
            &ReadOptions {
                sheet: SheetSelector::Index(99),
                ..options()
            },
            &mut missing,
        )
        .is_err()
    );

    let mut failing_transition = Probe {
        continue_reading: true,
        fail_head: true,
        ..Probe::default()
    };
    assert!(
        read_xlsx::<TestRow, _>(&path, &options(), &mut failing_transition).is_err(),
        "a header error emitted while advancing rows must propagate"
    );

    let single_path = fixture_directory.path().join("single.xlsx");
    let mut workbook = Workbook::new();
    workbook
        .add_worksheet()
        .write_string(0, 0, "Value")
        .map_err(test_error)?;
    workbook.save(&single_path).map_err(test_error)?;
    let mut failing_final = Probe {
        continue_reading: true,
        fail_head: true,
        ..Probe::default()
    };
    assert!(
        read_xlsx::<TestRow, _>(&single_path, &options(), &mut failing_final).is_err(),
        "a header error emitted at end-of-sheet must propagate"
    );
    let mut stopped_final = Probe::default();
    read_xlsx::<TestRow, _>(
        &single_path,
        &ReadOptions {
            head_row_number: 0,
            ..options()
        },
        &mut stopped_final,
    )?;
    assert_eq!(stopped_final.rows, vec![TestRow("Value".to_owned())]);
    assert!(stopped_final.after.is_empty());

    let source = XlsxSource::open(&path, None)?;
    let mut metadata = XlsxRowMetadata::new(source.reader()?)?;
    assert!(
        metadata
            .display_cells("Missing", false, false, ssfmt::Locale::default())
            .is_err()
    );

    let empty_path = fixture_directory.path().join("empty.xlsx");
    let mut empty_workbook = Workbook::new();
    empty_workbook.add_worksheet();
    empty_workbook.save(&empty_path).map_err(test_error)?;
    let mut empty_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(&empty_path, &options(), &mut empty_probe)?;
    assert!(empty_probe.rows.is_empty());
    assert_eq!(empty_probe.after, vec![("Sheet1".to_owned(), 0, 0)]);

    let out_of_order_path = fixture_directory.path().join("out-of-order.xlsx");
    let out_of_order_xml = worksheet_xml(
        r#"<c r="B1" t="inlineStr"><is><t>second</t></is></c>
<c r="A1" t="inlineStr"><is><t>first</t></is></c>"#,
    );
    rewrite_first_sheet(&path, &out_of_order_path, &out_of_order_xml)?;
    let mut out_of_order_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(
        &out_of_order_path,
        &ReadOptions {
            head_row_number: 0,
            ..options()
        },
        &mut out_of_order_probe,
    )?;
    assert_eq!(out_of_order_probe.rows, vec![TestRow("first".to_owned())]);

    let sparse_path = fixture_directory.path().join("sparse.xlsx");
    let sparse_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1"><c r="A1" t="inlineStr"><is><t>Value</t></is></c></row>
    <row r="4"><c r="A4" t="inlineStr"><is><t>one</t></is></c></row>
  </sheetData>
</worksheet>"#;
    rewrite_first_sheet(&path, &sparse_path, sparse_xml)?;
    let sparse_options = ReadOptions {
        ignore_empty_row: false,
        ..options()
    };
    let mut sparse_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(&sparse_path, &sparse_options, &mut sparse_probe)?;
    assert_eq!(
        sparse_probe.rows,
        vec![
            TestRow(String::new()),
            TestRow(String::new()),
            TestRow("one".to_owned())
        ]
    );

    let mut stopped_sparse = Probe {
        continue_reading: true,
        stop_after_callbacks: Some(2),
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(&sparse_path, &sparse_options, &mut stopped_sparse)?;
    assert_eq!(stopped_sparse.rows, vec![TestRow(String::new())]);
    assert!(stopped_sparse.after.is_empty());

    let mut failing_sparse = Probe {
        continue_reading: true,
        fail_invoke: true,
        ..Probe::default()
    };
    assert!(read_xlsx::<TestRow, _>(&sparse_path, &sparse_options, &mut failing_sparse).is_err());
    assert_eq!(failing_sparse.errors, 1);

    let trailing_empty_path = fixture_directory.path().join("trailing-empty.xlsx");
    let trailing_empty_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1"><c r="A1" t="inlineStr"><is><t>Value</t></is></c></row>
    <row r="2"><c r="A2" t="inlineStr"><is><t>one</t></is></c></row>
    <row r="5"/>
  </sheetData>
</worksheet>"#;
    rewrite_first_sheet(&path, &trailing_empty_path, trailing_empty_xml)?;
    let mut trailing_empty_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(
        &trailing_empty_path,
        &sparse_options,
        &mut trailing_empty_probe,
    )?;
    assert_eq!(
        trailing_empty_probe.rows,
        vec![
            TestRow("one".to_owned()),
            TestRow(String::new()),
            TestRow(String::new()),
            TestRow(String::new())
        ]
    );
    assert_eq!(trailing_empty_probe.after, vec![("First".to_owned(), 0, 4)]);

    let mut stopped_trailing = Probe {
        continue_reading: true,
        stop_after_callbacks: Some(3),
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(&trailing_empty_path, &sparse_options, &mut stopped_trailing)?;
    assert_eq!(
        stopped_trailing.rows,
        vec![TestRow("one".to_owned()), TestRow(String::new())]
    );
    assert!(stopped_trailing.after.is_empty());

    let mut failing_trailing = Probe {
        continue_reading: true,
        fail_invoke_at: Some(2),
        ..Probe::default()
    };
    assert!(
        read_xlsx::<TestRow, _>(&trailing_empty_path, &sparse_options, &mut failing_trailing,)
            .is_err()
    );
    assert_eq!(failing_trailing.errors, 1);

    let empty_rows_path = fixture_directory.path().join("only-empty-rows.xlsx");
    let empty_rows_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData><row/><row/><row/></sheetData>
</worksheet>"#;
    rewrite_first_sheet(&path, &empty_rows_path, empty_rows_xml)?;
    let mut empty_rows_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(
        &empty_rows_path,
        &ReadOptions {
            head_row_number: 0,
            ignore_empty_row: false,
            ..options()
        },
        &mut empty_rows_probe,
    )?;
    assert_eq!(
        empty_rows_probe.rows,
        vec![
            TestRow(String::new()),
            TestRow(String::new()),
            TestRow(String::new())
        ]
    );
    assert_eq!(empty_rows_probe.after, vec![("First".to_owned(), 0, 2)]);

    let invalid_row_path = fixture_directory.path().join("invalid-row.xlsx");
    let invalid_row_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData><row r="0"/></sheetData>
</worksheet>"#;
    rewrite_first_sheet(&path, &invalid_row_path, invalid_row_xml)?;
    let mut invalid_row_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    assert!(
        read_xlsx::<TestRow, _>(
            &invalid_row_path,
            &ReadOptions {
                ignore_empty_row: false,
                ..options()
            },
            &mut invalid_row_probe,
        )
        .is_err()
    );

    let missing_sheet_path = fixture_directory.path().join("missing-sheet-part.xlsx");
    remove_first_sheet(&path, &missing_sheet_path)?;
    let mut missing_sheet_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    assert!(
        read_xlsx::<TestRow, _>(
            &missing_sheet_path,
            &ReadOptions {
                ignore_empty_row: false,
                ..options()
            },
            &mut missing_sheet_probe,
        )
        .is_err()
    );

    let leading_sparse_path = fixture_directory.path().join("leading-sparse.xlsx");
    let leading_sparse_xml = worksheet_xml(r#"<c r="A3" t="inlineStr"><is><t>first</t></is></c>"#)
        .replace("<row r=\"1\">", "<row r=\"3\">");
    rewrite_first_sheet(&path, &leading_sparse_path, &leading_sparse_xml)?;
    let mut leading_sparse_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(
        &leading_sparse_path,
        &ReadOptions {
            head_row_number: 0,
            ignore_empty_row: false,
            ..options()
        },
        &mut leading_sparse_probe,
    )?;
    assert_eq!(
        leading_sparse_probe.rows,
        vec![
            TestRow(String::new()),
            TestRow(String::new()),
            TestRow("first".to_owned())
        ]
    );

    let wide_path = fixture_directory.path().join("wide.xlsx");
    let wide_column = column_name(u32::from(u16::MAX) + 1);
    let wide_xml = worksheet_xml(&format!(
        r#"<c r="{wide_column}1" t="inlineStr"><is><t>wide</t></is></c>"#
    ));
    rewrite_first_sheet(&path, &wide_path, &wide_xml)?;
    let mut wide_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    assert!(
        read_xlsx::<TestRow, _>(
            &wide_path,
            &ReadOptions {
                head_row_number: 0,
                ..options()
            },
            &mut wide_probe,
        )
        .is_err()
    );

    let truncated_path = fixture_directory.path().join("truncated.xlsx");
    rewrite_first_sheet(
        &path,
        &truncated_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>first</t></is></c>"#,
    )?;
    let mut truncated_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    assert!(
        read_xlsx::<TestRow, _>(
            &truncated_path,
            &ReadOptions {
                head_row_number: 0,
                ..options()
            },
            &mut truncated_probe,
        )
        .is_err()
    );

    let directory = tempdir()?;
    let invalid = directory.path().join("invalid.xlsx");
    fs::write(&invalid, b"not an xlsx")?;
    assert!(read_xlsx::<TestRow, _>(&invalid, &options(), &mut probe).is_err());
    assert!(
        read_xlsx::<TestRow, _>(
            &directory.path().join("missing.xlsx"),
            &options(),
            &mut probe,
        )
        .is_err()
    );
    let missing_source = XlsxSource::File(directory.path().join("missing-source.xlsx"));
    assert!(read_xlsx_source::<TestRow, _>(&missing_source, &options(), &mut probe).is_err());
    assert!(read_xlsx::<TestRow, _>(directory.path(), &options(), &mut probe).is_err());
    let invalid_encrypted = directory.path().join("invalid-encrypted.xlsx");
    fs::write(
        &invalid_encrypted,
        [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
    )?;
    assert!(
        read_xlsx::<TestRow, _>(
            &invalid_encrypted,
            &ReadOptions {
                password: Some("123456".to_owned()),
                ..options()
            },
            &mut probe,
        )
        .is_err()
    );
    Ok(())
}
