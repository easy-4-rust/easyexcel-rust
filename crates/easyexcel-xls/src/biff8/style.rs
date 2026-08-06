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

/// 由引擎解析为 BIFF8 调色板 ICV 的中立颜色请求。
/// 对应 Java：`org.apache.poi.hssf.util.HSSFColor`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8Color {
    /// 自动颜色（`ICV_AUTO`）。
    Automatic,
    /// 已有索引调色板颜色。
    Indexed(u8),
    /// 在工作簿自定义调色板中分配的 RGB 颜色。
    Rgb(u32),
}

/// BIFF8 语义水平对齐；数值协议码仅保留在本 crate 内部。
/// 对应 Java：`org.apache.poi.ss.usermodel.HorizontalAlignment`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8HorizontalAlignment {
    /// 按单元格值类型决定的常规对齐。
    General,
    /// 左对齐。
    Left,
    /// 居中对齐。
    Center,
    /// 右对齐。
    Right,
    /// 横向重复内容以填满单元格。
    Fill,
    /// 两端对齐。
    Justify,
    /// 跨相邻单元格居中。
    CenterAcross,
    /// 分散对齐。
    Distributed,
}

impl Biff8HorizontalAlignment {
    const fn code(self) -> u8 {
        match self {
            Self::General => 0,
            Self::Left => 1,
            Self::Center => 2,
            Self::Right => 3,
            Self::Fill => 4,
            Self::Justify => 5,
            Self::CenterAcross => 6,
            Self::Distributed => 7,
        }
    }
}

/// BIFF8 语义垂直对齐；数值协议码仅保留在本 crate 内部。
/// 对应 Java：`org.apache.poi.ss.usermodel.VerticalAlignment`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8VerticalAlignment {
    /// 顶部对齐。
    Top,
    /// 垂直居中。
    Center,
    /// 底部对齐。
    Bottom,
    /// 垂直两端对齐。
    Justify,
    /// 垂直分散对齐。
    Distributed,
}

impl Biff8VerticalAlignment {
    const fn code(self) -> u8 {
        match self {
            Self::Top => 0,
            Self::Center => 1,
            Self::Bottom => 2,
            Self::Justify => 3,
            Self::Distributed => 4,
        }
    }
}

/// BIFF8 语义填充图案。
/// 对应 Java：`org.apache.poi.ss.usermodel.FillPatternType`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8FillPattern {
    /// 无填充。
    None,
    /// 纯色填充。
    Solid,
    /// 中灰填充。
    MediumGray,
    /// 深灰填充。
    DarkGray,
    /// 浅灰填充。
    LightGray,
    /// 深色水平线。
    DarkHorizontal,
    /// 深色垂直线。
    DarkVertical,
    /// 深色向下斜线。
    DarkDown,
    /// 深色向上斜线。
    DarkUp,
    /// 深色网格。
    DarkGrid,
    /// 深色格架。
    DarkTrellis,
    /// 浅色水平线。
    LightHorizontal,
    /// 浅色垂直线。
    LightVertical,
    /// 浅色向下斜线。
    LightDown,
    /// 浅色向上斜线。
    LightUp,
    /// 浅色网格。
    LightGrid,
    /// 浅色格架。
    LightTrellis,
    /// 12.5% 灰色填充。
    Gray125,
    /// 6.25% 灰色填充。
    Gray0625,
}

impl Biff8FillPattern {
    const fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Solid => 1,
            Self::MediumGray => 2,
            Self::DarkGray => 3,
            Self::LightGray => 4,
            Self::DarkHorizontal => 5,
            Self::DarkVertical => 6,
            Self::DarkDown => 7,
            Self::DarkUp => 8,
            Self::DarkGrid => 9,
            Self::DarkTrellis => 10,
            Self::LightHorizontal => 11,
            Self::LightVertical => 12,
            Self::LightDown => 13,
            Self::LightUp => 14,
            Self::LightGrid => 15,
            Self::LightTrellis => 16,
            Self::Gray125 => 17,
            Self::Gray0625 => 18,
        }
    }
}

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
    /// Semantic font colour (`None` → automatic).
    pub font_color: Option<Biff8Color>,
    /// Horizontal alignment (`None` → general).
    pub horizontal_alignment: Option<Biff8HorizontalAlignment>,
    /// Vertical alignment (`None` → bottom).
    pub vertical_alignment: Option<Biff8VerticalAlignment>,
    /// Wrap text.
    pub wrap: bool,
    /// Semantic fill pattern (`None` / [`Biff8FillPattern::None`] → no fill).
    pub fill_pattern: Option<Biff8FillPattern>,
    /// Fill foreground colour.
    pub fill_foreground_color: Option<Biff8Color>,
    /// Fill background colour.
    pub fill_background_color: Option<Biff8Color>,
    /// Number format: built-in index or custom code.
    pub number_format: Option<Biff8NumberFormat>,
}

