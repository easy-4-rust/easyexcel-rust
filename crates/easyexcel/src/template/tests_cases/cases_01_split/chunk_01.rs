#[test]
fn template_data_and_xml_escaping_are_deterministic() {
    let mut data = TemplateData::new().with("name", "Alice").with("count", 2);
    assert_eq!(
        data.insert("name", "Bob"),
        Some(CellValue::String("Alice".to_owned()))
    );
    assert_eq!(data.insert("new", "value"), None);
    assert_eq!(
        data.values().get("name"),
        Some(&CellValue::String("Bob".to_owned()))
    );
    assert_eq!(escape_xml("<&>\"' text"), "&lt;&amp;&gt;&quot;&apos; text");
    assert!(!contains_unescaped(r"\{users.name}", "{users."));
    assert!(contains_unescaped("{users.name}", "{users."));
    assert_eq!(TemplateData::default(), TemplateData::new());

    let owned = "owned".to_owned();
    let typed_date = NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date");
    let date_time = typed_date.and_hms_opt(12, 34, 56).expect("valid time");
    for value in [
        "text".into_template_value(),
        owned.clone().into_template_value(),
        (&owned).into_template_value(),
        true.into_template_value(),
        i8::MIN.into_template_value(),
        i16::MIN.into_template_value(),
        i32::MIN.into_template_value(),
        i64::MIN.into_template_value(),
        isize::MIN.into_template_value(),
        i128::MIN.into_template_value(),
        u8::MAX.into_template_value(),
        u16::MAX.into_template_value(),
        u32::MAX.into_template_value(),
        usize::MAX.into_template_value(),
        u64::MAX.into_template_value(),
        u128::MAX.into_template_value(),
        BigInt::from(i128::MAX).into_template_value(),
        1.25_f32.into_template_value(),
        2.5_f64.into_template_value(),
        BigDecimal::from(42).into_template_value(),
        typed_date.into_template_value(),
        date_time.into_template_value(),
        Some(7_i32).into_template_value(),
        Option::<i32>::None.into_template_value(),
        CellValue::Error("#N/A".to_owned()).into_template_value(),
    ] {
        assert!(matches!(value, CellValue::Empty) || !value.as_text().is_empty());
    }
}

#[test]
fn exact_placeholders_preserve_java_scalar_cell_types() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("typed-template.xlsx");
    let output = directory.path().join("typed-output.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    for (row, placeholder) in [
        "{string}",
        "{boolean}",
        "{integer}",
        "{float}",
        "{decimal}",
        "{date}",
        "{datetime}",
        "{error}",
        "{formula}",
        "{empty}",
        "value={integer}",
    ]
    .into_iter()
    .enumerate()
    {
        worksheet
            .write_string(u32::try_from(row).expect("small row"), 0, placeholder)
            .map_err(test_error)?;
    }
    workbook.save(&template).map_err(test_error)?;

    let date = NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date");
    fill_xlsx_template(
        &template,
        &output,
        &TemplateData::new()
            .with("string", "Alice")
            .with("boolean", true)
            .with("integer", 42_i64)
            .with("float", 5.25_f64)
            .with("decimal", BigDecimal::from(12345))
            .with("date", date)
            .with(
                "datetime",
                date.and_hms_opt(13, 14, 15).expect("valid time"),
            )
            .with("error", CellValue::Error("#N/A".to_owned()))
            .with("formula", CellValue::Formula("SUM(20,22)".to_owned()))
            .with("empty", Option::<String>::None),
    )?;

    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get_value((0, 0)),
        Some(&Data::String("Alice".to_owned()))
    );
    assert_eq!(range.get_value((1, 0)), Some(&Data::Bool(true)));
    assert_eq!(range.get_value((2, 0)), Some(&Data::Float(42.0)));
    assert_eq!(range.get_value((3, 0)), Some(&Data::Float(5.25)));
    assert_eq!(range.get_value((4, 0)), Some(&Data::Float(12345.0)));
    assert_eq!(
        range.get_value((5, 0)),
        Some(&Data::DateTimeIso("2026-07-17".to_owned()))
    );
    assert_eq!(
        range.get_value((6, 0)),
        Some(&Data::DateTimeIso("2026-07-17T13:14:15".to_owned()))
    );
    assert_eq!(
        range.get_value((7, 0)),
        Some(&Data::Error(calamine::CellErrorType::NA))
    );
    assert_eq!(
        range.get_value((10, 0)),
        Some(&Data::String("value=42".to_owned()))
    );

    let entries = load_entries(&output)?;
    let sheet = entries
        .iter()
        .find(|entry| entry.name == "xl/worksheets/sheet1.xml")
        .and_then(|entry| std::str::from_utf8(&entry.bytes).ok())
        .expect("typed worksheet");
    assert!(sheet.contains("<f>SUM(20,22)</f>"));
    assert!(sheet.contains("r=\"A10\"></c>"));
    Ok(())
}

