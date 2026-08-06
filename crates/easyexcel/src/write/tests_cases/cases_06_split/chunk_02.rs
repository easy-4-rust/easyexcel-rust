#[test]
fn duplicate_manual_indexes_fail_before_handlers_filters_templates_and_output() -> Result<()> {
    struct DuplicateIndexRow;

    impl ExcelRow for DuplicateIndexRow {
        fn schema() -> &'static [ExcelColumn] {
            const SCHEMA: &[ExcelColumn] = &[
                ExcelColumn::new("first", "First", Some(1), 0, None),
                ExcelColumn::new("second", "Second", Some(1), 0, None),
            ];
            SCHEMA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            unreachable!("duplicate schema must fail before row conversion")
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            unreachable!("duplicate schema must fail before row conversion")
        }
    }

    struct WorkbookProbe(Arc<AtomicUsize>);

    impl WriteHandler for WorkbookProbe {
        fn before_workbook(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn assert_duplicate(error: &ExcelError) {
        assert_eq!(
            error.to_string(),
            "excel format error: The index of 'first' and 'second' must be inconsistent"
        );
    }

    let directory = tempdir()?;
    let options = WriteOptions {
        // Java validates the complete FieldCache before applying excludes.
        exclude_column_field_names: vec!["second".to_owned()],
        ..WriteOptions::default()
    };

    for extension in ["xlsx", "xls", "csv"] {
        let output = directory.path().join(format!("duplicate.{extension}"));
        let callbacks = Arc::new(AtomicUsize::new(0));
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(WorkbookProbe(Arc::clone(&callbacks)))];
        let error = match extension {
            "xlsx" => write_xlsx_with_handlers::<DuplicateIndexRow, _>(
                &output,
                &options,
                vec![DuplicateIndexRow],
                &mut handlers,
            )
            .expect_err("duplicate XLSX index must fail"),
            "xls" => write_xls_with_handlers::<DuplicateIndexRow, _>(
                &output,
                &options,
                vec![DuplicateIndexRow],
                &mut handlers,
            )
            .expect_err("duplicate XLS index must fail"),
            "csv" => write_csv_with_handlers::<DuplicateIndexRow, _>(
                &output,
                &options,
                vec![DuplicateIndexRow],
                &mut handlers,
            )
            .expect_err("duplicate CSV index must fail"),
            _ => unreachable!(),
        };
        assert_duplicate(&error);
        assert_eq!(callbacks.load(Ordering::SeqCst), 0);
        assert!(!output.exists(), "{extension} output must not be created");
    }

    let callbacks = Arc::new(AtomicUsize::new(0));
    let mut handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(WorkbookProbe(Arc::clone(&callbacks)))];
    let template_options = WriteOptions {
        template_bytes: Some(b"not a workbook".to_vec()),
        ..options.clone()
    };
    let error = write_xlsx_with_handlers::<DuplicateIndexRow, _>(
        &directory.path().join("duplicate-template.xlsx"),
        &template_options,
        vec![DuplicateIndexRow],
        &mut handlers,
    )
    .expect_err("schema validation must precede template parsing");
    assert_duplicate(&error);
    assert_eq!(callbacks.load(Ordering::SeqCst), 0);

    let stateful_output = directory.path().join("duplicate-stateful.xlsx");
    let callbacks = Arc::new(AtomicUsize::new(0));
    let mut writer = ExcelWriter::new(&stateful_output);
    writer.register_write_handler(Box::new(WorkbookProbe(Arc::clone(&callbacks))))?;
    let sheet = WriteSheet::<DuplicateIndexRow>::new("Data");
    let Err(error) = writer.write(vec![DuplicateIndexRow], &sheet) else {
        panic!("stateful schema validation must precede writer start");
    };
    assert_duplicate(&error);
    assert_eq!(callbacks.load(Ordering::SeqCst), 0);
    assert!(!stateful_output.exists());

    Ok(())
}

