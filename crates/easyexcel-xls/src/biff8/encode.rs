//! Low-level BIFF8 record framing and `XLUnicodeString` helpers.
//!
//! Record layout matches [MS-XLS] / `OpenOffice` BIFF8: 2-byte type, 2-byte length,
//! then payload (≤ 8224 bytes). Unicode strings use the short / long `XLUnicode`
//! forms that calamine and Excel expect.
//!
//! XF / COLINFO / ROW / MERGECELLS layouts follow the same field packing as
//! xlwt / Apache POI HSSF (see module docs on each writer helper).

#![allow(dead_code)]
#![allow(
    missing_docs,
    reason = "BIFF record identifiers preserve the canonical MS-XLS names used by the encoder"
)]

/// Maximum BIFF record data payload (excluding the 4-byte header).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MAX_RECORD_DATA: usize = 8224;

pub use super::record_sid::{
    BLANK_SID as BLANK, BOF_SID as BOF, BOOL_ERR_SID as BOOLERR, BOUND_SHEET_SID as BOUNDSHEET,
    CALC_MODE_SID as CALCMODE, CODE_NAME_SID as CODENAME, CODE_PAGE_SID as CODEPAGE,
    COLUMN_INFO_SID as COLINFO, CONTINUE_SID as CONTINUE, DATE_MODE_SID as DATEMODE,
    DIMENSION_SID as DIMENSION, EOF_SID as EOF, EXT_SST_SID as EXTSST,
    EXTERNAL_SHEET_SID as EXTERNSHEET, FILE_PASS_SID as FILEPASS, FONT_SID as FONT,
    FORMAT_SID as FORMAT, FORMULA_SID as FORMULA, HYPERLINK_SID as HYPERLINK,
    INTERFACE_END_SID as INTERFACEEND, INTERFACE_HEADER_SID as INTERFACEHDR, LABEL_SID as LABEL,
    LABEL_SST_SID as LABELSST, MERGE_CELLS_SID as MERGECELLS, MMS_SID as MMS,
    MSO_DRAWING_GROUP_SID as MSODRAWINGGROUP, MSO_DRAWING_SID as MSODRAWING,
    MUL_BLANK_SID as MULBLANK, MUL_RK_SID as MULRK, NOTE_SID as NOTE, NUMBER_SID as NUMBER,
    OBJECT_PROTECT_SID as OBJECTPROTECT, OBJ_SID as OBJ, PALETTE_SID as PALETTE,
    PANE_SID as PANE, PASSWORD_SID as PASSWORD, PROTECT_SID as PROTECT, RK_SID as RK,
    ROW_SID as ROW, SCENARIO_PROTECT_SID as SCENPROTECT, SST_SID as SST,
    STRING_SID as STRING, STYLE_SID as STYLE, SUP_BOOK_SID as SUPBOOK,
    TEXT_OBJECT_SID as TXO, WINDOW2_SID as WINDOW2, WRITE_ACCESS_SID as WRITEACCESS, XF_SID as XF,
};

/// BIFF8 DBCELL 记录。
pub const DBCELL: u16 = super::record_sid::DB_CELL_SID;

/// Workbook globals substream type.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const DT_GLOBALS: u16 = 0x0005;
/// Worksheet substream type.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const DT_WORKSHEET: u16 = 0x0010;
/// BIFF8 version word.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const BIFF8_VERSION: u16 = 0x0600;

/// Built-in XF index used for unstyled cells (last of the 16 style XFs).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const XF_GENERAL: u16 = 15;
/// First cell XF after the 16 built-in style XFs — date (`m/d/yy`, id 14).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const XF_DATE: u16 = 16;
/// Second cell XF — datetime (`m/d/yy h:mm`, id 22).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const XF_DATETIME: u16 = 17;
/// First custom cell XF index (after date / datetime helpers).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const XF_CUSTOM_BASE: u16 = 18;

