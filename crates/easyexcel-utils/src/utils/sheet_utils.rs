//! 对应 Java：`com.alibaba.excel.util.SheetUtils` 的可复用实现。

#![allow(dead_code)]

/// Excel/POI 允许的最大列宽字符数。
/// 对应 Java：com.alibaba.excel.util.SheetUtils。
pub const MAX_COLUMN_WIDTH_CHARS: u16 = 255;

/// 对应 Java：com.alibaba.excel.util.SheetUtils。 Mirrors `com.alibaba.excel.util.SheetUtils#match`.
///
/// Java matches a Java `RegEx` against the sheet name. Rust uses the
/// `regex` crate's syntax via `str::contains` for the placeholder, the
/// writer/reader crates own the real matching.
#[must_use]
pub fn match_sheet(sheet_name: &str, sheet_pattern: &str) -> bool {
    sheet_name == sheet_pattern || sheet_pattern.is_empty()
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn match_sheet_compares_or_accepts_empty_pattern() {
        // 对应 Java：SheetUtils.match
        assert!(match_sheet("Sheet1", "Sheet1"));
        assert!(match_sheet("Sheet1", ""));
        assert!(!match_sheet("Sheet1", "Sheet2"));
    }
}
