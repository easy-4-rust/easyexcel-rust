#[test]
fn template_stream_failures_are_propagated_and_owned_stream_is_closed() -> Result<()> {
    let (_directory, template) = template_fixture()?;
    let bytes = fs::read(&template)?;
    assert!(
        ExcelTemplateWriter::from_reader(
            FaultyIo::reading(bytes.clone(), 0),
            template.with_extension("read-error.xlsx")
        )
        .is_err()
    );

    let missing = template.with_extension("missing.xlsx");
    let mut constructor_output = Cursor::new(Vec::new());
    assert!(ExcelTemplateWriter::to_writer(&missing, &mut constructor_output).is_err());
    assert!(
        ExcelTemplateWriter::from_reader_to_writer(
            FaultyIo::reading(bytes.clone(), 0),
            &mut constructor_output
        )
        .is_err()
    );
    assert!(
        ExcelTemplateWriter::to_output_stream(
            &missing,
            ExcelOutputStream::new(SharedOutput::new(false, false))
        )
        .is_err()
    );
    assert!(
        ExcelTemplateWriter::from_reader_to_output_stream(
            FaultyIo::reading(bytes.clone(), 0),
            ExcelOutputStream::new(SharedOutput::new(false, false))
        )
        .is_err()
    );
    ExcelTemplateWriter::from_reader(
        FaultyIo::reading(bytes.clone(), usize::MAX),
        template.with_extension("fault-reader-success.xlsx"),
    )?
    .finish()?;
    assert!(
        ExcelTemplateWriter::from_reader(
            Cursor::new(b"not-a-zip".to_vec()),
            template.with_extension("invalid.xlsx")
        )
        .is_err()
    );

    for (fail_write, fail_flush) in [(true, false), (false, true)] {
        let state = SharedOutput::new(fail_write, fail_flush);
        let stream = ExcelOutputStream::new(state);
        let observer = stream.clone();
        let mut writer =
            ExcelTemplateWriter::from_reader_to_output_stream(Cursor::new(bytes.clone()), stream)?;
        assert!(writer.finish().is_err());
        assert!(observer.is_closed());
    }

    for (fail_write, fail_flush) in [(true, false), (false, true)] {
        let mut output = SharedOutput::new(fail_write, fail_flush);
        let mut writer = ExcelTemplateWriter::to_writer(&template, &mut output)?;
        assert!(writer.finish().is_err());
    }

    let entries = load_entries(&template)?;
    let wrong_type = write_entries_to(Box::new(FaultyIo::writing(usize::MAX)), &entries)?;
    assert!(archive_output_bytes(wrong_type).is_err());

    let invalid_entries = [TemplateEntry {
        name: "invalid.bin".to_owned(),
        is_dir: false,
        compression: CompressionMethod::AES,
        unix_mode: None,
        bytes: vec![1],
    }];
    assert!(encode_entries(&invalid_entries).is_err());
    let mut borrowed = Cursor::new(Vec::new());
    assert!(
        write_entries_to_output(
            &mut TemplateOutput::Borrowed(&mut borrowed),
            &invalid_entries,
            true
        )
        .is_err()
    );
    let invalid_stream = ExcelOutputStream::new(SharedOutput::new(false, false));
    let invalid_observer = invalid_stream.clone();
    assert!(
        write_entries_to_output(
            &mut TemplateOutput::Owned(Box::new(invalid_stream)),
            &invalid_entries,
            true
        )
        .is_err()
    );
    assert!(invalid_observer.is_closed());
    Ok(())
}

#[test]
fn stateful_template_writer_isolates_scalar_list_and_rows_by_sheet() -> Result<()> {
    let (directory, template) = multi_sheet_template_fixture()?;
    let output = directory.path().join("multi-sheet-filled.xlsx");
    let details = TemplateSheet::name("明细");
    let rows = FillWrapper::named(
        "items",
        [
            TemplateData::new().with("name", "A").with("value", 1),
            TemplateData::new().with("name", "B").with("value", 2),
        ],
    );

    let mut writer = ExcelTemplateWriter::new(&template, &output)?;
    writer
        .fill(&TemplateData::new().with("title", "首页"))?
        .fill_on_sheet(&details, &TemplateData::new().with("title", "详情"))?
        .fill_list_on_sheet(&details, &rows, FillConfig::new())?
        .fill_on_sheet(
            &TemplateSheet::index(1),
            &TemplateData::new().with("title", "详情覆盖"),
        )?
        .write_rows_on_sheet(
            &TemplateSheet::index(1),
            [vec![
                CellValue::String("合计".to_owned()),
                CellValue::Int(3),
            ]],
        )?
        .finish()?;

    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let summary = workbook.worksheet_range("摘要").map_err(test_error)?;
    assert_eq!(summary.get((0, 0)), Some(&Data::String("首页".to_owned())));
    let details = workbook.worksheet_range("明细").map_err(test_error)?;
    assert_eq!(
        details.get((0, 0)),
        Some(&Data::String("详情覆盖".to_owned()))
    );
    assert_eq!(details.get((1, 0)), Some(&Data::String("A".to_owned())));
    assert_eq!(details.get((1, 1)), Some(&Data::Float(1.0)));
    assert_eq!(details.get((2, 0)), Some(&Data::String("B".to_owned())));
    assert_eq!(details.get((2, 1)), Some(&Data::Float(2.0)));
    assert_eq!(details.get((3, 0)), Some(&Data::String("合计".to_owned())));
    assert_eq!(details.get((3, 1)), Some(&Data::Float(3.0)));
    let untouched = workbook.worksheet_range("未处理").map_err(test_error)?;
    assert_eq!(
        untouched.get((0, 0)),
        Some(&Data::String("{title}".to_owned()))
    );
    Ok(())
}

