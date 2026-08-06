#[test]
fn public_facade_round_trips_scalar_write_cell_data_and_emits_multiple_images() -> Result<()> {
    let bytes = tiny_png();
    let second_anchor = ClientAnchorData::new()
        .coordinates(
            CoordinateData::new()
                .relative_first_column_index(1)
                .relative_last_column_index(1),
        )
        .left(3)
        .top(4)
        .right(5)
        .bottom(6)
        .anchor_type(AnchorType::DontMoveAndResize);
    let absolute_anchor = ClientAnchorData::new().coordinates(
        CoordinateData::new()
            .first_row_index(1)
            .first_column_index(1)
            .last_row_index(1)
            .last_column_index(1),
    );
    let row = MultiImageRow {
        cell: WriteCellData::new(CellValue::String("three images".to_owned())).image_data_list([
            ImageData::new(bytes.clone()),
            ImageData::new(bytes.clone()).anchor(second_anchor),
            ImageData::new(bytes).anchor(absolute_anchor),
        ]),
    };
    let directory = tempdir()?;
    let path = directory.path().join("multi-image.xlsx");
    EasyExcel::write::<MultiImageRow>(&path)
        .sheet("Images")
        .column_width(0, 18)
        .column_width(1, 12)
        .do_write([row])?;

    let rows = EasyExcel::read_sync::<MultiImageRow>(&path)
        .sheet("Images")
        .do_read_sync()?;
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].cell.value(),
        &CellValue::String("three images".to_owned())
    );
    assert!(rows[0].cell.images().is_empty());

    let mut archive = ZipArchive::new(File::open(&path)?)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut drawing_xml = String::new();
    archive
        .by_name("xl/drawings/drawing1.xml")
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .read_to_string(&mut drawing_xml)?;
    assert_eq!(drawing_xml.matches("<xdr:twoCellAnchor").count(), 3);
    assert_eq!(drawing_xml.matches("editAs=\"absolute\"").count(), 1);
    Ok(())
}

#[test]
fn public_facade_writes_rich_text_and_reads_its_plain_value() -> Result<()> {
    let rich = RichTextStringData::new("红色😀下标")
        .apply_font(
            WriteFont::new()
                .font_name("Aptos")
                .bold(true)
                .type_offset(ExcelFontScript::None),
        )
        .apply_font_range(
            0,
            2,
            WriteFont::new()
                .color(ExcelColor::Indexed(10))
                .underline(ExcelUnderline::Single),
        )
        .apply_font_range(
            2,
            4,
            WriteFont::new()
                .color(ExcelColor::Rgb(0x00_80_00))
                .type_offset(ExcelFontScript::Subscript),
        )
        .apply_font_range(
            0,
            1,
            WriteFont::new().type_offset(ExcelFontScript::Superscript),
        );
    let directory = tempdir()?;
    let path = directory.path().join("rich-text.xlsx");
    EasyExcel::write::<RichTextFacadeRow>(&path)
        .sheet("Rich")
        .do_write([RichTextFacadeRow {
            value: rich.clone(),
        }])?;

    for (name, value) in [
        (
            "outside.xlsx",
            RichTextStringData::new("a").apply_font_range(0, 2, WriteFont::new()),
        ),
        (
            "surrogate.xlsx",
            RichTextStringData::new("😀").apply_font_range(0, 1, WriteFont::new()),
        ),
    ] {
        assert!(
            EasyExcel::write::<RichTextFacadeRow>(directory.path().join(name))
                .sheet("Rich")
                .do_write([RichTextFacadeRow { value }])
                .is_err()
        );
    }

    let rows = EasyExcel::read_sync::<RichTextFacadeRow>(&path)
        .sheet("Rich")
        .do_read_sync()?;
    assert_eq!(rows[0].value.text_string(), rich.text_string());
    assert!(rows[0].value.interval_fonts().is_empty());
    let mut archive = ZipArchive::new(File::open(&path)?)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut shared_strings = String::new();
    archive
        .by_name("xl/sharedStrings.xml")
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .read_to_string(&mut shared_strings)?;
    assert!(shared_strings.contains("<r>"));
    assert!(shared_strings.contains('红'));
    assert!(shared_strings.contains('色'));
    assert!(shared_strings.contains("😀"));
    Ok(())
}

