//! 对应 Java： com.alibaba.excel.util.PoiUtils.

#![allow(dead_code)]

/// Mirrors `com.alibaba.excel.util.PoiUtils#customHeight`.
///
/// Java reflects on `XSSFRow` / `HSSFRow` to read the `customHeight`
/// attribute. The Rust writer uses `rust_xlsxwriter`, which exposes
/// this directly via `Worksheet::set_row_height`, so the helper is an
/// inert placeholder that defaults to `false` to preserve the 1:1 file
/// mapping.
#[must_use]
pub fn custom_height() -> bool {
    false
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn custom_height_placeholder_is_false() {
        // 对应 Java：customHeight 占位实现（委托 rust_xlsxwriter）
        assert!(!custom_height());
    }
}
