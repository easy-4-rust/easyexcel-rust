#[test]
fn java_field_cache_order_and_forced_index_are_preserved_across_backends() -> Result<()> {
    struct OrderedBean;

    impl ExcelRow for OrderedBean {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("forced", "Forced", Some(2), i32::MAX, None),
                ExcelColumn::new("late", "Late", None, 20, None),
                ExcelColumn::new("early_a", "Early A", None, 10, None),
                ExcelColumn::new("early_b", "Early B", None, 10, None),
            ];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![
                CellValue::String("forced".to_owned()),
                CellValue::String("late".to_owned()),
                CellValue::String("early-a".to_owned()),
                CellValue::String("early-b".to_owned()),
            ])
        }
    }

    fn assert_range_order(range: &calamine::Range<Data>) {
        let expected_head = ["Early A", "Early B", "Forced", "Late"];
        let expected_data = ["early-a", "early-b", "forced", "late"];
        for (column, expected) in expected_head.into_iter().enumerate() {
            assert_eq!(
                range.get_value((0, u32::try_from(column).expect("column"))),
                Some(&Data::String(expected.to_owned()))
            );
        }
        for (column, expected) in expected_data.into_iter().enumerate() {
            assert_eq!(
                range.get_value((1, u32::try_from(column).expect("column"))),
                Some(&Data::String(expected.to_owned()))
            );
        }
    }

    let directory = tempdir()?;
    let options = WriteOptions {
        sheet_name: "Order".to_owned(),
        with_bom: false,
        ..WriteOptions::default()
    };

    let xlsx_path = directory.path().join("field-order.xlsx");
    write_xlsx::<OrderedBean, _>(&xlsx_path, &options, [OrderedBean])?;
    let mut xlsx: Xlsx<_> = open_workbook(&xlsx_path).map_err(test_error)?;
    assert_range_order(&xlsx.worksheet_range("Order").map_err(test_error)?);

    let xls_path = directory.path().join("field-order.xls");
    write_xls::<OrderedBean, _>(&xls_path, &options, [OrderedBean])?;
    let mut xls: Xls<_> = open_workbook(&xls_path).map_err(test_error)?;
    assert_range_order(&xls.worksheet_range("Order").map_err(test_error)?);

    let csv = write_csv_to_buffer::<OrderedBean, _>(
        Path::new("field-order.csv"),
        &options,
        [OrderedBean],
        &mut [],
    )?;
    assert_eq!(
        String::from_utf8(csv).map_err(test_error)?,
        "Early A,Early B,Forced,Late\nearly-a,early-b,forced,late\n"
    );

    let (template_rows, _, _, _) =
        collect_template_append_rows::<OrderedBean, _>(&options, [OrderedBean], true, 0)?;
    assert_eq!(
        template_rows,
        vec![
            vec![
                (0, CellValue::String("Early A".to_owned())),
                (1, CellValue::String("Early B".to_owned())),
                (2, CellValue::String("Forced".to_owned())),
                (3, CellValue::String("Late".to_owned())),
            ],
            vec![
                (0, CellValue::String("early-a".to_owned())),
                (1, CellValue::String("early-b".to_owned())),
                (2, CellValue::String("forced".to_owned())),
                (3, CellValue::String("late".to_owned())),
            ],
        ]
    );
    Ok(())
}

#[test]
fn writer_emits_headers_and_every_supported_cell_type() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("all.xlsx");
    write_xlsx::<EveryCell, _>(
        &path,
        &WriteOptions {
            sheet_name: "Values".to_owned(),
            constant_memory: false,
            need_head: true,
            freeze_head: true,
            freeze_panes: None,
            ..WriteOptions::default()
        },
        vec![every_cell()],
    )?;

    let mut workbook: Xlsx<_> = open_workbook(&path).map_err(test_error)?;
    let range = workbook.worksheet_range("Values").map_err(test_error)?;
    assert_eq!(
        range.get_value((0, 1)),
        Some(&Data::String("String".to_owned()))
    );
    assert_eq!(
        range.get_value((1, 1)),
        Some(&Data::String("text".to_owned()))
    );
    assert_eq!(
        range.get_value((1, 2)),
        Some(&Data::String("#DIV/0!".to_owned()))
    );
    assert_eq!(range.get_value((1, 3)), Some(&Data::Bool(true)));
    assert_eq!(range.get_value((1, 4)), Some(&Data::Float(-12.0)));
    assert_eq!(range.get_value((1, 5)), Some(&Data::Float(1.25)));
    assert!(matches!(range.get_value((1, 6)), Some(Data::DateTime(_))));
    assert!(matches!(range.get_value((1, 7)), Some(Data::DateTime(_))));
    assert_eq!(
        range.get_value((1, 8)),
        Some(&Data::String(i64::MAX.to_string()))
    );
    assert_eq!(range.get_value((1, 9)), Some(&Data::Empty));
    assert_eq!(
        range.get_value((1, 11)),
        Some(&Data::String("Rust".to_owned()))
    );
    assert_eq!(
        range.get_value((1, 12)),
        Some(&Data::String("annotated".to_owned()))
    );
    assert_eq!(range.get_value((1, 14)), Some(&Data::Float(123.45)));
    let formulas = workbook.worksheet_formula("Values").map_err(test_error)?;
    assert!(
        formulas
            .get_value((1, 10))
            .is_some_and(|formula| formula.contains("SUM(E2:F2)"))
    );

    let sheet = zip_entry(&path, "xl/worksheets/sheet1.xml")?;
    assert!(sheet.contains("<hyperlink ref=\"L2\""));
    let comments = zip_entry(&path, "xl/comments1.xml")?;
    assert!(comments.contains("cell note"));
    let names = zip_names(&path)?;
    assert!(names.iter().any(|name| name == "xl/media/image1.png"));
    Ok(())
}

