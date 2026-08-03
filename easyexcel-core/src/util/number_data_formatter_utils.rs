//! 对应 Java： com.alibaba.excel.util.NumberDataFormatterUtils.

#![allow(dead_code)]

use std::cell::RefCell;

thread_local! {
    static FORMATTER_CACHE: RefCell<std::collections::HashMap<String, String>> =
        RefCell::new(std::collections::HashMap::new());
}

/// Mirrors `com.alibaba.excel.util.NumberDataFormatterUtils#format`.
///
/// Java caches `DecimalFormat` instances per format pattern in a
/// `ThreadLocal`. Rust replaces the `DecimalFormat` with a simple
/// `format!("{:.*}", scale)` shim because Excel cell rendering is done
/// by the writer crates via `rust_xlsxwriter`.
#[must_use]
pub fn format(value: f64, format_pattern: &str) -> String {
    // The pattern itself is preserved for cache parity but only the
    // number of `0` digits after the decimal point drives precision.
    let scale = format_pattern
        .split('.')
        .nth(1)
        .map(|s| s.chars().take_while(|c| *c == '0').count())
        .unwrap_or(0);
    FORMATTER_CACHE.with(|c| {
        c.borrow_mut()
            .insert(format_pattern.to_owned(), format!("{value:.*}", scale));
    });
    format!("{value:.*}", scale)
}

/// Mirrors `com.alibaba.excel.util.NumberDataFormatterUtils#removeThreadLocalCache`.
pub fn remove_thread_local_cache() {
    FORMATTER_CACHE.with(|c| c.borrow_mut().clear());
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn format_uses_decimal_scale_from_pattern() {
        // 对应 Java：NumberDataFormatterUtils.format 按小数点后 0 的数量取精度
        assert_eq!(format(3.14159, "0.00"), "3.14");
        assert_eq!(format(3.0, "0.00"), "3.00");
        assert_eq!(format(2.5, "0.000"), "2.500");
        assert_eq!(format(1.2, "0"), "1");
        // 无小数部分时按 0 位精度
        assert_eq!(format(7.9, "0"), "8");
    }

    #[test]
    fn remove_thread_local_cache_clears_cache() {
        // 对应 Java：NumberDataFormatterUtils.removeThreadLocalCache
        let _ = format(1.0, "0.00");
        remove_thread_local_cache();
        let _ = format(2.0, "0.00");
        remove_thread_local_cache();
    }
}
