#[test]
#[allow(clippy::too_many_lines)]
fn facade_reads_and_writes_java_style_dynamic_rows() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("dynamic.xlsx");
    let source = DynamicRow::new(BTreeMap::from([
        (0, DynamicValue::String("string19".to_owned())),
        (1, DynamicValue::ActualData(CellValue::Int(109))),
        (2, DynamicValue::Null),
        (3, DynamicValue::String("tail".to_owned())),
    ]));
    EasyExcel::write::<DynamicRow>(&path).do_write([source.clone()])?;

    let strings = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()?;
    assert_eq!(
        strings[0].get(0),
        Some(&DynamicValue::String("string19".to_owned()))
    );
    assert_eq!(
        strings[0].get(1),
        Some(&DynamicValue::String("109".to_owned()))
    );
    assert_eq!(strings[0].get(2), Some(&DynamicValue::Null));

    let actual = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .read_default_return(ReadDefaultReturn::ActualData)
        .do_read_sync()?;
    let actual_cell = actual[0].get(1);
    // 守卫断言替代 let-else 兜底 panic 臂（ActualData 读取模式恒产出 ActualData）。
    assert!(
        matches!(actual_cell, Some(DynamicValue::ActualData(number)) if number.as_text() == "109"),
        "expected actual numeric cell, got {actual_cell:?}"
    );

    let listener = DynamicListener::default();
    let observed = Arc::clone(&listener.0);
    EasyExcel::read_dynamic(&path, listener)
        .head_row_number(0)
        .read_default_return(ReadDefaultReturn::ReadCellData)
        .do_read()?;
    let observed = observed.lock().expect("dynamic listener lock");
    let tail = observed[0].get(3).expect("tail cell");
    // 守卫断言替代 let-else 兜底 panic 臂（ReadDefaultReturn::ReadCellData 模式恒产出 ReadCellData）。
    assert!(
        matches!(tail, DynamicValue::ReadCellData(cell) if cell.data() == &CellValue::String("tail".to_owned())),
        "expected read cell data, got {tail:?}"
    );

    let csv_without_head = directory.path().join("dynamic-no-head.csv");
    EasyExcel::write::<DynamicRow>(&csv_without_head)
        .with_bom(false)
        .do_write([source.clone()])?;
    let no_head_rows = EasyExcel::read_dynamic_sync(&csv_without_head)
        .head_row_number(0)
        .do_read_sync()?;
    assert_eq!(
        no_head_rows[0].get(3),
        Some(&DynamicValue::String("tail".to_owned()))
    );
    assert!(matches!(
        EasyExcel::write::<DynamicRow>(directory.path().join("invalid-charset.csv"))
            .charset("not-a-charset")
            .do_write([source.clone()]),
        Err(ExcelError::Unsupported(_))
    ));
    assert!(
        EasyExcel::write::<DynamicRow>(directory.path().join("missing/dynamic.csv"))
            .do_write([source.clone()])
            .is_err()
    );

    let csv = directory.path().join("dynamic.csv");
    EasyExcel::write::<DynamicRow>(&csv)
        .head([["Text"], ["Number"], ["Empty"], ["Tail"]])
        .with_bom(false)
        .do_write([source])?;
    let csv_rows = EasyExcel::read_dynamic_sync(&csv).do_read_sync()?;
    assert_eq!(
        csv_rows[0].get(0),
        Some(&DynamicValue::String("string19".to_owned()))
    );
    assert_eq!(
        csv_rows[0].get(1),
        Some(&DynamicValue::String("109".to_owned()))
    );

    let filter_source = DynamicRow::new(BTreeMap::from([
        (0, DynamicValue::String("A".to_owned())),
        (1, DynamicValue::String("B".to_owned())),
        (2, DynamicValue::String("C".to_owned())),
    ]));
    let filtered = directory.path().join("dynamic-filtered.xlsx");
    EasyExcel::write::<DynamicRow>(&filtered)
        .include_column_indexes([2, 0])
        .exclude_column_indexes([2])
        .order_by_include_column(true)
        .do_write([filter_source.clone()])?;
    assert_eq!(
        EasyExcel::read_dynamic_sync(&filtered)
            .head_row_number(0)
            .do_read_sync()?[0]
            .get(0),
        Some(&DynamicValue::String("A".to_owned()))
    );

    EasyExcel::write::<DynamicRow>(directory.path().join("dynamic-ordered.xlsx"))
        .order_by_include_column(true)
        .do_write([filter_source.clone()])?;
    EasyExcel::write::<DynamicRow>(directory.path().join("dynamic-field-filter.xlsx"))
        .include_column_field_names(["unknown"])
        .do_write([filter_source.clone()])?;
    EasyExcel::write::<DynamicRow>(directory.path().join("dynamic-index-include.xlsx"))
        .include_column_indexes([1])
        .do_write([filter_source])?;
    EasyExcel::write::<Value>(directory.path().join("typed-field-include.xlsx"))
        .include_column_field_names(["value"])
        .do_write([Value("included".to_owned())])?;
    Ok(())
}

