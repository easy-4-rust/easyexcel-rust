//! BIFF8 workbook-level FONT / XF / palette registry.
//!
//! Maps Java `EasyExcel` / POI HSSF style knobs (`WriteCellStyle`, `WriteFont`,
//! `IndexedColors`, `CellStyle` builder) onto FONT + XF records. Borders,
//! custom number formats beyond date/datetime, rich-text runs, and conditional
//! formatting remain unsupported for Minimal BIFF8.

use std::collections::HashMap;

use easyexcel_core::{
    ExcelCellStyle, ExcelColor, ExcelFillPattern, ExcelFontStyle, ExcelHorizontalAlignment,
    ExcelVerticalAlignment,
};

use super::encode::{
    ICV_AUTO, ICV_PATTERN_BG_DEFAULT, XF_CUSTOM_BASE, XF_DATE, XF_DATETIME, pack_cell_xf, pack_font,
};

/// Resolved write-style inputs used when allocating an XF index.
// 语义敏感：bold/italic/strikeout/wrap 与 Java `WriteCellStyle`/`WriteFont`
// 布尔字段一一对应，合并会破坏 1:1 可追溯性，故豁免 struct_excessive_bools。
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default)]
pub struct Biff8StyleRequest {
    /// Bold font.
    pub bold: bool,
    /// Italic font.
    pub italic: bool,
    /// Strike-through font.
    pub strikeout: bool,
    /// Font height in points (`None` → 10pt Arial default).
    pub font_height_points: Option<u16>,
    /// Font family name (`None` → `"Arial"`).
    pub font_name: Option<String>,
    /// Font colour as palette ICV (`None` → automatic).
    pub font_color_icv: Option<u16>,
    /// Horizontal alignment POI code (`None` → general / 0).
    pub halign: Option<u8>,
    /// Vertical alignment POI code (`None` → bottom / 2).
    pub valign: Option<u8>,
    /// Wrap text.
    pub wrap: bool,
    /// Fill pattern POI code (`None` / 0 → no fill).
    pub fill_pattern: Option<u8>,
    /// Fill foreground palette ICV.
    pub fill_fg_icv: Option<u16>,
    /// Fill background palette ICV.
    pub fill_bg_icv: Option<u16>,
}

impl Biff8StyleRequest {
    /// Returns `true` when this request would produce `XF_GENERAL` with default font.
    #[must_use]
    pub fn is_default(&self) -> bool {
        !self.bold
            && !self.italic
            && !self.strikeout
            && self.font_height_points.is_none()
            && self.font_name.is_none()
            && self.font_color_icv.is_none()
            && self.halign.is_none()
            && self.valign.is_none()
            && !self.wrap
            && self.fill_pattern.unwrap_or(0) == 0
            && self.fill_fg_icv.is_none()
    }

    /// Merges annotation / strategy [`ExcelCellStyle`] (Java `WriteCellStyle`).
    pub fn apply_excel_cell_style(&mut self, style: ExcelCellStyle) {
        if let Some(align) = style.horizontal_alignment {
            self.halign = Some(excel_halign(align));
        }
        if let Some(align) = style.vertical_alignment {
            self.valign = Some(excel_valign(align));
        }
        if let Some(wrapped) = style.wrapped {
            self.wrap = wrapped;
        }
        if let Some(pattern) = style.fill_pattern {
            self.fill_pattern = Some(excel_fill_pattern(pattern));
        }
        if let Some(color) = style.fill_foreground_color {
            self.fill_fg_icv = Some(rgb_or_indexed_to_icv(color));
            if self.fill_pattern.unwrap_or(0) == 0 {
                self.fill_pattern = Some(1);
            }
        }
        if let Some(color) = style.fill_background_color {
            self.fill_bg_icv = Some(rgb_or_indexed_to_icv(color));
        }
        if let Some(font) = style.font {
            self.apply_excel_font_style(font);
        }
    }

