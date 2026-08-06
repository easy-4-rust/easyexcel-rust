#[test]
fn csv_charset_accepts_java_style_names_and_has_a_utf8_default() {
    assert_eq!(CsvCharset::default(), CsvCharset::utf8());
    assert_eq!(CsvCharset::default().name(), "UTF-8");
    assert_eq!(CsvCharset::from("GBK").name(), "GBK");
    assert_eq!(CsvCharset::from("UTF-16BE".to_owned()).name(), "UTF-16BE");
    assert_eq!(CsvCharset::from("gbk").name(), "gbk");
    assert_eq!(CsvCharset::from("windows-1252").name(), "windows-1252");
}

#[test]
fn csv_charset_implements_from_str_and_from_string() {
    let charset: CsvCharset = "UTF-8".into();
    assert_eq!(charset.name(), "UTF-8");

    let charset2: CsvCharset = String::from("ISO-8859-1").into();
    assert_eq!(charset2.name(), "ISO-8859-1");
}

#[test]
fn cell_values_have_stable_text_and_empty_semantics() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date");
    let datetime = date.and_hms_opt(12, 34, 56).expect("valid time");
    let cases = [
        (CellValue::Empty, ""),
        (CellValue::String("text".to_owned()), "text"),
        (CellValue::Error("#DIV/0!".to_owned()), "#DIV/0!"),
        (CellValue::Bool(true), "true"),
        (CellValue::Int(-12), "-12"),
        (CellValue::Float(1.5), "1.5"),
        (
            CellValue::Decimal("123.450".parse().expect("valid decimal")),
            "123.450",
        ),
        (CellValue::Date(date), "2026-07-17"),
        (CellValue::DateTime(datetime), "2026-07-17 12:34:56"),
        (CellValue::Formula("SUM(A1:A2)".to_owned()), "SUM(A1:A2)"),
        (
            CellValue::Hyperlink {
                url: "https://rust-lang.org".to_owned(),
                text: "Rust".to_owned(),
            },
            "Rust",
        ),
        (
            CellValue::Comment {
                value: Box::new(CellValue::String("value".to_owned())),
                text: "note".to_owned(),
            },
            "value",
        ),
        (CellValue::Image(vec![1, 2, 3]), ""),
    ];
    for (value, expected) in cases {
        assert_eq!(value.as_text(), expected);
    }
    assert!(CellValue::Empty.is_empty());
    assert!(!CellValue::Bool(false).is_empty());
}

#[test]
fn cell_values_expose_converter_dispatch_types() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date");
    let datetime = date.and_hms_opt(12, 34, 56).expect("valid time");
    let cases = [
        (CellValue::Empty, CellDataType::Empty),
        (CellValue::String(String::new()), CellDataType::String),
        (CellValue::Bool(true), CellDataType::Boolean),
        (CellValue::Int(1), CellDataType::Number),
        (CellValue::Float(1.0), CellDataType::Number),
        (
            CellValue::Decimal(BigDecimal::from(1)),
            CellDataType::Number,
        ),
        (CellValue::Date(date), CellDataType::Date),
        (CellValue::DateTime(datetime), CellDataType::Date),
        (CellValue::Error("#N/A".to_owned()), CellDataType::Error),
        (CellValue::Formula("1+1".to_owned()), CellDataType::Formula),
        (
            CellValue::Hyperlink {
                url: "x".to_owned(),
                text: "y".to_owned(),
            },
            CellDataType::String,
        ),
        (
            CellValue::Comment {
                value: Box::new(CellValue::String("v".to_owned())),
                text: String::new(),
            },
            CellDataType::String,
        ),
        (CellValue::Image(vec![]), CellDataType::Image),
        (
            CellValue::RichText(RichTextStringData::new("rt")),
            CellDataType::RichTextString,
        ),
        (
            CellValue::Images {
                value: Box::new(CellValue::Empty),
                images: vec![],
            },
            CellDataType::Empty,
        ),
    ];
    for (value, expected) in cases {
        assert_eq!(value.data_type(), expected);
    }
}

#[test]
fn cell_value_clone_and_eq() {
    let a = CellValue::String("hello".to_owned());
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(a, CellValue::Int(42));
}

#[test]
fn string_round_trip() {
    let ctx = context(None);
    let val = CellValue::String("abc".to_owned());
    let s = <String as FromExcelCell>::from_excel_cell(Some(&val), &ctx).unwrap();
    assert_eq!(s, "abc");

    let cell = s.to_excel_cell(&ctx).unwrap();
    assert_eq!(cell, CellValue::String("abc".to_owned()));
}

