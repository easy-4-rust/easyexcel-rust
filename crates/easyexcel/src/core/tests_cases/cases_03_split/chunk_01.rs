#[test]
fn row_data_display_values_override() {
    let headers = Arc::new(HashMap::new());
    let cells = vec![CellValue::Float(12_345_678.123_456_7)];
    let mut display_values = HashMap::new();
    display_values.insert(0, "12345678.12".to_owned());
    let row = RowData::new("S", 0, cells, headers).with_display_values(display_values);
    let col = ExcelColumn::new("v", "V", Some(0), 0, None);
    // When ReadDefaultReturn::String (default), dynamic_cell uses display_value
    assert_eq!(
        *row.cell(&col).unwrap(),
        CellValue::Float(12_345_678.123_456_7)
    );
}

#[test]
fn excel_error_data_with_none_column() {
    let err = ExcelError::Data {
        sheet: "S".to_owned(),
        row: 0,
        column: None,
        field: "x",
        value: String::new(),
        message: "err".to_owned(),
    };
    let msg = err.to_string();
    assert!(msg.contains("column=None"));
}

#[test]
fn read_listener_extra_is_noop_by_default() {
    struct NoopListener;
    impl ReadListener<String> for NoopListener {
        fn invoke(&mut self, _data: String, _ctx: &AnalysisContext) -> Result<()> {
            Ok(())
        }
        fn do_after_all_analysed(&mut self, _ctx: &AnalysisContext) -> Result<()> {
            Ok(())
        }
    }
    let mut listener = NoopListener;
    let ctx = AnalysisContext::new("S", 0, 0);
    let extra = CellExtra::new(CellExtraType::Merge, None, 0, 0, 0, 0);
    let _ = listener.extra(&extra, &ctx); // should not panic
}

#[test]
fn write_cell_data_with_images() {
    let ws = WriteCellData::new(CellValue::String("img".to_owned()))
        .image(ImageData::new(vec![0x89, 0x50, 0x4E, 0x47]).image_type(ImageType::Png));
    assert_eq!(ws.images().len(), 1);
    assert_eq!(ws.images()[0].get_image_type(), Some(ImageType::Png));

    let ws2 = ws.image(ImageData::new(vec![0x42, 0x4D]).image_type(ImageType::Dib));
    assert_eq!(ws2.images().len(), 2);
}

#[test]
fn excel_color_java_or_rgb_boundary() {
    assert_eq!(ExcelColor::java_or_rgb(0), ExcelColor::Indexed(0));
    assert_eq!(ExcelColor::java_or_rgb(64), ExcelColor::Indexed(64));
    assert_eq!(ExcelColor::java_or_rgb(65), ExcelColor::Rgb(65));
    assert_eq!(
        ExcelColor::java_or_rgb(0xFF_FFFF),
        ExcelColor::Rgb(0xFF_FFFF)
    );
}

#[test]
fn font_style_builder() {
    let fs = ExcelFontStyle {
        font_name: Some("Times New Roman"),
        font_height_in_points: Some(12.0),
        italic: Some(true),
        bold: Some(true),
        color: Some(ExcelColor::Rgb(0x00_FF00)),
        ..ExcelFontStyle::new()
    };
    assert_eq!(fs.font_name, Some("Times New Roman"));
    assert_eq!(fs.font_height_in_points, Some(12.0));
    assert_eq!(fs.italic, Some(true));
    assert_eq!(fs.bold, Some(true));
}

#[test]
fn excel_column_style_fields() {
    let style = ExcelCellStyle {
        hidden: Some(true),
        ..ExcelCellStyle::new()
    };
    let fs = ExcelFontStyle {
        bold: Some(false),
        ..ExcelFontStyle::new()
    };
    let col = ExcelColumn::new("c", "C", None, 0, None)
        .with_column_width(40)
        .with_head_style(style)
        .with_content_font_style(fs);
    assert!(col.head_style.is_some());
    assert!(col.content_font_style.is_some());
}

#[test]
fn write_metadata_merge_behavior() {
    let base = ExcelWriteMetadata::new()
        .column_width(10)
        .head_row_height(20);
    // Simulate inheritance by copying fields
    let derived = ExcelWriteMetadata {
        column_width: base.column_width,
        head_row_height: base.head_row_height.or(Some(25)),
        content_row_height: None,
        head_style: base.head_style,
        content_style: None,
        head_font_style: None,
        content_font_style: None,
        once_absolute_merge: None,
    };
    assert_eq!(derived.column_width, Some(10));
    assert_eq!(derived.head_row_height, Some(20));
    assert_eq!(derived.content_row_height, None);
}

