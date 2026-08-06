#[test]
fn write_cell_data_has_image_list_interface() {
    let ws = WriteCellData::new(CellValue::Empty)
        .image(ImageData::new(vec![0x89, 0x50, 0x4E, 0x47]))
        .image(ImageData::new(vec![0x42, 0x4D]).image_type(ImageType::Dib));
    assert_eq!(ws.images().len(), 2);
    assert_eq!(ws.value(), &CellValue::Empty);
}

#[test]
fn excel_row_from_row_with_converter() {
    // DynamicRow is a concrete ExcelRow
    let headers = Arc::new(HashMap::new());
    let cells = vec![CellValue::Int(1)];
    let row = RowData::new("S", 0, cells, headers);
    let mut registry = ConverterRegistry::default();
    registry.register::<i64, _>(IntConverter);
    let dynamic = DynamicRow::from_row_with_converters(&row, &registry).unwrap();
    assert_eq!(dynamic.values().len(), 1);
}

#[test]
fn images_variant_round_trip() {
    let img = CellValue::Images {
        value: Box::new(CellValue::String("base".to_owned())),
        images: vec![ImageData::new(vec![1, 2, 3])],
    };
    let img2 = img.clone();
    assert_eq!(img, img2);
    assert_eq!(img.as_text(), "base");
}

#[test]
fn richtext_multiple_ranges() {
    let rt = RichTextStringData::new("abcdef")
        .apply_font_range(0, 3, WriteFont::new().bold(true))
        .apply_font_range(3, 6, WriteFont::new().italic(true));
    assert_eq!(rt.interval_fonts().len(), 2);
    assert_eq!(rt.interval_fonts()[0].start_index(), 0);
    assert_eq!(rt.interval_fonts()[0].end_index(), 3);
    assert_eq!(rt.interval_fonts()[1].start_index(), 3);
    assert_eq!(rt.interval_fonts()[1].end_index(), 6);
}

#[test]
fn coordinate_data_all_getters() {
    let c = CoordinateData::new()
        .first_row_index(1)
        .first_column_index(2)
        .last_row_index(3)
        .last_column_index(4)
        .relative_first_row_index(5)
        .relative_first_column_index(6)
        .relative_last_row_index(7)
        .relative_last_column_index(8);
    assert_eq!(c.get_first_row_index(), Some(1));
    assert_eq!(c.get_first_column_index(), Some(2));
    assert_eq!(c.get_last_row_index(), Some(3));
    assert_eq!(c.get_last_column_index(), Some(4));
    assert_eq!(c.get_relative_first_row_index(), Some(5));
    assert_eq!(c.get_relative_first_column_index(), Some(6));
    assert_eq!(c.get_relative_last_row_index(), Some(7));
    assert_eq!(c.get_relative_last_column_index(), Some(8));
}

#[test]
fn client_anchor_all_fields() {
    let coord = CoordinateData::new()
        .first_row_index(1)
        .first_column_index(2)
        .last_row_index(3)
        .last_column_index(4);
    let anchor = ClientAnchorData::new()
        .coordinates(coord)
        .top(10)
        .right(20)
        .bottom(30)
        .left(40)
        .anchor_type(AnchorType::DontMoveAndResize);
    assert_eq!(anchor.get_top(), Some(10));
    assert_eq!(anchor.get_right(), Some(20));
    assert_eq!(anchor.get_bottom(), Some(30));
    assert_eq!(anchor.get_left(), Some(40));
    assert_eq!(
        anchor.get_anchor_type(),
        Some(AnchorType::DontMoveAndResize)
    );
    assert_eq!(anchor.get_coordinates().get_first_row_index(), Some(1));
}

#[test]
fn image_data_full_builder() {
    let coord = CoordinateData::new()
        .first_row_index(5)
        .first_column_index(6);
    let anchor = ClientAnchorData::new()
        .coordinates(coord)
        .top(100)
        .anchor_type(AnchorType::MoveAndResize);
    let img = ImageData::new(vec![1, 2, 3, 4, 5])
        .image_type(ImageType::Png)
        .anchor(anchor);
    assert_eq!(img.image(), &[1, 2, 3, 4, 5]);
    assert_eq!(img.get_image_type(), Some(ImageType::Png));
    assert_eq!(img.get_anchor().get_top(), Some(100));
}

