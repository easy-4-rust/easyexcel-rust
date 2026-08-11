//! `EasyExcel` 样式 metadata 到 BIFF8 样式请求的适配层。
//!
//! FONT、XF、FORMAT 与调色板分配算法位于 `easyexcel-xls`；本模块只负责
//! 将 Java `EasyExcel` 对应的 `ExcelCellStyle` / `ExcelFontStyle` 元数据转换为
//! 格式无关门面可使用的 BIFF8 请求。

use crate::core::{
    ExcelBorderStyle, ExcelCellStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern,
    ExcelFontStyle, ExcelHorizontalAlignment, ExcelUnderline, ExcelVerticalAlignment, WriteFont,
};
use crate::write::horizontal_alignment::HorizontalAlignment;
use crate::write::vertical_alignment::VerticalAlignment;

pub use easyexcel_xls::biff8::{
    Biff8BorderStyle, Biff8Color, Biff8FillPattern, Biff8HorizontalAlignment, Biff8NumberFormat,
    Biff8StyleRequest, Biff8Underline, Biff8VerticalAlignment,
};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 把 `EasyExcel` 单元格样式合并到 BIFF8 请求。
pub(crate) fn apply_excel_cell_style(request: &mut Biff8StyleRequest, style: ExcelCellStyle) {
    if let Some(align) = style.horizontal_alignment {
        request.horizontal_alignment = Some(excel_halign(align));
    }
    if let Some(align) = style.vertical_alignment {
        request.vertical_alignment = Some(excel_valign(align));
    }
    if let Some(wrapped) = style.wrapped {
        request.wrap = wrapped;
    }
    if let Some(pattern) = style.fill_pattern {
        request.fill_pattern = Some(excel_fill_pattern(pattern));
    }
    if let Some(color) = style.fill_foreground_color {
        request.fill_foreground_color = Some(excel_color(color));
        if request
            .fill_pattern
            .is_none_or(|pattern| pattern == Biff8FillPattern::None)
        {
            request.fill_pattern = Some(Biff8FillPattern::Solid);
        }
    }
    if let Some(color) = style.fill_background_color {
        request.fill_background_color = Some(excel_color(color));
    }
    if let Some(border) = style.border_left {
        request.border_left = Some(excel_border_style(border));
    }
    if let Some(border) = style.border_right {
        request.border_right = Some(excel_border_style(border));
    }
    if let Some(border) = style.border_top {
        request.border_top = Some(excel_border_style(border));
    }
    if let Some(border) = style.border_bottom {
        request.border_bottom = Some(excel_border_style(border));
    }
    request.border_left_color = style.left_border_color.map(excel_color);
    request.border_right_color = style.right_border_color.map(excel_color);
    request.border_top_color = style.top_border_color.map(excel_color);
    request.border_bottom_color = style.bottom_border_color.map(excel_color);
    if let Some(font) = style.font {
        apply_excel_font_style(request, font);
    }
    if let Some(format) = style.data_format {
        request.number_format = Some(match format {
            ExcelDataFormat::Builtin(index) => Biff8NumberFormat::Builtin(index),
            ExcelDataFormat::Custom(code) => Biff8NumberFormat::Custom(code.to_owned()),
        });
    }
}

