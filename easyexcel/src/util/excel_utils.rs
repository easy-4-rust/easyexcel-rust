//! Excel utility functions absorbed from hutool-poi.
//!
//! 对应 Java：`cn.hutool.poi.excel.ExcelUtil` (column name conversion),
//! `cn.hutool.poi.excel.ExcelFileUtil` (magic-byte detection), and
//! `cn.hutool.poi.excel.ExcelDateUtil` (date format detection).

/// Converts a zero-based column index to Excel column name (A, B, ..., Z, AA, AB, ...).
///
/// Mirrors hutool `ExcelUtil.indexToColName(int)`.
///
/// ```
/// use easyexcel::util::excel_utils::index_to_col_name;
/// assert_eq!(index_to_col_name(0), "A");
/// assert_eq!(index_to_col_name(25), "Z");
/// assert_eq!(index_to_col_name(26), "AA");
/// ```
#[must_use]
pub fn index_to_col_name(mut index: u32) -> String {
    let mut result = String::new();
    loop {
        let remainder = (index % 26) as u8;
        result.insert(0, (b'A' + remainder) as char);
        if index < 26 {
            break;
        }
        index = (index / 26).saturating_sub(1);
    }
    result
}

/// Converts an Excel column name to a zero-based index.
///
/// Mirrors hutool `ExcelUtil.colNameToIndex(String)`.
///
/// ```
/// use easyexcel::util::excel_utils::col_name_to_index;
/// assert_eq!(col_name_to_index("A"), Some(0));
/// assert_eq!(col_name_to_index("AA"), Some(26));
/// ```
///
/// # Panics
///
/// 不会 panic（列名长度远小于 `u32` 上限，`expect` 仅为静态证明）。
#[must_use]
pub fn col_name_to_index(name: &str) -> Option<u32> {
    let name = name.to_uppercase();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }
    let mut index = 0u32;
    for (i, ch) in name.chars().rev().enumerate() {
        let digit = (ch as u32).saturating_sub(u32::from(b'A'));
        if i == 0 {
            index += digit;
        } else {
            // 列名长度至多 7 位（ZZZZZZZ 约为 u32 上限），try_from 恒成功
            index += (digit + 1) * 26u32.pow(u32::try_from(i).expect("列名长度远小于 u32 上限"));
        }
    }
    Some(index)
}

/// Detects if bytes represent an XLS (BIFF8 / OLE2) file by magic bytes.
///
/// Mirrors hutool `ExcelFileUtil.isXls(InputStream)`.
/// OLE2 compound document magic: `D0 CF 11 E0 A1 B1 1A E1`
#[must_use]
pub fn is_xls_bytes(data: &[u8]) -> bool {
    data.len() >= 8 && data[..8] == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
}

/// Detects if bytes represent an XLSX (OOXML / ZIP) file by magic bytes.
///
/// Mirrors hutool `ExcelFileUtil.isXlsx(InputStream)`.
/// ZIP magic: `PK\x03\x04`
#[must_use]
pub fn is_xlsx_bytes(data: &[u8]) -> bool {
    data.len() >= 4 && data[..4] == [b'P', b'K', 0x03, 0x04]
}

/// Detects if bytes represent a CSV file by checking for common CSV patterns.
/// Returns true if first 4 bytes are printable ASCII (typical CSV start).
#[must_use]
pub fn is_csv_bytes(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    // CSV starts with printable ASCII or BOM
    if data.len() >= 3 && data[..3] == [0xEF, 0xBB, 0xBF] {
        return true; // UTF-8 BOM
    }
    data[0].is_ascii_graphic() || data[0].is_ascii_whitespace()
}