#[test]
fn write_font_builder_all_fields() {
    let f = WriteFont::new()
        .font_name("Courier".to_owned())
        .font_height_in_points(10.5)
        .italic(true)
        .strikeout(true)
        .color(ExcelColor::Rgb(0x00_FF00))
        .type_offset(ExcelFontScript::Subscript)
        .underline(ExcelUnderline::Double)
        .charset(0)
        .bold(false);
    assert_eq!(f.get_font_name(), Some("Courier"));
    assert_eq!(f.get_font_height_in_points(), Some(10.5));
    assert_eq!(f.get_italic(), Some(true));
    assert_eq!(f.get_strikeout(), Some(true));
    assert_eq!(f.get_color(), Some(ExcelColor::Rgb(0x00_FF00)));
    assert_eq!(f.get_type_offset(), Some(ExcelFontScript::Subscript));
    assert_eq!(f.get_underline(), Some(ExcelUnderline::Double));
    assert_eq!(f.get_charset(), Some(0));
    assert_eq!(f.get_bold(), Some(false));
}

#[test]
fn richtext_apply_font_whole_string() {
    let rt = RichTextStringData::new("Hello")
        .apply_font(WriteFont::new().bold(true).font_height_in_points(14.0));
    assert!(rt.write_font().is_some());
    assert_eq!(rt.write_font().unwrap().get_bold(), Some(true));
    assert_eq!(
        rt.write_font().unwrap().get_font_height_in_points(),
        Some(14.0)
    );
    assert!(rt.interval_fonts().is_empty());
}

#[test]
fn interval_font_fields() {
    let wf = WriteFont::new().italic(true);
    let if_ = IntervalFont::new(10, 20, wf);
    assert_eq!(if_.start_index(), 10);
    assert_eq!(if_.end_index(), 20);
    assert_eq!(if_.write_font().get_italic(), Some(true));
}

#[test]
fn url_image_converter_invalid_url() {
    let conv = UrlImageConverter::new(Duration::from_millis(10), Duration::from_millis(10));
    let ctx = context(None);
    let url = Url::parse("http://localhost:1/unreachable").unwrap();
    let col = ExcelColumn::new("u", "U", Some(0), 0, None);
    let wctx = WriteConverterContext::new(&url, &col, &ctx);
    let result = Converter::<Url>::convert_to_excel_data(&conv, &wctx);
    assert!(result.is_err()); // connection will fail
}

#[test]
fn url_into_excel_cell_delegates() {
    let url = Url::parse("http://localhost:1/unreachable").unwrap();
    let ctx = context(None);
    let result = url.to_excel_cell(&ctx);
    assert!(result.is_err());
}

#[test]
fn error_action_variants() {
    assert_eq!(ErrorAction::Continue, ErrorAction::Continue);
    assert_eq!(ErrorAction::SkipRow, ErrorAction::SkipRow);
    assert_eq!(ErrorAction::Stop, ErrorAction::Stop);
    assert_ne!(ErrorAction::Continue, ErrorAction::Stop);
    // Default is Stop
    assert_eq!(ErrorAction::default(), ErrorAction::Stop);
}

#[test]
fn boxed_read_listener_dispatches() {
    struct Impl;
    impl ReadListener<String> for Impl {
        fn invoke(&mut self, data: String, _ctx: &AnalysisContext) -> Result<()> {
            if data == "stop" {
                return Err(ExcelError::Format("stop".to_owned()));
            }
            Ok(())
        }
        fn do_after_all_analysed(&mut self, _ctx: &AnalysisContext) -> Result<()> {
            Ok(())
        }
    }

    let mut boxed: Box<dyn ReadListener<String>> = Box::new(Impl);
    let ctx = AnalysisContext::new("S", 0, 0);
    boxed.invoke("ok".to_owned(), &ctx).unwrap();
    let result = boxed.invoke("stop".to_owned(), &ctx);
    assert!(result.is_err());
}

#[test]
fn excel_error_eq() {
    let a = ExcelError::Format("x".to_owned());
    let b = ExcelError::Format("x".to_owned());
    assert_eq!(a, b);
    let c = ExcelError::Format("y".to_owned());
    assert_ne!(a, c);
}

#[test]
fn converter_registry_debug() {
    let mut r = ConverterRegistry::default();
    r.register::<String, _>(PrefixConverter);
    let debug = format!("{r:?}");
    assert!(debug.contains("PrefixConverter") || debug.contains("String"));
}

#[test]
fn converter_registry_empty_is_true() {
    let r = ConverterRegistry::default();
    assert!(r.is_empty());
}

#[test]
fn write_workbook_context_various_paths() {
    let ctx1 = WriteWorkbookContext::new("/tmp/out.xlsx");
    assert_eq!(ctx1.path().to_str(), Some("/tmp/out.xlsx"));
    let ctx2 = WriteWorkbookContext::new("relative/path.csv");
    assert_eq!(ctx2.path().to_str(), Some("relative/path.csv"));
}

#[test]
fn write_sheet_context_various_names() {
    let c1 = WriteSheetContext::new("Sheet1");
    assert_eq!(c1.sheet_name(), "Sheet1");
    let c2 = WriteSheetContext::new(String::from("Report"));
    assert_eq!(c2.sheet_name(), "Report");
}

