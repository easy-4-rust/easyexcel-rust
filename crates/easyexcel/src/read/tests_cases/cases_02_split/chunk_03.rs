#[test]
fn sheet_selection_supports_first_index_name_all_and_missing_values() -> Result<()> {
    let (_directory, path) = workbook_fixture()?;
    let workbook: Xlsx<_> = open_workbook(path).map_err(test_error)?;
    assert_eq!(
        selected_sheet_names(&workbook, &SheetSelector::First, true)?,
        vec![(0, "First".to_owned())]
    );
    assert_eq!(
        selected_sheet_names(&workbook, &SheetSelector::Index(1), true)?,
        vec![(1, "Second".to_owned())]
    );
    assert_eq!(
        selected_sheet_names(&workbook, &SheetSelector::Name("Second".to_owned()), true,)?,
        vec![(1, "Second".to_owned())]
    );
    assert_eq!(
        selected_sheet_names(&workbook, &SheetSelector::All, true)?.len(),
        2
    );
    assert!(selected_sheet_names(&workbook, &SheetSelector::Index(2), true).is_err());
    assert!(
        selected_sheet_names(&workbook, &SheetSelector::Name("Missing".to_owned()), true,).is_err()
    );
    assert!(select_sheet_names(Vec::new(), &SheetSelector::First, true).is_err());
    assert_eq!(
        select_sheet_names(
            vec![" First ".to_owned()],
            &SheetSelector::Name("First".to_owned()),
            true,
        )?,
        vec![(0, " First ".to_owned())]
    );
    assert!(
        select_sheet_names(
            vec![" First ".to_owned()],
            &SheetSelector::Name("First".to_owned()),
            false,
        )
        .is_err()
    );

    let legacy = || {
        vec![
            ("First".to_owned(), Range::empty()),
            ("Second".to_owned(), Range::empty()),
        ]
    };
    let first = select_xls_sheets(legacy(), &SheetSelector::First, true)?;
    assert_eq!((first[0].0, first[0].1.as_str()), (0, "First"));
    let second = select_xls_sheets(legacy(), &SheetSelector::Index(1), true)?;
    assert_eq!((second[0].0, second[0].1.as_str()), (1, "Second"));
    let named = select_xls_sheets(legacy(), &SheetSelector::Name("Second".to_owned()), true)?;
    assert_eq!((named[0].0, named[0].1.as_str()), (1, "Second"));
    assert_eq!(
        select_xls_sheets(legacy(), &SheetSelector::All, true)?.len(),
        2
    );
    assert!(select_xls_sheets(legacy(), &SheetSelector::Index(2), true).is_err());
    assert!(
        select_xls_sheets(legacy(), &SheetSelector::Name("Missing".to_owned()), true,).is_err()
    );
    assert!(select_xls_sheets(Vec::new(), &SheetSelector::First, true).is_err());
    Ok(())
}