/// BIFF8 数字格式描述，隔离 EasyExcel 注解模型与底层 XLS 引擎。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biff8NumberFormat {
    /// Excel 内建格式索引。
    Builtin(u8),
    /// 自定义格式代码。
    Custom(String),
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
            && self.font_color.is_none()
            && self.horizontal_alignment.is_none()
            && self.vertical_alignment.is_none()
            && !self.wrap
            && self
                .fill_pattern
                .is_none_or(|pattern| pattern == Biff8FillPattern::None)
            && self.fill_foreground_color.is_none()
            && self.fill_background_color.is_none()
            && self.number_format.is_none()
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
/// 自定义数字格式起始索引（BIFF8：ifmt ≥ 164 为自定义格式）。
const FORMAT_CUSTOM_BASE: u16 = 164;

/// Workbook-global FONT / XF / number-format allocator。
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
    /// Registered custom number formats `(ifmt, code)` in emission order.
    formats: Vec<(u16, String)>,
    /// Custom format code → ifmt lookup.
    format_lookup: HashMap<String, u16>,
}

impl Biff8StyleTable {
    /// Resolves an XF index for `request`, preserving `base_xf` number format
    /// (`XF_GENERAL` / `XF_DATE` / `XF_DATETIME`).
    pub fn resolve_xf(&mut self, request: &Biff8StyleRequest, base_xf: u16) -> u16 {
        let base_ifmt = match base_xf {
            XF_DATE => 14,
            XF_DATETIME => 22,
            _ => 0,
        };
        let ifmt = self.resolve_ifmt(request.number_format.as_ref(), base_ifmt);
        if request.is_default() {
            return base_xf;
        }
        let font_index = self.ensure_font(request);
        let fill_fg_icv = self.resolve_color(request.fill_foreground_color, 0x40);
        let fill_bg_icv = self.resolve_color(request.fill_background_color, ICV_PATTERN_BG_DEFAULT);
        let key = XfKey {
            font_index,
            ifmt,
            halign: request
                .horizontal_alignment
                .map_or(0, Biff8HorizontalAlignment::code),
            valign: request
                .vertical_alignment
                .map_or(2, Biff8VerticalAlignment::code),
            wrap: request.wrap,
            fill_pattern: request.fill_pattern.map_or(0, Biff8FillPattern::code),
            fill_fg_icv,
            fill_bg_icv,
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

    /// 解析数字格式为 BIFF8 ifmt：显式格式优先，其次 base（日期/时间），
    /// 最后 General(0)。自定义格式码从 164 起注册（同码复用）。
    fn resolve_ifmt(&mut self, format: Option<&Biff8NumberFormat>, base_ifmt: u16) -> u16 {
        let Some(format) = format else {
            return base_ifmt;
        };
        match format {
            Biff8NumberFormat::Builtin(index) => u16::from(*index),
            Biff8NumberFormat::Custom(code) => {
                if let Some(builtin) = builtin_format_id(&code) {
                    return builtin;
                }
                if let Some(existing) = self.format_lookup.get(code.as_str()) {
                    return *existing;
                }
                // 语义敏感：自定义格式数量远小于 u16 上限，保留 as 以对齐 BIFF8 索引
                #[allow(clippy::cast_possible_truncation)]
                let ifmt = FORMAT_CUSTOM_BASE + self.formats.len() as u16;
                self.formats.push((ifmt, code.clone()));
                self.format_lookup.insert(code.clone(), ifmt);
                ifmt
            }
        }
    }

    /// Registered custom FORMAT records in emission order.
    #[must_use]
    pub fn custom_formats(&self) -> &[(u16, String)] {
        &self.formats
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
            color_icv: self.resolve_color(request.font_color, ICV_AUTO),
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

    fn resolve_color(&mut self, color: Option<Biff8Color>, default: u16) -> u16 {
        match color {
            None => default,
            Some(Biff8Color::Automatic) => ICV_AUTO,
            Some(Biff8Color::Indexed(index)) => u16::from(index),
            Some(Biff8Color::Rgb(rgb)) => self.alloc_rgb_icv(rgb),
        }
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
}
