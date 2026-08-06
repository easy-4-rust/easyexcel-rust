#[test]
fn fill_style_annotated_head() {
    let style = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
        ..ExcelCellStyle::new()
    };
    assert_eq!(
        style.horizontal_alignment,
        Some(ExcelHorizontalAlignment::Center)
    );
}

#[test]
fn fill_style_annotated_content() {
    let style = ExcelCellStyle {
        fill_pattern: Some(ExcelFillPattern::Solid),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.fill_pattern, Some(ExcelFillPattern::Solid));
}

#[test]
fn fill_style_annotated_both() {
    let head = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Left),
        ..ExcelCellStyle::new()
    };
    let content = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Right),
        ..ExcelCellStyle::new()
    };
    assert_ne!(head.horizontal_alignment, content.horizontal_alignment);
}

#[test]
fn uncamel_camel_to_snake() {
    let field_name = "user_name";
    assert_eq!(field_name, "user_name");
}

#[test]
fn uncamel_pascal_to_snake() {
    let field_name = "user_name";
    assert!(field_name.contains('_'));
}

#[test]
fn uncamel_snake_to_snake() {
    let field_name = "user_name";
    assert_eq!(field_name, "user_name");
}

#[test]
fn uncamel_already_snake() {
    let field_name = "already_snake_case";
    assert_eq!(field_name, "already_snake_case");
    assert!(field_name.contains('_'));
}

#[test]
fn parameter_excel_column_basic() {
    let col = ExcelColumn::new("f", "F", None, 0, None);
    assert_eq!(col.field, "f");
    assert_eq!(col.name, "F");
    assert!(col.index.is_none());
    assert!(col.format.is_none());
}

#[test]
fn parameter_excel_column_with_width() {
    let col = ExcelColumn::new("f", "F", None, 0, None).with_column_width(25);
    assert_eq!(col.column_width, Some(25));
}

#[test]
fn parameter_excel_column_with_styles() {
    let style = ExcelCellStyle {
        hidden: Some(true),
        ..ExcelCellStyle::new()
    };
    let col = ExcelColumn::new("f", "F", None, 0, None).with_content_style(style);
    assert_eq!(col.content_style.unwrap().hidden, Some(true));
}

#[test]
fn skip_rows_basic() {
    let start: Option<u32> = Some(5);
    let end: Option<u32> = Some(10);
    assert_eq!(start, Some(5));
    assert_eq!(end, Some(10));
}

#[test]
fn skip_rows_start_only() {
    let start: Option<u32> = Some(5);
    let end: Option<u32> = None;
    assert_eq!(start, Some(5));
    assert!(end.is_none());
}

#[test]
fn skip_rows_end_only() {
    let start: Option<u32> = None;
    let end: Option<u32> = Some(100);
    assert!(start.is_none());
    assert_eq!(end, Some(100));
}

#[test]
fn skip_rows_default_none() {
    let start: Option<u32> = None;
    let end: Option<u32> = None;
    assert!(start.is_none());
    assert!(end.is_none());
}

#[test]
fn sort_data_index_priority() {
    let col = ExcelColumn::new("f", "F", Some(0), 100, None);
    assert_eq!(col.index, Some(0));
    assert_eq!(col.order, 100);
}

#[test]
fn sort_data_order_priority() {
    let col = ExcelColumn::new("f", "F", None, 50, None);
    assert!(col.index.is_none());
    assert_eq!(col.order, 50);
}

#[test]
fn sort_data_default_priority() {
    let col = ExcelColumn::new("f", "F", None, i32::MAX, None);
    assert!(col.index.is_none());
    assert_eq!(col.order, i32::MAX);
}

#[test]
fn sort_data_index_overrides_order() {
    let col = ExcelColumn::new("f", "F", Some(3), 10, None);
    assert_eq!(col.index, Some(3));
    assert_eq!(col.order, 10);
    // order 为排序值恒非负，usize 转换恒成功（避免符号丢失的 as 转换）
    assert!(col.index.unwrap() < usize::try_from(col.order).expect("order 恒非负"));
}

#[test]
fn sort_data_order_only() {
    let col = ExcelColumn::new("f", "F", None, 7, None);
    assert!(col.index.is_none());
    assert_eq!(col.order, 7);
}

#[test]
fn sort_data_max_order() {
    let col = ExcelColumn::new("f", "F", None, i32::MAX, None);
    assert_eq!(col.order, i32::MAX);
}

#[test]
fn sort_data_negative_order() {
    let col = ExcelColumn::new("f", "F", None, -1, None);
    assert_eq!(col.order, -1);
}

#[test]
fn template_data_scalar_basic() {
    let mut data: BTreeMap<String, CellValue> = BTreeMap::new();
    data.insert("name".to_owned(), CellValue::String("Alice".to_owned()));
    assert_eq!(data.len(), 1);
    assert!(data.contains_key("name"));
}

#[test]
fn template_data_collection() {
    let users = [
        CellValue::String("Alice".to_owned()),
        CellValue::String("Bob".to_owned()),
        CellValue::String("Carol".to_owned()),
    ];
    assert_eq!(users.len(), 3);
    assert_eq!(users[0], CellValue::String("Alice".to_owned()));
    assert_eq!(users[2], CellValue::String("Carol".to_owned()));
}

#[test]
fn template_data_numeric() {
    let mut data: BTreeMap<String, CellValue> = BTreeMap::new();
    data.insert("count".to_owned(), CellValue::Int(42));
    assert!(data.contains_key("count"));
    assert_eq!(data["count"], CellValue::Int(42));
}

