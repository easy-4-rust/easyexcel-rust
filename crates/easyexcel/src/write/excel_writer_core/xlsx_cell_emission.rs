fn write_rich_text(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    data: &RichTextStringData,
    cell_format: &Format,
) -> Result<()> {
    if data.text_string().is_empty() {
        generation::GeneratedCellValue::Text(String::new())
            .write(worksheet, row, column, Some(cell_format))
            .map_err(ExcelError::from)?;
        return Ok(());
    }
    let runs = rich_text_run_specs(data)?;
    generation::write_rich_string_with_font_specs(worksheet, row, column, &runs, cell_format)
        .map_err(format_error)
}

fn rich_text_run_specs(data: &RichTextStringData) -> Result<Vec<(FontFormatSpec, String)>> {
    let text = data.text_string();
    let intervals = data
        .interval_fonts()
        .iter()
        .map(|interval| (interval.start_index(), interval.end_index()))
        .collect::<Vec<_>>();
    let segments = easyexcel_model::segment_utf16_text(text, &intervals)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut runs = Vec::with_capacity(segments.len());
    for segment in segments {
        let font = segment.interval_index.map_or(data.write_font(), |index| {
            Some(data.interval_fonts()[index].write_font())
        });
        runs.push((
            font.map_or_else(FontFormatSpec::default, rich_text_font_spec),
            segment.text,
        ));
    }
    Ok(runs)
}

fn rich_text_font_spec(font: &WriteFont) -> FontFormatSpec {
    FontFormatSpec {
        name: font.get_font_name().map(str::to_owned),
        size: font.get_font_height_in_points(),
        italic: font.get_italic(),
        strikeout: font.get_strikeout(),
        color: font.get_color().map(annotation_color),
        script: font.get_type_offset().map(annotation_font_script),
        underline: font.get_underline().map(annotation_underline),
        charset: font.get_charset(),
        bold: font.get_bold(),
    }
}

/// 将 Java 富文本元数据转换为 XLSX 模板引擎的 typed-cell 值。
pub(crate) fn template_rich_text_cell_value(
    data: &RichTextStringData,
) -> Result<easyexcel_xlsx::xlsx::template_xml::TemplateCellValue> {
    let rich_text = if data.text_string().is_empty() {
        easyexcel_xlsx::TemplateRichText::plain("")
    } else {
        easyexcel_xlsx::TemplateRichText::from_runs(&rich_text_run_specs(data)?)
            .map_err(ExcelError::from)?
    };
    Ok(easyexcel_xlsx::xlsx::template_xml::TemplateCellValue::RichText(
        rich_text,
    ))
}

fn insert_image_data(
    worksheet: &mut Worksheet,
    current_row: u32,
    current_column: u16,
    data: &ImageData,
    layout: &ImageLayout,
) -> Result<()> {
    let anchor = data.get_anchor();
    let coordinates = anchor.get_coordinates();
    let resolved = easyexcel_xlsx::resolve_image_anchor(
        easyexcel_xlsx::ImageAnchorSpec {
            current_row,
            current_column,
            first_row: easyexcel_xlsx::AnchorCoordinate {
                absolute: coordinates.get_first_row_index(),
                relative: coordinates.get_relative_first_row_index(),
            },
            first_column: easyexcel_xlsx::AnchorCoordinate {
                absolute: coordinates.get_first_column_index().map(u32::from),
                relative: coordinates.get_relative_first_column_index(),
            },
            last_row: easyexcel_xlsx::AnchorCoordinate {
                absolute: coordinates.get_last_row_index(),
                relative: coordinates.get_relative_last_row_index(),
            },
            last_column: easyexcel_xlsx::AnchorCoordinate {
                absolute: coordinates.get_last_column_index().map(u32::from),
                relative: coordinates.get_relative_last_column_index(),
            },
            left: anchor.get_left().unwrap_or(0),
            right: anchor.get_right().unwrap_or(0),
            top: anchor.get_top().unwrap_or(0),
            bottom: anchor.get_bottom().unwrap_or(0),
        },
        |column| layout.column_width(column),
        |row| layout.row_height(row),
    )
    .map_err(ExcelError::from)?;
    let movement = match anchor
        .get_anchor_type()
        .unwrap_or(AnchorType::MoveAndResize)
    {
        AnchorType::MoveAndResize => easyexcel_xlsx::TemplateImageMovement::MoveAndResize,
        AnchorType::DontMoveDoResize | AnchorType::MoveDontResize => {
            easyexcel_xlsx::TemplateImageMovement::MoveDontResize
        }
        AnchorType::DontMoveAndResize => easyexcel_xlsx::TemplateImageMovement::DontMoveOrResize,
    };
    generation::insert_scaled_image_with_policy(
        worksheet,
        resolved.first_row,
        resolved.first_column,
        data.image(),
        resolved.width,
        resolved.height,
        movement,
        resolved.left,
        resolved.top,
    )
    .map_err(format_error)
}

