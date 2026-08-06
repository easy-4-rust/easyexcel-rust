#[test]
fn ordered_handlers_observe_transform_and_skip_the_full_lifecycle() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("handled.xlsx");
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![
        Box::new(RecordingHandler {
            order: 10,
            events: Rc::clone(&events),
        }),
        Box::new(RecordingHandler {
            order: -10,
            events: Rc::clone(&events),
        }),
    ];
    write_xlsx_with_handlers::<EveryCell, _>(
        &path,
        &WriteOptions::default(),
        vec![every_cell()],
        &mut handlers,
    )?;

    let actual = events.borrow();
    assert!(actual[0].starts_with("-10:before_workbook:"));
    assert!(actual[1].starts_with("10:before_workbook:"));
    assert!(actual.iter().any(|event| event == "-10:after_workbook"));
    assert!(actual.iter().any(|event| event == "10:after_workbook"));
    drop(actual);

    let mut workbook: Xlsx<_> = open_workbook(path).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(range.get_value((0, 0)), None);
    assert_eq!(range.get_value((0, 1)), Some(&Data::Bool(true)));
    assert_eq!(
        range.get_value((1, 1)),
        Some(&Data::String("transformed".to_owned()))
    );
    assert_eq!(range.get_value((1, 2)), Some(&Data::Empty));
    Ok(())
}

