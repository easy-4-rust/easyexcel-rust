#[test]
#[allow(clippy::too_many_lines)]
fn dynamic_head_validation_and_backend_failures_are_typed() -> Result<()> {
    let directory = tempdir()?;
    assert_eq!(head_level_to_row(0)?, 0);
    assert!(head_level_to_row(usize::MAX).is_err());
    assert_eq!(
        dynamic_head_rows(&WriteOptions {
            need_head: false,
            dynamic_head: Some(Vec::new()),
            ..WriteOptions::default()
        })?,
        0
    );
    assert!(
        dynamic_head_rows(&WriteOptions {
            dynamic_head: Some(Vec::new()),
            ..WriteOptions::default()
        })
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("empty-head.xlsx"),
            &WriteOptions {
                dynamic_head: Some(Vec::new()),
                ..WriteOptions::default()
            },
            Vec::new()
        )
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("empty-head-paths.xlsx"),
            &WriteOptions {
                dynamic_head: Some(vec![Vec::new(); EveryCell::schema().len()]),
                ..WriteOptions::default()
            },
            Vec::new()
        )
        .is_err()
    );
    let invalid_head_options = WriteOptions {
        dynamic_head: Some(vec![Vec::new(); EveryCell::schema().len()]),
        ..WriteOptions::default()
    };
    let mut workbook = Workbook::new();
    assert!(
        append_rows_to_worksheet::<EveryCell, _>(
            workbook.add_worksheet(),
            &invalid_head_options,
            Vec::new(),
            &mut [],
            WriteProgress {
                next_row: 0,
                next_data_index: 0,
            },
            true,
            EveryCell::write_metadata(),
        )
        .is_err()
    );
    let invalid_head_height = ExcelWriteMetadata::new().head_row_height(16);
    assert!(
        append_rows_to_worksheet::<EveryCell, _>(
            workbook.add_worksheet(),
            &WriteOptions {
                include_column_indexes: Some(Vec::new()),
                ..WriteOptions::default()
            },
            Vec::new(),
            &mut [],
            WriteProgress {
                next_row: 1_048_576,
                next_data_index: 0,
            },
            true,
            &invalid_head_height,
        )
        .is_err()
    );
    let invalid_content_height = ExcelWriteMetadata::new().content_row_height(16);
    assert!(
        append_rows_to_worksheet::<EveryCell, _>(
            workbook.add_worksheet(),
            &WriteOptions {
                need_head: false,
                ..WriteOptions::default()
            },
            vec![every_cell()],
            &mut [],
            WriteProgress {
                next_row: 1_048_576,
                next_data_index: 0,
            },
            true,
            &invalid_content_height,
        )
        .is_err()
    );
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("mismatched-head.xlsx"),
            &WriteOptions {
                include_column_indexes: Some(vec![0, 1]),
                dynamic_head: Some(vec![vec!["Only one".to_owned()]]),
                ..WriteOptions::default()
            },
            Vec::new()
        )
        .is_err()
    );
    USE_WIDE_SCHEMA.with(|wide| wide.set(true));
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("wide-dynamic-head.xlsx"),
            &WriteOptions {
                dynamic_head: Some(vec![vec!["Wide".to_owned()]]),
                ..WriteOptions::default()
            },
            Vec::new()
        )
        .is_err()
    );
    USE_WIDE_SCHEMA.with(|wide| wide.set(false));
    USE_ANNOTATED_WIDE_SCHEMA.with(|wide| wide.set(true));
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("annotated-wide-column.xlsx"),
            &WriteOptions::default(),
            Vec::new(),
        )
        .is_err()
    );
    USE_ANNOTATED_WIDE_SCHEMA.with(|wide| wide.set(false));
    USE_BACKEND_WIDE_SCHEMA.with(|wide| wide.set(true));
    assert!(
        write_xlsx::<EveryCell, _>(
            &directory.path().join("backend-wide-column.xlsx"),
            &WriteOptions::default(),
            Vec::new(),
        )
        .is_err()
    );
    USE_BACKEND_WIDE_SCHEMA.with(|wide| wide.set(false));

    let head = vec![vec!["Group".to_owned()], vec!["Group".to_owned()]];
    for columns in [
        vec![(65_536, 0, &TEST_COLUMN), (65_537, 0, &TEST_COLUMN)],
        vec![(65_535, 0, &TEST_COLUMN), (65_536, 0, &TEST_COLUMN)],
        vec![(16_383, 0, &TEST_COLUMN), (16_384, 0, &TEST_COLUMN)],
    ] {
        let mut raw = Workbook::new();
        let worksheet = raw.add_worksheet();
        assert!(
            merge_dynamic_head_groups(
                worksheet,
                &columns,
                &head,
                SheetStyleContext::head(
                    &CellStyle::default(),
                    &ExcelWriteMetadata::new(),
                    WriteGlobalFlags::default()
                ),
                0,
            )
            .is_err()
        );
    }
    assert!(
        dynamic_head_rows(&WriteOptions {
            dynamic_head: Some(vec![Vec::new()]),
            ..WriteOptions::default()
        })
        .is_err()
    );
    assert_eq!(
        dynamic_head_merge_ranges(
            &[(0, 0, &TEST_COLUMN), (1, 1, &TEST_COLUMN)],
            &[
                vec!["A".to_owned(), "X".to_owned()],
                vec!["B".to_owned(), "X".to_owned()]
            ],
            0
        )?,
        vec![MergeRange::new(1, 1, 0, 1)]
    );
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn columns_follow_java_field_cache_order_and_selection_rules() {
    const SCHEMA: &[ExcelColumn] = &[
        ExcelColumn::new("third", "Third", Some(2), 0, None),
        ExcelColumn::new("late", "Late", None, 5, None),
        ExcelColumn::new("first", "First", None, 1, None),
        ExcelColumn::new("implicit", "Implicit", None, 0, None),
    ];
    let actual = ordered_columns(SCHEMA)
        .expect("valid schema")
        .into_iter()
        .map(|(physical, schema, column)| (physical, schema, column.field))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            (0, 3, "implicit"),
            (1, 2, "first"),
            (2, 0, "third"),
            (3, 1, "late")
        ]
    );

    let by_index = selected_columns(
        SCHEMA,
        &WriteOptions {
            include_column_indexes: Some(vec![2, 1]),
            order_by_include_column: true,
            ..WriteOptions::default()
        },
    )
    .expect("valid selected columns");
    assert_eq!(
        by_index
            .iter()
            .map(|(_, _, column)| column.field)
            .collect::<Vec<_>>(),
        vec!["third", "first"]
    );
    assert_eq!(
        by_index
            .iter()
            .map(|(physical, _, _)| *physical)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );

    let by_name = selected_columns(
        SCHEMA,
        &WriteOptions {
            include_column_field_names: Some(vec!["implicit".to_owned(), "first".to_owned()]),
            order_by_include_column: true,
            ..WriteOptions::default()
        },
    )
    .expect("valid selected columns");
    assert_eq!(
        by_name
            .iter()
            .map(|(_, _, column)| column.field)
            .collect::<Vec<_>>(),
        vec!["implicit", "first"]
    );

    let excluded = selected_columns(
        SCHEMA,
        &WriteOptions {
            exclude_column_indexes: vec![2],
            exclude_column_field_names: vec!["late".to_owned()],
            ..WriteOptions::default()
        },
    )
    .expect("valid selected columns");
    assert_eq!(
        excluded
            .iter()
            .map(|(_, _, column)| column.field)
            .collect::<Vec<_>>(),
        vec!["implicit", "first"]
    );

    let dynamic = selected_columns(
        crate::core::DynamicRow::schema(),
        &WriteOptions {
            dynamic_head: Some(vec![
                vec!["First".to_owned()],
                vec!["Second".to_owned()],
                vec!["Third".to_owned()],
            ]),
            include_column_indexes: Some(vec![2, 0]),
            exclude_column_indexes: vec![1],
            order_by_include_column: true,
            ..WriteOptions::default()
        },
    )
    .expect("valid dynamic columns");
    assert_eq!(
        dynamic
            .iter()
            .map(|(physical, source, column)| (*physical, *source, column.field))
            .collect::<Vec<_>>(),
        vec![(0, 2, ""), (1, 0, "")]
    );
    assert_eq!(
        selected_dynamic_head_paths(
            &dynamic,
            &[
                vec!["First".to_owned()],
                vec!["Second".to_owned()],
                vec!["Third".to_owned()],
            ],
        )
        .expect("selected head paths"),
        vec![vec!["Third".to_owned()], vec!["First".to_owned()]]
    );
    assert!(
        selected_dynamic_columns(
            3,
            &WriteOptions {
                include_column_field_names: Some(vec!["unknown".to_owned()]),
                ..WriteOptions::default()
            }
        )
        .is_empty()
    );
    assert_eq!(
        selected_dynamic_columns(
            2,
            &WriteOptions {
                order_by_include_column: true,
                ..WriteOptions::default()
            }
        )
        .iter()
        .map(|(physical, source, _)| (*physical, *source))
        .collect::<Vec<_>>(),
        vec![(0, 0), (1, 1)]
    );
}