// 按值传入与调用点构造惯例一致，改引用会增加不必要的借用链
#[allow(clippy::large_types_passed_by_value)]
fn cell_format(context: CellFormatContext<'_>) -> Format {
    let mut format = generation::new_format();
    // Annotation style merged with handler strategy style
    // (Java `WriteCellStyle.merge(strategy, cellData.getOrCreateStyle())`).
    let mut annotation_cell = context.converted_cell.cloned();
    if let Some(annotation_style) = context.cell {
        let annotation_style = crate::WriteCellStyle::from(annotation_style);
        annotation_cell = Some(merge_write_cell_style(
            &annotation_style,
            annotation_cell.unwrap_or_default(),
        ));
    }
    if let Some(handler_style) = context.handler_cell {
        let handler_style = crate::WriteCellStyle::from(handler_style);
        annotation_cell = Some(merge_write_cell_style(
            &handler_style,
            annotation_cell.unwrap_or_default(),
        ));
    }
    // Nested WriteFont / ExcelFontStyle on merged cell style
    // (Java WriteCellStyle.writeFont merge onto annotation HeadFontStyle/ContentFontStyle).
    let mut font = context.font.map(write_font_from_excel_font_style);
    let merged_has_data_format = annotation_cell
        .as_ref()
        .is_some_and(|style| style.data_format.is_some());
    if let Some(style) = annotation_cell {
        if let Some(style_font) = style.font.as_ref() {
            font = Some(match font {
                Some(target) => merge_write_font(style_font, target),
                None => style_font.clone(),
            });
        }
        format = apply_annotation_cell_style(format, style.engine_cell_style());
    }
    if let Some(handler_font) = context.handler_font {
        font = Some(match font {
            Some(target) => merge_write_font(&handler_font, target),
            None => handler_font,
        });
    }
    if !merged_has_data_format && let Some(number_format) = context.converted_data_format {
        format = generation::with_number_format(format, number_format);
    }
    if let Some(font) = font {
        format = apply_write_font_style(format, &font);
    }
    let Some(style) = context.explicit else {
        return format;
    };
    generation::apply_format_spec(
        format,
        &FormatSpec {
            horizontal_alignment: style.horizontal_alignment.map(horizontal_format_align),
            vertical_alignment: style.vertical_alignment.map(vertical_format_align),
            wrap_text: style.wrap_text.then_some(true),
            fill_pattern: style.background_color.map(|_| FormatPattern::Solid),
            fill_background_color: style.background_color.map(generation::color_from_rgb),
            number_format: style
                .number_format
                .as_ref()
                .map(|value| NumberFormatSpec::Custom(value.clone())),
            font: FontFormatSpec {
                bold: style.bold.then_some(true),
                italic: style.italic.then_some(true),
                color: style.font_color.map(generation::color_from_rgb),
                ..FontFormatSpec::default()
            },
            ..FormatSpec::default()
        },
    )
}

