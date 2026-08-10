#[test]
fn annotation_styles_apply_field_type_and_builder_precedence() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("annotation-styles.xlsx");
    write_xlsx::<StyledAnnotationRow, _>(
        &path,
        &WriteOptions {
            head_style: CellStyle::new().bold(true).font_color(0x0000_ff00),
            ..WriteOptions::default()
        },
        vec![StyledAnnotationRow],
    )?;

    let styles = zip_entry(&path, "xl/styles.xml")?;
    assert!(styles.contains("rgb=\"FF00FF00\""));
    assert!(styles.contains("rgb=\"FFFF0000\""));
    assert!(styles.contains("rgb=\"FF00CCFF\""));
    assert!(styles.contains("rgb=\"FF008000\""));
    assert!(styles.contains("rgb=\"FF0000FF\""));
    assert!(styles.contains("rgb=\"FFC0C0C0\""));
    assert!(styles.contains("<sz val=\"50\"/>"));
    assert!(styles.contains("style=\"thin\""));

    let sheet = zip_entry(&path, "xl/worksheets/sheet1.xml")?;
    assert_ne!(cell_style_id(&sheet, "A1"), cell_style_id(&sheet, "B1"));
    assert_ne!(cell_style_id(&sheet, "A2"), cell_style_id(&sheet, "B2"));

    let java_path = directory.path().join("java-indexed-annotation-styles.xlsx");
    write_xlsx::<StyledAnnotationRow, _>(
        &java_path,
        &WriteOptions::default(),
        vec![StyledAnnotationRow],
    )?;
    let java_styles = zip_entry(&java_path, "xl/styles.xml")?;
    for expected in [
        "rgb=\"FFFF00FF\"",
        "rgb=\"FFFFCC00\"",
        "rgb=\"FF00CCFF\"",
        "rgb=\"FF0000FF\"",
        "rgb=\"FFFF0000\"",
        "rgb=\"FF00FFFF\"",
        "rgb=\"FF008000\"",
        "rgb=\"FFC0C0C0\"",
    ] {
        assert!(java_styles.contains(expected), "missing {expected}");
    }
    for expected in [20, 30, 40, 50] {
        assert!(java_styles.contains(&format!("<sz val=\"{expected}\"/>")));
    }
    Ok(())
}

#[test]
// 语义敏感：该测试端到端覆盖 Java 对应用例的完整流程，
// 拆分会降低可读性，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
fn excel_write_head_property_resolves_metadata_and_java_merge_ranges() -> Result<()> {
    let mut parent_head_style = ExcelCellStyle::new();
    parent_head_style.wrapped = Some(true);
    let mut parent_head_font = ExcelFontStyle::new();
    parent_head_font.bold = Some(true);
    let mut field_head_style = ExcelCellStyle::new();
    field_head_style.locked = Some(false);

    let columns = [
        ExcelColumn::new("a", "A", Some(0), 0, None)
            .with_column_width(18)
            .with_head_style(field_head_style),
        ExcelColumn::new("b", "B", Some(1), 0, None),
        ExcelColumn::new("c", "C", Some(2), 0, None),
        ExcelColumn::new("d", "D", Some(3), 0, None),
        ExcelColumn::new("e", "E", Some(4), 0, None),
    ];
    let effective_columns = columns.iter().enumerate().collect::<Vec<_>>();
    let head = vec![
        vec!["顶格".to_owned(), "顶格".to_owned(), "两格".to_owned()],
        vec!["顶格".to_owned(), "顶格".to_owned(), "两格".to_owned()],
        vec!["顶格".to_owned(), "四联".to_owned(), "四联".to_owned()],
        vec!["顶格".to_owned(), "四联".to_owned(), "四联".to_owned()],
        vec!["顶格".to_owned()],
    ];
    let once_merge = OnceAbsoluteMergeProperty::new(10, 11, 1, 2);
    let property = ExcelWriteHeadProperty::from_columns(
        Some("DemoData".to_owned()),
        &effective_columns,
        Some(&head),
        ExcelWriteMetadata::new()
            .column_width(12)
            .head_row_height(25)
            .content_row_height(17)
            .head_style(parent_head_style)
            .head_font_style(parent_head_font)
            .once_absolute_merge(once_merge),
    )?;

    assert_eq!(property.head_kind(), HeadKind::Class);
    assert_eq!(property.head_row_number(), 3);
    assert_eq!(
        property.head_row_height_property(),
        Some(&RowHeightProperty::new(25))
    );
    assert_eq!(
        property.content_row_height_property(),
        Some(&RowHeightProperty::new(17))
    );
    assert_eq!(property.once_absolute_merge_property(), Some(&once_merge));
    assert_eq!(
        property.head_map()[&0]
            .column_width_property
            .expect("field width")
            .width(),
        18
    );
    assert_eq!(
        property.head_map()[&1]
            .column_width_property
            .expect("parent width")
            .width(),
        12
    );
    assert_eq!(
        property.head_map()[&0]
            .head_style_property
            .as_ref()
            .expect("field style")
            .write_cell_style(),
        &field_head_style.into()
    );
    assert_eq!(
        property.head_map()[&1]
            .head_style_property
            .as_ref()
            .expect("parent style")
            .write_cell_style(),
        &parent_head_style.into()
    );
    assert_eq!(
        property.head_map()[&1]
            .head_font_property
            .as_ref()
            .expect("parent font")
            .bold,
        Some(true)
    );
    assert_eq!(
        property.head_map()[&4].head_name_list(),
        ["顶格", "顶格", "顶格"]
    );
    assert_eq!(
        property.head_cell_range_list(),
        vec![
            CellRange::new(0, 0, 0, 4),
            CellRange::new(1, 1, 0, 1),
            CellRange::new(2, 2, 0, 1),
            CellRange::new(1, 2, 2, 3),
            CellRange::new(1, 2, 4, 4),
        ]
    );
    Ok(())
}