    /// Merges annotation / strategy [`ExcelFontStyle`] (Java `WriteFont`).
    pub fn apply_excel_font_style(&mut self, style: ExcelFontStyle) {
        if let Some(name) = style.font_name {
            self.font_name = Some(name.to_owned());
        }
        if let Some(height) = style.font_height_in_points {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                self.font_height_points = Some(height.round().clamp(1.0, 409.0) as u16);
            }
        }
        if let Some(italic) = style.italic {
            self.italic = italic;
        }
        if let Some(strikeout) = style.strikeout {
            self.strikeout = strikeout;
        }
        if let Some(bold) = style.bold {
            self.bold = bold;
        }
        if let Some(color) = style.color {
            self.font_color_icv = Some(rgb_or_indexed_to_icv(color));
        }
    }
}

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
}

/// Workbook-global FONT / XF allocator shared by all sheets.
///
/// Java mapping: POI `HSSFWorkbook` font/style tables. Built-in XF 0..15 are
/// style XFs; 16/17 are date/datetime helpers; custom cell XFs start at
/// [`XF_CUSTOM_BASE`] (18).
#[derive(Debug, Clone, Default)]
pub struct Biff8StyleTable {
    /// Custom fonts beyond the five default Arial records.
    fonts: Vec<FontKey>,
    font_cache: HashMap<FontKey, u16>,
    /// Custom cell XF payloads (indices `XF_CUSTOM_BASE..`).
    xfs: Vec<[u8; 20]>,
    xf_cache: HashMap<XfKey, u16>,
    /// RGB colours allocated into the customizable palette (indices 8..).
    palette_rgb: Vec<(u8, u8, u8)>,
}

impl Biff8StyleTable {
    /// Resolves an XF index for `request`, preserving `base_xf` number format
    /// (`XF_GENERAL` / `XF_DATE` / `XF_DATETIME`).
    pub fn resolve_xf(&mut self, request: &Biff8StyleRequest, base_xf: u16) -> u16 {
        let ifmt = match base_xf {
            XF_DATE => 14,
            XF_DATETIME => 22,
            _ => 0,
        };
        if request.is_default() {
            return base_xf;
        }
        let font_index = self.ensure_font(request);
        let key = XfKey {
            font_index,
            ifmt,
            halign: request.halign.unwrap_or(0),
            valign: request.valign.unwrap_or(2),
            wrap: request.wrap,
            fill_pattern: request.fill_pattern.unwrap_or(0),
            fill_fg_icv: request.fill_fg_icv.unwrap_or(0x40),
            fill_bg_icv: request.fill_bg_icv.unwrap_or(ICV_PATTERN_BG_DEFAULT),
        };
        if let Some(existing) = self.xf_cache.get(&key) {
            return *existing;
        }
        let packed = pack_cell_xf(
            key.font_index,
            key.ifmt,
            key.halign,
            key.valign,
            key.wrap,
            key.fill_pattern,
            key.fill_fg_icv,
            key.fill_bg_icv,
        );
        // 语义敏感：自定义 XF 数量远小于 u16 上限，保留 as 以对齐 BIFF8 索引。
        #[allow(clippy::cast_possible_truncation)]
        let index = XF_CUSTOM_BASE + self.xfs.len() as u16;
        self.xfs.push(packed);
        self.xf_cache.insert(key, index);
        index
    }

    /// FONT records after the five defaults (emission order).
    #[must_use]
    pub fn custom_fonts(&self) -> Vec<Vec<u8>> {
        self.fonts
            .iter()
            .map(|font| {
                pack_font(
                    font.height_points,
                    font.bold,
                    font.italic,
                    font.strikeout,
                    font.color_icv,
                    &font.name,
                )
            })
            .collect()
    }

    /// Custom cell XF payloads in emission order.
    #[must_use]
    pub fn custom_xfs(&self) -> &[[u8; 20]] {
        &self.xfs
    }

    /// Whether a PALETTE record is required for custom RGB colours.
    #[must_use]
    pub fn needs_palette(&self) -> bool {
        !self.palette_rgb.is_empty()
    }

    /// Custom RGB colours keyed by palette index starting at 8.
    #[must_use]
    pub fn palette_overrides(&self) -> &[(u8, u8, u8)] {
        &self.palette_rgb
    }