/// Returns true if the format string represents a date format.
///
/// Mirrors hutool `ExcelDateUtil.isDateFormat(int, String)`.
/// Excludes "General" (which contains 'G') and pure number formats.
#[must_use]
pub fn is_date_format(format_str: &str) -> bool {
    if format_str.is_empty() || format_str.eq_ignore_ascii_case("General") {
        return false;
    }
    let chars: Vec<char> = format_str.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let mut count = 1;
        while i + count < chars.len() && chars[i + count] == ch {
            count += 1;
        }
        // 对应 Java：日期格式标记判定（y/m/d/h/s/a），各标记判定条件互不相同，
        // 合并为单一布尔条件以避免 match 同体分支告警，语义与逐臂 return true 完全一致
        if matches!(ch, 'y' | 'Y' | 'h' | 'H' | 'a' | 'A')
            || matches!(ch, 'm' | 'M' | 'd' | 'D') && count <= 4
            || matches!(ch, 's' | 'S') && count <= 2
        {
            return true;
        }
        i += count;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_name_round_trip() {
        for i in [0, 1, 25, 26, 51, 52, 701, 702, 1000] {
            let name = index_to_col_name(i);
            let back = col_name_to_index(&name);
            assert_eq!(
                back,
                Some(i),
                "round-trip failed for index {i}, name={name}"
            );
        }
    }

    #[test]
    fn col_name_to_index_examples() {
        assert_eq!(col_name_to_index("A"), Some(0));
        assert_eq!(col_name_to_index("Z"), Some(25));
        assert_eq!(col_name_to_index("AA"), Some(26));
        assert_eq!(col_name_to_index("AB"), Some(27));
        assert_eq!(col_name_to_index("ZZ"), Some(701));
        assert_eq!(col_name_to_index("AAA"), Some(702));
    }

    #[test]
    fn index_to_col_name_examples() {
        assert_eq!(index_to_col_name(0), "A");
        assert_eq!(index_to_col_name(25), "Z");
        assert_eq!(index_to_col_name(26), "AA");
        assert_eq!(index_to_col_name(27), "AB");
        assert_eq!(index_to_col_name(701), "ZZ");
        assert_eq!(index_to_col_name(702), "AAA");
    }

    #[test]
    fn xls_xlsx_csv_detection() {
        assert!(is_xls_bytes(&[
            0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1
        ]));
        assert!(!is_xls_bytes(b"not xls data"));
        assert!(is_xlsx_bytes(b"PK\x03\x04more"));
        assert!(!is_xlsx_bytes(b"not zip"));
        assert!(is_csv_bytes(b"name,age"));
        assert!(!is_csv_bytes(&[]));
    }

    #[test]
    fn date_format_detection() {
        assert!(is_date_format("yyyy-MM-dd"));
        assert!(is_date_format("yyyy/MM/dd HH:mm:ss"));
        assert!(is_date_format("m/d/yy"));
        // "$-409" locale prefix + "d" + "mmm" + "yy" = date format
        assert!(is_date_format("d-mmm-yy"));
        assert!(!is_date_format("0.00"));
        assert!(!is_date_format("#,##0"));
        assert!(!is_date_format("General"));
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn col_name_to_index_rejects_invalid_names() {
        // 对应 Java（hutool）：非法列名返回 None
        assert_eq!(col_name_to_index(""), None);
        assert_eq!(col_name_to_index("a1"), None);
        assert_eq!(col_name_to_index("A!"), None);
        assert_eq!(col_name_to_index("ab c"), None);
    }

    #[test]
    fn is_csv_bytes_accepts_utf8_bom() {
        // 对应 Java（hutool）：UTF-8 BOM 开头视为 CSV
        assert!(is_csv_bytes(&[0xEF, 0xBB, 0xBF, b'n', b'a']));
        // 空白开头也可识别
        assert!(is_csv_bytes(b" name"));
    }

    #[test]
    fn is_date_format_extra_cases() {
        // 对应 Java（hutool）：其余格式识别边界
        assert!(is_date_format("h:mm AM/PM"));
        assert!(is_date_format("yy"));
        assert!(is_date_format("s.000"));
        assert!(!is_date_format(""));
        assert!(!is_date_format("mmmmm")); // 5 个月字母视为字面量
        assert!(!is_date_format("sss")); // 3 个秒字母视为字面量
        assert!(!is_date_format("General"));
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    #[test]
    fn date_format_detection_accepts_am_pm_marker() {
        // 对应 Java（hutool）：a/A 上午下午标记视为日期格式
        assert!(is_date_format("AM/PM"));
        assert!(is_date_format("a"));
    }
}
