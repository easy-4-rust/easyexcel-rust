#[test]
fn sort_order_multiple_fields() {
    let cols = [
        ExcelColumn::new("c", "C", None, 3, None),
        ExcelColumn::new("a", "A", None, 1, None),
        ExcelColumn::new("b", "B", None, 2, None),
    ];
    // Verify order values are accessible
    assert_eq!(cols[0].order, 3);
    assert_eq!(cols[1].order, 1);
    assert_eq!(cols[2].order, 2);
}

// --- Complex header (via ExcelColumn name) ---
#[test]
fn complex_head_multi_level_names() {
    // @ExcelProperty({"主标题", "子标题"})
    let col = ExcelColumn::new("field", "子标题", None, 0, None);
    assert_eq!(col.name, "子标题");
}

// --- List head (dynamic head via ExcelColumn) ---
#[test]
fn list_head_column_names() {
    let cols = [
        ExcelColumn::new("name", "Name", None, 0, None),
        ExcelColumn::new("age", "Age", None, 1, None),
    ];
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].name, "Name");
    assert_eq!(cols[1].name, "Age");
}

// --- No head data ---
#[test]
fn no_head_data_column_width() {
    // When no head, column_width is None by default
    let col = ExcelColumn::new("f", "F", None, 0, None);
    assert!(col.column_width.is_none());
}

// --- Parameter tests ---
#[test]
fn parameter_excel_column_all_fields() {
    let col = ExcelColumn::new("myField", "MyField", Some(5), 100, Some("yyyy-MM-dd"))
        .with_column_width(25)
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
            italic: Some(true),
            ..ExcelFontStyle::new()
        });
    assert_eq!(col.field, "myField");
    assert_eq!(col.name, "MyField");
    assert_eq!(col.index, Some(5));
    assert_eq!(col.order, 100);
    assert_eq!(col.format, Some("yyyy-MM-dd"));
    assert_eq!(col.column_width, Some(25));
    assert!(col.head_style.is_some());
    assert!(col.content_style.is_some());
    assert!(col.head_font_style.is_some());
    assert!(col.content_font_style.is_some());
}

// LoopMergeStrategy tests are in easyexcel-writer crate
// See: easyexcel-writer/src/tests.rs

// --- FillStyle tests ---
#[test]
fn fill_style_content_font() {
    let fs = ExcelFontStyle {
        bold: Some(true),
        font_name: Some("Arial"),
        ..ExcelFontStyle::new()
    };
    let col = ExcelColumn::new("f", "F", None, 0, None).with_content_font_style(fs);
    let fs = col.content_font_style.unwrap();
    assert_eq!(fs.bold, Some(true));
    assert_eq!(fs.font_name, Some("Arial"));
}

// --- FillAnnotation tests ---
#[test]
fn fill_annotation_data_format() {
    let col = ExcelColumn::new("date", "Date", None, 0, Some("yyyy-MM-dd"));
    assert_eq!(col.format, Some("yyyy-MM-dd"));
}

#[test]
fn fill_annotation_column_width() {
    let col = ExcelColumn::new("name", "Name", None, 0, None).with_column_width(25);
    assert_eq!(col.column_width, Some(25));
}

// --- FillAnnotation data format edge cases ---
#[test]
fn fill_annotation_format_long() {
    let col = ExcelColumn::new("ts", "Timestamp", None, 0, Some("yyyy-MM-dd HH:mm:ss.SSS"));
    assert_eq!(col.format, Some("yyyy-MM-dd HH:mm:ss.SSS"));
}

// --- ExcelProperty edge cases ---
#[test]
fn annotation_excel_property_empty_format() {
    let col = ExcelColumn::new("f", "F", None, 0, Some(""));
    assert_eq!(col.format, Some(""));
}

// --- ExcelCellStyle edge cases ---
#[test]
fn cell_style_fill_pattern_none() {
    let style = ExcelCellStyle {
        fill_pattern: Some(ExcelFillPattern::None),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.fill_pattern, Some(ExcelFillPattern::None));
}

#[test]
fn cell_style_border_all_sides() {
    let style = ExcelCellStyle {
        border_left: Some(ExcelBorderStyle::Thin),
        border_right: Some(ExcelBorderStyle::Thin),
        border_top: Some(ExcelBorderStyle::Thin),
        border_bottom: Some(ExcelBorderStyle::Thin),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.border_left, Some(ExcelBorderStyle::Thin));
    assert_eq!(style.border_right, Some(ExcelBorderStyle::Thin));
    assert_eq!(style.border_top, Some(ExcelBorderStyle::Thin));
    assert_eq!(style.border_bottom, Some(ExcelBorderStyle::Thin));
}

// --- ExcelFontStyle edge cases ---
#[test]
fn font_style_superscript() {
    let fs = ExcelFontStyle {
        type_offset: Some(ExcelFontScript::Superscript),
        ..ExcelFontStyle::new()
    };
    assert_eq!(fs.type_offset, Some(ExcelFontScript::Superscript));
}