#[test]
fn repeated_fill_reuses_java_cursor_when_direction_changes() -> Result<()> {
    let directory = tempdir().map_err(test_error)?;
    let template = directory.path().join("direction-change-template.xlsx");
    let output = directory.path().join("direction-change-filled.xlsx");
    let mut workbook = Workbook::new();
    workbook
        .add_worksheet()
        .set_name("纵转横")
        .map_err(test_error)?
        .write_string(0, 0, "{items.name}")
        .map_err(test_error)?;
    workbook
        .add_worksheet()
        .set_name("横转纵")
        .map_err(test_error)?
        .write_string(0, 0, "{items.name}")
        .map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;

    let first = FillWrapper::named(
        "items",
        [
            TemplateData::new().with("name", "A"),
            TemplateData::new().with("name", "B"),
        ],
    );
    let second = FillWrapper::named(
        "items",
        [
            TemplateData::new().with("name", "C"),
            TemplateData::new().with("name", "D"),
        ],
    );
    let mut writer = ExcelTemplateWriter::new(&template, &output)?;
    writer
        .fill_list_on_sheet(&TemplateSheet::name("纵转横"), &first, FillConfig::new())?
        .fill_list_on_sheet(
            &TemplateSheet::name("纵转横"),
            &second,
            FillConfig::new().direction(FillDirection::Horizontal),
        )?
        .fill_list_on_sheet(
            &TemplateSheet::name("横转纵"),
            &first,
            FillConfig::new().direction(FillDirection::Horizontal),
        )?
        .fill_list_on_sheet(&TemplateSheet::name("横转纵"), &second, FillConfig::new())?
        .finish()?;

    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let vertical_then_horizontal = workbook.worksheet_range("纵转横").map_err(test_error)?;
    assert_eq!(
        vertical_then_horizontal.get((0, 0)),
        Some(&Data::String("A".to_owned()))
    );
    assert_eq!(
        vertical_then_horizontal.get((1, 0)),
        Some(&Data::String("B".to_owned()))
    );
    assert_eq!(
        vertical_then_horizontal.get((0, 2)),
        Some(&Data::String("C".to_owned()))
    );
    assert_eq!(
        vertical_then_horizontal.get((0, 3)),
        Some(&Data::String("D".to_owned()))
    );

    let horizontal_then_vertical = workbook.worksheet_range("横转纵").map_err(test_error)?;
    assert_eq!(
        horizontal_then_vertical.get((0, 0)),
        Some(&Data::String("A".to_owned()))
    );
    assert_eq!(
        horizontal_then_vertical.get((0, 1)),
        Some(&Data::String("B".to_owned()))
    );
    assert_eq!(
        horizontal_then_vertical.get((2, 0)),
        Some(&Data::String("C".to_owned()))
    );
    assert_eq!(
        horizontal_then_vertical.get((3, 0)),
        Some(&Data::String("D".to_owned()))
    );
    Ok(())
}

#[test]
fn repeated_fill_applies_each_calls_force_row_and_auto_style_config() -> Result<()> {
    let directory = tempdir().map_err(test_error)?;
    let template = directory.path().join("config-change-template.xlsx");
    let output = directory.path().join("config-change-filled.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .write_string_with_format(0, 0, "{items.name}", &Format::new().set_bold())
        .map_err(test_error)?;
    worksheet.write_string(1, 0, "Footer").map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;

    let mut writer = ExcelTemplateWriter::new(&template, &output)?;
    writer
        .fill_list(
            &FillWrapper::named("items", [TemplateData::new().with("name", "A")]),
            FillConfig::new(),
        )?
        .fill_list(
            &FillWrapper::named(
                "items",
                [
                    TemplateData::new().with("name", "B"),
                    TemplateData::new().with("name", "C"),
                ],
            ),
            FillConfig::new().force_new_row(true).auto_style(false),
        )?
        .finish()?;

    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    for (row, value) in [(0, "A"), (1, "B"), (2, "C"), (3, "Footer")] {
        assert_eq!(range.get((row, 0)), Some(&Data::String(value.to_owned())));
    }

    let entries = load_entries(&output)?;
    let sheet = entries
        .iter()
        .find(|entry| entry.name == "xl/worksheets/sheet1.xml")
        .expect("sheet1 exists");
    let xml = std::str::from_utf8(&sheet.bytes).map_err(test_error)?;
    let style = |reference| {
        all_cells(xml)
            .into_iter()
            .find(|(_, _, cell)| attribute_value(cell, "r") == Some(reference))
            .and_then(|(_, _, cell)| attribute_value(cell, "s"))
    };
    assert!(style("A1").is_some());
    assert_eq!(style("A2"), None);
    assert_eq!(style("A3"), None);
    Ok(())
}

