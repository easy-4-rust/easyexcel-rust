//! BIFF8 数字单元格与 XF/FORMAT 覆盖信息扫描。
//!
//! 该层只解析原始 BIFF 记录，不执行 EasyExcel/POI 的本地化显示格式化。

use std::collections::HashMap;
use std::path::Path;

use easyexcel_format::{SpreadsheetLocale, builtin_format_code, format_with_code};
use easyexcel_io::Result;

/// 一个数字单元格及其 BIFF8 数字格式元数据。
#[derive(Debug, Clone, PartialEq)]
pub struct Biff8NumericCell {
    /// 原始 IEEE754 数值。
    pub value: f64,
    /// XF 引用的 `ifmt` 索引。
    pub format_index: u16,
    /// FORMAT 记录定义的自定义格式；内建格式为 `None`。
    pub custom_format: Option<String>,
}

/// 每个工作表的 `(row, col) -> numeric cell` 映射。
pub type Biff8NumericSheets = Vec<HashMap<(u32, usize), Biff8NumericCell>>;

/// 每个工作表的 `(row, col) -> Excel 格式化显示文本` 映射。
pub type Biff8SheetDisplays = Vec<HashMap<(u32, usize), String>>;

/// 从 `.xls` 文件加载所有数字单元格的 Excel 显示文本。
///
/// # Errors
///
/// OLE/CFB 或 Workbook 流读取失败时返回错误。
pub fn load_numeric_displays(
    path: &Path,
    date_1904: bool,
    locale: &SpreadsheetLocale,
) -> Result<Biff8SheetDisplays> {
    let workbook = super::record_stream::read_workbook_stream(path)?;
    Ok(format_numeric_displays(&workbook, date_1904, locale))
}

/// 从原始 BIFF8 Workbook 流生成数字显示文本。
#[must_use]
pub fn format_numeric_displays(
    workbook: &[u8],
    date_1904: bool,
    locale: &SpreadsheetLocale,
) -> Biff8SheetDisplays {
    scan_numeric_cells(workbook)
        .into_iter()
        .map(|cells| {
            cells
                .into_iter()
                .filter_map(|(position, cell)| {
                    if !cell.value.is_finite() {
                        return None;
                    }
                    let code = cell
                        .custom_format
                        .as_deref()
                        .or_else(|| builtin_format_code(cell.format_index))?;
                    if code.eq_ignore_ascii_case("General") || code == "@" {
                        return None;
                    }
                    format_with_code(cell.value, code, date_1904, locale)
                        .map(|display| (position, display))
                })
                .collect()
        })
        .collect()
}

/// 顺序扫描 Workbook 流中的 FORMAT、XF、NUMBER、RK 与 MULRK 记录。
#[must_use]
pub fn scan_numeric_cells(workbook: &[u8]) -> Biff8NumericSheets {
    let mut custom_formats: HashMap<u16, String> = HashMap::new();
    let mut xfs: Vec<u16> = Vec::new();
    let mut sheets = Vec::new();
    let mut sheet_index: Option<usize> = None;
    let mut offset = 0usize;

    while offset + 4 <= workbook.len() {
        let typ = u16::from_le_bytes([workbook[offset], workbook[offset + 1]]);
        let length = usize::from(u16::from_le_bytes([
            workbook[offset + 2],
            workbook[offset + 3],
        ]));
        offset += 4;
        if offset + length > workbook.len() {
            break;
        }
        let payload = &workbook[offset..offset + length];
        offset += length;

        match typ {
            0x0809 if payload.len() >= 4 => {
                let data_type = u16::from_le_bytes([payload[2], payload[3]]);
                if data_type == 0x0010 {
                    let index = sheets.len();
                    sheets.push(HashMap::new());
                    sheet_index = Some(index);
                } else {
                    sheet_index = None;
                }
            }
            0x041E => {
                if let Some((format_index, code)) = parse_format_record(payload) {
                    custom_formats.insert(format_index, code);
                }
            }
            0x00E0 if payload.len() >= 4 => {
                xfs.push(u16::from_le_bytes([payload[2], payload[3]]));
            }
            0x0203 if payload.len() >= 14 => {
                push_numeric(
                    &mut sheets,
                    sheet_index,
                    payload,
                    f64::from_le_bytes(payload[6..14].try_into().unwrap_or([0; 8])),
                    &xfs,
                    &custom_formats,
                );
            }
            0x027E if payload.len() >= 10 => {
                push_numeric(
                    &mut sheets,
                    sheet_index,
                    payload,
                    decode_rk(&payload[6..10]),
                    &xfs,
                    &custom_formats,
                );
            }
            0x00BD if payload.len() >= 6 => {
                let Some(sheet) = sheet_index.and_then(|index| sheets.get_mut(index)) else {
                    continue;
                };
                let row = u32::from(u16::from_le_bytes([payload[0], payload[1]]));
                let first_col = usize::from(u16::from_le_bytes([payload[2], payload[3]]));
                let last_col = usize::from(u16::from_le_bytes([
                    payload[payload.len() - 2],
                    payload[payload.len() - 1],
                ]));
                let mut cursor = 4usize;
                let mut col = first_col;
                while col <= last_col && cursor + 6 <= payload.len().saturating_sub(2) {
                    let xf =
                        usize::from(u16::from_le_bytes([payload[cursor], payload[cursor + 1]]));
                    if let Some(format_index) = xfs.get(xf).copied() {
                        sheet.insert(
                            (row, col),
                            Biff8NumericCell {
                                value: decode_rk(&payload[cursor + 2..cursor + 6]),
                                format_index,
                                custom_format: custom_formats.get(&format_index).cloned(),
                            },
                        );
                    }
                    cursor += 6;
                    col += 1;
                }
            }
            _ => {}
        }
    }
    sheets
}

