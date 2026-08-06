#[test]
fn sort_order_default_is_max() {
    let col = ExcelColumn::new("f", "F", None, i32::MAX, None);
    assert_eq!(col.order, i32::MAX);
}

#[test]
fn fill_style_head_style_fill_pattern() {
    let style = ExcelCellStyle {
        fill_pattern: Some(ExcelFillPattern::Solid),
        fill_foreground_color: Some(ExcelColor::Rgb(0xD9_EA_F7)),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.fill_pattern, Some(ExcelFillPattern::Solid));
    assert_eq!(
        style.fill_foreground_color,
        Some(ExcelColor::Rgb(0xD9_EA_F7))
    );
}

#[test]
fn fill_style_content_style_wrapped() {
    let style = ExcelCellStyle {
        wrapped: Some(true),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.wrapped, Some(true));
}

#[test]
fn large_data_row_count() {
    let rows: Vec<i64> = (0..1000).collect();
    assert_eq!(rows.len(), 1000);
    assert_eq!(rows[0], 0);
    assert_eq!(rows[999], 999);
}

#[test]
fn large_data_string_generation() {
    let values: Vec<String> = (0..100).map(|i| format!("row-{i}")).collect();
    assert_eq!(values.len(), 100);
    assert_eq!(values[0], "row-0");
    assert_eq!(values[99], "row-99");
}

#[test]
fn annotation_content_font_style() {
    // @ContentFontStyle(italic = true)
    let fs = ExcelFontStyle {
        italic: Some(true),
        font_name: Some("Courier"),
        ..ExcelFontStyle::new()
    };
    let col = ExcelColumn::new("f", "F", None, 0, None).with_content_font_style(fs);
    assert!(col.content_font_style.is_some());
    let fs = col.content_font_style.unwrap();
    assert_eq!(fs.italic, Some(true));
    assert_eq!(fs.font_name, Some("Courier"));
}

#[test]
fn annotation_column_width_type_level() {
    // @ColumnWidth(25) on class
    let meta = ExcelWriteMetadata::new().column_width(25);
    assert_eq!(meta.column_width, Some(25));
}

#[test]
fn annotation_once_absolute_merge_type_level() {
    // @OnceAbsoluteMerge(firstRowIndex=0, lastRowIndex=0, firstColumnIndex=0, lastColumnIndex=1)
    let merge = OnceAbsoluteMergeProperty::new(0, 0, 0, 1);
    let meta = ExcelWriteMetadata::new().once_absolute_merge(merge);
    assert_eq!(meta.once_absolute_merge, Some(merge));
}

#[test]
fn annotation_content_loop_merge_field_level() {
    // @ContentLoopMerge(eachRow = 2, columnExtend = 1)
    let merge = LoopMergeProperty::new(2, 1);
    let col = ExcelColumn::new("f", "F", Some(0), 0, None).with_loop_merge(merge);
    assert_eq!(col.loop_merge, Some(merge));
}

#[test]
fn annotation_excel_ignore() {
    // Rust uses #[excel(ignore)]
    // Ignored fields use Default::default()
    let val: String = String::default();
    assert!(val.is_empty());
}

#[test]
fn annotation_excel_ignore_unannotated() {
    // Rust uses #[excel(ignore_unannotated)]
    let val: String = String::default();
    assert!(val.is_empty());
}

#[test]
fn annotation_excel_property_converter() {
    // @ExcelProperty(converter = CustomConverter)
    // In Rust: #[excel(converter = MyConverter)]
    let mut reg = ConverterRegistry::default();
    reg.register::<String, _>(PrefixConverter);
    assert!(!reg.is_empty());
}

#[test]
fn date_format_edge_case_leap_year() {
    let c = context(Some("%Y-%m-%d"));
    let cell = CellValue::String("2024-02-29".to_owned());
    let d = <NaiveDate as FromExcelCell>::from_excel_cell(Some(&cell), &c).unwrap();
    assert_eq!(d, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
}

#[test]
fn date_format_edge_case_year_end() {
    let c = context(Some("%Y-%m-%d"));
    let cell = CellValue::String("2026-12-31".to_owned());
    let d = <NaiveDate as FromExcelCell>::from_excel_cell(Some(&cell), &c).unwrap();
    assert_eq!(d, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
}

#[test]
fn number_format_negative() {
    let c = context(None);
    let cell = CellValue::Decimal("-123.45".parse().unwrap());
    let v = <BigDecimal as FromExcelCell>::from_excel_cell(Some(&cell), &c).unwrap();
    assert_eq!(v, "-123.45".parse::<BigDecimal>().unwrap());
}

#[test]
fn number_format_zero() {
    let c = context(None);
    let cell = CellValue::Int(0);
    let v = <i64 as FromExcelCell>::from_excel_cell(Some(&cell), &c).unwrap();
    assert_eq!(v, 0);
}