#[test]
fn fills_java_official_simple_template_with_typed_number() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("java-simple.xlsx");
    let output = directory.path().join("java-simple-filled.xlsx");
    write_compressed_java_fixture(
        &template,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-demo-simple.xlsx.gz.b64"
        )),
    )?;

    let mut source: Xlsx<_> = open_workbook(&template).map_err(test_error)?;
    let source_range = source.worksheet_range("Sheet1").map_err(test_error)?;
    let name_coordinate = find_string_coordinate(&source_range, "{name}").expect("name marker");
    let number_coordinate =
        find_string_coordinate(&source_range, "{number}").expect("number marker");

    fill_xlsx_template(
        &template,
        &output,
        &TemplateData::new()
            .with("name", "张三")
            .with("number", 5.2_f64),
    )?;

    let mut result: Xlsx<_> = open_workbook(output).map_err(test_error)?;
    let range = result.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get_value(name_coordinate),
        Some(&Data::String("张三".to_owned()))
    );
    assert_eq!(range.get_value(number_coordinate), Some(&Data::Float(5.2)));
    Ok(())
}

#[test]
fn java_complex_fill_with_table_appends_summary_after_repeated_fill() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("java-complex-table.xlsx");
    let output = directory.path().join("java-complex-table-filled.xlsx");
    write_compressed_java_fixture(
        &template,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-demo-complex-fill-with-table.xlsx.gz.b64"
        )),
    )?;

    let first = [
        TemplateData::new().with("name", "A").with("number", 1),
        TemplateData::new().with("name", "B").with("number", 2),
        TemplateData::new().with("name", "C").with("number", 3),
    ];
    let second = [
        TemplateData::new().with("name", "D").with("number", 4),
        TemplateData::new().with("name", "E").with("number", 5),
        TemplateData::new().with("name", "F").with("number", 6),
    ];
    let mut writer = ExcelTemplateWriter::new(&template, &output)?;
    writer
        .fill_list(&FillWrapper::new(first), FillConfig::new())?
        .fill_list(&FillWrapper::new(second), FillConfig::new())?
        .fill(&TemplateData::new().with("date", "2019年10月9日13:28:28"))?
        .write_rows([vec![
            CellValue::Empty,
            CellValue::Empty,
            CellValue::Empty,
            CellValue::String("统计:1000".to_owned()),
        ]])?
        .finish()?;

    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    let first_data_row = find_string_coordinate(&range, "A")
        .map(|(row, _)| usize::try_from(row).expect("row index fits usize"))
        .expect("first filled collection row");
    for (offset, (name, number)) in [
        ("A", 1.0),
        ("B", 2.0),
        ("C", 3.0),
        ("D", 4.0),
        ("E", 5.0),
        ("F", 6.0),
    ]
    .into_iter()
    .enumerate()
    {
        let row = u32::try_from(first_data_row + offset).expect("small row");
        assert_eq!(
            range.get_value((row, 0)),
            Some(&Data::String(name.to_owned()))
        );
        assert_eq!(range.get_value((row, 1)), Some(&Data::Float(number)));
    }
    let summary_row = u32::try_from(first_data_row + 6).expect("small row");
    assert_eq!(
        range.get_value((summary_row, 3)),
        Some(&Data::String("统计:1000".to_owned()))
    );
    Ok(())
}