#[test]
fn cell_value_clone_preserves_all_variants() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();
    let cases = vec![
        CellValue::Empty,
        CellValue::String("abc".to_owned()),
        CellValue::Bool(true),
        CellValue::Int(42),
        CellValue::Float(3.5),
        CellValue::Decimal("1.5".parse().unwrap()),
        CellValue::Date(date),
        CellValue::DateTime(datetime),
        CellValue::Error("#N/A".to_owned()),
        CellValue::Formula("SUM(A1:A2)".to_owned()),
        CellValue::Hyperlink {
            url: "u".to_owned(),
            text: "t".to_owned(),
        },
        CellValue::Comment {
            value: Box::new(CellValue::Empty),
            text: "c".to_owned(),
        },
        CellValue::Image(vec![1]),
        CellValue::RichText(RichTextStringData::new("rt")),
        CellValue::Images {
            value: Box::new(CellValue::Empty),
            images: vec![],
        },
    ];
    for case in &cases {
        let cloned = case.clone();
        assert_eq!(*case, cloned);
    }
}

#[test]
fn excel_error_data_full_display() {
    let err = ExcelError::Data {
        sheet: "Report".to_owned(),
        row: 100,
        column: Some(15),
        field: "revenue",
        value: "not-a-number".to_owned(),
        message: "cannot parse as f64".to_owned(),
    };
    let s = err.to_string();
    assert!(s.contains("Report"));
    assert!(s.contains("100"));
    assert!(s.contains("15"));
    assert!(s.contains("revenue"));
    assert!(s.contains("not-a-number"));
    assert!(s.contains("cannot parse as f64"));
}

#[test]
fn analysis_context_custom_object_downcast() {
    let ctx = AnalysisContext::new("S", 0, 0)
        .with_custom_object(Some(CustomReadObject::new(vec![1u8, 2u8, 3u8])));
    let vec = ctx.custom::<Vec<u8>>();
    assert_eq!(vec, Some(&vec![1u8, 2u8, 3u8]));
    assert!(ctx.custom::<String>().is_none());
}

#[test]
fn analysis_context_no_custom_object() {
    let ctx = AnalysisContext::new("S", 0, 0);
    assert!(ctx.custom_object().is_none());
    assert!(ctx.custom::<String>().is_none());
}

#[test]
fn row_data_dynamic_cell_uses_display_when_string_mode() {
    let cells = vec![CellValue::Float(123_456_789.123_456_79)];
    let mut display = HashMap::new();
    display.insert(0, "123456789.12".to_owned());
    let row = RowData::new("S", 0, cells, Arc::new(HashMap::new())).with_display_values(display);
    let dynamic = DynamicRow::from_row(&row).unwrap();
    assert_eq!(
        dynamic.get(0),
        Some(&DynamicValue::String("123456789.12".to_owned()))
    );
}

#[test]
fn row_data_dynamic_cell_formula_preserved() {
    let cells = vec![CellValue::Int(42)];
    let mut formulas = HashMap::new();
    formulas.insert(0, FormulaData::new("A1+1".to_owned()));
    let row = RowData::new("S", 0, cells, Arc::new(HashMap::new()))
        .with_formulas(formulas)
        .with_read_default_return(ReadDefaultReturn::ReadCellData);
    let dynamic = DynamicRow::from_row(&row).unwrap();
    let cell = dynamic.get(0).unwrap();
    // 守卫断言替代 match 兜底 panic 臂（ReadDefaultReturn::ReadCellData 模式恒产出 ReadCellData）。
    assert!(
        matches!(cell, DynamicValue::ReadCellData(rcd) if rcd.formula().unwrap().formula_value() == "A1+1"),
        "expected ReadCellData, got {cell:?}"
    );
}

