#[test]
#[allow(clippy::too_many_lines)]
fn facade_executes_event_sync_and_iterator_workflows() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("values.xlsx");
    let rows = vec![Value("one".to_owned()), Value("two".to_owned())];
    EasyExcel::write::<Value>(&path)
        .sheet("Values")
        .freeze_head(true)
        .do_write_iter(rows.clone())?;

    let actual = EasyExcel::read_sync::<Value>(&path)
        .sheet("Values".to_owned())
        .do_read_sync()?;
    assert_eq!(actual, rows);

    let csv = directory.path().join("values.CSV");
    EasyExcel::write::<Value>(&csv).do_write(rows.clone())?;
    assert_eq!(EasyExcel::read_sync::<Value>(&csv).do_read_sync()?, rows);
    EasyExcel::read::<Value, _>(&csv, Listener::default())
        .sheet("CsvSheet")
        .do_read()?;

    let gbk_csv = directory.path().join("values-gbk.csv");
    let chinese = vec![Value("姓名".repeat(5_000))];
    EasyExcel::write::<Value>(&gbk_csv)
        .charset("GBK")
        .with_bom(false)
        .do_write(chinese.clone())?;
    assert_eq!(
        EasyExcel::read_sync::<Value>(&gbk_csv)
            .charset("gbk")
            .do_read_sync()?,
        chinese
    );
    EasyExcel::read::<Value, _>(&gbk_csv, Listener::default())
        .charset("GBK")
        .do_read()?;
    assert!(matches!(
        EasyExcel::write::<Value>(directory.path().join("protected.csv"))
            .password("secret")
            .do_write(rows.clone()),
        Err(ExcelError::Unsupported(_))
    ));

    let encrypted = directory.path().join("protected.xlsx");
    EasyExcel::write::<Value>(&encrypted)
        .password("123456")
        .do_write(rows.clone())?;
    assert_eq!(
        &fs::read(&encrypted)?[..8],
        &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
    );
    assert_eq!(
        EasyExcel::read_sync::<Value>(&encrypted)
            .password("123456")
            .do_read_sync()?,
        rows
    );
    assert!(
        EasyExcel::read_sync::<Value>(&encrypted)
            .password("wrong")
            .do_read_sync()
            .is_err()
    );
    assert!(
        EasyExcel::read_sync::<Value>(&encrypted)
            .do_read_sync()
            .is_err()
    );
    let invalid_encrypted = directory.path().join("invalid-encrypted.xlsx");
    fs::write(
        &invalid_encrypted,
        [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
    )?;
    assert!(
        EasyExcel::read_sync::<Value>(&invalid_encrypted)
            .password("123456")
            .do_read_sync()
            .is_err()
    );
    assert_eq!(
        EasyExcel::read_sync::<Value>(&path)
            .password("ignored-for-plain-xlsx")
            .sheet("Values")
            .do_read_sync()?,
        rows
    );
    assert!(
        EasyExcel::read_sync::<Value>(&path)
            .sheet(99_usize)
            .do_read_sync()
            .is_err()
    );
    assert!(
        EasyExcel::read::<Value, _>(&path, FailingListener)
            .do_read()
            .is_err()
    );

    EasyExcel::read::<Value, _>(&path, Listener::default())
        .all_sheets()
        .do_read()?;

    let no_head = directory.path().join("no-head.xlsx");
    EasyExcel::write::<Value>(&no_head)
        .need_head(false)
        .constant_memory(true)
        .do_write(rows.clone())?;
    assert_eq!(
        EasyExcel::read_sync::<Value>(&no_head)
            .head_row_number(0)
            .do_read_sync()?
            .len(),
        2
    );

    let multi = directory.path().join("multi.xlsx");
    let first = EasyExcel::writer_sheet::<Value>("First").freeze_head(true);
    let second = EasyExcel::writer_sheet::<Value>("Second")
        .need_head(false)
        .constant_memory(true);
    let mut writer = EasyExcel::write::<Value>(&multi)
        .register_write_handler(NoopWriteHandler)
        .build();
    writer
        .write(vec![Value("first".to_owned())], &first)?
        .write(vec![Value("second".to_owned())], &second)?;
    writer.finish()?;
    assert_eq!(
        EasyExcel::read_sync::<Value>(&multi)
            .sheet("First")
            .do_read_sync()?,
        vec![Value("first".to_owned())]
    );
    assert_eq!(
        EasyExcel::read_sync::<Value>(&multi)
            .sheet("Second")
            .head_row_number(0)
            .do_read_sync()?,
        vec![Value("second".to_owned())]
    );

    let encrypted_multi = directory.path().join("encrypted-multi.xlsx");
    let mut encrypted_writer = EasyExcel::write::<Value>(&encrypted_multi)
        .password("stateful")
        .build();
    encrypted_writer.write(rows.clone(), &first)?.finish()?;
    assert_eq!(
        EasyExcel::read_sync::<Value>(&encrypted_multi)
            .password("stateful")
            .sheet("First")
            .do_read_sync()?,
        rows
    );

    let template = directory.path().join("template.xlsx");
    let filled = directory.path().join("filled.xlsx");
    EasyExcel::write::<Value>(&template)
        .need_head(false)
        .do_write(vec![Value("Hello {name}".to_owned())])?;
    EasyExcel::fill_template(
        &template,
        &filled,
        &TemplateData::new().with("name", "Rust"),
    )?;
    assert_eq!(
        EasyExcel::read_sync::<Value>(&filled)
            .head_row_number(0)
            .do_read_sync()?,
        vec![Value("Hello Rust".to_owned())]
    );

    let typed_template = directory.path().join("typed-template.xlsx");
    let typed_filled = directory.path().join("typed-filled.xlsx");
    EasyExcel::write::<Value>(&typed_template)
        .need_head(false)
        .do_write(vec![Value("{number}".to_owned())])?;
    EasyExcel::fill_template(
        &typed_template,
        &typed_filled,
        &TemplateData::new().with("number", BigDecimal::from(42)),
    )?;
    assert_eq!(
        EasyExcel::read_sync::<Value>(&typed_filled)
            .head_row_number(0)
            .do_read_sync()?,
        vec![Value("42".to_owned())]
    );

    let list_template = directory.path().join("list-template.xlsx");
    let list_filled = directory.path().join("list-filled.xlsx");
    EasyExcel::write::<Value>(&list_template)
        .need_head(false)
        .do_write(vec![Value("{.name}".to_owned())])?;
    EasyExcel::fill_template_list(
        &list_template,
        &list_filled,
        &FillWrapper::new([
            TemplateData::new().with("name", "one"),
            TemplateData::new().with("name", "two"),
        ]),
        FillConfig::new(),
    )?;
    assert_eq!(
        EasyExcel::read_sync::<Value>(&list_filled)
            .head_row_number(0)
            .do_read_sync()?,
        vec![Value("one".to_owned()), Value("two".to_owned())]
    );

    let repeated_filled = directory.path().join("list-repeated-filled.xlsx");
    let mut template_writer =
        EasyExcel::template_writer(list_template.clone(), repeated_filled.clone())?;
    template_writer
        .fill(&TemplateData::new())?
        .fill_list(
            &FillWrapper::new([TemplateData::new().with("name", "first")]),
            FillConfig::new(),
        )?
        .fill_list(
            &FillWrapper::new([TemplateData::new().with("name", "second")]),
            FillConfig::new(),
        )?
        .fill_list(&FillWrapper::default(), FillConfig::new())?
        .write_rows([vec![CellValue::String("summary".to_owned())]])?;
    template_writer.fill_list(
        &FillWrapper::new([TemplateData::new().with("name", "horizontal")]),
        FillConfig::new().direction(FillDirection::Horizontal),
    )?;
    template_writer.finish()?;
    template_writer.finish()?;
    assert!(template_writer.fill(&TemplateData::new()).is_err());
    assert!(
        template_writer
            .write_rows([Vec::<CellValue>::new()])
            .is_err()
    );
    assert!(
        template_writer
            .fill_list(&FillWrapper::default(), FillConfig::new())
            .is_err()
    );
    assert_eq!(
        EasyExcel::read_sync::<Value>(&repeated_filled)
            .head_row_number(0)
            .do_read_sync()?,
        vec![
            Value("first".to_owned()),
            Value("second".to_owned()),
            Value("summary".to_owned())
        ]
    );
    assert!(
        EasyExcel::template_writer(
            directory.path().join("missing-template.xlsx"),
            directory.path().join("missing-output.xlsx"),
        )
        .is_err()
    );
    assert!(
        EasyExcel::fill_template(
            directory.path().join("missing-template.xlsx"),
            directory.path().join("missing-output.xlsx"),
            &TemplateData::new(),
        )
        .is_err()
    );
    assert!(
        EasyExcel::fill_template_list(
            directory.path().join("missing-template.xlsx"),
            directory.path().join("missing-output.xlsx"),
            &FillWrapper::default(),
            FillConfig::new(),
        )
        .is_err()
    );
    assert!(
        EasyExcel::fill_template_list(
            &list_template,
            directory.path().join("missing/template-output.xlsx"),
            &FillWrapper::default(),
            FillConfig::new(),
        )
        .is_err()
    );

    let malformed_template = directory.path().join("malformed-template.xlsx");
    let malformed_output = directory.path().join("malformed-output.xlsx");
    write_minimal_template(
        &malformed_template,
        "<sst><si><t>{.name}</t></si><si><t</si><si><t>missing</si></sst>",
        concat!(
            "<worksheet><sheetData><row r=\"1\">",
            "<c t=\"s\"></c><c t=\"s\"><v>broken</c><c t=\"s\"><v>9</v></c>",
            "<c t=\"inlineStr\"><is><t</is></c>",
            "<c t=\"inlineStr\"><is><t>missing</is></c>",
            "<c r=\"A1\" t=\"s\"><v>0</v></c>",
            "<c r=\"B1\"><v>{.name}</v></c>",
            "</row></sheetData></worksheet>"
        ),
    )?;
    EasyExcel::fill_template_list(
        &malformed_template,
        &malformed_output,
        &FillWrapper::new([TemplateData::new().with("name", "covered")]),
        FillConfig::new(),
    )?;

    let untyped_template = directory.path().join("untyped-template.xlsx");
    write_minimal_template(
        &untyped_template,
        "<sst></sst>",
        concat!(
            "<worksheet><sheetData><row r=\"1\">",
            "<c r=\"A1\"><v>{.name}</v></c>",
            "</row></sheetData></worksheet>"
        ),
    )?;
    EasyExcel::fill_template_list(
        &untyped_template,
        directory.path().join("untyped-output.xlsx"),
        &FillWrapper::new([TemplateData::new().with("name", "covered")]),
        FillConfig::new(),
    )?;
    Ok(())
}

