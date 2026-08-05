//! EasyExcel 样式 metadata 到 BIFF8 样式请求的适配层。
//!
//! FONT、XF、FORMAT 与调色板分配算法位于 `easyexcel-xls`；本模块只负责
//! 将 Java EasyExcel 对应的 `ExcelCellStyle` / `ExcelFontStyle` 元数据转换为
//! 格式无关门面可使用的 BIFF8 请求。

use crate::core::{
    ExcelCellStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern, ExcelFontStyle,
    ExcelHorizontalAlignment, ExcelVerticalAlignment,
};

pub use easyexcel_xls::biff8::{Biff8NumberFormat, Biff8StyleRequest, Biff8StyleTable};

/// 把 EasyExcel 单元格样式合并到 BIFF8 请求。
pub(crate) fn apply_excel_cell_style(request: &mut Biff8StyleRequest, style: ExcelCellStyle) {
    if let Some(align) = style.horizontal_alignment {
        request.halign = Some(excel_halign(align));
    }
    if let Some(align) = style.vertical_alignment {
        request.valign = Some(excel_valign(align));
    }
    if let Some(wrapped) = style.wrapped {
        request.wrap = wrapped;
    }
    if let Some(pattern) = style.fill_pattern {
        request.fill_pattern = Some(excel_fill_pattern(pattern));
    }
    if let Some(color) = style.fill_foreground_color {
        request.fill_fg_icv = Some(indexed_color_to_icv(color));
        if request.fill_pattern.unwrap_or(0) == 0 {
            request.fill_pattern = Some(1);
        }
    }
    if let Some(color) = style.fill_background_color {
        request.fill_bg_icv = Some(indexed_color_to_icv(color));
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
        request.font_color_icv = Some(indexed_color_to_icv(color));
    }
}

fn indexed_color_to_icv(color: ExcelColor) -> u16 {
    match color {
        ExcelColor::Indexed(64) => 0x7fff,
        ExcelColor::Indexed(index) => u16::from(index),
        // RGB 在调用本适配器前已由工作簿调色板分配为 Indexed。
        ExcelColor::Rgb(_) => 8,
    }
}

const fn excel_fill_pattern(pattern: ExcelFillPattern) -> u8 {
    match pattern {
        ExcelFillPattern::None => 0,
        ExcelFillPattern::Solid => 1,
        ExcelFillPattern::MediumGray => 2,
        ExcelFillPattern::DarkGray => 3,
        ExcelFillPattern::LightGray => 4,
        ExcelFillPattern::DarkHorizontal => 5,
        ExcelFillPattern::DarkVertical => 6,
        ExcelFillPattern::DarkDown => 7,
        ExcelFillPattern::DarkUp => 8,
        ExcelFillPattern::DarkGrid => 9,
        ExcelFillPattern::DarkTrellis => 10,
        ExcelFillPattern::LightHorizontal => 11,
        ExcelFillPattern::LightVertical => 12,
        ExcelFillPattern::LightDown => 13,
        ExcelFillPattern::LightUp => 14,
        ExcelFillPattern::LightGrid => 15,
        ExcelFillPattern::LightTrellis => 16,
        ExcelFillPattern::Gray125 => 17,
        ExcelFillPattern::Gray0625 => 18,
    }
}

const fn excel_halign(align: ExcelHorizontalAlignment) -> u8 {
    match align {
        ExcelHorizontalAlignment::General => 0,
        ExcelHorizontalAlignment::Left => 1,
        ExcelHorizontalAlignment::Center => 2,
        ExcelHorizontalAlignment::Right => 3,
        ExcelHorizontalAlignment::Fill => 4,
        ExcelHorizontalAlignment::Justify => 5,
        ExcelHorizontalAlignment::CenterAcross => 6,
        ExcelHorizontalAlignment::Distributed => 7,
    }
}

const fn excel_valign(align: ExcelVerticalAlignment) -> u8 {
    match align {
        ExcelVerticalAlignment::Top => 0,
        ExcelVerticalAlignment::Center => 1,
        ExcelVerticalAlignment::Bottom => 2,
        ExcelVerticalAlignment::Justify => 3,
        ExcelVerticalAlignment::Distributed => 4,
    }
}