#[test]
fn write_handler_all_default_methods() {
    struct AllDefaults;
    impl WriteHandler for AllDefaults {
        fn order(&self) -> i32 {
            0
        }
    }
    let mut h = AllDefaults;
    let wb_ctx = WriteWorkbookContext::new("x.xlsx");
    let sh_ctx = WriteSheetContext::new("S");
    let rw_ctx = WriteRowContext::new("S", 0, Some(0), true);
    let mut cl_ctx = WriteCellContext::new("S", 0, 0, CellValue::Empty);
    assert!(h.before_workbook(&wb_ctx).is_ok());
    assert!(h.after_workbook(&wb_ctx).is_ok());
    assert!(h.before_sheet(&sh_ctx).is_ok());
    assert!(h.after_sheet(&sh_ctx).is_ok());
    assert!(h.before_row(&rw_ctx).is_ok());
    assert!(h.after_row(&rw_ctx).is_ok());
    assert!(h.before_cell(&mut cl_ctx).is_ok());
    assert!(h.after_cell(&cl_ctx).is_ok());
}

#[test]
fn read_cell_data_clone() {
    let rd = ReadCellData::new(
        1,
        2,
        CellValue::Int(3),
        CellValue::Int(3),
        "3".to_owned(),
        Some(FormulaData::new("f".to_owned())),
    );
    let rd2 = rd.clone();
    assert_eq!(rd.row_index(), rd2.row_index());
    assert_eq!(rd.formula().unwrap().formula_value(), "f");
}

#[test]
fn dynamic_value_variants() {
    let vals = vec![
        DynamicValue::Null,
        DynamicValue::String("s".to_owned()),
        DynamicValue::ActualData(CellValue::Bool(true)),
        DynamicValue::ReadCellData(ReadCellData::new(
            0,
            0,
            CellValue::Empty,
            CellValue::Empty,
            String::new(),
            None,
        )),
    ];
    for v in &vals {
        assert_eq!(*v, v.clone());
    }
}

#[test]
fn write_metadata_full_chain() {
    let m = ExcelWriteMetadata::new()
        .column_width(100)
        .head_row_height(50)
        .content_row_height(30)
        .head_style(ExcelCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..ExcelCellStyle::new()
        })
        .content_style(ExcelCellStyle {
            hidden: Some(true),
            ..ExcelCellStyle::new()
        })
        .head_font_style(ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::new()
        })
        .content_font_style(ExcelFontStyle {
            italic: Some(true),
            ..ExcelFontStyle::new()
        });
    assert_eq!(m.column_width, Some(100));
    assert_eq!(m.head_row_height, Some(50));
    assert_eq!(m.content_row_height, Some(30));
    assert!(m.head_style.is_some());
    assert!(m.content_style.is_some());
    assert!(m.head_font_style.is_some());
    assert!(m.content_font_style.is_some());
}

#[test]
fn excel_column_with_format() {
    let col = ExcelColumn::new("date", "Date", None, 0, Some("%Y/%m/%d"));
    assert_eq!(col.format, Some("%Y/%m/%d"));
}

#[test]
fn cell_extra_type_hashset() {
    let mut set = HashSet::new();
    set.insert(CellExtraType::Comment);
    set.insert(CellExtraType::Hyperlink);
    set.insert(CellExtraType::Merge);
    assert!(set.contains(&CellExtraType::Comment));
    // Merge was inserted, so it should be present
    assert!(set.contains(&CellExtraType::Merge));
    // duplicate insert
    set.insert(CellExtraType::Comment);
    assert_eq!(set.len(), 3);
}

#[test]
fn dynamic_row_schema_is_empty() {
    assert!(DynamicRow::schema().is_empty());
}

#[test]
fn dynamic_row_to_row_roundtrip() {
    let mut map = BTreeMap::new();
    map.insert(0, DynamicValue::String("hello".to_owned()));
    map.insert(2, DynamicValue::Null);
    let row = DynamicRow::new(map);
    let cells = row.to_row().unwrap();
    assert_eq!(cells.len(), 3);
    assert_eq!(cells[0], CellValue::String("hello".to_owned()));
    assert_eq!(cells[1], CellValue::Empty);
    assert_eq!(cells[2], CellValue::Empty);
}

#[test]
fn dynamic_row_to_row_empty() {
    let row = DynamicRow::new(BTreeMap::new());
    assert!(row.to_row().unwrap().is_empty());
}

#[test]
fn excel_column_no_style_by_default() {
    let col = ExcelColumn::new("f", "F", None, 0, None);
    assert!(col.head_style.is_none());
    assert!(col.content_style.is_none());
    assert!(col.head_font_style.is_none());
    assert!(col.content_font_style.is_none());
}

#[test]
fn write_cell_data_empty() {
    let ws = WriteCellData::new(CellValue::Empty);
    assert_eq!(*ws.value(), CellValue::Empty);
    assert!(ws.images().is_empty());
}

