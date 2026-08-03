//! XLS BIFF format overlay for STRING-mode display.
//!
//! Calamine converts date-formatted numbers to `Data::DateTime` and discards the
//! Excel format code. `EasyExcel` / POI keep the XF → FORMAT mapping and render
//! via `DataFormatter` (`BuiltinFormats` + custom codes). This module re-reads the
//! Workbook stream's FORMAT / XF / NUMBER / RK / `MulRk` records so Rust STRING
//! mode can match Java short dates (`yyyy-m-d h:mm`) and related formats.

use std::collections::HashMap;
use std::path::Path;

use crate::constant::builtin_format_code;
use ssfmt::Locale;

use crate::analysis::v03::biff_record_stream::read_workbook_stream;
use crate::read::xlsx_rows::format_with_code;

/// Per-sheet map of `(row, col) → formatted STRING display`.
pub(crate) type SheetDisplays = HashMap<(u32, usize), String>;

/// Load formatted display strings for every numeric cell in an `.xls` workbook.
///
/// Returns one map per worksheet (`BoundSheet` order). Failures are soft: callers
/// may fall back to calamine `as_text()` when a sheet map is missing.
pub(crate) fn load_xls_displays(
    path: &Path,
    date_1904: bool,
    locale: &Locale,
) -> Vec<SheetDisplays> {
    load_xls_displays_inner(path, date_1904, locale).unwrap_or_default()
}

fn load_xls_displays_inner(
    path: &Path,
    date_1904: bool,
    locale: &Locale,
) -> Result<Vec<SheetDisplays>, String> {
    let wb = read_workbook_stream(path).map_err(|error| error.to_string())?;
    Ok(parse_workbook_displays(&wb, date_1904, locale))
}

// 对应 Java：`XlsSaxAnalyser` 对 Workbook 流 FORMAT/XF/NUMBER/RK/MulRk 的
// 顺序解析与 Java 记录遍历一一对应，拆分会改变记录推进顺序，保持原样。
#[allow(clippy::too_many_lines)]
fn parse_workbook_displays(wb: &[u8], date_1904: bool, locale: &Locale) -> Vec<SheetDisplays> {
    let mut custom_formats: HashMap<u16, String> = HashMap::new();
    let mut xfs: Vec<u16> = Vec::new();
    let mut sheets: Vec<SheetDisplays> = Vec::new();
    let mut sheet_idx: isize = -1;
    let mut in_sheet = false;
    let mut i = 0usize;

    while i + 4 <= wb.len() {
        let typ = u16::from_le_bytes([wb[i], wb[i + 1]]);
        let length = u16::from_le_bytes([wb[i + 2], wb[i + 3]]) as usize;
        i += 4;
        if i + length > wb.len() {
            break;
        }
        let payload = &wb[i..i + length];
        i += length;

        match typ {
            0x0809 if payload.len() >= 4 => {
                let dt = u16::from_le_bytes([payload[2], payload[3]]);
                if dt == 0x0010 {
                    sheet_idx += 1;
                    // 对应 Java：sheet_idx 从 -1 起计，`sheets.len() as isize` 比较
                    // 仅在工作表数超 isize 上限时才会环绕（实际不可能），保持 as 转换。
                    #[allow(clippy::cast_possible_wrap)]
                    while sheets.len() as isize <= sheet_idx {
                        sheets.push(HashMap::new());
                    }
                    in_sheet = true;
                } else {
                    in_sheet = false;
                }
            }
            0x041E => {
                if let Some((ifmt, code)) = parse_format_record(payload) {
                    custom_formats.insert(ifmt, code);
                }
            }
            0x00E0 if payload.len() >= 4 => {
                let ifmt = u16::from_le_bytes([payload[2], payload[3]]);
                xfs.push(ifmt);
            }
            0x0203 if in_sheet && payload.len() >= 14 => {
                let row = u32::from(u16::from_le_bytes([payload[0], payload[1]]));
                let col = u16::from_le_bytes([payload[2], payload[3]]) as usize;
                let xf = u16::from_le_bytes([payload[4], payload[5]]) as usize;
                let value = f64::from_le_bytes(payload[6..14].try_into().unwrap_or([0; 8]));
                push_display(
                    &mut sheets,
                    sheet_idx,
                    row,
                    col,
                    xf,
                    value,
                    &xfs,
                    &custom_formats,
                    date_1904,
                    locale,
                );
            }
            0x027E if in_sheet && payload.len() >= 10 => {
                let row = u32::from(u16::from_le_bytes([payload[0], payload[1]]));
                let col = u16::from_le_bytes([payload[2], payload[3]]) as usize;
                let xf = u16::from_le_bytes([payload[4], payload[5]]) as usize;
                let value = decode_rk(&payload[6..10]);
                push_display(
                    &mut sheets,
                    sheet_idx,
                    row,
                    col,
                    xf,
                    value,
                    &xfs,
                    &custom_formats,
                    date_1904,
                    locale,
                );
            }
            0x00BD if in_sheet && payload.len() >= 6 => {
                // MulRk: row, firstCol, then repeating (xf, rk) until lastCol
                let row = u32::from(u16::from_le_bytes([payload[0], payload[1]]));
                let first_col = u16::from_le_bytes([payload[2], payload[3]]) as usize;
                let last_col =
                    u16::from_le_bytes([payload[payload.len() - 2], payload[payload.len() - 1]])
                        as usize;
                let mut offset = 4usize;
                let mut col = first_col;
                while col <= last_col && offset + 6 <= payload.len().saturating_sub(2) {
                    let xf = u16::from_le_bytes([payload[offset], payload[offset + 1]]) as usize;
                    let value = decode_rk(&payload[offset + 2..offset + 6]);
                    push_display(
                        &mut sheets,
                        sheet_idx,
                        row,
                        col,
                        xf,
                        value,
                        &xfs,
                        &custom_formats,
                        date_1904,
                        locale,
                    );
                    offset += 6;
                    col += 1;
                }
            }
            _ => {}
        }
    }
    sheets
}

