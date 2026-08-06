#[test]
fn reads_java_easyexcel_legacy_multisheet_fixture() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("java-multiplesheets.xls");
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-multiplesheets.xls.gz.b64")).trim())
        .map_err(test_error)?;
    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut workbook = Vec::new();
    decoder.read_to_end(&mut workbook)?;
    fs::write(&path, workbook)?;
    let mut probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xls::<TestRow, _>(
        &path,
        &ReadOptions {
            sheet: SheetSelector::All,
            ..options()
        },
        &mut probe,
    )?;
    assert_eq!(
        probe.rows,
        (1..=6)
            .map(|index| TestRow(format!("表{index}数据")))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        probe.after,
        (0..6)
            .map(|index| (format!("Sheet{}", index + 1), index, 1))
            .collect::<Vec<_>>()
    );
    assert_eq!(probe.heads.len(), 6);
    for (index, head) in probe.heads.iter().enumerate() {
        assert_eq!(head.get(&format!("表{}头", index + 1)), Some(&0));
    }
    let mut stopped = Probe::default();
    read_xls::<TestRow, _>(
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
    assert!(
        read_xls::<TestRow, _>(
            &path,
            &ReadOptions {
                sheet: SheetSelector::Index(99),
                ..options()
            },
            &mut probe,
        )
        .is_err()
    );
    let mut failing_head = Probe {
        continue_reading: true,
        fail_head: true,
        ..Probe::default()
    };
    assert!(read_xls::<TestRow, _>(&path, &options(), &mut failing_head).is_err());
    let invalid = directory.path().join("invalid.xls");
    fs::write(&invalid, b"not an XLS workbook")?;
    assert!(read_xls::<TestRow, _>(&invalid, &options(), &mut probe).is_err());

    let mut dynamic = DynamicProbe::default();
    read_xls::<DynamicRow, _>(
        &path,
        &ReadOptions {
            read_default_return: ReadDefaultReturn::ReadCellData,
            ..options()
        },
        &mut dynamic,
    )?;
    let DynamicValue::ReadCellData(cell) = dynamic.0[0].get(0).expect("legacy cell") else {
        panic!("expected legacy read cell data");
    };
    assert_eq!(cell.raw_value(), &CellValue::String("表1数据".to_owned()));
    assert_eq!(cell.data(), &CellValue::String("表1数据".to_owned()));
    assert_eq!(cell.row_index(), 1);
    assert_eq!(cell.column_index(), 0);
    Ok(())
}

#[test]
fn reads_java_official_compatibility_fixtures() -> Result<()> {
    let directory = tempdir().map_err(test_error)?;
    let t01 = read_java_compatibility_rows(&directory, "t01.xls", 1, ReadDefaultReturn::String)?;
    assert_eq!(t01.len(), 2);
    assert_eq!(
        t01[1].get(0),
        Some(&DynamicValue::String("Q235(碳钢)".to_owned()))
    );

    let t02 = read_java_compatibility_rows(&directory, "t02.xlsx", 0, ReadDefaultReturn::String)?;
    assert_eq!(t02.len(), 3);
    assert_eq!(
        t02[2].get(2),
        Some(&DynamicValue::String("1，2-戊二醇".to_owned()))
    );

    let t03 = read_java_compatibility_rows(&directory, "t03.xlsx", 1, ReadDefaultReturn::String)?;
    assert_eq!(t03.len(), 1);
    assert_eq!(t03[0].values().len(), 12);

    let t04 = read_java_compatibility_rows(&directory, "t04.xlsx", 1, ReadDefaultReturn::String)?;
    assert_eq!(t04.len(), 56);
    assert_eq!(
        t04[0].get(5),
        Some(&DynamicValue::String("QQSJK28F152A012242S0081".to_owned()))
    );

    let t05 = read_java_compatibility_rows(&directory, "t05.xlsx", 1, ReadDefaultReturn::String)?;
    for (row, expected) in [
        "2023-01-01 00:00:00",
        "2023-01-01 00:00:00",
        "2023-01-01 00:00:00",
        "2023-01-01 00:00:01",
        "2023-01-01 00:00:01",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            t05[row].get(0),
            Some(&DynamicValue::String(expected.to_owned()))
        );
    }

    let t06 = read_java_compatibility_rows(&directory, "t06.xlsx", 0, ReadDefaultReturn::String)?;
    assert_eq!(
        t06[0].get(2),
        Some(&DynamicValue::String("2087.03".to_owned()))
    );

    let t07_actual =
        read_java_compatibility_rows(&directory, "t07.xlsx", 1, ReadDefaultReturn::ActualData)?;
    let Some(DynamicValue::ActualData(CellValue::Decimal(actual))) = t07_actual[0].get(11) else {
        panic!("expected actual decimal value");
    };
    assert_eq!(actual.to_string(), "24.1998124");
    let t07_string =
        read_java_compatibility_rows(&directory, "t07.xlsx", 1, ReadDefaultReturn::String)?;
    assert_eq!(
        t07_string[0].get(11),
        Some(&DynamicValue::String("24.20".to_owned()))
    );
    // Full-table STRING: `_ ` pads dropped; negative `\ ` keeps trailing space (Java POI).
    assert_eq!(
        t07_string[0].get(12),
        Some(&DynamicValue::String("-1.07 ".to_owned()))
    );
    assert_eq!(
        t07_string[0].get(13),
        Some(&DynamicValue::String("14.11".to_owned()))
    );
    assert_eq!(
        t07_string[0].get(15),
        Some(&DynamicValue::String("0.00".to_owned()))
    );

    let t09 = read_java_compatibility_rows(&directory, "t09.xlsx", 0, ReadDefaultReturn::String)?;
    assert_eq!(t09.len(), 1);
    assert_eq!(
        t09[0].get(0),
        Some(&DynamicValue::String("SH_x000D_Z002".to_owned()))
    );
    Ok(())
}

#[test]
fn reads_java_easyexcel_encrypted_xlsx_fixture() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("java-encrypt07.xlsx");
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-encrypt07.xlsx.gz.b64"))
            .trim(),
        )
        .map_err(test_error)?;
    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut workbook = Vec::new();
    decoder.read_to_end(&mut workbook)?;
    assert!(is_compound_document(&mut workbook.as_slice()));
    assert!(!is_compound_document(&mut &workbook[..4]));
    assert!(!is_compound_document(&mut &b"not-cfb!"[..]));
    fs::write(&path, workbook)?;

    let mut probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(
        &path,
        &ReadOptions {
            password: Some("123456".to_owned()),
            extra_read: HashSet::from([crate::core::CellExtraType::Merge]),
            ..options()
        },
        &mut probe,
    )?;
    assert!(read_xlsx::<TestRow, _>(&path, &options(), &mut probe).is_err());
    assert_eq!(
        probe.rows,
        (0..10)
            .map(|index| TestRow(format!("姓名{index}")))
            .collect::<Vec<_>>()
    );
    assert_eq!(probe.heads[0].get("姓名"), Some(&0));
    assert_eq!(probe.after, vec![("0".to_owned(), 0, 10)]);

    let mut empty_row_probe = Probe {
        continue_reading: true,
        ..Probe::default()
    };
    read_xlsx::<TestRow, _>(
        &path,
        &ReadOptions {
            ignore_empty_row: false,
            password: Some("123456".to_owned()),
            ..options()
        },
        &mut empty_row_probe,
    )?;
    assert_eq!(empty_row_probe.rows.len(), 10);
    assert_eq!(empty_row_probe.after, vec![("0".to_owned(), 0, 10)]);
    Ok(())
}

