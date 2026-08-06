//! EasyExcel 样式 metadata 到 BIFF8 样式请求的适配层。
//!
//! FONT、XF、FORMAT 与调色板分配算法位于 `easyexcel-xls`；本模块只负责
//! 将 Java EasyExcel 对应的 `ExcelCellStyle` / `ExcelFontStyle` 元数据转换为
//! 格式无关门面可使用的 BIFF8 请求。

use crate::core::{
    ExcelCellStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern, ExcelFontStyle,
    ExcelHorizontalAlignment, ExcelVerticalAlignment,
};
use crate::write::horizontal_alignment::HorizontalAlignment;
use crate::write::vertical_alignment::VerticalAlignment;

pub use easyexcel_xls::biff8::{
    Biff8Color, Biff8FillPattern, Biff8HorizontalAlignment, Biff8NumberFormat, Biff8StyleRequest,
    Biff8VerticalAlignment,
};

/// 把 EasyExcel 单元格样式合并到 BIFF8 请求。
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

/// 把 EasyExcel 字体样式合并到 BIFF8 请求。
pub(crate) fn apply_excel_font_style(request: &mut Biff8StyleRequest, style: ExcelFontStyle) {
    if let Some(name) = style.font_name {
        request.font_name = Some(name.to_owned());
    }
    if let Some(height) = style.font_height_in_points {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            request.font_height_points = Some(height.round().clamp(1.0, 409.0) as u16);
        }
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

pub(crate) const fn writer_vertical_alignment(align: VerticalAlignment) -> Biff8VerticalAlignment {
    match align {
        VerticalAlignment::Top => Biff8VerticalAlignment::Top,
        VerticalAlignment::Center => Biff8VerticalAlignment::Center,
        VerticalAlignment::Bottom => Biff8VerticalAlignment::Bottom,
        VerticalAlignment::Justify => Biff8VerticalAlignment::Justify,
        VerticalAlignment::Distributed => Biff8VerticalAlignment::Distributed,
    }
}