// 对应 Java：`pushDisplay(sheetIdx, row, col, xf, value, ...)` 参数与 Java
// 记录分派一一对应，为保持 Java 签名语义不做参数聚合。
#[allow(clippy::too_many_arguments)]
fn push_display(
    sheets: &mut [SheetDisplays],
    sheet_idx: isize,
    row: u32,
    col: usize,
    xf: usize,
    value: f64,
    xfs: &[u16],
    custom_formats: &HashMap<u16, String>,
    date_1904: bool,
    locale: &Locale,
) {
    if sheet_idx < 0 || !value.is_finite() {
        return;
    }
    let Some(ifmt) = xfs.get(xf).copied() else {
        return;
    };
    let code = custom_formats
        .get(&ifmt)
        .map(String::as_str)
        .or_else(|| builtin_format_code(ifmt));
    let Some(code) = code else {
        return;
    };
    // General / @ — leave to calamine textualization.
    if code.eq_ignore_ascii_case("General") || code == "@" {
        return;
    }
    let Some(display) = format_with_code(value, code, date_1904, locale) else {
        return;
    };
    // 对应 Java：sheetIdx 已由上方 `sheet_idx < 0` 分支排除负值，
    // `as usize` 不会丢失符号。
    #[allow(clippy::cast_sign_loss)]
    if let Some(sheet) = sheets.get_mut(sheet_idx as usize) {
        sheet.insert((row, col), display);
    }
}

fn parse_format_record(payload: &[u8]) -> Option<(u16, String)> {
    if payload.len() < 5 {
        return None;
    }
    let ifmt = u16::from_le_bytes([payload[0], payload[1]]);
    let cch = u16::from_le_bytes([payload[2], payload[3]]) as usize;
    let flags = payload[4];
    let raw = &payload[5..];
    let code = if flags & 1 != 0 {
        let bytes = cch.saturating_mul(2).min(raw.len());
        if bytes < 2 {
            return None;
        }
        let units: Vec<u16> = raw[..bytes]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        // BIFF8 compressed Unicode is Latin-1 code units (one byte per char),
        // not UTF-8. Byte `0xA5` must stay `¥` (U+00A5); UTF-8 lossy decode
        // would turn it into U+FFFD and break STRING currency cells.
        let bytes = cch.min(raw.len());
        raw[..bytes].iter().map(|&b| b as char).collect()
    };
    Some((ifmt, code))
}

