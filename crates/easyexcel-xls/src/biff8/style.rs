//! BIFF8 workbook-level FONT / XF / palette registry.
//!
//! Maps Java `EasyExcel` / POI HSSF style knobs (`WriteCellStyle`, `WriteFont`,
//! `IndexedColors`, `CellStyle` builder) onto FONT + XF records. Borders,
//! custom number formats beyond date/datetime, rich-text runs, and conditional
//! formatting remain unsupported for Minimal BIFF8.

use std::collections::HashMap;

use super::encode::{
    ICV_AUTO, ICV_PATTERN_BG_DEFAULT, XF_CUSTOM_BASE, XF_DATE, XF_DATETIME, pack_cell_xf, pack_font,
};
use super::format::builtin_format_id;

include!("style/biff8color.rs");

include!("style/biff8horizontal_alignment.rs");

include!("style/biff8vertical_alignment.rs");

include!("style/biff8fill_pattern.rs");

include!("style/biff8_border_style.rs");

include!("style/biff8style_request.rs");

include!("style/biff8number_format.rs");

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct FontKey {
    height_points: u16,
    bold: bool,
    italic: bool,
    strikeout: bool,
    color_icv: u16,
    name: String,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct XfKey {
    font_index: u16,
    ifmt: u16,
    halign: u8,
    valign: u8,
    wrap: bool,
    fill_pattern: u8,
    fill_fg_icv: u16,
    fill_bg_icv: u16,
    border_left: u8,
    border_right: u8,
    border_top: u8,
    border_bottom: u8,
    border_left_icv: u16,
    border_right_icv: u16,
    border_top_icv: u16,
    border_bottom_icv: u16,
}

/// Workbook-global FONT / XF allocator shared by all sheets.
///
/// Java mapping: POI `HSSFWorkbook` font/style tables. Built-in XF 0..15 are
/// style XFs; 16/17 are date/datetime helpers; custom cell XFs start at
/// [`XF_CUSTOM_BASE`] (18).
/// 自定义数字格式起始索引（BIFF8：ifmt ≥ 164 为自定义格式）。
const FORMAT_CUSTOM_BASE: u16 = 164;

include!("style/biff8style_table.rs");

/// Maps FONT record ordinal (0-based among all FONT records) to XF font index.
///
/// Excel / HSSF skip index 4: records `[0,1,2,3,4]` → indices `[0,1,2,3,5]`.
// 语义敏感：slot 来自 FONT 记录表长度（远小于 u16 上限），保留 as 转换。
#[allow(clippy::cast_possible_truncation)]
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const fn font_index_for_slot(slot: usize) -> u16 {
    if slot < 4 {
        slot as u16
    } else {
        (slot + 1) as u16
    }
}

// 语义敏感：距离平方和 dr²+dg²+db² 恒非负且远小于 i32::MAX，
// i32->u32 转换不可能出现符号丢失。
#[allow(clippy::cast_sign_loss)]
fn nearest_indexed(r: u8, g: u8, b: u8) -> u16 {
    // Minimal subset of POI IndexedColors used by Style / Annotation tests.
    const TABLE: &[(u8, u8, u8, u16)] = &[
        (0, 0, 0, 8),
        (255, 255, 255, 9),
        (255, 0, 0, 10),
        (0, 255, 0, 11),
        (0, 0, 255, 12),
        (255, 255, 0, 13),
        (255, 0, 255, 14),
        (0, 255, 255, 15),
        (128, 0, 0, 16),
        (0, 128, 0, 17),
        (0, 0, 128, 18),
        (128, 128, 0, 19),
        (128, 0, 128, 20),
        (0, 128, 128, 21),
        (192, 192, 192, 22),
        (128, 128, 128, 23),
    ];
    let mut best = 8u16;
    let mut best_dist = u32::MAX;
    for &(tr, tg, tb, idx) in TABLE {
        let dr = i32::from(r) - i32::from(tr);
        let dg = i32::from(g) - i32::from(tg);
        let db = i32::from(b) - i32::from(tb);
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best = idx;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biff8::encode::XF_GENERAL;

    #[test]
    fn custom_number_format_registers_from_164_and_reuses() {
        // 对应 Java：POI createDataFormat().getFormat("0.00") → ifmt ≥ 164
        let mut table = Biff8StyleTable::default();
        let request = Biff8StyleRequest {
            number_format: Some(Biff8NumberFormat::Custom("0.000".to_owned())),
            ..Biff8StyleRequest::default()
        };
        let xf1 = table.resolve_xf(&request, XF_GENERAL);
        let xf2 = table.resolve_xf(&request, XF_GENERAL);
        assert_eq!(xf1, xf2, "同码复用同一 XF");
        assert_eq!(table.custom_formats(), &[(164, "0.000".to_owned())]);
        assert_eq!(table.custom_xfs()[0][2..4], [164u8, 0u8]); // ifmt 字段
        // 不同格式码注册新 ifmt
        let other = Biff8StyleRequest {
            number_format: Some(Biff8NumberFormat::Custom("0.0000".to_owned())),
            ..Biff8StyleRequest::default()
        };
        let xf3 = table.resolve_xf(&other, XF_GENERAL);
        assert_ne!(xf1, xf3);
        assert_eq!(table.custom_formats().len(), 2);
        assert_eq!(table.custom_formats()[1].0, 165);
    }

    #[test]
    fn builtin_number_format_maps_to_builtin_ifmt() {
        let mut table = Biff8StyleTable::default();
        let request = Biff8StyleRequest {
            number_format: Some(Biff8NumberFormat::Builtin(2)), // "0.00"
            ..Biff8StyleRequest::default()
        };
        let xf = table.resolve_xf(&request, XF_GENERAL);
        assert_eq!(table.custom_xfs()[0][2..4], [2u8, 0u8]);
        assert!(
            table.custom_formats().is_empty(),
            "内置格式不注册 FORMAT 记录"
        );
        // 格式码精确匹配内置表（如 "#,##0.00" → 4）
        let request2 = Biff8StyleRequest {
            number_format: Some(Biff8NumberFormat::Custom("#,##0.00".to_owned())),
            ..Biff8StyleRequest::default()
        };
        let xf2 = table.resolve_xf(&request2, XF_GENERAL);
        let _ = (xf, xf2);
        assert_eq!(table.custom_formats().len(), 0, "内置码不注册自定义");
    }

    #[test]
    fn resolve_default_keeps_general_xf() {
        let mut table = Biff8StyleTable::default();
        assert_eq!(
            table.resolve_xf(&Biff8StyleRequest::default(), XF_GENERAL),
            XF_GENERAL
        );
    }

    #[test]
    fn resolve_bold_allocates_custom_xf() {
        let mut table = Biff8StyleTable::default();
        let req = Biff8StyleRequest {
            bold: true,
            ..Biff8StyleRequest::default()
        };
        let xf = table.resolve_xf(&req, XF_GENERAL);
        assert!(xf >= XF_CUSTOM_BASE);
        assert_eq!(table.custom_xfs().len(), 1);
        assert_eq!(table.custom_fonts().len(), 1);
    }

    #[test]
    fn semantic_style_values_are_encoded_only_inside_the_biff8_engine() {
        assert_eq!(Biff8HorizontalAlignment::Distributed.code(), 7);
        assert_eq!(Biff8VerticalAlignment::Distributed.code(), 4);
        assert_eq!(Biff8FillPattern::Gray0625.code(), 18);

        let mut table = Biff8StyleTable::default();
        let request = Biff8StyleRequest {
            font_color: Some(Biff8Color::Rgb(0x11_22_33)),
            horizontal_alignment: Some(Biff8HorizontalAlignment::Center),
            vertical_alignment: Some(Biff8VerticalAlignment::Top),
            fill_pattern: Some(Biff8FillPattern::Solid),
            fill_foreground_color: Some(Biff8Color::Rgb(0x44_55_66)),
            ..Biff8StyleRequest::default()
        };
        let xf = table.resolve_xf(&request, XF_GENERAL);
        assert!(xf >= XF_CUSTOM_BASE);
        assert_eq!(table.palette_overrides().len(), 2);
        assert_eq!(table.custom_xfs().len(), 1);
    }

    #[test]
    fn font_index_skips_four() {
        assert_eq!(font_index_for_slot(0), 0);
        assert_eq!(font_index_for_slot(4), 5);
        assert_eq!(font_index_for_slot(5), 6);
    }

    #[test]
    fn border_styles_and_colours_are_packed_into_xf() {
        let mut table = Biff8StyleTable::default();
        let request = Biff8StyleRequest {
            border_left: Some(Biff8BorderStyle::Thin),
            border_right: Some(Biff8BorderStyle::Medium),
            border_top: Some(Biff8BorderStyle::Dashed),
            border_bottom: Some(Biff8BorderStyle::Double),
            border_left_color: Some(Biff8Color::Indexed(10)),
            border_right_color: Some(Biff8Color::Indexed(11)),
            border_top_color: Some(Biff8Color::Indexed(12)),
            border_bottom_color: Some(Biff8Color::Indexed(13)),
            ..Biff8StyleRequest::default()
        };
        let _ = table.resolve_xf(&request, XF_GENERAL);
        let xf = table.custom_xfs()[0];
        let brd1 = u32::from_le_bytes(xf[10..14].try_into().expect("brd1"));
        let brd2 = u32::from_le_bytes(xf[14..18].try_into().expect("brd2"));
        assert_eq!(brd1 & 0x0F, 1);
        assert_eq!((brd1 >> 4) & 0x0F, 2);
        assert_eq!((brd1 >> 8) & 0x0F, 3);
        assert_eq!((brd1 >> 12) & 0x0F, 6);
        assert_eq!((brd1 >> 16) & 0x7F, 10);
        assert_eq!((brd1 >> 23) & 0x7F, 11);
        assert_eq!(brd2 & 0x7F, 12);
        assert_eq!((brd2 >> 7) & 0x7F, 13);
    }
}