#[test]
fn derive_selected_converter_transforms_read_and_write_values() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("converted.xlsx");
    let expected = vec![ConvertedName {
        name: "alice".to_owned(),
    }];
    EasyExcel::write::<ConvertedName>(&path).do_write(expected.clone())?;
    assert_eq!(
        EasyExcel::read_sync::<RawName>(&path).do_read_sync()?,
        vec![RawName {
            name: "excel:alice".to_owned()
        }]
    );
    assert_eq!(
        EasyExcel::read_sync::<ConvertedName>(&path).do_read_sync()?,
        expected
    );
    Ok(())
}

#[test]
fn converter_write_cell_data_style_reaches_xlsx_xls_csv_and_template_backends() -> Result<()> {
    let directory = tempdir()?;
    let row = RuntimeStyledValue { value: 1.25 };
    let (_, converted) = row.to_excel_write_row(&easyexcel::ConverterRegistry::default())?;
    assert_eq!(converted[0].effective_value(), CellValue::Float(1.25));
    assert_eq!(
        converted[0]
            .write_cell_style()
            .and_then(|style| style.fill_foreground_color),
        Some(ExcelColor::Rgb(0x00_ff_00))
    );
    assert_eq!(
        converted[0]
            .data_format_data()
            .and_then(|data| data.format()),
        Some("0.0000")
    );

    let xlsx = directory.path().join("runtime-style.xlsx");
    EasyExcel::write::<RuntimeStyledValue>(&xlsx).do_write([row.clone()])?;
    let mut archive = ZipArchive::new(File::open(&xlsx)?)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut styles = String::new();
    archive
        .by_name("xl/styles.xml")
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .read_to_string(&mut styles)?;
    assert!(
        styles.contains("FF00FF00"),
        "converter fill missing: {styles}"
    );
    assert!(
        styles.contains("formatCode=\"0.0000\""),
        "converter number format missing: {styles}"
    );

    let template = directory.path().join("runtime-style-template.xlsx");
    EasyExcel::write::<RawName>(&template)
        .sheet("Sheet1")
        .do_write([RawName {
            name: "seed".to_owned(),
        }])?;
    let template_output = directory.path().join("runtime-style-template-output.xlsx");
    EasyExcel::write::<RuntimeStyledValue>(&template_output)
        .with_template(&template)
        .sheet("Sheet1")
        .do_write([row.clone()])?;
    let mut archive = ZipArchive::new(File::open(&template_output)?)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut template_styles = String::new();
    archive
        .by_name("xl/styles.xml")
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .read_to_string(&mut template_styles)?;
    assert!(template_styles.contains("FF00FF00"));
    assert!(template_styles.contains("formatCode=\"0.0000\""));

    let xls = directory.path().join("runtime-style.xls");
    EasyExcel::write::<RuntimeStyledValue>(&xls).do_write([row.clone()])?;
    assert_eq!(
        EasyExcel::read_sync::<RawStyledValue>(&xls)
            .head_row_number(1)
            .do_read_sync()?,
        vec![RawStyledValue { value: 1.25 }]
    );

    let csv = directory.path().join("runtime-style.csv");
    EasyExcel::write::<RuntimeStyledValue>(&csv).do_write([row])?;
    let csv_text = std::fs::read_to_string(csv)?;
    assert!(csv_text.contains("1.25"));
    Ok(())
}

