#[allow(clippy::too_many_lines)]
#[test]
fn template_sheet_selection_reports_missing_names_and_indexes() -> Result<()> {
    let (directory, template) = multi_sheet_template_fixture()?;
    assert_eq!(TemplateSheet::default(), TemplateSheet::first());
    assert!(same_sheet(
        &TemplateSheet::first(),
        &TemplateSheet::index(0)
    ));
    assert!(!same_sheet(
        &TemplateSheet::name("摘要"),
        &TemplateSheet::index(0)
    ));
    assert!(same_sheet(
        &TemplateSheet::index(2),
        &TemplateSheet::index(2)
    ));
    assert!(!same_sheet(
        &TemplateSheet::index(1),
        &TemplateSheet::index(2)
    ));

    for (sheet, name) in [
        (TemplateSheet::index(99), "missing-index.xlsx"),
        (TemplateSheet::name("不存在"), "missing-name.xlsx"),
    ] {
        let mut writer = ExcelTemplateWriter::new(&template, directory.path().join(name))?;
        writer.fill_on_sheet(&sheet, &TemplateData::new().with("title", "x"))?;
        assert!(matches!(writer.finish(), Err(ExcelError::SheetNotFound(_))));
        assert!(!writer.is_finished());
    }

    let rows = FillWrapper::named(
        "items",
        [TemplateData::new().with("name", "A").with("value", 1)],
    );
    let mut writer = ExcelTemplateWriter::new(
        &template,
        directory.path().join("conflicting-sheet-alias.xlsx"),
    )?;
    writer
        .fill_list_on_sheet(&TemplateSheet::name("明细"), &rows, FillConfig::new())?
        .fill_list_on_sheet(
            &TemplateSheet::index(1),
            &rows,
            FillConfig::new().direction(FillDirection::Horizontal),
        )?;
    writer.finish()?;
    assert!(writer.is_finished());

    let mut writer = ExcelTemplateWriter::new(
        &template,
        directory.path().join("distinct-sheet-alias.xlsx"),
    )?;
    writer
        .fill_list_on_sheet(&TemplateSheet::name("明细"), &rows, FillConfig::new())?
        .fill_list_on_sheet(
            &TemplateSheet::index(1),
            &FillWrapper::named("others", [TemplateData::new().with("name", "B")]),
            FillConfig::new(),
        )?;
    let resolved = writer.resolved_sheet_fills()?;
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[1].collections.len(), 2);

    let mut writer =
        ExcelTemplateWriter::new(&template, directory.path().join("merged-sheet-alias.xlsx"))?;
    writer
        .fill_list_on_sheet(&TemplateSheet::name("明细"), &rows, FillConfig::new())?
        .fill_list_on_sheet(&TemplateSheet::index(1), &rows, FillConfig::new())?;
    let resolved = writer.resolved_sheet_fills()?;
    assert_eq!(resolved[1].collections.len(), 2);
    assert_eq!(
        resolved[1]
            .collections
            .iter()
            .map(|fill| fill.wrapper.rows().len())
            .sum::<usize>(),
        2
    );
    Ok(())
}

#[test]
fn worksheet_part_resolution_covers_relationship_and_fallback_failures() {
    let workbook = br#"<workbook><sheets><sheet name="Data" r:id="rId1"/></sheets></workbook>"#;
    let missing_relationship = vec![
        synthetic_entry("xl/workbook.xml", workbook.to_vec()),
        synthetic_entry("xl/_rels/workbook.xml.rels", b"<Relationships/>".to_vec()),
    ];
    assert!(matches!(
        worksheet_path(&missing_relationship, &TemplateSheet::first()),
        Err(ExcelError::Format(message)) if message.contains("relationship rId1")
    ));

    let missing_part = vec![
        synthetic_entry("xl/workbook.xml", workbook.to_vec()),
        synthetic_entry(
            "xl/_rels/workbook.xml.rels",
            br#"<Relationships><Relationship Id="rId1" Target="worksheets/missing.xml"/></Relationships>"#.to_vec(),
        ),
    ];
    assert!(matches!(
        worksheet_path(&missing_part, &TemplateSheet::name("Data")),
        Err(ExcelError::Format(message)) if message.contains("worksheet part")
    ));

    for entries in [
        vec![
            synthetic_entry("xl/workbook.xml", vec![0xff]),
            synthetic_entry("xl/_rels/workbook.xml.rels", b"<Relationships/>".to_vec()),
        ],
        vec![
            synthetic_entry("xl/workbook.xml", workbook.to_vec()),
            synthetic_entry("xl/_rels/workbook.xml.rels", vec![0xff]),
        ],
    ] {
        assert!(matches!(
            worksheet_path(&entries, &TemplateSheet::first()),
            Err(ExcelError::Format(_))
        ));
    }
    let invalid_target = vec![
        synthetic_entry("xl/workbook.xml", workbook.to_vec()),
        synthetic_entry(
            "xl/_rels/workbook.xml.rels",
            br#"<Relationships><Relationship Id="rId1" Target="../../outside.xml"/></Relationships>"#.to_vec(),
        ),
    ];
    assert!(matches!(
        worksheet_path(&invalid_target, &TemplateSheet::first()),
        Err(ExcelError::Format(message)) if message.contains("escapes package root")
    ));
    assert!(
        workbook_sheets(
            r#"<sheets><sheet name="missing-id"/><sheet r:id="missing-name"/></sheets>"#
        )
        .is_empty()
    );

    let fallback = vec![synthetic_entry(
        "xl/worksheets/custom.xml",
        b"<worksheet/>".to_vec(),
    )];
    assert_eq!(
        worksheet_path(&fallback, &TemplateSheet::index(0)).expect("fallback index"),
        "xl/worksheets/custom.xml"
    );
    assert!(matches!(
        worksheet_path(&fallback, &TemplateSheet::name("Data")),
        Err(ExcelError::SheetNotFound(name)) if name == "Data"
    ));
    assert!(matches!(
        worksheet_path(&fallback, &TemplateSheet::index(1)),
        Err(ExcelError::SheetNotFound(index)) if index == "1"
    ));

    assert_eq!(
        normalize_workbook_target("../worksheets/sheet.xml").expect("relative target"),
        "worksheets/sheet.xml"
    );
    assert!(normalize_workbook_target("../../outside.xml").is_err());
    assert!(normalize_workbook_target("/").is_err());
    assert_eq!(
        xml_elements("<sheets><sheet name=\"A\"/><sheet", "sheet").collect::<Vec<_>>(),
        vec!["<sheet name=\"A\"/>"]
    );
}