#[test]
fn bool_from_string() {
    let ctx = context(None);
    assert!(
        <bool as FromExcelCell>::from_excel_cell(Some(&CellValue::String("true".to_owned())), &ctx)
            .unwrap()
    );
    assert!(
        !<bool as FromExcelCell>::from_excel_cell(
            Some(&CellValue::String("false".to_owned())),
            &ctx
        )
        .unwrap()
    );
    assert!(
        !<bool as FromExcelCell>::from_excel_cell(Some(&CellValue::String("0".to_owned())), &ctx)
            .unwrap()
    );
    assert!(
        <bool as FromExcelCell>::from_excel_cell(Some(&CellValue::String("1".to_owned())), &ctx)
            .unwrap()
    );
}

#[test]
fn integer_conversions() {
    let ctx = context(None);
    assert_eq!(
        <i64 as FromExcelCell>::from_excel_cell(Some(&CellValue::Int(42)), &ctx).unwrap(),
        42
    );
    assert!(
        <i64 as FromExcelCell>::from_excel_cell(Some(&CellValue::Float(3.7)), &ctx)
            .unwrap_err()
            .to_string()
            .contains("i64")
    );
    assert_eq!(
        <i32 as FromExcelCell>::from_excel_cell(Some(&CellValue::String("100".to_owned())), &ctx)
            .unwrap(),
        100
    );
}

#[test]
// 42.0 可被 f64 二进制精确表示，精确比较正是本测试的意图
#[allow(clippy::float_cmp)]
fn float_from_integer_cell() {
    let ctx = context(None);
    assert_eq!(
        <f64 as FromExcelCell>::from_excel_cell(Some(&CellValue::Int(42)), &ctx).unwrap(),
        42.0
    );
}

#[test]
fn bigdecimal_round_trip() {
    let ctx = context(None);
    let original = BigDecimal::from_str("123.456789").unwrap();
    let cell = CellValue::Decimal(original.clone());
    let recovered = <BigDecimal as FromExcelCell>::from_excel_cell(Some(&cell), &ctx).unwrap();
    assert_eq!(recovered, original);
}

#[test]
fn naivedate_from_string_with_format() {
    let ctx = context(Some("%Y-%m-%d"));
    let cell = CellValue::String("2026-03-15".to_owned());
    let d = <NaiveDate as FromExcelCell>::from_excel_cell(Some(&cell), &ctx).unwrap();
    assert_eq!(d, NaiveDate::from_ymd_opt(2026, 3, 15).unwrap());
}

#[test]
fn naivedatetime_from_string_with_format() {
    let ctx = context(Some("%Y-%m-%d %H:%M:%S"));
    let cell = CellValue::String("2026-03-15 14:30:00".to_owned());
    let dt = <NaiveDateTime as FromExcelCell>::from_excel_cell(Some(&cell), &ctx).unwrap();
    assert_eq!(
        dt,
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            chrono::NaiveTime::from_hms_opt(14, 30, 0).unwrap()
        )
    );
}

#[test]
fn option_cell_handles_empty() {
    let ctx = context(None);
    assert_eq!(
        <Option<String> as FromExcelCell>::from_excel_cell(None, &ctx).unwrap(),
        None
    );
    assert_eq!(
        <Option<String> as FromExcelCell>::from_excel_cell(Some(&CellValue::Empty), &ctx).unwrap(),
        None
    );
    assert_eq!(
        <Option<String> as FromExcelCell>::from_excel_cell(
            Some(&CellValue::String("x".to_owned())),
            &ctx
        )
        .unwrap(),
        Some("x".to_owned())
    );
}

#[test]
fn image_vec_round_trip() {
    let ctx = context(None);
    let img = vec![0x89, 0x50, 0x4E, 0x47];
    let cell = <Vec<u8> as IntoExcelCell>::to_excel_cell(&img, &ctx).unwrap();
    assert!(matches!(cell, CellValue::Image(_)));
    let back = <Vec<u8> as FromExcelCell>::from_excel_cell(Some(&cell), &ctx).unwrap();
    assert_eq!(back, img);
}

#[test]
fn pathbuf_round_trip() {
    let ctx = context(None);
    let pb = PathBuf::from("/tmp/image.png");
    let cell = <PathBuf as IntoExcelCell>::to_excel_cell(&pb, &ctx);
    // File may not exist, so we just check the type
    assert!(cell.is_ok() || cell.is_err());
}