#[test]
fn formula_converter_receives_expression_while_scalar_receives_cached_value() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("formula.xlsx");
    let expected = vec![FormulaExpression {
        formula: "SUM(1,2)".to_owned(),
    }];
    EasyExcel::write::<FormulaExpression>(&path).do_write(expected.clone())?;

    assert_eq!(
        EasyExcel::read_sync::<FormulaExpression>(&path).do_read_sync()?,
        expected
    );
    assert_eq!(
        EasyExcel::read_sync::<CachedFormulaValue>(&path).do_read_sync()?,
        vec![CachedFormulaValue { value: 0.0 }]
    );
    Ok(())
}

#[test]
fn derive_exposes_java_style_dimension_annotations() -> Result<()> {
    let metadata = AnnotatedDimensions::write_metadata();
    assert_eq!(AnnotatedDimensions::schema()[0].column_width, Some(30));
    assert_eq!(AnnotatedDimensions::schema()[1].column_width, None);
    assert_eq!(metadata.column_width, Some(18));
    assert_eq!(metadata.head_row_height, Some(24));
    assert_eq!(metadata.content_row_height, Some(16));

    let directory = tempdir()?;
    EasyExcel::write::<AnnotatedDimensions>(directory.path().join("dimensions.xlsx"))
        .column_width(1, 40)
        .do_write([AnnotatedDimensions {
            name: "Alice".to_owned(),
            age: 30,
        }])?;
    Ok(())
}

#[test]
fn derive_writes_java_style_cell_and_font_annotations() -> Result<()> {
    let metadata = AnnotatedStyles::write_metadata();
    assert!(metadata.head_style.is_some());
    assert!(metadata.content_style.is_some());
    assert!(metadata.head_font_style.is_some());
    assert!(metadata.content_font_style.is_some());
    assert_eq!(
        metadata.once_absolute_merge,
        Some(OnceAbsoluteMergeProperty::new(0, 0, 0, 1))
    );
    assert!(AnnotatedStyles::schema()[0].head_style.is_some());
    assert!(AnnotatedStyles::schema()[0].head_font_style.is_some());
    assert_eq!(
        AnnotatedStyles::schema()[0].loop_merge,
        Some(LoopMergeProperty::new(2, 1))
    );

    let directory = tempdir()?;
    EasyExcel::write::<AnnotatedStyles>(directory.path().join("annotated-styles.xlsx")).do_write(
        [
            AnnotatedStyles {
                name: "Alice".to_owned(),
                age: 30,
            },
            AnnotatedStyles {
                name: "Bob".to_owned(),
                age: 31,
            },
        ],
    )?;
    Ok(())
}

#[test]
fn page_listener_receives_batches_and_contexts() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("users.xlsx");
    let users = (0..4)
        .map(|age| User {
            name: format!("user-{age}"),
            age: Some(age),
            registered_on: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            transient: String::new(),
        })
        .collect::<Vec<_>>();
    EasyExcel::write::<User>(&path).do_write(users)?;

    let batches = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&batches);
    let listener = PageReadListener::new(2, move |rows: Vec<User>, context| {
        captured
            .borrow_mut()
            .push((rows.len(), context.batch_index()));
        Ok(())
    });
    EasyExcel::read::<User, _>(&path, listener).do_read()?;

    assert_eq!(&*batches.borrow(), &[(2, 0), (2, 1)]);
    Ok(())
}

struct StopListener;

impl ReadListener<User> for StopListener {
    fn invoke(&mut self, _data: User, _context: &AnalysisContext) -> Result<()> {
        panic!("has_next prevents invocation")
    }

    fn has_next(&mut self, _context: &AnalysisContext) -> bool {
        false
    }
}

#[test]
fn listener_can_stop_before_data_rows() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("users.xlsx");
    let user = User {
        name: "stop".to_owned(),
        age: Some(1),
        registered_on: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
        transient: String::new(),
    };
    EasyExcel::write::<User>(&path).do_write([user])?;
    EasyExcel::read::<User, _>(&path, StopListener)
        .sheet(0_usize)
        .head_row_number(1)
        .ignore_empty_row(false)
        .do_read()
}
