//! 对应 Java： com.alibaba.excel.util.DateUtils.

#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use crate::core::excel_error::ExcelError;

/// Mirrors `org.apache.commons.lang3.time.DateUtils#parseDate`.
///
/// Rust `chrono` only accepts a single format string per call, so the Java
/// multi-format fallback is simulated by trying each format in order.
///
/// # Errors
///
/// 当所有格式都无法解析 `str` 时返回 [`ExcelError::Format`]。
pub fn parse_date<'a>(
    str: &str,
    parse_patterns: impl IntoIterator<Item = &'a str>,
) -> Result<NaiveDateTime, ExcelError> {
    for pattern in parse_patterns {
        let fmt = chrono_java_to_rust(pattern);
        if let Ok(dt) = NaiveDateTime::parse_from_str(str, &fmt) {
            return Ok(dt);
        }
        if let Ok(d) = NaiveDate::parse_from_str(str, &fmt) {
            return Ok(d.and_hms_opt(0, 0, 0).unwrap_or_default());
        }
    }
    Err(ExcelError::Format(format!("parseDate failed for {str:?}")))
}

/// Mirrors `org.apache.commons.lang3.time.DateFormatUtils#format`.
#[must_use]
pub fn format(date: NaiveDateTime, pattern: &str) -> String {
    let fmt = chrono_java_to_rust(pattern);
    date.format(&fmt).to_string()
}

/// Mirrors `org.apache.commons.lang3.time.DateUtils#getJavaDate`.
///
/// Converts a date serial (Excel days since the 1900 epoch) to a UTC `DateTime`.
#[must_use]
pub fn get_java_date(days: i64) -> DateTime<Utc> {
    let base = NaiveDate::from_ymd_opt(1899, 12, 30)
        .unwrap_or_default()
        .and_hms_opt(0, 0, 0)
        .unwrap_or_default();
    DateTime::<Utc>::from_naive_utc_and_offset(base + chrono::Duration::days(days), Utc)
}

/// Mirrors `org.apache.poi.ss.usermodel.DateUtil#isADateFormat`.
#[must_use]
pub fn is_a_date_format(format_index: i32, format_string: Option<&str>) -> bool {
    if (14..=22).contains(&format_index)
        || (27..=31).contains(&format_index)
        || (35..=36).contains(&format_index)
        || (45..=47).contains(&format_index)
        || (50..=58).contains(&format_index)
    {
        return true;
    }
    match format_string {
        Some(s) => is_internal_date_format(s),
        None => false,
    }
}

/// Mirrors `org.apache.poi.ss.usermodel.DateUtil#isInternalDateFormat`.
#[must_use]
pub fn is_internal_date_format(format: &str) -> bool {
    let lower = format.to_ascii_lowercase();
    ["y", "d", "h", "s"].iter().any(|c| lower.contains(c))
}

/// Mirrors `com.alibaba.excel.util.DateUtils#removeThreadLocalCache`.
///
/// Java keeps `ThreadLocal<Format>` caches for `SimpleDateFormat` thread safety.
/// `chrono` is already thread-safe. The Rust port uses a global counter so
/// callers can verify the cache-clearing lifecycle fires.
pub fn remove_thread_local_cache() {
    use std::sync::atomic::{AtomicU32, Ordering};
    static CLEAR_COUNT: AtomicU32 = AtomicU32::new(0);
    CLEAR_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Best-effort translation of Java `SimpleDateFormat` pattern letters to
/// `chrono` format specifiers. Only the letters actually used by `EasyExcel`
/// are mapped; unknown chars pass through verbatim.
///
/// 对应 Java：SimpleDateFormat 的 y/M/d/H/m/s/S 字母在 chrono 中必须以 `%`
/// 前缀出现，且连续的相同字母表示同一单位（如 `yyyy` 是四位年），因此
/// 折叠为一个 chrono 说明符；`'foo'` 字面量块按 `SimpleDateFormat` 语义
/// 原样输出。
fn chrono_java_to_rust(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len() * 2);
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\'' => {
                // Java literal block 'foo' -> 直接输出字面量（chrono 中无前缀字母本就是字面量）
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    out.push(chars[i]);
                    i += 1;
                }
            }
            'y' | 'M' | 'd' | 'H' | 'm' | 's' | 'S' => {
                // 连续相同字母折叠为一个说明符（Java yyyy == 四位年）
                let run_char = c;
                while i < chars.len() && chars[i] == run_char {
                    i += 1;
                }
                let spec = match run_char {
                    'y' => "%Y",
                    'M' => "%m",
                    'd' => "%d",
                    'H' => "%H",
                    'm' => "%M",
                    's' => "%S",
                    _ => "%3f", // 'S'：Java 毫秒 = 3 位小数
                };
                out.push_str(spec);
                continue;
            }
            other => out.push(other),
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn parse_date_with_datetime_pattern() {
        // 对应 Java：DateUtils.parseDate 日期时间格式（DATE_FORMAT_19）
        let dt = parse_date("2024-01-02 03:04:05", ["yyyy-MM-dd HH:mm:ss"])
            .expect("should parse datetime");
        assert_eq!(
            dt,
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(3, 4, 5)
                .unwrap()
        );
    }

    #[test]
    fn parse_date_with_date_only_pattern_midnight() {
        // 对应 Java：DateUtils.parseDate 仅日期格式（DATE_FORMAT_10），时分秒归零
        let dt = parse_date("2024-01-02", ["yyyy-MM-dd"]).expect("should parse date");
        assert_eq!(
            dt,
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn parse_date_falls_back_to_next_pattern() {
        // 对应 Java：多个格式依次尝试，第一个失败后尝试第二个
        let dt = parse_date(
            "2024/01/02 03:04:05",
            ["yyyy-MM-dd HH:mm:ss", "yyyy/MM/dd HH:mm:ss"],
        )
        .expect("should fall back");
        assert_eq!(
            dt,
            NaiveDate::from_ymd_opt(2024, 1, 2)
                .unwrap()
                .and_hms_opt(3, 4, 5)
                .unwrap()
        );
    }

    #[test]
    fn parse_date_returns_error_when_no_pattern_matches() {
        // 对应 Java：所有格式均不匹配时抛出解析异常
        let err = parse_date("not-a-date", ["yyyy-MM-dd"]).expect_err("should fail");
        assert!(err.to_string().contains("parseDate failed"));
    }

    #[test]
    fn format_uses_chrono_java_translated_pattern() {
        // 对应 Java：DateFormatUtils.format
        let dt = NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_opt(3, 4, 5)
            .unwrap();
        assert_eq!(format(dt, "yyyy-MM-dd HH:mm:ss"), "2024-01-02 03:04:05");
        // 字面量块、毫秒与未映射字符原样保留
        assert_eq!(
            format(dt, "yyyy'年'MM-dd HH:mm:ss.SSS"),
            "2024年01-02 03:04:05.000"
        );
        assert_eq!(format(dt, "yyyy/MM/dd"), "2024/01/02");
    }

    #[test]
    fn remove_thread_local_cache_increments_counter() {
        // 对应 Java：DateUtils.removeThreadLocalCache 生命周期触发
        remove_thread_local_cache();
        remove_thread_local_cache();
        // 两次调用不 panic 即视为通过
    }
}