fn push_numeric(
    sheets: &mut Biff8NumericSheets,
    sheet_index: Option<usize>,
    payload: &[u8],
    value: f64,
    xfs: &[u16],
    custom_formats: &HashMap<u16, String>,
) {
    let Some(sheet) = sheet_index.and_then(|index| sheets.get_mut(index)) else {
        return;
    };
    let row = u32::from(u16::from_le_bytes([payload[0], payload[1]]));
    let col = usize::from(u16::from_le_bytes([payload[2], payload[3]]));
    let xf = usize::from(u16::from_le_bytes([payload[4], payload[5]]));
    let Some(format_index) = xfs.get(xf).copied() else {
        return;
    };
    sheet.insert(
        (row, col),
        Biff8NumericCell {
            value,
            format_index,
            custom_format: custom_formats.get(&format_index).cloned(),
        },
    );
}

/// 解析 BIFF8 FORMAT 记录负载。
#[must_use]
pub fn parse_format_record(payload: &[u8]) -> Option<(u16, String)> {
    if payload.len() < 5 {
        return None;
    }
    let format_index = u16::from_le_bytes([payload[0], payload[1]]);
    let count = usize::from(u16::from_le_bytes([payload[2], payload[3]]));
    let raw = &payload[5..];
    let code = if payload[4] & 1 != 0 {
        let bytes = count.saturating_mul(2).min(raw.len());
        if bytes < 2 {
            return None;
        }
        let units = raw[..bytes]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&units)
    } else {
        raw[..count.min(raw.len())]
            .iter()
            .map(|byte| char::from(*byte))
            .collect()
    };
    Some((format_index, code))
}

/// 解码 BIFF8 RK 压缩数值。
#[must_use]
pub fn decode_rk(bytes: &[u8]) -> f64 {
    if bytes.len() < 4 {
        return 0.0;
    }
    let rk = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let value = if rk & 0x02 != 0 {
        f64::from((rk as i32) >> 2)
    } else {
        f64::from_bits(u64::from(rk & 0xFFFF_FFFC) << 32)
    };
    if rk & 0x01 != 0 { value / 100.0 } else { value }
}

#[cfg(test)]
mod tests {
    use easyexcel_format::SpreadsheetLocale;

    use super::{decode_rk, format_numeric_displays, load_numeric_displays, parse_format_record};

    fn record(sid: u16, payload: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();
        output.extend_from_slice(&sid.to_le_bytes());
        output.extend_from_slice(
            &u16::try_from(payload.len())
                .expect("test BIFF payload length")
                .to_le_bytes(),
        );
        output.extend_from_slice(payload);
        output
    }

    fn worksheet_bof() -> Vec<u8> {
        record(0x0809, &[0, 0, 0x10, 0x00])
    }

    fn custom_format_record(format_index: u16, code: &str) -> Vec<u8> {
        let mut payload = vec![0, 0, 0, 0, 0];
        payload[0..2].copy_from_slice(&format_index.to_le_bytes());
        payload[2..4].copy_from_slice(
            &u16::try_from(code.len())
                .expect("test format code length")
                .to_le_bytes(),
        );
        payload.extend_from_slice(code.as_bytes());
        record(0x041E, &payload)
    }

    fn xf_record(format_index: u16) -> Vec<u8> {
        let mut payload = vec![0, 0, 0, 0];
        payload[2..4].copy_from_slice(&format_index.to_le_bytes());
        record(0x00E0, &payload)
    }