/// Automatic / default font colour ICV.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const ICV_AUTO: u16 = 0x7FFF;
/// Default pattern background (automatic).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const ICV_PATTERN_BG_DEFAULT: u16 = 64;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends a framed BIFF record (`type` + `len` + `data`) to `out`.
// 语义敏感：上方 debug_assert 已保证 data.len() <= MAX_RECORD_DATA（远小于
// u16 上限），记录长度字段按 BIFF8 规范为 u16，保留 as 转换。
#[allow(clippy::cast_possible_truncation)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn record(out: &mut Vec<u8>, typ: u16, data: &[u8]) {
    debug_assert!(data.len() <= MAX_RECORD_DATA);
    out.extend_from_slice(&typ.to_le_bytes());
    out.extend_from_slice(&(data.len() as u16).to_le_bytes());
    out.extend_from_slice(data);
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Encodes a long `XLUnicodeString` (`cch:u16` + `grbit` + chars).
// 语义敏感：Excel 单元格文本上限 32767 字符（远小于 u16 上限）；压缩模式下
// 每字符必 <= 0xFF，u16->u8 无损。保留 as 以对齐 BIFF8 规范。
#[allow(clippy::cast_possible_truncation)]
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn encode_unicode_string(s: &str) -> Vec<u8> {
    let chars: Vec<u16> = s.encode_utf16().collect();
    let compressed = chars.iter().all(|&c| c <= 0xFF);
    let mut out = Vec::with_capacity(3 + chars.len() * if compressed { 1 } else { 2 });
    out.extend_from_slice(&(chars.len() as u16).to_le_bytes());
    if compressed {
        out.push(0x00);
        for &c in &chars {
            out.push(c as u8);
        }
    } else {
        out.push(0x01);
        for &c in &chars {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    out
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Encodes a short `XLUnicodeString` (`cch:u8` + `grbit` + chars) for BOUNDSHEET / FONT.
// 语义敏感：上方 take(255) 已保证字符数 <= 255，且压缩模式下每字符必 <= 0xFF。
#[allow(clippy::cast_possible_truncation)]
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn encode_short_unicode_string(s: &str) -> Vec<u8> {
    let chars: Vec<u16> = s.encode_utf16().take(255).collect();
    let compressed = chars.iter().all(|&c| c <= 0xFF);
    let mut out = Vec::with_capacity(2 + chars.len() * if compressed { 1 } else { 2 });
    out.push(chars.len() as u8);
    if compressed {
        out.push(0x00);
        for &c in &chars {
            out.push(c as u8);
        }
    } else {
        out.push(0x01);
        for &c in &chars {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    out
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Tries to pack `v` into an RK record value; `None` means emit a NUMBER record.
#[must_use]
pub fn encode_rk(v: f64) -> Option<u32> {
    if !v.is_finite() {
        return None;
    }
    // Integer form (bit0=0, bit1=1)：值占低 30 位。
    // 语义敏感：数值已限制在 [-0x1FFF_FFFF, 0x1FFF_FFFF]，i32->u32 符号转换无损。
    if v.fract() == 0.0 && v >= f64::from(-0x1FFF_FFFF) && v <= f64::from(0x1FFF_FFFF) {
        // 语义敏感：数值已限制在 [-0x1FFF_FFFF, 0x1FFF_FFFF]，两个转换均无损。
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            let n = v as i32;
            return Some(((n as u32) << 2) | 0x02);
        }
    }
    // Integer / 100 form (bit0=1, bit1=1)。
    let scaled = v * 100.0;
    if scaled.fract() == 0.0
        && scaled >= f64::from(-0x1FFF_FFFF)
        && scaled <= f64::from(0x1FFF_FFFF)
    {
        // 语义敏感：同上，转换无损。
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            let n = scaled as i32;
            return Some(((n as u32) << 2) | 0x03);
        }
    }
    // Truncated IEEE754 high 30 bits (bit0=0, bit1=0)：要求低 34 位全零，
    // 即 trailing_zeros >= 34，截断后无损。
    let bits = v.to_bits();
    let high = (bits >> 32) as u32;
    if bits.trailing_zeros() >= 34 {
        return Some(high);
    }
    None
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Packs a BIFF8 cell XF (20 bytes) with optional solid fill / alignment.
///
/// Field packing matches xlwt `XFRecord` / `OpenOffice` BIFF8 XF (Java HSSF
/// `ExtendedFormatRecord`).
///
/// `halign` / `valign` use POI codes (`HorizontalAlignment` /
/// `VerticalAlignment` ordinals). `fill_pattern` uses POI `FillPatternType`
/// codes (`SolidForeground = 1`).
// 语义敏感：fill_fg_icv/fill_bg_icv 名称与 Java HSSF `ExtendedFormatRecord` 的
// fg/bg ICV 字段一一对应，保持原名便于对照；8 个参数均直接映射该 Java
// 记录的字段，拆分结构体会破坏 1:1 可追溯性。
#[allow(clippy::similar_names, clippy::too_many_arguments)]
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn pack_cell_xf(
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
) -> [u8; 20] {
    let mut d = [0u8; 20];
    d[0..2].copy_from_slice(&font_index.to_le_bytes());
    d[2..4].copy_from_slice(&ifmt.to_le_bytes());
    // fLocked cell XF (not style).
    d[4..6].copy_from_slice(&0x0001u16.to_le_bytes());
    let mut align = halign & 0x07;
    if wrap {
        align |= 0x08;
    }
    // Vertical alignment in bits 4-6 (default bottom = 2 when unset).
    align |= (valign & 0x07) << 4;
    d[6] = align;
    d[7] = 0; // rotation
    d[8] = 0; // indent / shrink
    d[9] = 0xF8; // XF_USED_ATTRIB — all groups used (cell XF)
    let brd1 = u32::from(border_left & 0x0F)
        | (u32::from(border_right & 0x0F) << 4)
        | (u32::from(border_top & 0x0F) << 8)
        | (u32::from(border_bottom & 0x0F) << 12)
        | (u32::from(border_left_icv & 0x7F) << 16)
        | (u32::from(border_right_icv & 0x7F) << 23);
    d[10..14].copy_from_slice(&brd1.to_le_bytes());
    // brd2 (bytes 14-17): fill pattern in bits 26-31.
    let brd2 = u32::from(border_top_icv & 0x7F)
        | (u32::from(border_bottom_icv & 0x7F) << 7)
        | (u32::from(fill_pattern & 0x3F) << 26);
    d[14..18].copy_from_slice(&brd2.to_le_bytes());
    // pattern colours (bytes 18-19).
    let pat = (fill_fg_icv & 0x7F) | ((fill_bg_icv & 0x7F) << 7);
    d[18..20].copy_from_slice(&pat.to_le_bytes());
    d
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Packs a BIFF8 FONT record payload (Java HSSF `FontRecord`).
///
/// `height_points` is converted to twips (`* 20`). Bold uses `bls=700`.
#[must_use]
pub fn pack_font(
    height_points: u16,
    bold: bool,
    italic: bool,
    strikeout: bool,
    color_icv: u16,
    name: &str,
) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&(height_points.saturating_mul(20)).to_le_bytes());
    let mut grbit = 0u16;
    if italic {
        grbit |= 0x02;
    }
    if strikeout {
        grbit |= 0x08;
    }
    data.extend_from_slice(&grbit.to_le_bytes());
    data.extend_from_slice(&color_icv.to_le_bytes());
    data.extend_from_slice(&(if bold { 700u16 } else { 400u16 }).to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes()); // sss
    data.extend_from_slice(&[0, 0, 0, 0]); // uls, family, charset, reserved
    data.extend_from_slice(&encode_short_unicode_string(name));
    data
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Packs one MERGECELLS range (8 bytes): `rwFirst..rwLast`, `colFirst..colLast`.
///
/// Java HSSF `MergedCellsTable` / record 0x00E5.
#[must_use]
pub fn pack_merge_range(first_row: u16, last_row: u16, first_col: u16, last_col: u16) -> [u8; 8] {
    let mut d = [0u8; 8];
    d[0..2].copy_from_slice(&first_row.to_le_bytes());
    d[2..4].copy_from_slice(&last_row.to_le_bytes());
    d[4..6].copy_from_slice(&first_col.to_le_bytes());
    d[6..8].copy_from_slice(&last_col.to_le_bytes());
    d
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Emits one or more MERGECELLS records (max 1027 ranges each).
// 语义敏感：chunks(1027) 保证每记录范围数 <= 1027（远小于 u16 上限），
// MERGECELLS 计数按 BIFF8 规范为 u16。
#[allow(clippy::cast_possible_truncation)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn write_merge_cells(out: &mut Vec<u8>, ranges: &[[u8; 8]]) {
    const MAX_PER_RECORD: usize = 1027;
    for chunk in ranges.chunks(MAX_PER_RECORD) {
        let mut data = Vec::with_capacity(2 + chunk.len() * 8);
        data.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        for range in chunk {
            data.extend_from_slice(range);
        }
        record(out, MERGECELLS, &data);
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes a PALETTE record with optional RGB overrides at indices 8..
///
/// Java HSSF `PaletteRecord` — first override replaces palette slot 8, etc.
// 语义敏感：BIFF8 调色板最多 56 色，usize->u16 不可能截断。
#[allow(clippy::cast_possible_truncation)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn write_palette_record(out: &mut Vec<u8>, overrides: &[(u8, u8, u8)]) {
    // Standard BIFF8 customizable palette (56 colours, indices 8..63).
    let mut colours: [(u8, u8, u8); 56] = [
        (0, 0, 0),
        (255, 255, 255),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 255, 0),
        (255, 0, 255),
        (0, 255, 255),
        (128, 0, 0),
        (0, 128, 0),
        (0, 0, 128),
        (128, 128, 0),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (153, 153, 255),
        (153, 51, 102),
        (255, 255, 204),
        (204, 255, 255),
        (102, 0, 102),
        (255, 128, 128),
        (0, 102, 204),
        (204, 204, 255),
        (0, 0, 128),
        (255, 0, 255),
        (255, 255, 0),
        (0, 255, 255),
        (128, 0, 128),
        (128, 0, 0),
        (0, 128, 128),
        (0, 0, 255),
        (0, 204, 255),
        (204, 255, 255),
        (204, 255, 204),
        (255, 255, 153),
        (153, 204, 255),
        (255, 153, 204),
        (204, 153, 255),
        (255, 204, 153),
        (51, 102, 255),
        (51, 204, 204),
        (153, 204, 0),
        (255, 204, 0),
        (255, 153, 0),
        (255, 102, 0),
        (102, 102, 153),
        (150, 150, 150),
        (0, 51, 102),
        (51, 153, 102),
        (0, 51, 0),
        (51, 51, 0),
        (153, 51, 0),
        (153, 51, 102),
        (51, 51, 153),
        (51, 51, 51),
    ];
    for (i, rgb) in overrides.iter().enumerate() {
        if i < colours.len() {
            colours[i] = *rgb;
        }
    }
    let mut data = Vec::with_capacity(2 + colours.len() * 4);
    data.extend_from_slice(&(colours.len() as u16).to_le_bytes());
    for &(r, g, b) in &colours {
        data.push(r);
        data.push(g);
        data.push(b);
        data.push(0);
    }
    record(out, PALETTE, &data);
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Packs a COLINFO record payload (12 bytes).
///
/// `width_chars` is Excel's character width; stored as `width * 256` (POI
/// `sheet.setColumnWidth(col, chars * 256)`).
#[must_use]
pub fn pack_colinfo(first_col: u8, last_col: u8, width_chars: u16, xf_index: u16) -> [u8; 12] {
    let coldx = width_chars.saturating_mul(256);
    let mut d = [0u8; 12];
    d[0..2].copy_from_slice(&u16::from(first_col).to_le_bytes());
    d[2..4].copy_from_slice(&u16::from(last_col).to_le_bytes());
    d[4..6].copy_from_slice(&coldx.to_le_bytes());
    d[6..8].copy_from_slice(&xf_index.to_le_bytes());
    // options + unused remain zero.
    d
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Packs a ROW record payload (16 bytes).
///
/// `height_points` is converted to twips (`* 20`), matching POI
/// `row.setHeightInPoints` / Java `StyleDataTest` (`40pt → 800`).
#[must_use]
pub fn pack_row(row: u16, first_col: u8, last_col_exclusive: u8, height_points: u16) -> [u8; 16] {
    let miy = height_points.saturating_mul(20) & 0x7FFF; // bit15=0 → custom height
    let mut d = [0u8; 16];
    d[0..2].copy_from_slice(&row.to_le_bytes());
    d[2..4].copy_from_slice(&u16::from(first_col).to_le_bytes());
    d[4..6].copy_from_slice(&u16::from(last_col_exclusive).to_le_bytes());
    d[6..8].copy_from_slice(&miy.to_le_bytes());
    // unused + reserved
    // option flags: bit8 always 1 (0x100); bit6 = height unsynced (0x40)
    let options: u32 = 0x0100 | 0x0040;
    d[12..16].copy_from_slice(&options.to_le_bytes());
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_colinfo_matches_poi_character_units() {
        let bytes = pack_colinfo(0, 0, 50, XF_GENERAL);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 50 * 256);
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), XF_GENERAL);
    }

    #[test]
    fn pack_row_matches_poi_twips() {
        let bytes = pack_row(0, 0, 2, 40);
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 800);
    }

    #[test]
    fn pack_cell_xf_solid_yellow() {
        // IndexedColors.YELLOW = 13, solid pattern = 1, valign bottom = 2.
        let bytes = pack_cell_xf(
            0,
            0,
            0,
            2,
            false,
            1,
            13,
            ICV_PATTERN_BG_DEFAULT,
            0,
            0,
            0,
            0,
            ICV_AUTO,
            ICV_AUTO,
            ICV_AUTO,
            ICV_AUTO,
        );
        let brd2 = u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
        assert_eq!((brd2 >> 26) & 0x3F, 1);
        let pat = u16::from_le_bytes([bytes[18], bytes[19]]);
        assert_eq!(pat & 0x7F, 13);
        assert_eq!((pat >> 7) & 0x7F, ICV_PATTERN_BG_DEFAULT);
        assert_eq!((bytes[6] >> 4) & 0x07, 2);
    }

    #[test]
    fn pack_font_bold_arial_12() {
        let bytes = pack_font(12, true, false, false, ICV_AUTO, "Arial");
        assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), 240);
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 700);
    }

    #[test]
    fn pack_merge_range_layout() {
        let bytes = pack_merge_range(1, 2, 0, 1);
        assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), 1);
        assert_eq!(u16::from_le_bytes([bytes[2], bytes[3]]), 2);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 0);
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 1);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn encode_unicode_string_supports_16bit_chars() {
        let encoded = encode_unicode_string("中文");
        assert_eq!(encoded[0], 2);
        assert_eq!(encoded[1], 0);
        assert_eq!(encoded[2], 0x01);
    }

    #[test]
    fn encode_short_unicode_string_supports_16bit_chars() {
        let encoded = encode_short_unicode_string("中文");
        assert_eq!(encoded[0], 2);
        assert_eq!(encoded[1], 0x01);
    }

    #[test]
    fn encode_rk_handles_non_finite_and_fractional_values() {
        assert_eq!(encode_rk(f64::NAN), None);
        assert_eq!(encode_rk(f64::INFINITY), None);
        let hundredth = encode_rk(1.5).expect("1.5 packs as /100 RK");
        assert_eq!(hundredth & 0x03, 0x03);
        let truncated = encode_rk(4_294_967_296.0).expect("2^32 packs as truncated RK");
        assert_eq!(truncated & 0x03, 0x00);
        assert_eq!(encode_rk(1.0 / 3.0), None);
    }

    #[test]
    fn pack_cell_xf_wrap_flag_sets_alignment_bit() {
        let xf = pack_cell_xf(
            0, 0, 0, 0, true, 0, 0, 0, 0, 0, 0, 0, ICV_AUTO, ICV_AUTO, ICV_AUTO, ICV_AUTO,
        );
        assert_eq!(xf[6] & 0x08, 0x08);
    }

    #[test]
    fn pack_font_italic_and_strikeout_set_grbit() {
        let font = pack_font(12, false, true, true, 0, "Test");
        let grbit = u16::from_le_bytes([font[2], font[3]]);
        assert_eq!(grbit & 0x0A, 0x0A);
    }
}