const fn excel_border_style(style: ExcelBorderStyle) -> Biff8BorderStyle {
    match style {
        ExcelBorderStyle::None => Biff8BorderStyle::None,
        ExcelBorderStyle::Thin => Biff8BorderStyle::Thin,
        ExcelBorderStyle::Medium => Biff8BorderStyle::Medium,
        ExcelBorderStyle::Dashed => Biff8BorderStyle::Dashed,
        ExcelBorderStyle::Dotted => Biff8BorderStyle::Dotted,
        ExcelBorderStyle::Thick => Biff8BorderStyle::Thick,
        ExcelBorderStyle::Double => Biff8BorderStyle::Double,
        ExcelBorderStyle::Hair => Biff8BorderStyle::Hair,
        ExcelBorderStyle::MediumDashed => Biff8BorderStyle::MediumDashed,
        ExcelBorderStyle::DashDot => Biff8BorderStyle::DashDot,
        ExcelBorderStyle::MediumDashDot => Biff8BorderStyle::MediumDashDot,
        ExcelBorderStyle::DashDotDot => Biff8BorderStyle::DashDotDot,
        ExcelBorderStyle::MediumDashDotDot => Biff8BorderStyle::MediumDashDotDot,
        ExcelBorderStyle::SlantDashDot => Biff8BorderStyle::SlantDashDot,
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 把 `EasyExcel` 字体样式合并到 BIFF8 请求。
pub(crate) fn apply_excel_font_style(request: &mut Biff8StyleRequest, style: ExcelFontStyle) {
    if let Some(name) = style.font_name {
        request.font_name = Some(name.to_owned());
    }
    if let Some(height) = style.font_height_in_points {
        request.font_height_twips = Some(biff8_font_height_twips(height));
        request.font_height_points = None;
    }
    if let Some(italic) = style.italic {
        request.italic = italic;
    }
    if let Some(strikeout) = style.strikeout {
        request.strikeout = strikeout;
    }
    if let Some(bold) = style.bold {
        request.bold = bold;
    }
    if let Some(color) = style.color {
        request.font_color = Some(excel_color(color));
    }
    if let Some(underline) = style.underline {
        request.underline = biff8_underline(underline);
    }
}

/// 把运行时 `WriteFont` 合并到 BIFF8 字体请求。
pub(crate) fn apply_write_font(request: &mut Biff8StyleRequest, font: &WriteFont) {
    if let Some(name) = font.get_font_name() {
        request.font_name = Some(name.to_owned());
    }
    if let Some(height) = font.get_font_height_in_points() {
        request.font_height_twips = Some(biff8_font_height_twips(height));
        request.font_height_points = None;
    }
    if let Some(italic) = font.get_italic() {
        request.italic = italic;
    }
    if let Some(strikeout) = font.get_strikeout() {
        request.strikeout = strikeout;
    }
    if let Some(bold) = font.get_bold() {
        request.bold = bold;
    }
    if let Some(color) = font.get_color() {
        request.font_color = Some(excel_color(color));
    }
    if let Some(underline) = font.get_underline() {
        request.underline = biff8_underline(underline);
    }
}

fn biff8_font_height_twips(height: f64) -> u16 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        (height * 20.0).round().clamp(1.0, f64::from(u16::MAX)) as u16
    }
}

const fn biff8_underline(underline: ExcelUnderline) -> Biff8Underline {
    match underline {
        ExcelUnderline::None => Biff8Underline::None,
        ExcelUnderline::Single => Biff8Underline::Single,
        ExcelUnderline::Double => Biff8Underline::Double,
        ExcelUnderline::SingleAccounting => Biff8Underline::SingleAccounting,
        ExcelUnderline::DoubleAccounting => Biff8Underline::DoubleAccounting,
    }
}

const fn excel_color(color: ExcelColor) -> Biff8Color {
    match color {
        ExcelColor::Indexed(64) => Biff8Color::Automatic,
        ExcelColor::Indexed(index) => Biff8Color::Indexed(index),
        ExcelColor::Rgb(rgb) => Biff8Color::Rgb(rgb),
    }
}

const fn excel_fill_pattern(pattern: ExcelFillPattern) -> Biff8FillPattern {
    match pattern {
        ExcelFillPattern::None => Biff8FillPattern::None,
        ExcelFillPattern::Solid => Biff8FillPattern::Solid,
        ExcelFillPattern::MediumGray => Biff8FillPattern::MediumGray,
        ExcelFillPattern::DarkGray => Biff8FillPattern::DarkGray,
        ExcelFillPattern::LightGray => Biff8FillPattern::LightGray,
        ExcelFillPattern::DarkHorizontal => Biff8FillPattern::DarkHorizontal,
        ExcelFillPattern::DarkVertical => Biff8FillPattern::DarkVertical,
        ExcelFillPattern::DarkDown => Biff8FillPattern::DarkDown,
        ExcelFillPattern::DarkUp => Biff8FillPattern::DarkUp,
        ExcelFillPattern::DarkGrid => Biff8FillPattern::DarkGrid,
        ExcelFillPattern::DarkTrellis => Biff8FillPattern::DarkTrellis,
        ExcelFillPattern::LightHorizontal => Biff8FillPattern::LightHorizontal,
        ExcelFillPattern::LightVertical => Biff8FillPattern::LightVertical,
        ExcelFillPattern::LightDown => Biff8FillPattern::LightDown,
        ExcelFillPattern::LightUp => Biff8FillPattern::LightUp,
        ExcelFillPattern::LightGrid => Biff8FillPattern::LightGrid,
        ExcelFillPattern::LightTrellis => Biff8FillPattern::LightTrellis,
        ExcelFillPattern::Gray125 => Biff8FillPattern::Gray125,
        ExcelFillPattern::Gray0625 => Biff8FillPattern::Gray0625,
    }
}