#[test]
fn dynamic_multi_level_head_merges_parents_and_offsets_data_rows() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("dynamic-head.xlsx");
    let options = WriteOptions {
        sheet_name: "Dynamic".to_owned(),
        include_column_indexes: Some(vec![0, 1, 2]),
        dynamic_head: Some(vec![
            vec!["User".to_owned(), "Empty".to_owned()],
            vec!["User".to_owned(), "String".to_owned()],
            vec!["Meta".to_owned()],
        ]),
        relative_head_row_index: 2,
        freeze_head: true,
        ..WriteOptions::default()
    };
    assert_eq!(dynamic_head_rows(&options)?, 2);
    write_xlsx::<EveryCell, _>(&path, &options, vec![every_cell()])?;

    let mut workbook: Xlsx<_> = open_workbook(&path).map_err(test_error)?;
    let range = workbook.worksheet_range("Dynamic").map_err(test_error)?;
    assert_eq!(
        range.get_value((2, 0)),
        Some(&Data::String("User".to_owned()))
    );
    assert_eq!(
        range.get_value((3, 1)),
        Some(&Data::String("String".to_owned()))
    );
    assert_eq!(
        range.get_value((4, 1)),
        Some(&Data::String("text".to_owned()))
    );
    assert_eq!(
        workbook
            .merge_cells_by_sheet_name("Dynamic")
            .map_err(test_error)?,
        vec![
            Dimensions::new((2, 0), (2, 1)),
            Dimensions::new((2, 2), (3, 2)),
        ]
    );

    let xls_path = directory.path().join("dynamic-head.xls");
    write_xls::<EveryCell, _>(&xls_path, &options, vec![every_cell()])?;
    let xls: Xls<_> = open_workbook(&xls_path).map_err(test_error)?;
    assert_eq!(
        xls.merge_cells_by_sheet_name("Dynamic")
            .map_err(test_error)?,
        vec![
            Dimensions::new((2, 0), (2, 1)),
            Dimensions::new((2, 2), (3, 2)),
        ]
    );

    let unmerged_path = directory.path().join("dynamic-head-unmerged.xlsx");
    write_xlsx::<EveryCell, _>(
        &unmerged_path,
        &WriteOptions {
            automatic_merge_head: false,
            ..options
        },
        vec![every_cell()],
    )?;
    let mut unmerged: Xlsx<_> = open_workbook(&unmerged_path).map_err(test_error)?;
    assert!(
        unmerged
            .merge_cells_by_sheet_name("Dynamic")
            .map_err(test_error)?
            .is_empty()
    );
    Ok(())
}

#[test]
// 语义敏感：xlsx/xls 双后端模板并行断言，命名刻意对照，故豁免 similar_names。
#[allow(clippy::similar_names)]
fn dynamic_head_merges_are_preserved_on_xlsx_and_xls_templates() -> Result<()> {
    let directory = tempdir()?;
    let dynamic_head = vec![
        vec!["User".to_owned(), "Empty".to_owned()],
        vec!["User".to_owned(), "String".to_owned()],
        vec!["Meta".to_owned()],
    ];

    let xlsx_template = directory.path().join("dynamic-template.xlsx");
    let mut workbook = Workbook::new();
    workbook
        .add_worksheet()
        .set_name("Dynamic")
        .map_err(test_error)?
        .write_string(0, 0, "seed")
        .map_err(test_error)?;
    workbook.save(&xlsx_template).map_err(test_error)?;
    let xlsx_output = directory.path().join("dynamic-template-output.xlsx");
    write_xlsx::<EveryCell, _>(
        &xlsx_output,
        &WriteOptions {
            sheet_name: "Dynamic".to_owned(),
            template_file: Some(xlsx_template),
            include_column_indexes: Some(vec![0, 1, 2]),
            dynamic_head: Some(dynamic_head.clone()),
            relative_head_row_index: 2,
            ..WriteOptions::default()
        },
        vec![every_cell()],
    )?;
    let mut xlsx: Xlsx<_> = open_workbook(&xlsx_output).map_err(test_error)?;
    assert_eq!(
        xlsx.merge_cells_by_sheet_name("Dynamic")
            .map_err(test_error)?,
        vec![
            Dimensions::new((3, 0), (3, 1)),
            Dimensions::new((3, 2), (4, 2)),
        ]
    );

    let xls_template = directory.path().join("dynamic-template.xls");
    write_xls::<EveryCell, _>(
        &xls_template,
        &WriteOptions {
            sheet_name: "Dynamic".to_owned(),
            need_head: false,
            include_column_indexes: Some(vec![0]),
            ..WriteOptions::default()
        },
        vec![every_cell()],
    )?;
    let xls_output = directory.path().join("dynamic-template-output.xls");
    write_xls::<EveryCell, _>(
        &xls_output,
        &WriteOptions {
            sheet_name: "Dynamic".to_owned(),
            template_file: Some(xls_template),
            include_column_indexes: Some(vec![0, 1, 2]),
            dynamic_head: Some(dynamic_head),
            relative_head_row_index: 2,
            ..WriteOptions::default()
        },
        vec![every_cell()],
    )?;
    let xls: Xls<_> = open_workbook(&xls_output).map_err(test_error)?;
    assert_eq!(
        xls.merge_cells_by_sheet_name("Dynamic")
            .map_err(test_error)?,
        vec![
            Dimensions::new((3, 0), (3, 1)),
            Dimensions::new((3, 2), (4, 2)),
        ]
    );
    Ok(())
}

