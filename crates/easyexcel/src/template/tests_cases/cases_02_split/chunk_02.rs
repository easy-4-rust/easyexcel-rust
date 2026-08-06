#[test]
#[allow(clippy::too_many_lines)]
fn stateful_template_writer_matches_java_repeated_and_composite_fill() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("composite-template.xlsx");
    let output = directory.path().join("composite-output.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .write_string(0, 0, "Report {date}")
        .map_err(test_error)?;
    worksheet
        .write_string(0, 1, r"\{date\}")
        .map_err(test_error)?;
    worksheet
        .write_string(1, 0, "{data1.name}")
        .map_err(test_error)?;
    worksheet
        .write_string(3, 0, "{data2.name}")
        .map_err(test_error)?;
    worksheet.write_string(6, 0, "Footer").map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;

    let horizontal = FillConfig::new().direction(FillDirection::Horizontal);
    let vertical = FillConfig::new().force_new_row(true);
    let mut writer = ExcelTemplateWriter::new(&template, &output)?;
    assert!(!writer.is_finished());
    writer
        .fill(&TemplateData::new().with("date", "old"))?
        .fill_list(
            &FillWrapper::named(
                "data1",
                [
                    TemplateData::new().with("name", "A"),
                    TemplateData::new().with("name", "B"),
                ],
            ),
            horizontal,
        )?
        .fill_list(
            &FillWrapper::named("data1", [TemplateData::new().with("name", "C")]),
            horizontal,
        )?
        .fill_list(
            &FillWrapper::named("data2", [TemplateData::new().with("name", "X")]),
            vertical,
        )?
        .fill_list(
            &FillWrapper::named(
                "data2",
                [
                    TemplateData::new().with("name", "Y"),
                    TemplateData::new().with("name", "Z"),
                ],
            ),
            vertical,
        )?
        .fill_list(&FillWrapper::default(), FillConfig::new())?
        .fill(&TemplateData::new().with("date", 2026))?
        .write_rows([vec![
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::String("统计:1000".to_owned()),
        ]])?;
    writer.finish()?;
    writer.finish()?;
    assert!(writer.is_finished());
    assert!(writer.fill(&TemplateData::new()).is_err());
    assert!(writer.write_rows([Vec::<CellValue>::new()]).is_err());
    assert!(
        writer
            .fill_list(&FillWrapper::default(), FillConfig::new())
            .is_err()
    );

    let mut workbook: Xlsx<_> = open_workbook(output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get_value((0, 0)),
        Some(&Data::String("Report 2026".to_owned()))
    );
    assert_eq!(
        range.get_value((0, 1)),
        Some(&Data::String("{date}".to_owned()))
    );
    for (column, expected) in [(0_u32, "A"), (1, "B"), (2, "C")] {
        assert_eq!(
            range.get_value((1, column)),
            Some(&Data::String(expected.to_owned()))
        );
    }
    for (row, expected) in [(3_u32, "X"), (4, "Y"), (5, "Z")] {
        assert_eq!(
            range.get_value((row, 0)),
            Some(&Data::String(expected.to_owned()))
        );
    }
    assert_eq!(
        range.get_value((8, 0)),
        Some(&Data::String("Footer".to_owned()))
    );
    assert_eq!(
        range.get_value((9, 3)),
        Some(&Data::String("统计:1000".to_owned()))
    );
    Ok(())
}

#[test]
fn fills_java_official_composite_template_across_all_analysis_cells() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("java-composite.xlsx");
    let output = directory.path().join("java-composite-filled.xlsx");
    write_java_composite_fixture(&template)?;

    let horizontal = FillConfig::new().direction(FillDirection::Horizontal);
    let mut writer = ExcelTemplateWriter::new(&template, &output)?;
    for row in [
        TemplateData::new().with("name", "A").with("number", 1),
        TemplateData::new().with("name", "B").with("number", 2),
    ] {
        writer.fill_list(&FillWrapper::named("data1", [row]), horizontal)?;
    }
    for row in [
        TemplateData::new().with("name", "X").with("number", 10),
        TemplateData::new().with("name", "Y").with("number", 20),
    ] {
        writer.fill_list(&FillWrapper::named("data2", [row]), FillConfig::new())?;
    }
    for row in [
        TemplateData::new().with("name", "P").with("number", 100),
        TemplateData::new().with("name", "Q").with("number", 200),
    ] {
        writer.fill_list(&FillWrapper::named("data3", [row]), FillConfig::new())?;
    }
    writer
        .fill(&TemplateData::new().with("date", "2026-07-17"))?
        .finish()?;

    let mut workbook: Xlsx<_> = open_workbook(output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    for (coordinate, expected) in [
        ((0, 2), "A"),
        ((0, 3), "B"),
        ((2, 2), "A"),
        ((2, 3), "B"),
        ((4, 0), "时间：2026-07-17"),
        ((8, 0), "X"),
        ((9, 0), "Y"),
        ((10, 3), "P"),
        ((11, 3), "Q"),
    ] {
        assert_eq!(
            range.get_value(coordinate),
            Some(&Data::String(expected.to_owned())),
            "coordinate {coordinate:?}"
        );
    }
    for (coordinate, expected) in [
        ((1, 2), 1.0),
        ((1, 3), 2.0),
        ((3, 2), 1.0),
        ((3, 3), 2.0),
        ((8, 1), 10.0),
        ((9, 1), 20.0),
        ((10, 4), 100.0),
        ((11, 4), 200.0),
    ] {
        assert_eq!(
            range.get_value(coordinate),
            Some(&Data::Float(expected)),
            "coordinate {coordinate:?}"
        );
    }
    Ok(())
}

