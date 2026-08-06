#[test]
fn write_workbook_context() {
    let ctx = WriteWorkbookContext::new("test.xlsx");
    assert_eq!(ctx.path(), std::path::Path::new("test.xlsx"));
}

#[test]
fn write_sheet_context() {
    let ctx = WriteSheetContext::new("Sheet1");
    assert_eq!(ctx.sheet_name(), "Sheet1");
}

#[test]
fn write_row_context() {
    let ctx = WriteRowContext::new("Sheet1", 5, None, false);
    assert_eq!(ctx.row_index, 5);
    assert!(!ctx.is_head);
}

#[test]
fn write_cell_context_skip_value() {
    let mut ctx = WriteCellContext::new("Sheet1", 0, 0, CellValue::String("Alice".to_owned()));
    ctx.field = Some("name");
    assert!(!ctx.skip);
}

#[test]
fn boolean_enum_tristate() {
    assert_eq!(BooleanEnum::Default.value(), None);
    assert_eq!(BooleanEnum::True.value(), Some(true));
    assert_eq!(BooleanEnum::False.value(), Some(false));
}

#[test]
fn alignment_enums_variants() {
    assert_eq!(
        ExcelHorizontalAlignment::Center,
        ExcelHorizontalAlignment::Center
    );
    assert_ne!(
        ExcelHorizontalAlignment::Center,
        ExcelHorizontalAlignment::Left
    );
    assert_eq!(
        ExcelVerticalAlignment::Bottom,
        ExcelVerticalAlignment::Bottom
    );
}

#[test]
fn border_style_and_fill_pattern_enum_variants() {
    // Verify key enum variants exist and are distinct.
    assert_ne!(ExcelBorderStyle::None, ExcelBorderStyle::Thin);
    assert_ne!(ExcelBorderStyle::Thin, ExcelBorderStyle::Medium);
    assert_ne!(ExcelBorderStyle::Double, ExcelBorderStyle::Hair);
    assert_ne!(ExcelBorderStyle::SlantDashDot, ExcelBorderStyle::DashDotDot);

    assert_ne!(ExcelUnderline::None, ExcelUnderline::Single);
    assert_ne!(ExcelUnderline::Single, ExcelUnderline::Double);
    assert_ne!(
        ExcelUnderline::SingleAccounting,
        ExcelUnderline::DoubleAccounting
    );

    assert_ne!(ExcelFontScript::None, ExcelFontScript::Superscript);
    assert_ne!(ExcelFontScript::Superscript, ExcelFontScript::Subscript);

    assert_ne!(ExcelFillPattern::None, ExcelFillPattern::Solid);
    assert_ne!(ExcelFillPattern::Gray125, ExcelFillPattern::Gray0625);
}

#[test]
fn excel_color_indexed_and_rgb() {
    let c1 = ExcelColor::java_or_rgb(5);
    assert_eq!(c1, ExcelColor::Indexed(5));
    let c2 = ExcelColor::java_or_rgb(0xFF_0000);
    assert_eq!(c2, ExcelColor::Rgb(0xFF_0000));
}

#[test]
fn excel_data_format_variants() {
    let builtin = ExcelDataFormat::Builtin(14);
    let custom = ExcelDataFormat::Custom("yyyy/m/d");
    assert_eq!(builtin, ExcelDataFormat::Builtin(14));
    assert_ne!(builtin, custom);
}

#[test]
fn read_write_cell_data_round_trip() {
    let ctx = context(None);
    // Scalar WriteCellData stays unwrapped; Images only wraps when image_data_list is non-empty.
    let ws = WriteCellData::new(CellValue::Int(100));
    let ws_cell = ws.to_excel_cell(&ctx).unwrap();
    assert_eq!(ws_cell, CellValue::Int(100));
    let rd = ReadCellData::new(0, 0, ws_cell.clone(), ws_cell, "100".to_owned(), None);
    assert_eq!(rd.row_index(), 0);
    assert_eq!(rd.raw_value(), &CellValue::Int(100));
    assert_eq!(rd.display_value(), "100");

    let with_image =
        WriteCellData::new(CellValue::Int(100)).image(ImageData::new(vec![0x89, 0x50, 0x4e, 0x47]));
    let imaged = with_image.to_excel_cell(&ctx).unwrap();
    // 整值断言替代 match 兜底 panic 臂（image_data_list 非空时 to_excel_cell 恒构造 Images 包装）。
    assert_eq!(
        imaged,
        CellValue::Images {
            value: Box::new(CellValue::Int(100)),
            images: vec![ImageData::new(vec![0x89, 0x50, 0x4e, 0x47])],
        }
    );
}

#[test]
fn dynamic_row_from_row_data() {
    let headers = Arc::new(HashMap::new());
    let cells = vec![
        CellValue::String("Alice".to_owned()),
        CellValue::Int(30),
        CellValue::Empty,
    ];
    let row_data = RowData::new("S", 0, cells, headers);
    let dynamic = DynamicRow::from_row(&row_data).unwrap();
    assert_eq!(
        dynamic.get(0),
        Some(&DynamicValue::String("Alice".to_owned()))
    );
    // Empty cells become empty strings in default ReadDefaultReturn::String mode
    assert_eq!(dynamic.get(2), Some(&DynamicValue::String(String::new())));
}