#[test]
fn dynamic_row_layout_omits_a_synthetic_head_and_accepts_a_dynamic_head() -> Result<()> {
    let options = WriteOptions::default();
    assert_eq!(head_rows_for_schema_state(true, &options)?, 0);
    assert!(dynamic_columns_for_row(true, 3, &options).is_some());
    assert!(dynamic_columns_for_row(false, 3, &options).is_none());

    let headed_options = WriteOptions {
        dynamic_head: Some(vec![
            vec!["Name".to_owned()],
            vec!["Unused".to_owned()],
            vec!["Score".to_owned()],
        ]),
        ..WriteOptions::default()
    };
    assert_eq!(head_rows_for_schema_state(true, &headed_options)?, 1);
    assert_eq!(
        dynamic_columns_for_row(true, 3, &headed_options)
            .expect("dynamic basic row mapping")
            .iter()
            .map(|(physical, source, _)| (*physical, *source))
            .collect::<Vec<_>>(),
        vec![(0, 0), (1, 1), (2, 2)]
    );
    assert_eq!(
        dynamic_columns_for_row(true, 1, &headed_options)
            .expect("short dynamic basic row mapping")
            .iter()
            .map(|(physical, source, _)| (*physical, *source))
            .collect::<Vec<_>>(),
        vec![(0, 0)]
    );
    assert_eq!(
        dynamic_columns_for_row(
            true,
            3,
            &WriteOptions {
                dynamic_head: headed_options.dynamic_head.clone(),
                include_column_indexes: Some(vec![2, 0]),
                order_by_include_column: true,
                ..WriteOptions::default()
            },
        )
        .expect("filtered head map")
        .iter()
        .map(|(physical, source, _)| (*physical, *source))
        .collect::<Vec<_>>(),
        vec![(0, 0), (1, 1), (2, 2)]
    );

    let mut writer = create_csv_record_writer(Box::new(Vec::<u8>::new()), &options.charset, true)?;
    let cells = vec![
        CellValue::String("Alice".to_owned()),
        CellValue::Empty,
        CellValue::Int(7),
    ];
    let converted = cells.iter().cloned().map(WriteCellData::new).collect();
    let mut rows = [Ok(PreparedWriteRow {
        absent: false,
        original_cells: cells,
        cells: converted,
    })]
    .into_iter();
    let progress = append_csv_records(
        &mut writer,
        &options,
        &[],
        true,
        &mut rows,
        &mut [],
        0,
        0,
        true,
        None,
    )?;
    assert_eq!(progress.next_row, 1);
    assert_eq!(progress.next_data_index, 1);
    finish_csv_record_writer(writer)
}

#[test]
// 语义敏感：xlsx/xls 双后端并行断言，命名刻意对照，故豁免 similar_names。
#[allow(clippy::similar_names)]
fn dynamic_basic_row_keeps_values_beyond_the_head_map_across_backends() -> Result<()> {
    let directory = tempdir()?;
    let options = WriteOptions {
        sheet_name: "Dynamic".to_owned(),
        dynamic_head: Some(vec![vec!["First".to_owned()], vec!["Second".to_owned()]]),
        ..WriteOptions::default()
    };
    let row = DynamicRow::new(
        [
            (0, DynamicValue::String("alpha".to_owned())),
            (1, DynamicValue::String("beta".to_owned())),
            (2, DynamicValue::String("after-head".to_owned())),
        ]
        .into_iter()
        .collect(),
    );

    let xlsx_path = directory.path().join("dynamic-extra.xlsx");
    write_xlsx::<DynamicRow, _>(&xlsx_path, &options, [row.clone()])?;
    let mut xlsx: Xlsx<_> = open_workbook(&xlsx_path).map_err(test_error)?;
    let xlsx_range = xlsx.worksheet_range("Dynamic").map_err(test_error)?;
    assert_eq!(
        xlsx_range.get_value((1, 2)),
        Some(&Data::String("after-head".to_owned()))
    );

    let xls_path = directory.path().join("dynamic-extra.xls");
    write_xls::<DynamicRow, _>(&xls_path, &options, [row.clone()])?;
    let mut xls: Xls<_> = open_workbook(&xls_path).map_err(test_error)?;
    let xls_range = xls.worksheet_range("Dynamic").map_err(test_error)?;
    assert_eq!(
        xls_range.get_value((1, 2)),
        Some(&Data::String("after-head".to_owned()))
    );

    let csv = write_csv_to_buffer::<DynamicRow, _>(
        Path::new("dynamic-extra.csv"),
        &options,
        [row.clone()],
        &mut [],
    )?;
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv.as_slice());
    let records = csv_reader
        .records()
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(test_error)?;
    assert_eq!(records[1].get(2), Some("after-head"));

    let (template_rows, _, _, _) =
        collect_template_append_rows::<DynamicRow, _>(&options, [row], true, 0)?;
    assert_eq!(
        template_rows[1].get(2),
        Some(&(2, CellValue::String("after-head".to_owned())))
    );
    Ok(())
}