#[test]
fn font_style_double_underline() {
    let fs = ExcelFontStyle {
        underline: Some(ExcelUnderline::Double),
        ..ExcelFontStyle::new()
    };
    assert_eq!(fs.underline, Some(ExcelUnderline::Double));
}

// --- ExcelColor edge cases ---
#[test]
fn color_indexed_boundary() {
    // 0..=64 are indexed colors
    assert_eq!(ExcelColor::java_or_rgb(0), ExcelColor::Indexed(0));
    assert_eq!(ExcelColor::java_or_rgb(64), ExcelColor::Indexed(64));
    // 65+ are RGB
    assert_eq!(ExcelColor::java_or_rgb(65), ExcelColor::Rgb(65));
}

// --- ExcelWriteMetadata edge cases ---
#[test]
fn write_metadata_all_fields() {
    let m = ExcelWriteMetadata::new()
        .column_width(25)
        .head_row_height(30)
        .content_row_height(20);
    assert_eq!(m.column_width, Some(25));
    assert_eq!(m.head_row_height, Some(30));
    assert_eq!(m.content_row_height, Some(20));
}

// --- DynamicRow edge cases ---
#[test]
fn dynamic_row_empty() {
    let row = DynamicRow::new(BTreeMap::new());
    assert!(row.values().is_empty());
    assert_eq!(row.to_row().unwrap().len(), 0);
}

#[test]
fn dynamic_row_sparse() {
    let mut m = BTreeMap::new();
    m.insert(0, DynamicValue::ActualData(CellValue::Int(1)));
    m.insert(5, DynamicValue::String("hello".to_owned()));
    let row = DynamicRow::new(m);
    let cells = row.to_row().unwrap();
    assert_eq!(cells.len(), 6); // 0..6
    assert_eq!(cells[0], CellValue::Int(1));
    assert_eq!(cells[5], CellValue::String("hello".to_owned()));
}

// --- ConverterRegistry edge cases ---
#[test]
fn converter_registry_clone_independence() {
    struct AnotherConverter;
    impl Converter<String> for AnotherConverter {
        fn convert_to_rust_data(&self, _: &ReadConverterContext<'_>) -> Result<String> {
            Ok("another".to_owned())
        }
    }
    let mut r1 = ConverterRegistry::default();
    r1.register::<String, _>(PrefixConverter);
    let r2 = r1.clone();
    // Both point to same underlying converters
    assert_eq!(r1, r2);
    // Adding to r1 doesn't affect r2
    r1.register::<String, _>(AnotherConverter);
    assert_ne!(r1, r2); // Different now
}

// --- AnalysisContext edge cases ---
#[test]
fn analysis_context_custom_object_none() {
    let ctx = AnalysisContext::new("S", 0, 0);
    assert!(ctx.custom_object().is_none());
    assert!(ctx.custom::<String>().is_none());
}

#[test]
fn analysis_context_with_batch_index_preserves_sheet() {
    let ctx = AnalysisContext::new("MySheet", 3, 42).with_batch_index(7);
    assert_eq!(ctx.sheet_name(), "MySheet");
    assert_eq!(ctx.sheet_no(), 3);
    assert_eq!(ctx.row_index(), 42);
    assert_eq!(ctx.batch_index(), 7);
}

// --- ErrorAction edge cases ---
#[test]
fn error_action_all_variants() {
    assert_eq!(ErrorAction::Continue, ErrorAction::Continue);
    assert_eq!(ErrorAction::SkipRow, ErrorAction::SkipRow);
    assert_eq!(ErrorAction::Stop, ErrorAction::Stop);
    assert_ne!(ErrorAction::Continue, ErrorAction::Stop);
    assert_ne!(ErrorAction::SkipRow, ErrorAction::Stop);
}

// --- ExcelError edge cases ---
#[test]
fn excel_error_io_kinds() {
    let not_found = ExcelError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
    let perm_denied = ExcelError::Io(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "no access",
    ));
    assert!(not_found.to_string().contains("missing"));
    assert!(perm_denied.to_string().contains("no access"));
}

// --- WriteHandler edge cases ---
#[test]
fn write_handler_before_cell_can_skip() {
    struct Skipper;
    impl WriteHandler for Skipper {
        fn before_cell(&mut self, ctx: &mut WriteCellContext) -> Result<()> {
            ctx.skip = true;
            Ok(())
        }
    }
    let mut h = Skipper;
    let mut cl = WriteCellContext::new("S", 0, 0, CellValue::Empty);
    h.before_cell(&mut cl).unwrap();
    assert!(cl.skip);
}

#[test]
fn write_handler_before_cell_can_transform() {
    struct Transformer;
    impl WriteHandler for Transformer {
        fn before_cell(&mut self, ctx: &mut WriteCellContext) -> Result<()> {
            ctx.value = CellValue::String("transformed".to_owned());
            Ok(())
        }
    }
    let mut h = Transformer;
    let mut cl = WriteCellContext::new("S", 0, 0, CellValue::Int(42));
    h.before_cell(&mut cl).unwrap();
    assert_eq!(cl.value, CellValue::String("transformed".to_owned()));
}