#[test]
fn fills_shared_string_placeholders_and_preserves_unknown_values() -> Result<()> {
    let (directory, template) = template_fixture()?;
    let output = directory.path().join("filled.xlsx");
    let data = TemplateData::new()
        .with("name", "A&B <Admin>")
        .with("count", 3);
    fill_xlsx_template(&template, &output, &data)?;

    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get_value((0, 0)),
        Some(&Data::String("Hello A&B <Admin>".to_owned()))
    );
    assert_eq!(
        range.get_value((1, 0)),
        Some(&Data::String("Count: 3".to_owned()))
    );
    assert_eq!(
        range.get_value((2, 0)),
        Some(&Data::String("Unknown: {unknown}".to_owned()))
    );

    fill_xlsx_template(
        &output,
        &output,
        &TemplateData::new().with("unknown", "done"),
    )?;
    let mut workbook: Xlsx<_> = open_workbook(output).map_err(test_error)?;
    assert_eq!(
        workbook
            .worksheet_range("Sheet1")
            .map_err(test_error)?
            .get_value((2, 0)),
        Some(&Data::String("Unknown: done".to_owned()))
    );
    Ok(())
}

#[test]
fn package_entries_directories_permissions_and_binary_data_round_trip() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("entries.zip");
    let entries = vec![
        TemplateEntry {
            name: "folder/".to_owned(),
            is_dir: true,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: Vec::new(),
        },
        TemplateEntry {
            name: "folder/data.bin".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Deflated,
            unix_mode: Some(0o644),
            bytes: vec![0, 1, 2, 3],
        },
    ];
    write_entries(&path, &entries)?;
    let actual = load_entries(&path)?;
    assert_eq!(actual.len(), 2);
    assert!(actual[0].is_dir);
    assert_eq!(actual[1].bytes, vec![0, 1, 2, 3]);
    Ok(())
}

#[test]
fn invalid_archives_xml_and_output_paths_return_typed_errors() -> Result<()> {
    let directory = tempdir()?;
    let corrupt = directory.path().join("corrupt.xlsx");
    fs::write(&corrupt, b"not a zip")?;
    assert!(
        fill_xlsx_template(
            &corrupt,
            &directory.path().join("out.xlsx"),
            &TemplateData::new()
        )
        .is_err()
    );

    let invalid_xml = directory.path().join("invalid-xml.xlsx");
    write_entries(
        &invalid_xml,
        &[TemplateEntry {
            name: "bad.xml".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: vec![0xff],
        }],
    )?;
    assert!(
        fill_xlsx_template(
            &invalid_xml,
            &directory.path().join("invalid-out.xlsx"),
            &TemplateData::new()
        )
        .is_err()
    );

    let (_template_directory, template) = template_fixture()?;
    assert!(fill_xlsx_template(&template, directory.path(), &TemplateData::new()).is_err());
    assert!(load_entries(&directory.path().join("missing.xlsx")).is_err());
    assert_eq!(
        format_error("broken").to_string(),
        "excel format error: broken"
    );
    Ok(())
}

#[test]
fn fill_config_and_wrapper_match_java_defaults_and_builders() {
    let rows = vec![TemplateData::new().with("name", "Alice")];
    let unnamed = FillWrapper::new(rows.clone());
    assert_eq!(unnamed.name(), None);
    assert_eq!(unnamed.rows(), rows);

    let named = FillWrapper::named("users", rows.clone());
    assert_eq!(named.name(), Some("users"));
    assert_eq!(named.rows(), rows);
    assert_eq!(FillWrapper::default().rows(), &[]);

    let defaults = FillConfig::default();
    assert_eq!(defaults, FillConfig::new());
    assert_eq!(defaults.get_direction(), FillDirection::Vertical);
    assert!(!defaults.get_force_new_row());
    assert!(defaults.get_auto_style());

    let configured = FillConfig::new()
        .direction(FillDirection::Horizontal)
        .force_new_row(true)
        .auto_style(false);
    assert_eq!(configured.get_direction(), FillDirection::Horizontal);
    assert!(configured.get_force_new_row());
    assert!(!configured.get_auto_style());
}