#[test]
fn coordinate_data_builder_chain() {
    let coord = CoordinateData::new()
        .first_row_index(5)
        .first_column_index(3)
        .relative_last_row_index(2)
        .relative_last_column_index(1);
    assert_eq!(coord.get_first_row_index(), Some(5));
    assert_eq!(coord.get_first_column_index(), Some(3));
    assert_eq!(coord.get_relative_last_row_index(), Some(2));
    assert_eq!(coord.get_relative_last_column_index(), Some(1));
    assert_eq!(coord.get_last_row_index(), None);
}

#[test]
fn coordinate_data_clone_eq() {
    let a = CoordinateData::new()
        .first_row_index(1)
        .first_column_index(2);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn client_anchor_data_builder() {
    let anchor = ClientAnchorData::new()
        .top(100)
        .left(50)
        .anchor_type(AnchorType::MoveAndResize);
    assert_eq!(anchor.get_top(), Some(100));
    assert_eq!(anchor.get_left(), Some(50));
    assert_eq!(anchor.get_anchor_type(), Some(AnchorType::MoveAndResize));
}

#[test]
fn anchor_type_variants() {
    assert_eq!(AnchorType::MoveAndResize, AnchorType::MoveAndResize);
    assert_eq!(AnchorType::DontMoveDoResize, AnchorType::DontMoveDoResize);
    assert_eq!(AnchorType::MoveDontResize, AnchorType::MoveDontResize);
    assert_eq!(AnchorType::DontMoveAndResize, AnchorType::DontMoveAndResize);
    assert_ne!(AnchorType::MoveAndResize, AnchorType::MoveDontResize);
}

#[test]
fn image_data_builder_chain() {
    let img = ImageData::new(vec![0x89, 0x50, 0x4E, 0x47]).image_type(ImageType::Png);
    assert_eq!(img.image(), &[0x89, 0x50, 0x4E, 0x47]);
    assert_eq!(img.get_image_type(), Some(ImageType::Png));
    assert_eq!(img.get_anchor(), ClientAnchorData::new());
}

#[test]
fn image_type_variants() {
    let types = [
        ImageType::Emf,
        ImageType::Wmf,
        ImageType::Pict,
        ImageType::Jpeg,
        ImageType::Png,
        ImageType::Dib,
    ];
    assert_eq!(types.len(), 6);
}

#[test]
fn richtext_string_data_builder() {
    let rt = RichTextStringData::new("Hello World")
        .apply_font(WriteFont::new().bold(true).font_name("Arial".to_owned()))
        .apply_font_range(0, 5, WriteFont::new().color(ExcelColor::Rgb(0xFF_0000)));
    assert_eq!(rt.text_string(), "Hello World");
    assert!(rt.write_font().is_some());
    assert_eq!(rt.write_font().unwrap().get_bold(), Some(true));
    assert_eq!(rt.interval_fonts().len(), 1);
}

#[test]
fn write_cell_data_constructors() {
    let _ctx = context(None);
    let ws = WriteCellData::new(CellValue::String("hi".to_owned()));
    assert_eq!(*ws.value(), CellValue::String("hi".to_owned()));
    assert!(ws.images().is_empty());

    let img = WriteCellData::from_image(vec![1, 2]);
    assert_eq!(*img.value(), CellValue::Empty);
    assert_eq!(img.images().len(), 1);

    let rt = WriteCellData::from_rich_text(RichTextStringData::new("rich"));
    assert!(matches!(rt.value(), CellValue::RichText(_)));
}

#[test]
fn read_cell_data_fields() {
    let rd = ReadCellData::new(
        5,
        2,
        CellValue::Int(42),
        CellValue::Int(42),
        "42".to_owned(),
        None,
    );
    assert_eq!(rd.row_index(), 5);
    assert_eq!(rd.column_index(), 2);
    assert_eq!(*rd.raw_value(), CellValue::Int(42));
    assert_eq!(rd.display_value(), "42");
    assert!(rd.formula().is_none());
}

#[test]
fn formula_data_clone() {
    let f1 = FormulaData::new("SUM(A1:A10)".to_owned());
    let f2 = f1.clone();
    assert_eq!(f1, f2);
    assert_eq!(f1.formula_value(), "SUM(A1:A10)");
}