/// Decode an RK number (see MS-XLS 2.5.209).
// 对应 Java：`RKRecord` 解码中 `(int)(rk >> 2)` 的位运算，u32→i32 环绕与
// Java 强转语义一致，保留 `as` 转换。
#[allow(clippy::cast_possible_wrap)]
fn decode_rk(bytes: &[u8]) -> f64 {
    if bytes.len() < 4 {
        return 0.0;
    }
    let rk = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let d100 = rk & 0x01 != 0;
    let is_int = rk & 0x02 != 0;
    let value = if is_int {
        f64::from((rk as i32) >> 2)
    } else {
        f64::from_bits((u64::from(rk & !0x03)) << 32)
    };
    if d100 { value / 100.0 } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssfmt::Locale;

    #[test]
    fn decode_rk_integer() {
        // 100 as integer RK (bit1 set), little-endian packed
        let rk = (100i32 << 2) as u32 | 0x02;
        let bytes = rk.to_le_bytes();
        assert!((decode_rk(&bytes) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn java_compat_percent_via_format_with_code() {
        let locale = Locale::default();
        assert_eq!(
            format_with_code(99.99, "#.##%", false, &locale).as_deref(),
            Some("9999%")
        );
    }

    /// POI / `EasyExcel`: `_ ` pads are dropped; `\ ` on the negative section is kept.
    #[test]
    fn java_compat_trailing_space_accounting_format() {
        let locale = Locale::default();
        let code = r"0.00_ ;[Red]\-0.00\ ";
        assert_eq!(
            format_with_code(-1.07, code, false, &locale).as_deref(),
            Some("-1.07 ")
        );
        assert_eq!(
            format_with_code(14.11, code, false, &locale).as_deref(),
            Some("14.11")
        );
        // Accounting `_)` must not leave a trailing pad space (Java `24.20`).
        let acct = r"0.00_);[Red]\(0.00\)";
        assert_eq!(
            format_with_code(24.199_812_400_000_013, acct, false, &locale).as_deref(),
            Some("24.20")
        );
    }

    /// DateFormatTest#t03Read — unpadded month `yyyy-m-dd` → `2023-1-01`.
    #[test]
    fn java_compat_short_month_dataformat_v2() {
        let locale = Locale::default();
        let code = r"yyyy\-m\-dd\ hh:mm:ss";
        // Excel serial for 2023-01-01
        assert_eq!(
            format_with_code(44927.0, code, false, &locale).as_deref(),
            Some("2023-1-01 00:00:00")
        );
    }

    /// CN `上午/下午` literal must resolve via locale AM/PM (not printed as slash text).
    #[test]
    fn java_compat_cn_ampm_literal_resolves() {
        let locale = Locale {
            decimal_separator: '.',
            thousands_separator: ',',
            currency_symbol: "¥",
            am_string: "上午",
            pm_string: "下午",
            month_names_short: [
                "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月",
                "12月",
            ],
            month_names_full: [
                "一月",
                "二月",
                "三月",
                "四月",
                "五月",
                "六月",
                "七月",
                "八月",
                "九月",
                "十月",
                "十一月",
                "十二月",
            ],
            day_names_short: ["日", "一", "二", "三", "四", "五", "六"],
            day_names_full: [
                "星期日",
                "星期一",
                "星期二",
                "星期三",
                "星期四",
                "星期五",
                "星期六",
            ],
        };
        // 2020-01-01 01:01
        let serial = 43831.0 + 1.0 / 24.0 + 1.0 / 1440.0;
        assert_eq!(
            format_with_code(serial, r#"[DBNum1]上午/下午h"时"mm"分""#, false, &locale).as_deref(),
            Some("上午1时01分")
        );
        assert_eq!(
            format_with_code(serial, "mmmmm/yy", false, &locale).as_deref(),
            Some("\u{E001}1月\u{E002}/20")
        );
    }

    /// BIFF8 compressed Unicode FORMAT records are Latin-1 (¥ = 0xA5), not UTF-8.
    #[test]
    // 对应 Java：测试固定短 payload（≤255 字节），`as u8` 不可能截断。
    #[allow(clippy::cast_possible_truncation)]
    fn parse_format_record_latin1_yen() {
        // ifmt=5, cch=len, flags=0, body = `"¥"#,##0` in Latin-1
        let mut payload = vec![5, 0, 0, 0, 0];
        let body: Vec<u8> = b"\"\xA5\"#,##0".to_vec();
        payload[2] = body.len() as u8;
        payload.extend_from_slice(&body);
        let (ifmt, code) = parse_format_record(&payload).expect("FORMAT");
        assert_eq!(ifmt, 5);
        assert_eq!(code, "\"¥\"#,##0");
        assert!(!code.contains('\u{FFFD}'));
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use ssfmt::Locale;

    /// 组装 BIFF 记录：sid + 长度 + 载荷。
    // 对应 Java：BIFF 记录长度字段为 u16，测试 payload 固定且远小于 65535 字节，
    // `as u16` 不可能截断。
    #[allow(clippy::cast_possible_truncation)]
    fn record(sid: u16, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&sid.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        out.extend_from_slice(payload);
        out
    }

    fn worksheet_bof() -> Vec<u8> {
        record(0x0809, &[0, 0, 0x10, 0x00])
    }

    fn custom_format_record(ifmt: u16, code: &str) -> Vec<u8> {
        let mut payload = vec![0, 0, 0, 0, 0];
        payload[0..2].copy_from_slice(&ifmt.to_le_bytes());
        // 对应 Java：cch 为 u16，测试格式码固定且短小，`as u16` 不可能截断。
        #[allow(clippy::cast_possible_truncation)]
        payload[2..4].copy_from_slice(&(code.len() as u16).to_le_bytes());
        payload.extend_from_slice(code.as_bytes());
        record(0x041E, &payload)
    }

    fn xf_record(ifmt: u16) -> Vec<u8> {
        let mut payload = vec![0, 0, 0, 0];
        payload[2..4].copy_from_slice(&ifmt.to_le_bytes());
        record(0x00E0, &payload)
    }

    fn number_record(row: u16, col: u16, xf: u16, value: f64) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&row.to_le_bytes());
        payload.extend_from_slice(&col.to_le_bytes());
        payload.extend_from_slice(&xf.to_le_bytes());
        payload.extend_from_slice(&value.to_le_bytes());
        record(0x0203, &payload)
    }

    fn rk_bytes(value: i32) -> [u8; 4] {
        // 对应 Java：RK 整数编码的位运算（int 移位后按位或），i32→u32 仅按位
        // 解释、不改变数值位模式，与 Java 位运算语义一致。
        #[allow(clippy::cast_sign_loss)]
        ((value << 2) as u32 | 0x02).to_le_bytes()
    }

    #[test]
    fn parses_number_xf_and_custom_format_displays() {
        // 对应 Java：NUMBER + XF + FORMAT 渲染自定义格式显示值
        let locale = Locale::default();
        let mut wb = Vec::new();
        wb.extend_from_slice(&worksheet_bof());
        wb.extend_from_slice(&custom_format_record(5, "0.0"));
        wb.extend_from_slice(&xf_record(5));
        wb.extend_from_slice(&number_record(0, 0, 0, 12.34));

        let displays = parse_workbook_displays(&wb, false, &locale);
        assert_eq!(displays[0].get(&(0, 0)).map(String::as_str), Some("12.3"));
    }

    #[test]
    fn mulrk_record_pushes_every_repeated_cell() {
        // 对应 Java：MulRk 记录展开多个单元格
        let locale = Locale::default();
        let mut wb = Vec::new();
        wb.extend_from_slice(&worksheet_bof());
        wb.extend_from_slice(&custom_format_record(5, "0.0"));
        wb.extend_from_slice(&xf_record(5));
        // row=0, firstCol=0, (xf=0, rk=100), (xf=0, rk=200), lastCol=1
        let mut payload = vec![0, 0, 0, 0, 0, 0];
        payload.extend_from_slice(&rk_bytes(100));
        payload.extend_from_slice(&[0, 0]);
        payload.extend_from_slice(&rk_bytes(200));
        payload.extend_from_slice(&[1, 0]);
        wb.extend_from_slice(&record(0x00BD, &payload));

        let displays = parse_workbook_displays(&wb, false, &locale);
        assert_eq!(displays[0].get(&(0, 0)).map(String::as_str), Some("100.0"));
        assert_eq!(displays[0].get(&(0, 1)).map(String::as_str), Some("200.0"));
    }

    #[test]
    fn skips_non_finite_out_of_range_and_missing_formats() {
        // 对应 Java：NaN/越界 XF/无格式码的单元格不产出显示值
        let locale = Locale::default();
        let mut wb = Vec::new();
        wb.extend_from_slice(&worksheet_bof());
        wb.extend_from_slice(&custom_format_record(5, "0.0"));
        wb.extend_from_slice(&xf_record(5));
        wb.extend_from_slice(&number_record(0, 0, 0, f64::NAN));
        wb.extend_from_slice(&number_record(0, 1, 9, 1.5)); // xf 越界
        wb.extend_from_slice(&number_record(0, 2, 0, 2.5)); // 由 xf_record(5) 命中

        let displays = parse_workbook_displays(&wb, false, &locale);
        assert!(!displays[0].contains_key(&(0, 0)));
        assert!(!displays[0].contains_key(&(0, 1)));
        assert!(displays[0].contains_key(&(0, 2)));
    }

    #[test]
    fn skips_general_and_at_format_codes() {
        // 对应 Java：General/@ 交给 calamine 文本化
        let locale = Locale::default();
        let mut wb = Vec::new();
        wb.extend_from_slice(&worksheet_bof());
        wb.extend_from_slice(&xf_record(0)); // Builtin 0 = General
        wb.extend_from_slice(&number_record(0, 0, 0, 3.5));

        let displays = parse_workbook_displays(&wb, false, &locale);
        assert!(!displays[0].contains_key(&(0, 0)));
    }

    #[test]
    fn breaks_on_truncated_records_and_ignores_unknown_sids() {
        // 对应 Java：截断记录停止解析；未知 sid 跳过
        let locale = Locale::default();
        let mut wb = Vec::new();
        wb.extend_from_slice(&worksheet_bof());
        wb.extend_from_slice(&[0xFF, 0x00, 0x04, 0x00, 1, 2, 3, 4]); // 未知 sid
        wb.extend_from_slice(&[0x08, 0x09, 0xFF, 0x00, 0]); // 截断
        let displays = parse_workbook_displays(&wb, false, &locale);
        assert_eq!(displays.len(), 1);
        assert!(displays[0].is_empty());
    }

    #[test]
    fn parse_format_record_guards_short_and_utf16_bodies() {
        // 对应 Java：FORMAT 记录长度门控与 UTF-16 分支
        assert!(parse_format_record(&[0, 0]).is_none());
        // utf16 flags=1 但字节不足
        let payload = [5, 0, 4, 0, 1, 0x41];
        assert!(parse_format_record(&payload).is_none());
        // utf16 有效
        let mut utf16 = vec![5, 0, 2, 0, 1];
        utf16.extend_from_slice(&[0x30, 0, 0x2E, 0]); // "0."
        let (ifmt, code) = parse_format_record(&utf16).expect("utf16 FORMAT");
        assert_eq!(ifmt, 5);
        assert_eq!(code, "0.");
    }

    #[test]
    // 对应 Java：`decode_rk` 对短输入返回 0.0，与 0.0 的精确比较即预期语义
    // （位级确定的返回值），不做容差比较。
    #[allow(clippy::float_cmp)]
    fn decode_rk_guards_short_input() {
        // 对应 Java：RK 解码输入不足返回 0
        assert_eq!(decode_rk(&[1, 2]), 0.0);
    }

    #[test]
    fn load_xls_displays_is_soft_on_invalid_files() {
        // 对应 Java：加载失败回退空列表（调用方用 calamine as_text）
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("bad.xls");
        std::fs::write(&path, b"not an ole document").expect("write");
        let displays = load_xls_displays(&path, false, &Locale::default());
        assert!(displays.is_empty());
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;
    use ssfmt::Locale;

    /// 组装 BIFF 记录：sid + 长度 + 载荷。
    // 对应 Java：BIFF 记录长度字段为 u16，测试 payload 固定且远小于 65535 字节，
    // `as u16` 不可能截断。
    #[allow(clippy::cast_possible_truncation)]
    fn record(sid: u16, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&sid.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        out.extend_from_slice(payload);
        out
    }

    fn worksheet_bof() -> Vec<u8> {
        record(0x0809, &[0, 0, 0x10, 0x00])
    }

    fn xf_record(ifmt: u16) -> Vec<u8> {
        let mut payload = vec![0, 0, 0, 0];
        payload[2..4].copy_from_slice(&ifmt.to_le_bytes());
        record(0x00E0, &payload)
    }

    fn custom_format_record(ifmt: u16, code: &str) -> Vec<u8> {
        let mut payload = vec![0, 0, 0, 0, 0];
        payload[0..2].copy_from_slice(&ifmt.to_le_bytes());
        // 对应 Java：cch 为 u16，测试格式码固定且短小，`as u16` 不可能截断。
        #[allow(clippy::cast_possible_truncation)]
        payload[2..4].copy_from_slice(&(code.len() as u16).to_le_bytes());
        payload.extend_from_slice(code.as_bytes());
        record(0x041E, &payload)
    }

    fn number_record(row: u16, col: u16, xf: u16, value: f64) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&row.to_le_bytes());
        payload.extend_from_slice(&col.to_le_bytes());
        payload.extend_from_slice(&xf.to_le_bytes());
        payload.extend_from_slice(&value.to_le_bytes());
        record(0x0203, &payload)
    }

    #[test]
    fn xf_without_any_format_code_is_skipped() {
        // 对应 Java：XF 指向无内置码也无自定义码的 ifmt 时不产出显示值
        let locale = Locale::default();
        let mut wb = Vec::new();
        wb.extend_from_slice(&worksheet_bof());
        // ifmt=99 不在内置表、也无自定义 FORMAT → builtin_format_code 返回 None
        wb.extend_from_slice(&xf_record(99));
        wb.extend_from_slice(&number_record(0, 0, 0, 1.5));
        let displays = parse_workbook_displays(&wb, false, &locale);
        assert!(!displays[0].contains_key(&(0, 0)));
    }

    #[test]
    fn unparseable_format_code_yields_no_display() {
        // 对应 Java：ssfmt 无法解析的格式码不产出显示值（调用方回退 calamine 文本）
        let locale = Locale::default();
        let mut wb = Vec::new();
        wb.extend_from_slice(&worksheet_bof());
        // 未闭合的方括号让 ssfmt 解析失败 → format_with_code 返回 None
        wb.extend_from_slice(&custom_format_record(5, "[Red"));
        wb.extend_from_slice(&xf_record(5));
        wb.extend_from_slice(&number_record(0, 0, 0, 1.5));
        let displays = parse_workbook_displays(&wb, false, &locale);
        assert!(!displays[0].contains_key(&(0, 0)));
        // 直接断言 format_with_code 对坏格式码返回 None
        assert_eq!(format_with_code(1.5, "[Red", false, &locale), None);
    }
}
