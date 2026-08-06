#[test]
#[allow(clippy::too_many_lines)]
fn style_model_maps_every_alignment_and_cycles_content_rows() -> Result<()> {
    for (alignment, expected) in [
        (HorizontalAlignment::General, FormatAlign::General),
        (HorizontalAlignment::Left, FormatAlign::Left),
        (HorizontalAlignment::Center, FormatAlign::Center),
        (HorizontalAlignment::Right, FormatAlign::Right),
        (HorizontalAlignment::Fill, FormatAlign::Fill),
        (HorizontalAlignment::Justify, FormatAlign::Justify),
        (HorizontalAlignment::CenterAcross, FormatAlign::CenterAcross),
    ] {
        assert_eq!(horizontal_format_align(alignment), expected);
    }
    for (alignment, expected) in [
        (VerticalAlignment::Top, FormatAlign::Top),
        (VerticalAlignment::Center, FormatAlign::VerticalCenter),
        (VerticalAlignment::Bottom, FormatAlign::Bottom),
        (VerticalAlignment::Justify, FormatAlign::VerticalJustify),
        (
            VerticalAlignment::Distributed,
            FormatAlign::VerticalDistributed,
        ),
    ] {
        assert_eq!(vertical_format_align(alignment), expected);
    }
    for (alignment, expected) in [
        (ExcelHorizontalAlignment::General, FormatAlign::General),
        (ExcelHorizontalAlignment::Left, FormatAlign::Left),
        (ExcelHorizontalAlignment::Center, FormatAlign::Center),
        (ExcelHorizontalAlignment::Right, FormatAlign::Right),
        (ExcelHorizontalAlignment::Fill, FormatAlign::Fill),
        (ExcelHorizontalAlignment::Justify, FormatAlign::Justify),
        (
            ExcelHorizontalAlignment::CenterAcross,
            FormatAlign::CenterAcross,
        ),
        (
            ExcelHorizontalAlignment::Distributed,
            FormatAlign::Distributed,
        ),
    ] {
        assert_eq!(annotation_horizontal_format_align(alignment), expected);
    }
    for (alignment, expected) in [
        (ExcelVerticalAlignment::Top, FormatAlign::Top),
        (ExcelVerticalAlignment::Center, FormatAlign::VerticalCenter),
        (ExcelVerticalAlignment::Bottom, FormatAlign::Bottom),
        (
            ExcelVerticalAlignment::Justify,
            FormatAlign::VerticalJustify,
        ),
        (
            ExcelVerticalAlignment::Distributed,
            FormatAlign::VerticalDistributed,
        ),
    ] {
        assert_eq!(annotation_vertical_format_align(alignment), expected);
    }
    for (border, expected) in [
        (ExcelBorderStyle::None, FormatBorder::None),
        (ExcelBorderStyle::Thin, FormatBorder::Thin),
        (ExcelBorderStyle::Medium, FormatBorder::Medium),
        (ExcelBorderStyle::Dashed, FormatBorder::Dashed),
        (ExcelBorderStyle::Dotted, FormatBorder::Dotted),
        (ExcelBorderStyle::Thick, FormatBorder::Thick),
        (ExcelBorderStyle::Double, FormatBorder::Double),
        (ExcelBorderStyle::Hair, FormatBorder::Hair),
        (ExcelBorderStyle::MediumDashed, FormatBorder::MediumDashed),
        (ExcelBorderStyle::DashDot, FormatBorder::DashDot),
        (ExcelBorderStyle::MediumDashDot, FormatBorder::MediumDashDot),
        (ExcelBorderStyle::DashDotDot, FormatBorder::DashDotDot),
        (
            ExcelBorderStyle::MediumDashDotDot,
            FormatBorder::MediumDashDotDot,
        ),
        (ExcelBorderStyle::SlantDashDot, FormatBorder::SlantDashDot),
    ] {
        assert_eq!(annotation_border_style(border), expected);
    }
    for (pattern, expected) in [
        (ExcelFillPattern::None, FormatPattern::None),
        (ExcelFillPattern::Solid, FormatPattern::Solid),
        (ExcelFillPattern::MediumGray, FormatPattern::MediumGray),
        (ExcelFillPattern::DarkGray, FormatPattern::DarkGray),
        (ExcelFillPattern::LightGray, FormatPattern::LightGray),
        (
            ExcelFillPattern::DarkHorizontal,
            FormatPattern::DarkHorizontal,
        ),
        (ExcelFillPattern::DarkVertical, FormatPattern::DarkVertical),
        (ExcelFillPattern::DarkDown, FormatPattern::DarkDown),
        (ExcelFillPattern::DarkUp, FormatPattern::DarkUp),
        (ExcelFillPattern::DarkGrid, FormatPattern::DarkGrid),
        (ExcelFillPattern::DarkTrellis, FormatPattern::DarkTrellis),
        (
            ExcelFillPattern::LightHorizontal,
            FormatPattern::LightHorizontal,
        ),
        (
            ExcelFillPattern::LightVertical,
            FormatPattern::LightVertical,
        ),
        (ExcelFillPattern::LightDown, FormatPattern::LightDown),
        (ExcelFillPattern::LightUp, FormatPattern::LightUp),
        (ExcelFillPattern::LightGrid, FormatPattern::LightGrid),
        (ExcelFillPattern::LightTrellis, FormatPattern::LightTrellis),
        (ExcelFillPattern::Gray125, FormatPattern::Gray125),
        (ExcelFillPattern::Gray0625, FormatPattern::Gray0625),
    ] {
        assert_eq!(annotation_fill_pattern(pattern), expected);
    }
    for (underline, expected) in [
        (ExcelUnderline::None, FormatUnderline::None),
        (ExcelUnderline::Single, FormatUnderline::Single),
        (ExcelUnderline::Double, FormatUnderline::Double),
        (
            ExcelUnderline::SingleAccounting,
            FormatUnderline::SingleAccounting,
        ),
        (
            ExcelUnderline::DoubleAccounting,
            FormatUnderline::DoubleAccounting,
        ),
    ] {
        assert_eq!(annotation_underline(underline), expected);
    }
    for (script, expected) in [
        (ExcelFontScript::None, FormatScript::None),
        (ExcelFontScript::Superscript, FormatScript::Superscript),
        (ExcelFontScript::Subscript, FormatScript::Subscript),
    ] {
        assert_eq!(annotation_font_script(script), expected);
    }
    assert_eq!(
        annotation_color(ExcelColor::Rgb(0x0012_3456)),
        Color::RGB(0x0012_3456)
    );
    for index in 0..=65 {
        let _ = annotation_color(ExcelColor::Indexed(index));
    }
    assert_eq!(
        annotation_color(ExcelColor::Indexed(10)),
        Color::RGB(0x00ff_0000)
    );
    assert_eq!(annotation_color(ExcelColor::Indexed(64)), Color::Automatic);
    assert_eq!(annotation_color(ExcelColor::Indexed(65)), Color::Default);

    let annotation_cell = ExcelCellStyle {
        hidden: Some(true),
        locked: Some(false),
        quote_prefix: Some(true),
        horizontal_alignment: Some(ExcelHorizontalAlignment::Distributed),
        wrapped: Some(true),
        vertical_alignment: Some(ExcelVerticalAlignment::Distributed),
        rotation: Some(45),
        indent: Some(2),
        border_left: Some(ExcelBorderStyle::Thin),
        border_right: Some(ExcelBorderStyle::Medium),
        border_top: Some(ExcelBorderStyle::Dashed),
        border_bottom: Some(ExcelBorderStyle::Double),
        left_border_color: Some(ExcelColor::Rgb(0x0011_2233)),
        right_border_color: Some(ExcelColor::Rgb(0x0022_3344)),
        top_border_color: Some(ExcelColor::Rgb(0x0033_4455)),
        bottom_border_color: Some(ExcelColor::Rgb(0x0044_5566)),
        fill_pattern: Some(ExcelFillPattern::Solid),
        fill_background_color: Some(ExcelColor::Rgb(0x0055_6677)),
        fill_foreground_color: Some(ExcelColor::Rgb(0x0066_7788)),
        shrink_to_fit: Some(true),
        data_format: Some(ExcelDataFormat::Custom("0.00")),
        font: None,
    };
    assert_ne!(
        apply_annotation_cell_style(Format::new(), annotation_cell),
        Format::new()
    );
    assert_ne!(
        apply_annotation_cell_style(
            Format::new(),
            ExcelCellStyle {
                data_format: Some(ExcelDataFormat::Builtin(14)),
                ..ExcelCellStyle::new()
            }
        ),
        Format::new()
    );
    let disabled_cell = ExcelCellStyle {
        hidden: Some(false),
        locked: Some(true),
        quote_prefix: Some(false),
        wrapped: Some(false),
        shrink_to_fit: Some(false),
        ..ExcelCellStyle::new()
    };
    let _ = apply_annotation_cell_style(Format::new(), disabled_cell);

    let annotation_font = ExcelFontStyle {
        font_name: Some("Arial"),
        font_height_in_points: Some(12.5),
        italic: Some(true),
        strikeout: Some(true),
        color: Some(ExcelColor::Rgb(0x0077_8899)),
        type_offset: Some(ExcelFontScript::Superscript),
        underline: Some(ExcelUnderline::DoubleAccounting),
        charset: Some(1),
        bold: Some(true),
    };
    assert_ne!(
        apply_annotation_font_style(Format::new(), annotation_font),
        Format::new()
    );
    let disabled_font = ExcelFontStyle {
        italic: Some(false),
        strikeout: Some(false),
        bold: Some(false),
        ..ExcelFontStyle::new()
    };
    let _ = apply_annotation_font_style(Format::new(), disabled_font);

    let head_style = CellStyle::new()
        .bold(true)
        .italic(true)
        .font_color(0x00ff_0000)
        .background_color(0x0000_ff00)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .wrap_text(true)
        .number_format("0.00");
    let content_styles = vec![
        CellStyle::new().font_color(0x0000_00ff),
        CellStyle::new().font_color(0x00ff_0000),
    ];
    let directory = tempdir()?;
    let path = directory.path().join("styles.xlsx");
    write_xlsx::<EveryCell, _>(
        &path,
        &WriteOptions {
            head_style,
            content_styles,
            ..WriteOptions::default()
        },
        vec![every_cell(), every_cell()],
    )?;

    let styles = zip_entry(&path, "xl/styles.xml")?;
    assert!(styles.contains("<b/>"));
    assert!(styles.contains("<i/>"));
    assert!(styles.contains("rgb=\"FFFF0000\""));
    assert!(styles.contains("rgb=\"FF00FF00\""));
    assert!(styles.contains("formatCode=\"0.00\""));
    assert!(styles.contains("horizontal=\"center\""));
    assert!(styles.contains("vertical=\"center\""));
    assert!(styles.contains("wrapText=\"1\""));
    let sheet = zip_entry(&path, "xl/worksheets/sheet1.xml")?;
    assert_ne!(
        cell_style_id(&sheet, "A2").expect("first content style"),
        cell_style_id(&sheet, "A3").expect("second content style")
    );
    Ok(())
}