#[test]
fn template_reader_and_owned_output_follow_java_default_close_lifecycle() -> Result<()> {
    let (_directory, template) = template_fixture()?;
    let input_dropped = Arc::new(AtomicBool::new(false));
    let input = DropReader::new(fs::read(&template)?, Arc::clone(&input_dropped));
    let state = SharedOutput::new(false, false);
    let stream = ExcelOutputStream::new(state.clone());
    let observer = stream.clone();

    let mut writer = ExcelTemplateWriter::from_reader_to_output_stream(input, stream)?;
    assert!(input_dropped.load(Ordering::SeqCst));
    assert!(format!("{writer:?}").contains("owned stream"));
    assert_eq!(
        worksheet_path(&writer.entries, &TemplateSheet::first())?,
        "xl/worksheets/sheet1.xml"
    );
    writer.fill(&TemplateData::new().with("name", "stream"))?;
    assert_eq!(
        writer.sheets[0].scalar.values().get("name"),
        Some(&CellValue::String("stream".to_owned()))
    );
    writer.finish()?;

    assert!(observer.is_closed());
    let bytes = state.0.lock().expect("output state lock").bytes.clone();
    let mut workbook = Xlsx::new(Cursor::new(bytes)).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get((0, 0)),
        Some(&Data::String("Hello stream".to_owned()))
    );
    Ok(())
}

#[test]
fn template_path_to_owned_output_can_retain_stream() -> Result<()> {
    let (_directory, template) = template_fixture()?;
    let state = SharedOutput::new(false, false);
    let stream = ExcelOutputStream::new(state.clone());
    let observer = stream.clone();
    let mut writer =
        ExcelTemplateWriter::to_output_stream(&template, stream)?.auto_close_stream(false);

    writer.finish()?;
    assert!(!observer.is_closed());
    assert!(observer.with_inner(|_| ()).is_some());
    observer.close()?;
    assert!(observer.is_closed());
    assert!(!state.0.lock().expect("output state lock").bytes.is_empty());
    Ok(())
}

#[test]
fn template_borrowed_output_remains_usable_for_path_and_reader_inputs() -> Result<()> {
    let (directory, template) = template_fixture()?;
    let reader_output = directory.path().join("reader-output.xlsx");
    let mut path_writer =
        ExcelTemplateWriter::from_reader(Cursor::new(fs::read(&template)?), &reader_output)?;
    assert!(format!("{path_writer:?}").contains("path"));
    path_writer.finish()?;
    Xlsx::new(Cursor::new(fs::read(reader_output)?)).map_err(test_error)?;

    let mut first = Cursor::new(Vec::new());
    {
        let mut writer = ExcelTemplateWriter::to_writer(&template, &mut first)?;
        assert!(format!("{writer:?}").contains("borrowed stream"));
        writer.finish()?;
        writer.finish()?;
    }
    let first_bytes = first.get_ref().clone();
    first.write_all(b"caller-owned")?;
    assert!(first.get_ref().ends_with(b"caller-owned"));
    Xlsx::new(Cursor::new(first_bytes)).map_err(test_error)?;

    let mut second = Cursor::new(Vec::new());
    ExcelTemplateWriter::from_reader_to_writer(Cursor::new(fs::read(&template)?), &mut second)?
        .finish()?;
    Xlsx::new(Cursor::new(second.into_inner())).map_err(test_error)?;
    Ok(())
}