#[test]
fn write_cell_data_emits_multiple_images_with_java_anchor_semantics() -> Result<()> {
    let bytes = tiny_png();
    let spanning = ClientAnchorData::new()
        .coordinates(
            CoordinateData::new()
                .relative_last_row_index(1)
                .relative_last_column_index(1),
        )
        .left(5)
        .top(6)
        .right(7)
        .bottom(8)
        .anchor_type(AnchorType::MoveDontResize);
    let zero_absolute_defers = ClientAnchorData::new()
        .coordinates(
            CoordinateData::new()
                .first_row_index(0)
                .first_column_index(0)
                .relative_first_row_index(1)
                .relative_first_column_index(1)
                .relative_last_row_index(1)
                .relative_last_column_index(1),
        )
        .anchor_type(AnchorType::DontMoveDoResize);
    let absolute = ClientAnchorData::new()
        .coordinates(
            CoordinateData::new()
                .first_row_index(4)
                .first_column_index(3)
                .last_row_index(4)
                .last_column_index(3),
        )
        .anchor_type(AnchorType::DontMoveAndResize);
    let cell = WriteCellData::new(CellValue::String("caption".to_owned())).image_data_list([
        ImageData::new(bytes.clone()).image_type(ImageType::Png),
        ImageData::new(bytes.clone()).anchor(spanning),
        ImageData::new(bytes.clone()).anchor(zero_absolute_defers),
        ImageData::new(bytes).anchor(absolute),
    ]);
    let directory = tempdir()?;
    let path = directory.path().join("multiple-images.xlsx");
    write_xlsx::<AnchoredImageRow, _>(
        &path,
        &WriteOptions {
            sheet_name: "Images".to_owned(),
            column_widths: vec![(1, 12)],
            ..WriteOptions::default()
        },
        [AnchoredImageRow { cell }],
    )?;

    let mut workbook: Xlsx<_> = open_workbook(&path).map_err(test_error)?;
    let range = workbook.worksheet_range("Images").map_err(test_error)?;
    assert_eq!(
        range.get_value((1, 0)),
        Some(&Data::String("caption".to_owned()))
    );
    let drawing = zip_entry(&path, "xl/drawings/drawing1.xml")?;
    assert_eq!(drawing.matches("<xdr:twoCellAnchor").count(), 4);
    assert_eq!(drawing.matches("editAs=\"oneCell\"").count(), 2);
    assert_eq!(drawing.matches("editAs=\"absolute\"").count(), 1);
    assert!(drawing.contains("<xdr:col>3</xdr:col>"));
    assert!(drawing.contains("<xdr:row>4</xdr:row>"));
    Ok(())
}

#[test]
fn rich_text_writer_applies_java_whole_and_utf16_interval_fonts() -> Result<()> {
    let whole = WriteFont::new()
        .font_name("Aptos")
        .font_height_in_points(13.0)
        .italic(true)
        .strikeout(true)
        .color(ExcelColor::Indexed(10))
        .type_offset(ExcelFontScript::Subscript)
        .underline(ExcelUnderline::Single)
        .charset(1)
        .bold(true);
    let override_font = WriteFont::new()
        .italic(false)
        .strikeout(false)
        .color(ExcelColor::Rgb(0x00_80_00))
        .type_offset(ExcelFontScript::None)
        .underline(ExcelUnderline::None)
        .bold(false);
    let rich = RichTextStringData::new("A😀BC")
        .apply_font(whole)
        .apply_font_range(1, 3, override_font.clone())
        .apply_font_range(3, 5, WriteFont::new().color(ExcelColor::Indexed(11)))
        .apply_font_range(4, 5, override_font);
    // NOTE: `rich_text_runs` and `rich_text_format` are now private helpers
    // (`rich_text_run_specs`, `rich_text_font_spec`) inside `xlsx_cell_emission`.
    // Rich-text segmentation is validated indirectly via the xlsx round-trip below.

    let directory = tempdir()?;
    let path = directory.path().join("rich-text.xlsx");
    write_xlsx::<RichTextRow, _>(
        &path,
        &WriteOptions::default(),
        [
            RichTextRow {
                value: rich.clone(),
            },
            RichTextRow {
                value: RichTextStringData::new(""),
            },
        ],
    )?;
    let shared_strings = zip_entry(&path, "xl/sharedStrings.xml")?;
    assert!(shared_strings.contains("<t>A</t>"));
    assert!(shared_strings.contains("<t>😀</t>"));
    assert!(shared_strings.contains("rgb=\"FF008000\""));
    assert!(shared_strings.contains("<vertAlign val=\"subscript\"/>"));

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let invalid = RichTextStringData::new("abc").apply_font_range(2, 2, WriteFont::new());
    assert!(write_rich_text(worksheet, 0, 0, &invalid, &Format::new()).is_err());
    let metadata = ExcelWriteMetadata::new();
    assert!(
        write_cell(
            worksheet,
            0,
            0,
            &TEST_COLUMN,
            &CellValue::RichText(invalid),
            SheetStyleContext::content(None, &metadata, WriteGlobalFlags::default())
                .column(&TEST_COLUMN),
            &ImageLayout::default(),
        )
        .is_err()
    );
    assert!(
        write_rich_text(
            worksheet,
            u32::MAX,
            0,
            &RichTextStringData::new(""),
            &Format::new(),
        )
        .is_err()
    );
    assert!(write_rich_text(worksheet, u32::MAX, 0, &rich, &Format::new()).is_err());
    Ok(())
}