#[test]
fn write_row_context_fields() {
    let ctx = WriteRowContext::new("MySheet", 123, Some(123), true);
    assert_eq!(ctx.sheet_name, "MySheet");
    assert_eq!(ctx.row_index, 123);
    assert!(ctx.is_head);
}

#[test]
fn write_cell_context_skip_and_value() {
    let mut ctx = WriteCellContext::new("S", 0, 0, CellValue::String("v".to_owned()));
    ctx.field = Some("f");
    ctx.skip = true;
    ctx.value = CellValue::Int(42);
    assert!(ctx.skip);
    assert_eq!(ctx.value, CellValue::Int(42));
}

#[test]
fn excel_color_eq_across_variants() {
    assert_ne!(ExcelColor::Indexed(5), ExcelColor::Rgb(5));
    assert_ne!(ExcelColor::Rgb(0xFF_0000), ExcelColor::Indexed(0xFF));
}

#[test]
fn dynamic_row_into_values_ownership() {
    let mut map = BTreeMap::new();
    map.insert(0, DynamicValue::String("x".to_owned()));
    let row = DynamicRow::new(map);
    let vals = row.into_values();
    // row was moved
    assert_eq!(vals.len(), 1);
}

#[test]
fn annotation_excel_property_name_and_index() {
    let col = ExcelColumn::new("name", "Name", Some(0), 0, None);
    assert_eq!(col.field, "name");
    assert_eq!(col.name, "Name");
    assert_eq!(col.index, Some(0));
}

#[test]
fn annotation_excel_property_order() {
    let col = ExcelColumn::new("f", "F", None, 100, None);
    assert_eq!(col.order, 100);
}

#[test]
fn annotation_excel_property_format() {
    let col = ExcelColumn::new("date", "Date", None, 0, Some("yyyy-MM-dd"));
    assert_eq!(col.format, Some("yyyy-MM-dd"));
}

#[test]
fn annotation_column_width_field_level() {
    let col = ExcelColumn::new("name", "Name", None, 0, None).with_column_width(30);
    assert_eq!(col.column_width, Some(30));
}

#[test]
fn annotation_head_row_height() {
    let meta = ExcelWriteMetadata::new().head_row_height(24);
    assert_eq!(meta.head_row_height, Some(24));
}

#[test]
fn annotation_content_row_height() {
    let meta = ExcelWriteMetadata::new().content_row_height(16);
    assert_eq!(meta.content_row_height, Some(16));
}

#[test]
fn annotation_head_style() {
    let style = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
        ..ExcelCellStyle::new()
    };
    let col = ExcelColumn::new("f", "F", None, 0, None).with_head_style(style);
    assert!(col.head_style.is_some());
    assert_eq!(
        col.head_style.unwrap().horizontal_alignment,
        Some(ExcelHorizontalAlignment::Center)
    );
}

#[test]
fn annotation_content_style() {
    let style = ExcelCellStyle {
        vertical_alignment: Some(ExcelVerticalAlignment::Center),
        ..ExcelCellStyle::new()
    };
    let col = ExcelColumn::new("f", "F", None, 0, None).with_content_style(style);
    assert!(col.content_style.is_some());
}

#[test]
fn annotation_head_font_style() {
    let fs = ExcelFontStyle {
        bold: Some(true),
        font_name: Some("Arial"),
        ..ExcelFontStyle::new()
    };
    let col = ExcelColumn::new("f", "F", None, 0, None).with_head_font_style(fs);
    assert!(col.head_font_style.is_some());
    assert_eq!(col.head_font_style.unwrap().bold, Some(true));
}

#[test]
fn date_format_yyyy_mm_dd() {
    let c = context(Some("%Y-%m-%d"));
    let cell = CellValue::String("2026-03-15".to_owned());
    let d = <NaiveDate as FromExcelCell>::from_excel_cell(Some(&cell), &c).unwrap();
    assert_eq!(d, NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
}

#[test]
fn datetime_format_with_time() {
    let c = context(Some("%Y-%m-%d %H:%M:%S"));
    let cell = CellValue::String("2026-03-15 14:30:00".to_owned());
    let dt = <NaiveDateTime as FromExcelCell>::from_excel_cell(Some(&cell), &c).unwrap();
    let expected = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
        chrono::NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
    );
    assert_eq!(dt, expected);
}

#[test]
fn sort_order_field_level() {
    let col = ExcelColumn::new("b", "B", None, 2, None);
    assert_eq!(col.order, 2);
}

#[test]
fn sort_order_index_priority_over_order() {
    let col_with_index = ExcelColumn::new("a", "A", Some(5), 10, None);
    let col_with_order = ExcelColumn::new("b", "B", None, 20, None);
    assert_eq!(col_with_index.index, Some(5));
    assert_eq!(col_with_order.index, None);
    assert_eq!(col_with_order.order, 20);
}