    fn ensure_font(&mut self, request: &Biff8StyleRequest) -> u16 {
        let key = FontKey {
            height_points: request.font_height_points.unwrap_or(10),
            bold: request.bold,
            italic: request.italic,
            strikeout: request.strikeout,
            color_icv: request.font_color_icv.unwrap_or(ICV_AUTO),
            name: request
                .font_name
                .clone()
                .unwrap_or_else(|| "Arial".to_owned()),
        };
        // Default Arial 10 / not bold / auto colour → built-in font 0.
        if key.height_points == 10
            && !key.bold
            && !key.italic
            && !key.strikeout
            && key.color_icv == ICV_AUTO
            && key.name == "Arial"
        {
            return 0;
        }
        if let Some(existing) = self.font_cache.get(&key) {
            return *existing;
        }
        // BIFF8 skips font index 4: slots 0..3 → indices 0..3, slot 4 → index 5, …
        let slot = 5 + self.fonts.len(); // 5th default is index 5; first custom → 6
        let index = font_index_for_slot(slot);
        self.fonts.push(key.clone());
        self.font_cache.insert(key, index);
        index
    }

    /// Allocates or reuses a palette ICV for an RGB triple.
    // 语义敏感：BIFF8 调色板最多 56 色（索引 8..=63），usize->u16 不可能截断。
    #[allow(clippy::cast_possible_truncation)]
    pub fn alloc_rgb_icv(&mut self, rgb: u32) -> u16 {
        let r = ((rgb >> 16) & 0xFF) as u8;
        let g = ((rgb >> 8) & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;
        if let Some(pos) = self.palette_rgb.iter().position(|&c| c == (r, g, b)) {
            return (8 + pos) as u16;
        }
        if self.palette_rgb.len() >= 56 {
            // Fall back to nearest built-in when palette is full.
            return nearest_indexed(r, g, b);
        }
        let index = (8 + self.palette_rgb.len()) as u16;
        self.palette_rgb.push((r, g, b));
        index
    }
}

/// Maps FONT record ordinal (0-based among all FONT records) to XF font index.
///
/// Excel / HSSF skip index 4: records `[0,1,2,3,4]` → indices `[0,1,2,3,5]`.
// 语义敏感：slot 来自 FONT 记录表长度（远小于 u16 上限），保留 as 转换。
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub const fn font_index_for_slot(slot: usize) -> u16 {
    if slot < 4 {
        slot as u16
    } else {
        (slot + 1) as u16
    }
}

fn rgb_or_indexed_to_icv(color: ExcelColor) -> u16 {
    match color {
        ExcelColor::Indexed(64) => ICV_AUTO,
        ExcelColor::Indexed(index) => u16::from(index),
        // RGB is approximated to a standard palette entry here; workbook-level
        // custom palette allocation happens via [`Biff8StyleTable::alloc_rgb_icv`]
        // when callers pass through the table. For annotation Indexed colours
        // this path is unused.
        ExcelColor::Rgb(rgb) => nearest_indexed(
            ((rgb >> 16) & 0xFF) as u8,
            ((rgb >> 8) & 0xFF) as u8,
            (rgb & 0xFF) as u8,
        ),
    }
}

/// Converts [`ExcelColor`] using the style table for RGB palette allocation.
#[allow(dead_code)]
pub fn color_to_icv(table: &mut Biff8StyleTable, color: ExcelColor) -> u16 {
    match color {
        ExcelColor::Indexed(64) => ICV_AUTO,
        ExcelColor::Indexed(index) => u16::from(index),
        ExcelColor::Rgb(rgb) => table.alloc_rgb_icv(rgb),
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

fn excel_fill_pattern(pattern: ExcelFillPattern) -> u8 {
    // POI `FillPatternType` ordinals.
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

fn excel_halign(align: ExcelHorizontalAlignment) -> u8 {
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

fn excel_valign(align: ExcelVerticalAlignment) -> u8 {
    match align {
        ExcelVerticalAlignment::Top => 0,
        ExcelVerticalAlignment::Center => 1,
        ExcelVerticalAlignment::Bottom => 2,
        ExcelVerticalAlignment::Justify => 3,
        ExcelVerticalAlignment::Distributed => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biff8::encode::XF_GENERAL;

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
    fn font_index_skips_four() {
        assert_eq!(font_index_for_slot(0), 0);
        assert_eq!(font_index_for_slot(4), 5);
        assert_eq!(font_index_for_slot(5), 6);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn apply_excel_cell_style_maps_every_field() {
        let mut req = Biff8StyleRequest::default();
        let style = ExcelCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Distributed),
            vertical_alignment: Some(ExcelVerticalAlignment::Top),
            wrapped: Some(true),
            fill_pattern: Some(ExcelFillPattern::DarkTrellis),
            fill_foreground_color: Some(ExcelColor::Indexed(13)),
            fill_background_color: Some(ExcelColor::Indexed(64)),
            font: Some(ExcelFontStyle {
                font_name: Some("Arial"),
                font_height_in_points: Some(18.0),
                italic: Some(true),
                strikeout: Some(true),
                color: Some(ExcelColor::Indexed(10)),
                bold: Some(true),
                ..ExcelFontStyle::default()
            }),
            ..ExcelCellStyle::default()
        };
        req.apply_excel_cell_style(style);
        assert_eq!(req.halign, Some(7));
        assert_eq!(req.valign, Some(0));
        assert!(req.wrap);
        assert_eq!(req.fill_pattern, Some(10));
        assert_eq!(req.fill_fg_icv, Some(13));
        assert_eq!(req.fill_bg_icv, Some(ICV_AUTO));
        assert_eq!(req.font_name, Some("Arial".to_owned()));
        assert_eq!(req.font_height_points, Some(18));
        assert!(req.italic);
        assert!(req.strikeout);
        assert!(req.bold);
        assert_eq!(req.font_color_icv, Some(10));
    }

    #[test]
    fn fill_foreground_implies_solid_pattern_and_approximates_rgb() {
        let mut req = Biff8StyleRequest::default();
        req.apply_excel_cell_style(ExcelCellStyle {
            fill_foreground_color: Some(ExcelColor::Rgb(0xFF_00_00)),
            ..ExcelCellStyle::default()
        });
        assert_eq!(req.fill_pattern, Some(1));
        assert_eq!(req.fill_fg_icv, Some(10));
    }

    #[test]
    fn apply_excel_cell_style_without_fill_colors() {
        let mut req = Biff8StyleRequest::default();
        req.apply_excel_cell_style(ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            ..ExcelCellStyle::default()
        });
        assert_eq!(req.fill_pattern, Some(1));
        assert!(req.fill_fg_icv.is_none());
        assert!(req.fill_bg_icv.is_none());
    }

    #[test]
    fn empty_font_style_is_a_noop() {
        let mut req = Biff8StyleRequest::default();
        req.apply_excel_font_style(ExcelFontStyle::default());
        assert!(req.font_name.is_none());
        assert!(req.font_height_points.is_none());
        assert!(!req.italic);
        assert!(!req.strikeout);
    }

    #[test]
    fn font_height_is_rounded_and_clamped() {
        let mut req = Biff8StyleRequest::default();
        req.apply_excel_font_style(ExcelFontStyle {
            font_height_in_points: Some(0.4),
            ..ExcelFontStyle::default()
        });
        assert_eq!(req.font_height_points, Some(1));
        req.apply_excel_font_style(ExcelFontStyle {
            font_height_in_points: Some(500.0),
            ..ExcelFontStyle::default()
        });
        assert_eq!(req.font_height_points, Some(409));
        req.apply_excel_font_style(ExcelFontStyle {
            font_height_in_points: Some(12.6),
            ..ExcelFontStyle::default()
        });
        assert_eq!(req.font_height_points, Some(13));
    }

    #[test]
    fn resolve_xf_preserves_date_and_datetime_number_formats() {
        let mut table = Biff8StyleTable::default();
        let req = Biff8StyleRequest {
            bold: true,
            ..Biff8StyleRequest::default()
        };
        let date_xf = table.resolve_xf(&req, XF_DATE);
        let datetime_xf = table.resolve_xf(&req, XF_DATETIME);
        assert_eq!(date_xf, XF_CUSTOM_BASE);
        assert_eq!(datetime_xf, XF_CUSTOM_BASE + 1);
        let xfs = table.custom_xfs();
        assert_eq!(u16::from_le_bytes([xfs[0][2], xfs[0][3]]), 14);
        assert_eq!(u16::from_le_bytes([xfs[1][2], xfs[1][3]]), 22);
        // Repeated requests are cached.
        assert_eq!(table.resolve_xf(&req, XF_DATE), date_xf);
        assert_eq!(table.custom_xfs().len(), 2);
    }

    #[test]
    fn alloc_rgb_icv_reuses_then_allocates_then_falls_back() {
        let mut table = Biff8StyleTable::default();
        assert_eq!(table.alloc_rgb_icv(0x10_20_30), 8);
        assert_eq!(table.alloc_rgb_icv(0x10_20_30), 8);
        assert_eq!(table.palette_rgb.len(), 1);
        for i in 1..56u32 {
            table.alloc_rgb_icv((i & 0xFF) << 16);
        }
        assert_eq!(table.palette_rgb.len(), 56);
        assert!(table.needs_palette());
        assert_eq!(table.palette_overrides().len(), 56);
        // Palette full → nearest built-in fallback.
        assert_eq!(table.alloc_rgb_icv(0x01_02_03), 8);
        assert_eq!(table.alloc_rgb_icv(0xFF_FF_FF), 9);
    }

    #[test]
    fn color_to_icv_handles_indexed_and_rgb() {
        let mut table = Biff8StyleTable::default();
        assert_eq!(color_to_icv(&mut table, ExcelColor::Indexed(64)), ICV_AUTO);
        assert_eq!(color_to_icv(&mut table, ExcelColor::Indexed(10)), 10);
        assert_eq!(color_to_icv(&mut table, ExcelColor::Rgb(0x00_FF_00)), 8);
        assert_eq!(table.palette_rgb.len(), 1);
        assert_eq!(table.palette_rgb[0], (0, 255, 0));
    }

    #[test]
    fn pattern_and_alignment_codes_match_poi() {
        for (pattern, code) in [
            (ExcelFillPattern::None, 0u8),
            (ExcelFillPattern::Solid, 1),
            (ExcelFillPattern::MediumGray, 2),
            (ExcelFillPattern::DarkGray, 3),
            (ExcelFillPattern::LightGray, 4),
            (ExcelFillPattern::DarkHorizontal, 5),
            (ExcelFillPattern::DarkVertical, 6),
            (ExcelFillPattern::DarkDown, 7),
            (ExcelFillPattern::DarkUp, 8),
            (ExcelFillPattern::DarkGrid, 9),
            (ExcelFillPattern::DarkTrellis, 10),
            (ExcelFillPattern::LightHorizontal, 11),
            (ExcelFillPattern::LightVertical, 12),
            (ExcelFillPattern::LightDown, 13),
            (ExcelFillPattern::LightUp, 14),
            (ExcelFillPattern::LightGrid, 15),
            (ExcelFillPattern::LightTrellis, 16),
            (ExcelFillPattern::Gray125, 17),
            (ExcelFillPattern::Gray0625, 18),
        ] {
            assert_eq!(excel_fill_pattern(pattern), code);
        }
        for (align, code) in [
            (ExcelHorizontalAlignment::General, 0u8),
            (ExcelHorizontalAlignment::Left, 1),
            (ExcelHorizontalAlignment::Center, 2),
            (ExcelHorizontalAlignment::Right, 3),
            (ExcelHorizontalAlignment::Fill, 4),
            (ExcelHorizontalAlignment::Justify, 5),
            (ExcelHorizontalAlignment::CenterAcross, 6),
            (ExcelHorizontalAlignment::Distributed, 7),
        ] {
            assert_eq!(excel_halign(align), code);
        }
        for (align, code) in [
            (ExcelVerticalAlignment::Top, 0u8),
            (ExcelVerticalAlignment::Center, 1),
            (ExcelVerticalAlignment::Bottom, 2),
            (ExcelVerticalAlignment::Justify, 3),
            (ExcelVerticalAlignment::Distributed, 4),
        ] {
            assert_eq!(excel_valign(align), code);
        }
    }
}