const fn excel_halign(align: ExcelHorizontalAlignment) -> Biff8HorizontalAlignment {
    match align {
        ExcelHorizontalAlignment::General => Biff8HorizontalAlignment::General,
        ExcelHorizontalAlignment::Left => Biff8HorizontalAlignment::Left,
        ExcelHorizontalAlignment::Center => Biff8HorizontalAlignment::Center,
        ExcelHorizontalAlignment::Right => Biff8HorizontalAlignment::Right,
        ExcelHorizontalAlignment::Fill => Biff8HorizontalAlignment::Fill,
        ExcelHorizontalAlignment::Justify => Biff8HorizontalAlignment::Justify,
        ExcelHorizontalAlignment::CenterAcross => Biff8HorizontalAlignment::CenterAcross,
        ExcelHorizontalAlignment::Distributed => Biff8HorizontalAlignment::Distributed,
    }
}

const fn excel_valign(align: ExcelVerticalAlignment) -> Biff8VerticalAlignment {
    match align {
        ExcelVerticalAlignment::Top => Biff8VerticalAlignment::Top,
        ExcelVerticalAlignment::Center => Biff8VerticalAlignment::Center,
        ExcelVerticalAlignment::Bottom => Biff8VerticalAlignment::Bottom,
        ExcelVerticalAlignment::Justify => Biff8VerticalAlignment::Justify,
        ExcelVerticalAlignment::Distributed => Biff8VerticalAlignment::Distributed,
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) const fn writer_horizontal_alignment(
    align: HorizontalAlignment,
) -> Biff8HorizontalAlignment {
    match align {
        HorizontalAlignment::General => Biff8HorizontalAlignment::General,
        HorizontalAlignment::Left => Biff8HorizontalAlignment::Left,
        HorizontalAlignment::Center => Biff8HorizontalAlignment::Center,
        HorizontalAlignment::Right => Biff8HorizontalAlignment::Right,
        HorizontalAlignment::Fill => Biff8HorizontalAlignment::Fill,
        HorizontalAlignment::Justify => Biff8HorizontalAlignment::Justify,
        HorizontalAlignment::CenterAcross => Biff8HorizontalAlignment::CenterAcross,
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) const fn writer_vertical_alignment(align: VerticalAlignment) -> Biff8VerticalAlignment {
    match align {
        VerticalAlignment::Top => Biff8VerticalAlignment::Top,
        VerticalAlignment::Center => Biff8VerticalAlignment::Center,
        VerticalAlignment::Bottom => Biff8VerticalAlignment::Bottom,
        VerticalAlignment::Justify => Biff8VerticalAlignment::Justify,
        VerticalAlignment::Distributed => Biff8VerticalAlignment::Distributed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_excel_cell_style_sets_horizontal_alignment() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(
            request.horizontal_alignment,
            Some(Biff8HorizontalAlignment::Center)
        );
    }

    #[test]
    fn apply_excel_cell_style_sets_vertical_alignment() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            vertical_alignment: Some(ExcelVerticalAlignment::Top),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(
            request.vertical_alignment,
            Some(Biff8VerticalAlignment::Top)
        );
    }

    #[test]
    fn apply_excel_cell_style_sets_wrap() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            wrapped: Some(true),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert!(request.wrap);
    }

    #[test]
    fn apply_excel_cell_style_sets_fill_pattern() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(request.fill_pattern, Some(Biff8FillPattern::Solid));
    }

    #[test]
    fn apply_excel_cell_style_sets_fill_foreground_and_auto_pattern() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            fill_foreground_color: Some(ExcelColor::Rgb(0xFF0000)),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(
            request.fill_foreground_color,
            Some(Biff8Color::Rgb(0xFF0000))
        );
        // 自动设置 Solid 填充模式
        assert_eq!(request.fill_pattern, Some(Biff8FillPattern::Solid));
    }

    #[test]
    fn apply_excel_cell_style_sets_fill_background() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            fill_background_color: Some(ExcelColor::Indexed(1)),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(request.fill_background_color, Some(Biff8Color::Indexed(1)));
    }

    #[test]
    fn apply_excel_cell_style_sets_borders() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            border_left: Some(ExcelBorderStyle::Thin),
            border_right: Some(ExcelBorderStyle::Medium),
            border_top: Some(ExcelBorderStyle::Thick),
            border_bottom: Some(ExcelBorderStyle::Dashed),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(request.border_left, Some(Biff8BorderStyle::Thin));
        assert_eq!(request.border_right, Some(Biff8BorderStyle::Medium));
        assert_eq!(request.border_top, Some(Biff8BorderStyle::Thick));
        assert_eq!(request.border_bottom, Some(Biff8BorderStyle::Dashed));
    }

    #[test]
    fn apply_excel_cell_style_sets_border_colors() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            left_border_color: Some(ExcelColor::Rgb(0xFF0000)),
            right_border_color: Some(ExcelColor::Rgb(0x00FF00)),
            top_border_color: Some(ExcelColor::Rgb(0x0000FF)),
            bottom_border_color: Some(ExcelColor::Indexed(1)),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(request.border_left_color, Some(Biff8Color::Rgb(0xFF0000)));
        assert_eq!(request.border_right_color, Some(Biff8Color::Rgb(0x00FF00)));
        assert_eq!(request.border_top_color, Some(Biff8Color::Rgb(0x0000FF)));
        assert_eq!(request.border_bottom_color, Some(Biff8Color::Indexed(1)));
    }

    #[test]
    fn apply_excel_cell_style_sets_data_format() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            data_format: Some(ExcelDataFormat::Builtin(14)),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(request.number_format, Some(Biff8NumberFormat::Builtin(14)));
    }

    #[test]
    fn apply_excel_cell_style_sets_custom_data_format() {
        let mut request = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            data_format: Some(ExcelDataFormat::Custom("yyyy-mm-dd")),
            ..ExcelCellStyle::default()
        };
        apply_excel_cell_style(&mut request, style);
        assert_eq!(
            request.number_format,
            Some(Biff8NumberFormat::Custom("yyyy-mm-dd".to_owned()))
        );
    }

    #[test]
    fn apply_excel_font_style_sets_all_properties() {
        let mut request = Biff8StyleRequest::default();
        let font = ExcelFontStyle {
            font_name: Some("Arial"),
            font_height_in_points: Some(12.0),
            italic: Some(true),
            strikeout: Some(true),
            bold: Some(true),
            color: Some(ExcelColor::Rgb(0xFF0000)),
            underline: Some(ExcelUnderline::Single),
            ..ExcelFontStyle::new()
        };
        apply_excel_font_style(&mut request, font);
        assert_eq!(request.font_name, Some("Arial".to_owned()));
        assert!(request.italic);
        assert!(request.strikeout);
        assert!(request.bold);
        assert_eq!(request.font_color, Some(Biff8Color::Rgb(0xFF0000)));
        assert_eq!(request.underline, Biff8Underline::Single);
    }

    #[test]
    fn apply_write_font_sets_properties() {
        let mut request = Biff8StyleRequest::default();
        let font = WriteFont::new()
            .font_name("Times New Roman")
            .font_height_in_points(14.0)
            .italic(true)
            .bold(true);
        apply_write_font(&mut request, &font);
        assert_eq!(request.font_name, Some("Times New Roman".to_owned()));
        assert!(request.italic);
        assert!(request.bold);
    }

    #[test]
    fn writer_horizontal_alignment_maps_all_variants() {
        assert_eq!(
            writer_horizontal_alignment(HorizontalAlignment::General),
            Biff8HorizontalAlignment::General
        );
        assert_eq!(
            writer_horizontal_alignment(HorizontalAlignment::Left),
            Biff8HorizontalAlignment::Left
        );
        assert_eq!(
            writer_horizontal_alignment(HorizontalAlignment::Center),
            Biff8HorizontalAlignment::Center
        );
        assert_eq!(
            writer_horizontal_alignment(HorizontalAlignment::Right),
            Biff8HorizontalAlignment::Right
        );
        assert_eq!(
            writer_horizontal_alignment(HorizontalAlignment::Fill),
            Biff8HorizontalAlignment::Fill
        );
        assert_eq!(
            writer_horizontal_alignment(HorizontalAlignment::Justify),
            Biff8HorizontalAlignment::Justify
        );
        assert_eq!(
            writer_horizontal_alignment(HorizontalAlignment::CenterAcross),
            Biff8HorizontalAlignment::CenterAcross
        );
    }

    #[test]
    fn writer_vertical_alignment_maps_all_variants() {
        assert_eq!(
            writer_vertical_alignment(VerticalAlignment::Top),
            Biff8VerticalAlignment::Top
        );
        assert_eq!(
            writer_vertical_alignment(VerticalAlignment::Center),
            Biff8VerticalAlignment::Center
        );
        assert_eq!(
            writer_vertical_alignment(VerticalAlignment::Bottom),
            Biff8VerticalAlignment::Bottom
        );
        assert_eq!(
            writer_vertical_alignment(VerticalAlignment::Justify),
            Biff8VerticalAlignment::Justify
        );
        assert_eq!(
            writer_vertical_alignment(VerticalAlignment::Distributed),
            Biff8VerticalAlignment::Distributed
        );
    }
}
