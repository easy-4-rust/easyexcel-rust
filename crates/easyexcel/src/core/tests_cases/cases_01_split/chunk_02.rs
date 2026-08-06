#[test]
fn dynamic_row_get_by_column() {
    let mut map = BTreeMap::new();
    map.insert(0, DynamicValue::String("hello".to_owned()));
    map.insert(2, DynamicValue::ActualData(CellValue::Int(42)));
    let row = DynamicRow::new(map);
    assert_eq!(row.get(0), Some(&DynamicValue::String("hello".to_owned())));
    assert_eq!(row.get(1), None);
    assert_eq!(
        row.get(2),
        Some(&DynamicValue::ActualData(CellValue::Int(42)))
    );
    assert_eq!(row.values().len(), 2);
    assert_eq!(row.into_values().len(), 2);
}

#[test]
fn dynamic_row_clone_eq() {
    let map = BTreeMap::new();
    let a = DynamicRow::new(map.clone());
    let b = DynamicRow::new(map);
    assert_eq!(a, b);
}

#[test]
fn read_default_return_variants() {
    assert_eq!(ReadDefaultReturn::String, ReadDefaultReturn::String);
    assert_eq!(ReadDefaultReturn::ActualData, ReadDefaultReturn::ActualData);
    assert_eq!(
        ReadDefaultReturn::ReadCellData,
        ReadDefaultReturn::ReadCellData
    );
}

#[test]
fn excel_error_display() {
    let err = ExcelError::SheetNotFound("Sheet2".to_owned());
    assert!(err.to_string().contains("Sheet2"));

    let err2 = ExcelError::Format("bad xml".to_owned());
    assert!(err2.to_string().contains("bad xml"));

    let err3 = ExcelError::Unsupported("write xls".to_owned());
    assert!(err3.to_string().contains("write xls"));

    let err4 = ExcelError::Data {
        sheet: "S1".to_owned(),
        row: 5,
        column: Some(3),
        field: "age",
        value: "abc".to_owned(),
        message: "bad int".to_owned(),
    };
    let msg = err4.to_string();
    assert!(msg.contains("S1"));
    assert!(msg.contains('5'));
    assert!(msg.contains("age"));
    assert!(msg.contains("bad int"));
}

#[test]
fn excel_error_from_io() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let excel_err = ExcelError::Io(io_err);
    assert!(excel_err.to_string().contains("file missing"));
}

#[test]
fn excel_column_builder_chain() {
    let col = ExcelColumn::new("age", "Age", Some(1), 10, None)
        .with_column_width(20)
        .with_head_style(ExcelCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..ExcelCellStyle::new()
        })
        .with_content_style(ExcelCellStyle {
            hidden: Some(true),
            ..ExcelCellStyle::new()
        })
        .with_head_font_style(ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::new()
        })
        .with_content_font_style(ExcelFontStyle {
            font_name: Some("Arial"),
            ..ExcelFontStyle::new()
        });
    assert_eq!(col.field, "age");
    assert_eq!(col.name, "Age");
    assert_eq!(col.index, Some(1));
    assert_eq!(col.column_width, Some(20));
}

#[test]
fn excel_cell_style_fields() {
    let style = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
        vertical_alignment: Some(ExcelVerticalAlignment::Top),
        border_left: Some(ExcelBorderStyle::Thin),
        fill_pattern: Some(ExcelFillPattern::Solid),
        data_format: Some(ExcelDataFormat::Custom("0.00")),
        ..ExcelCellStyle::new()
    };
    assert_eq!(
        style.horizontal_alignment,
        Some(ExcelHorizontalAlignment::Center)
    );
    assert_eq!(style.vertical_alignment, Some(ExcelVerticalAlignment::Top));
    assert_eq!(style.border_left, Some(ExcelBorderStyle::Thin));
    assert_eq!(style.fill_pattern, Some(ExcelFillPattern::Solid));
    assert_eq!(style.data_format, Some(ExcelDataFormat::Custom("0.00")));
}

#[test]
fn excel_write_metadata_builder_chain() {
    let meta = ExcelWriteMetadata::new()
        .column_width(25)
        .head_row_height(30)
        .content_row_height(20)
        .head_style(ExcelCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..ExcelCellStyle::new()
        })
        .head_font_style(ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::new()
        });
    assert_eq!(meta.column_width, Some(25));
    assert_eq!(meta.head_row_height, Some(30));
    assert_eq!(meta.content_row_height, Some(20));
}

#[test]
fn cell_extra_fields() {
    let extra = CellExtra::new(
        CellExtraType::Comment,
        Some("this is a comment".to_owned()),
        0,
        0,
        1,
        1,
    );
    assert_eq!(extra.extra_type(), CellExtraType::Comment);
    assert_eq!(extra.text(), Some("this is a comment"));
    assert_eq!(extra.first_row_index(), 0);
    assert_eq!(extra.last_column_index(), 1);
}

#[test]
fn cell_extra_merge_range() {
    let merge = CellExtra::new(CellExtraType::Merge, None, 1, 5, 0, 3);
    assert_eq!(merge.first_row_index(), 1);
    assert_eq!(merge.last_row_index(), 5);
}

#[test]
fn row_data_cell_resolution() {
    let mut headers = HashMap::new();
    headers.insert("Name".to_owned(), 0);
    headers.insert("Age".to_owned(), 1);
    let cells = vec![CellValue::String("Alice".to_owned()), CellValue::Int(30)];
    let row = RowData::new("Sheet1", 0, cells, Arc::new(headers));

    let name_col = ExcelColumn::new("name", "Name", None, 0, None);
    let age_col = ExcelColumn::new("age", "Age", Some(1), 10, None);

    assert_eq!(
        row.cell(&name_col),
        Some(&CellValue::String("Alice".to_owned()))
    );
    assert_eq!(row.cell(&age_col), Some(&CellValue::Int(30)));
}

#[test]
fn row_data_formula_resolution() {
    let headers = Arc::new(HashMap::new());
    let cells = vec![CellValue::Empty, CellValue::Float(10.0)];
    let mut formulas = HashMap::new();
    formulas.insert(1, FormulaData::new("SUM(A1:A5)".to_owned()));
    let row = RowData::new("S", 0, cells, headers).with_formulas(formulas);

    let col = ExcelColumn::new("total", "Total", Some(1), 0, None);
    let formula = row.formula(&col).expect("formula present");
    assert_eq!(formula.formula_value(), "SUM(A1:A5)");
}