    fn number_record(row: u16, column: u16, xf: u16, value: f64) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&row.to_le_bytes());
        payload.extend_from_slice(&column.to_le_bytes());
        payload.extend_from_slice(&xf.to_le_bytes());
        payload.extend_from_slice(&value.to_le_bytes());
        record(0x0203, &payload)
    }

    fn rk_bytes(value: i32) -> [u8; 4] {
        (u32::from_ne_bytes((value << 2).to_ne_bytes()) | 0x02).to_le_bytes()
    }

    #[test]
    fn decodes_integer_rk_and_guards_short_input() {
        let bytes = rk_bytes(100);
        assert!((decode_rk(&bytes) - 100.0).abs() < f64::EPSILON);
        assert_eq!(decode_rk(&[1, 2]), 0.0);
    }

    #[test]
    fn parses_latin1_and_utf16_format_records() {
        let body = b"\"\xA5\"#,##0";
        let mut latin1 = vec![
            5,
            0,
            u8::try_from(body.len()).expect("test body length"),
            0,
            0,
        ];
        latin1.extend_from_slice(body);
        let (format_index, code) = parse_format_record(&latin1).expect("latin1 FORMAT");
        assert_eq!(format_index, 5);
        assert_eq!(code, "\"¥\"#,##0");

        let mut utf16 = vec![5, 0, 2, 0, 1];
        utf16.extend_from_slice(&[0x30, 0, 0x2E, 0]);
        assert_eq!(parse_format_record(&utf16), Some((5, "0.".to_owned())));
        assert!(parse_format_record(&[0, 0]).is_none());
        assert!(parse_format_record(&[5, 0, 4, 0, 1, 0x41]).is_none());
    }

    #[test]
    fn formats_number_records_with_xf_and_custom_format() {
        let mut workbook = worksheet_bof();
        workbook.extend_from_slice(&custom_format_record(5, "0.0"));
        workbook.extend_from_slice(&xf_record(5));
        workbook.extend_from_slice(&number_record(0, 0, 0, 12.34));

        let displays = format_numeric_displays(&workbook, false, &SpreadsheetLocale::default());
        assert_eq!(displays[0].get(&(0, 0)).map(String::as_str), Some("12.3"));
    }

    #[test]
    fn expands_mulrk_cells() {
        let mut workbook = worksheet_bof();
        workbook.extend_from_slice(&custom_format_record(5, "0.0"));
        workbook.extend_from_slice(&xf_record(5));
        let mut payload = vec![0, 0, 0, 0, 0, 0];
        payload.extend_from_slice(&rk_bytes(100));
        payload.extend_from_slice(&[0, 0]);
        payload.extend_from_slice(&rk_bytes(200));
        payload.extend_from_slice(&[1, 0]);
        workbook.extend_from_slice(&record(0x00BD, &payload));

        let displays = format_numeric_displays(&workbook, false, &SpreadsheetLocale::default());
        assert_eq!(displays[0].get(&(0, 0)).map(String::as_str), Some("100.0"));
        assert_eq!(displays[0].get(&(0, 1)).map(String::as_str), Some("200.0"));
    }

    #[test]
    fn skips_unrenderable_numeric_cells() {
        let mut workbook = worksheet_bof();
        workbook.extend_from_slice(&custom_format_record(5, "0.0"));
        workbook.extend_from_slice(&xf_record(5));
        workbook.extend_from_slice(&number_record(0, 0, 0, f64::NAN));
        workbook.extend_from_slice(&number_record(0, 1, 9, 1.5));

        let displays = format_numeric_displays(&workbook, false, &SpreadsheetLocale::default());
        assert!(!displays[0].contains_key(&(0, 0)));
        assert!(!displays[0].contains_key(&(0, 1)));

        let mut general = worksheet_bof();
        general.extend_from_slice(&xf_record(0));
        general.extend_from_slice(&number_record(0, 0, 0, 3.5));
        let displays = format_numeric_displays(&general, false, &SpreadsheetLocale::default());
        assert!(!displays[0].contains_key(&(0, 0)));

        let mut unknown = worksheet_bof();
        unknown.extend_from_slice(&xf_record(99));
        unknown.extend_from_slice(&number_record(0, 0, 0, 1.5));
        let displays = format_numeric_displays(&unknown, false, &SpreadsheetLocale::default());
        assert!(!displays[0].contains_key(&(0, 0)));
    }

    #[test]
    fn stops_at_truncated_record_and_ignores_unknown_sid() {
        let mut workbook = worksheet_bof();
        workbook.extend_from_slice(&[0xFF, 0x00, 0x04, 0x00, 1, 2, 3, 4]);
        workbook.extend_from_slice(&[0x08, 0x09, 0xFF, 0x00, 0]);
        let displays = format_numeric_displays(&workbook, false, &SpreadsheetLocale::default());
        assert_eq!(displays.len(), 1);
        assert!(displays[0].is_empty());
    }

    #[test]
    fn invalid_ole_file_is_reported_by_engine() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("invalid.xls");
        std::fs::write(&path, b"not an ole document").expect("write fixture");
        assert!(load_numeric_displays(&path, false, &SpreadsheetLocale::default()).is_err());
    }
}