fn apply_annotation_cell_style(mut format: Format, style: ExcelCellStyle) -> Format {
    let font = style.font;
    let spec = FormatSpec {
        hidden: style.hidden,
        locked: style.locked,
        quote_prefix: style.quote_prefix,
        horizontal_alignment: style
            .horizontal_alignment
            .map(annotation_horizontal_format_align),
        vertical_alignment: style
            .vertical_alignment
            .map(annotation_vertical_format_align),
        wrap_text: style.wrapped,
        rotation: style.rotation,
        indent: style.indent,
        border_left: style.border_left.map(annotation_border_style),
        border_right: style.border_right.map(annotation_border_style),
        border_top: style.border_top.map(annotation_border_style),
        border_bottom: style.border_bottom.map(annotation_border_style),
        left_border_color: style.left_border_color.map(annotation_color),
        right_border_color: style.right_border_color.map(annotation_color),
        top_border_color: style.top_border_color.map(annotation_color),
        bottom_border_color: style.bottom_border_color.map(annotation_color),
        fill_pattern: style.fill_pattern.map(annotation_fill_pattern),
        fill_background_color: style.fill_background_color.map(annotation_color),
        fill_foreground_color: style.fill_foreground_color.map(annotation_color),
        shrink_to_fit: style.shrink_to_fit,
        number_format: style.data_format.map(|value| match value {
            ExcelDataFormat::Builtin(index) => NumberFormatSpec::Builtin(index),
            ExcelDataFormat::Custom(value) => NumberFormatSpec::Custom(value.to_owned()),
        }),
        font: FontFormatSpec::default(),
    };
    format = generation::apply_format_spec(format, &spec);
    // Nested WriteFont / ExcelFontStyle (Java WriteCellStyle.writeFont)
    if let Some(font) = font {
        format = apply_annotation_font_style(format, font);
    }
    format
}

fn apply_annotation_font_style(format: Format, style: ExcelFontStyle) -> Format {
    generation::apply_font_format_spec(
        format,
        &FontFormatSpec {
            name: style.font_name.map(str::to_owned),
            size: style.font_height_in_points,
            italic: style.italic,
            strikeout: style.strikeout,
            color: style.color.map(annotation_color),
            script: style.type_offset.map(annotation_font_script),
            underline: style.underline.map(annotation_underline),
            charset: style.charset,
            bold: style.bold,
        },
    )
}

fn apply_write_font_style(format: Format, style: &WriteFont) -> Format {
    generation::apply_font_format_spec(
        format,
        &FontFormatSpec {
            name: style.get_font_name().map(str::to_owned),
            size: style.get_font_height_in_points(),
            italic: style.get_italic(),
            strikeout: style.get_strikeout(),
            color: style.get_color().map(annotation_color),
            script: style.get_type_offset().map(annotation_font_script),
            underline: style.get_underline().map(annotation_underline),
            charset: style.get_charset(),
            bold: style.get_bold(),
        },
    )
}

fn annotation_color(color: ExcelColor) -> Color {
    match color {
        ExcelColor::Rgb(value) => generation::color_from_rgb(value),
        ExcelColor::Indexed(index) => generation::color_from_indexed(index),
    }
}

const fn annotation_horizontal_format_align(alignment: ExcelHorizontalAlignment) -> FormatAlign {
    match alignment {
        ExcelHorizontalAlignment::General => FormatAlign::General,
        ExcelHorizontalAlignment::Left => FormatAlign::Left,
        ExcelHorizontalAlignment::Center => FormatAlign::Center,
        ExcelHorizontalAlignment::Right => FormatAlign::Right,
        ExcelHorizontalAlignment::Fill => FormatAlign::Fill,
        ExcelHorizontalAlignment::Justify => FormatAlign::Justify,
        ExcelHorizontalAlignment::CenterAcross => FormatAlign::CenterAcross,
        ExcelHorizontalAlignment::Distributed => FormatAlign::Distributed,
    }
}

const fn annotation_vertical_format_align(alignment: ExcelVerticalAlignment) -> FormatAlign {
    match alignment {
        ExcelVerticalAlignment::Top => FormatAlign::Top,
        ExcelVerticalAlignment::Center => FormatAlign::VerticalCenter,
        ExcelVerticalAlignment::Bottom => FormatAlign::Bottom,
        ExcelVerticalAlignment::Justify => FormatAlign::VerticalJustify,
        ExcelVerticalAlignment::Distributed => FormatAlign::VerticalDistributed,
    }
}

const fn annotation_border_style(border: ExcelBorderStyle) -> FormatBorder {
    match border {
        ExcelBorderStyle::None => FormatBorder::None,
        ExcelBorderStyle::Thin => FormatBorder::Thin,
        ExcelBorderStyle::Medium => FormatBorder::Medium,
        ExcelBorderStyle::Dashed => FormatBorder::Dashed,
        ExcelBorderStyle::Dotted => FormatBorder::Dotted,
        ExcelBorderStyle::Thick => FormatBorder::Thick,
        ExcelBorderStyle::Double => FormatBorder::Double,
        ExcelBorderStyle::Hair => FormatBorder::Hair,
        ExcelBorderStyle::MediumDashed => FormatBorder::MediumDashed,
        ExcelBorderStyle::DashDot => FormatBorder::DashDot,
        ExcelBorderStyle::MediumDashDot => FormatBorder::MediumDashDot,
        ExcelBorderStyle::DashDotDot => FormatBorder::DashDotDot,
        ExcelBorderStyle::MediumDashDotDot => FormatBorder::MediumDashDotDot,
        ExcelBorderStyle::SlantDashDot => FormatBorder::SlantDashDot,
    }
}