#[test]
fn annotation_dimensions_apply_field_type_and_explicit_precedence() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("annotation-dimensions.xlsx");
    write_xlsx::<DimensionRow, _>(
        &path,
        &WriteOptions {
            column_widths: vec![(2, 40)],
            ..WriteOptions::default()
        },
        vec![DimensionRow],
    )?;

    let sheet = zip_entry(&path, "xl/worksheets/sheet1.xml")?;
    let field_width = sheet_column_width(&sheet, 1)?;
    let type_width = sheet_column_width(&sheet, 2)?;
    let explicit_width = sheet_column_width(&sheet, 3)?;
    assert!((field_width - type_width - 12.0).abs() < f64::EPSILON);
    assert!((explicit_width - type_width - 22.0).abs() < f64::EPSILON);
    assert!((sheet_row_height(&sheet, 1)? - 24.0).abs() < f64::EPSILON);
    assert!((sheet_row_height(&sheet, 2)? - 16.0).abs() <= 0.25);
    Ok(())
}

#[test]
fn custom_row_height_handler_overrides_annotation_height() -> Result<()> {
    let directory = tempdir()?;
    let path = directory
        .path()
        .join("handler-overrides-annotation-height.xlsx");
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(
        SimpleRowHeightStyleStrategy::new(Some(30), Some(22)),
    )];
    write_xlsx_with_handlers::<DimensionRow, _>(
        &path,
        &WriteOptions::default(),
        vec![DimensionRow],
        &mut handlers,
    )?;

    let sheet = zip_entry(&path, "xl/worksheets/sheet1.xml")?;
    assert!((sheet_row_height(&sheet, 1)? - 30.0).abs() < f64::EPSILON);
    // XLSX stores row heights in quarter-point increments. The writer may
    // normalize the requested value to the nearest representable height.
    assert!((sheet_row_height(&sheet, 2)? - 22.0).abs() <= 0.25);
    Ok(())
}