#[test]
fn expands_vertical_named_rows_and_shifts_footer() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("vertical-template.xlsx");
    let output = directory.path().join("vertical-output.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Name").map_err(test_error)?;
    worksheet
        .write_string(1, 0, "{users.name}")
        .map_err(test_error)?;
    worksheet
        .write_string(1, 1, "Age {users.age}")
        .map_err(test_error)?;
    worksheet
        .write_string(1, 2, "Template static")
        .map_err(test_error)?;
    worksheet.write_string(2, 0, "Footer").map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;

    let wrapper = FillWrapper::named(
        "users",
        [
            TemplateData::new().with("name", "Alice").with("age", 20),
            TemplateData::new().with("name", "Bob").with("age", 30),
            TemplateData::new().with("name", "Carol").with("age", 40),
        ],
    );
    fill_xlsx_template_list(
        &template,
        &output,
        &wrapper,
        FillConfig::new().force_new_row(true),
    )?;

    let mut workbook: Xlsx<_> = open_workbook(output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get_value((1, 0)),
        Some(&Data::String("Alice".to_owned()))
    );
    assert_eq!(
        range.get_value((2, 0)),
        Some(&Data::String("Bob".to_owned()))
    );
    assert_eq!(
        range.get_value((3, 1)),
        Some(&Data::String("Age 40".to_owned()))
    );
    assert_eq!(range.get_value((2, 2)), Some(&Data::Empty));
    assert_eq!(range.get_value((3, 2)), Some(&Data::Empty));
    assert_eq!(
        range.get_value((4, 0)),
        Some(&Data::String("Footer".to_owned()))
    );
    Ok(())
}

#[test]
fn default_vertical_fill_reuses_existing_rows_without_copying_static_cells() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("reuse-template.xlsx");
    let output = directory.path().join("reuse-output.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Name").map_err(test_error)?;
    worksheet
        .write_string(1, 0, "{.name}")
        .map_err(test_error)?;
    worksheet
        .write_string(1, 1, "Template static")
        .map_err(test_error)?;
    worksheet.write_string(2, 0, "old").map_err(test_error)?;
    worksheet
        .write_string(2, 1, "Preserve")
        .map_err(test_error)?;
    worksheet.write_string(3, 0, "Footer").map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;

    fill_xlsx_template_list(
        &template,
        &output,
        &FillWrapper::new([
            TemplateData::new().with("name", "Alice"),
            TemplateData::new().with("name", "Bob"),
        ]),
        FillConfig::new(),
    )?;

    let mut workbook: Xlsx<_> = open_workbook(output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get_value((1, 0)),
        Some(&Data::String("Alice".to_owned()))
    );
    assert_eq!(
        range.get_value((1, 1)),
        Some(&Data::String("Template static".to_owned()))
    );
    assert_eq!(
        range.get_value((2, 0)),
        Some(&Data::String("Bob".to_owned()))
    );
    assert_eq!(
        range.get_value((2, 1)),
        Some(&Data::String("Preserve".to_owned()))
    );
    assert_eq!(
        range.get_value((3, 0)),
        Some(&Data::String("Footer".to_owned()))
    );
    Ok(())
}

#[test]
fn expands_horizontal_unnamed_cells_and_can_drop_style() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("horizontal-template.xlsx");
    let output = directory.path().join("horizontal-output.xlsx");
    let mut workbook = Workbook::new();
    workbook
        .add_worksheet()
        .write_string(0, 0, "{.name}")
        .map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;

    let wrapper = FillWrapper::new([
        TemplateData::new().with("name", "A"),
        TemplateData::new().with("name", "B"),
        TemplateData::new().with("name", "C"),
    ]);
    fill_xlsx_template_list(
        &template,
        &output,
        &wrapper,
        FillConfig::new()
            .direction(FillDirection::Horizontal)
            .auto_style(false),
    )?;

    let mut workbook: Xlsx<_> = open_workbook(output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(range.get_value((0, 0)), Some(&Data::String("A".to_owned())));
    assert_eq!(range.get_value((0, 1)), Some(&Data::String("B".to_owned())));
    assert_eq!(range.get_value((0, 2)), Some(&Data::String("C".to_owned())));
    Ok(())
}

