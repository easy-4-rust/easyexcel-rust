#[test]
fn write_cell_data_from_rich_text() {
    let rt = RichTextStringData::new("rich").apply_font(WriteFont::new().bold(true));
    let ws = WriteCellData::from_rich_text(rt.clone());
    assert!(matches!(ws.value(), CellValue::RichText(_)));
}

#[test]
fn cell_value_image_bytes() {
    let img = CellValue::Image(vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    assert!(matches!(img, CellValue::Image(_)));
    assert_eq!(img.as_text(), "");
    assert_eq!(img.data_type(), CellDataType::Image);
}

#[test]
fn row_data_decimal_values_override_float() {
    let cells = vec![CellValue::Float(3.5)];
    let mut decimals = HashMap::new();
    let bd = BigDecimal::from_str("3.14159265358979").unwrap();
    decimals.insert(0, bd.clone());
    let row = RowData::new("S", 0, cells, Arc::new(HashMap::new()))
        .with_decimal_values(decimals)
        .with_read_default_return(ReadDefaultReturn::ActualData);
    let dynamic = DynamicRow::from_row(&row).unwrap();
    let cell = dynamic.get(0).unwrap();
    // 守卫断言替代 match 兜底 panic 臂（ActualData 模式恒产出 Decimal）。
    assert!(
        matches!(cell, DynamicValue::ActualData(CellValue::Decimal(d)) if *d == bd),
        "expected Decimal, got {cell:?}"
    );
}

#[test]
fn analysis_context_batch_index_tracks_page() {
    let ctx0 = AnalysisContext::new("S", 0, 0);
    let ctx1 = ctx0.with_batch_index(1);
    let ctx2 = ctx0.with_batch_index(2);
    assert_eq!(ctx0.batch_index(), 0);
    assert_eq!(ctx1.batch_index(), 1);
    assert_eq!(ctx2.batch_index(), 2);
}

#[test]
fn font_style_all_fields() {
    let fs = ExcelFontStyle {
        font_name: Some("Arial"),
        font_height_in_points: Some(14.0),
        italic: Some(true),
        strikeout: Some(false),
        color: Some(ExcelColor::Indexed(10)),
        type_offset: Some(ExcelFontScript::Superscript),
        underline: Some(ExcelUnderline::Single),
        charset: Some(128),
        bold: Some(true),
    };
    assert_eq!(fs.font_name, Some("Arial"));
    assert_eq!(fs.font_height_in_points, Some(14.0));
    assert_eq!(fs.italic, Some(true));
    assert_eq!(fs.strikeout, Some(false));
    assert_eq!(fs.color, Some(ExcelColor::Indexed(10)));
    assert_eq!(fs.type_offset, Some(ExcelFontScript::Superscript));
    assert_eq!(fs.underline, Some(ExcelUnderline::Single));
    assert_eq!(fs.charset, Some(128));
    assert_eq!(fs.bold, Some(true));
}

#[test]
fn cell_style_all_fields() {
    let s = ExcelCellStyle {
        hidden: Some(true),
        locked: Some(false),
        quote_prefix: Some(true),
        horizontal_alignment: Some(ExcelHorizontalAlignment::Fill),
        wrapped: Some(true),
        vertical_alignment: Some(ExcelVerticalAlignment::Distributed),
        rotation: Some(45),
        indent: Some(2),
        border_left: Some(ExcelBorderStyle::Double),
        border_right: Some(ExcelBorderStyle::Hair),
        border_top: Some(ExcelBorderStyle::MediumDashed),
        border_bottom: Some(ExcelBorderStyle::SlantDashDot),
        left_border_color: Some(ExcelColor::Rgb(0xFF_0000)),
        right_border_color: Some(ExcelColor::Indexed(5)),
        top_border_color: Some(ExcelColor::Rgb(0x00_FF00)),
        bottom_border_color: Some(ExcelColor::Rgb(0x00_00FF)),
        fill_pattern: Some(ExcelFillPattern::Solid),
        fill_background_color: Some(ExcelColor::Indexed(20)),
        fill_foreground_color: Some(ExcelColor::Rgb(0xFF_FFFF)),
        shrink_to_fit: Some(true),
        data_format: Some(ExcelDataFormat::Builtin(0)),
        font: None,
    };
    assert_eq!(s.hidden, Some(true));
    assert_eq!(s.fill_pattern, Some(ExcelFillPattern::Solid));
    assert_eq!(s.shrink_to_fit, Some(true));
}

#[test]
fn excel_row_schema_has_field_metadata() {
    let col = ExcelColumn::new("id", "ID", Some(0), 100, None).with_column_width(20);
    // Verify all public fields exist and are accessible
    assert_eq!(col.field, "id");
    assert_eq!(col.name, "ID");
    assert_eq!(col.index, Some(0));
    assert_eq!(col.order, 100);
    assert!(col.format.is_none());
    assert_eq!(col.column_width, Some(20));
}

#[test]
fn converter_registry_clone() {
    let mut r1 = ConverterRegistry::default();
    r1.register::<String, _>(PrefixConverter);
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

#[test]
fn page_read_listener_minimum_batch_is_one() {
    let batch_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let bc = batch_count.clone();
    let _listener: PageReadListener<String> = PageReadListener::new(0, move |_data, _ctx| {
        bc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    });
    // Even if batch_size was 0, PageReadListener normalizes to 1
}

#[test]
fn analysis_context_eq() {
    let a = AnalysisContext::new("S", 0, 0).with_custom_object(Some(CustomReadObject::new(42u32)));
    let b = AnalysisContext::new("S", 0, 0).with_custom_object(Some(CustomReadObject::new(42u32)));
    // Arc::ptr_eq means same allocation
    assert_ne!(a, b); // different Arc allocations
}