const fn annotation_fill_pattern(pattern: ExcelFillPattern) -> FormatPattern {
    match pattern {
        ExcelFillPattern::None => FormatPattern::None,
        ExcelFillPattern::Solid => FormatPattern::Solid,
        ExcelFillPattern::MediumGray => FormatPattern::MediumGray,
        ExcelFillPattern::DarkGray => FormatPattern::DarkGray,
        ExcelFillPattern::LightGray => FormatPattern::LightGray,
        ExcelFillPattern::DarkHorizontal => FormatPattern::DarkHorizontal,
        ExcelFillPattern::DarkVertical => FormatPattern::DarkVertical,
        ExcelFillPattern::DarkDown => FormatPattern::DarkDown,
        ExcelFillPattern::DarkUp => FormatPattern::DarkUp,
        ExcelFillPattern::DarkGrid => FormatPattern::DarkGrid,
        ExcelFillPattern::DarkTrellis => FormatPattern::DarkTrellis,
        ExcelFillPattern::LightHorizontal => FormatPattern::LightHorizontal,
        ExcelFillPattern::LightVertical => FormatPattern::LightVertical,
        ExcelFillPattern::LightDown => FormatPattern::LightDown,
        ExcelFillPattern::LightUp => FormatPattern::LightUp,
        ExcelFillPattern::LightGrid => FormatPattern::LightGrid,
        ExcelFillPattern::LightTrellis => FormatPattern::LightTrellis,
        ExcelFillPattern::Gray125 => FormatPattern::Gray125,
        ExcelFillPattern::Gray0625 => FormatPattern::Gray0625,
    }
}

const fn annotation_underline(underline: ExcelUnderline) -> FormatUnderline {
    match underline {
        ExcelUnderline::None => FormatUnderline::None,
        ExcelUnderline::Single => FormatUnderline::Single,
        ExcelUnderline::Double => FormatUnderline::Double,
        ExcelUnderline::SingleAccounting => FormatUnderline::SingleAccounting,
        ExcelUnderline::DoubleAccounting => FormatUnderline::DoubleAccounting,
    }
}

const fn annotation_font_script(script: ExcelFontScript) -> FormatScript {
    match script {
        ExcelFontScript::None => FormatScript::None,
        ExcelFontScript::Superscript => FormatScript::Superscript,
        ExcelFontScript::Subscript => FormatScript::Subscript,
    }
}

const fn horizontal_format_align(alignment: HorizontalAlignment) -> FormatAlign {
    match alignment {
        HorizontalAlignment::General => FormatAlign::General,
        HorizontalAlignment::Left => FormatAlign::Left,
        HorizontalAlignment::Center => FormatAlign::Center,
        HorizontalAlignment::Right => FormatAlign::Right,
        HorizontalAlignment::Fill => FormatAlign::Fill,
        HorizontalAlignment::Justify => FormatAlign::Justify,
        HorizontalAlignment::CenterAcross => FormatAlign::CenterAcross,
    }
}

const fn vertical_format_align(alignment: VerticalAlignment) -> FormatAlign {
    match alignment {
        VerticalAlignment::Top => FormatAlign::Top,
        VerticalAlignment::Center => FormatAlign::VerticalCenter,
        VerticalAlignment::Bottom => FormatAlign::Bottom,
        VerticalAlignment::Justify => FormatAlign::VerticalJustify,
        VerticalAlignment::Distributed => FormatAlign::VerticalDistributed,
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn finite_decimal_f64(value: &BigDecimal, format: &str) -> Result<f64> {
    easyexcel_format::finite_decimal_f64(value, format).map_err(format_error)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn decimal_integer_requires_text(value: &BigDecimal) -> Result<bool> {
    easyexcel_format::decimal_integer_requires_text(value).map_err(format_error)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn to_column(index: usize) -> Result<u16> {
    generation::column_index(index).map_err(format_error)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn format_error(error: impl std::fmt::Display) -> ExcelError {
    ExcelError::Format(error.to_string())
}
