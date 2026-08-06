//! 对应 Java：`com.alibaba.excel.util.StringUtils` 的可复用实现。

#![allow(dead_code)]

use std::borrow::Cow;

/// 对应 Java：com.alibaba.excel.util.StringUtils。 返回字符串的 UTF-8 字节长度；超过 `u16` 可表示范围时返回 `None`。
///
/// Java `EasyExcel` 的最长列宽策略使用 `String#getBytes().length`，门面可用
/// 该基础原语实现相同的有界长度计算而不自行处理整数收窄。
#[must_use]
pub fn utf8_byte_len_u16(value: &str) -> Option<u16> {
    u16::try_from(value.len()).ok()
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 按配置使用 Java `String#trim` 语义裁剪字符串两端字符。
///
/// 无论是否启用均返回原字符串的借用切片，避免写入热路径产生分配。
#[must_use]
pub fn maybe_trim(value: &str, enabled: bool) -> Cow<'_, str> {
    if enabled {
        Cow::Borrowed(java_trim(value))
    } else {
        Cow::Borrowed(value)
    }
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 按 Java `EasyExcel` `FieldUtils.resolveCglibFieldName` 规则规范字段名。
///
/// 长度小于两个字符，或前两个字符同为大写/同为非大写时保持不变；否则切换
/// 首字符大小写。未发生变化时返回借用，避免分配。
#[must_use]
pub fn resolve_cglib_field_name(value: &str) -> Cow<'_, str> {
    let mut characters = value.char_indices();
    let Some((_, first)) = characters.next() else {
        return Cow::Borrowed(value);
    };
    let Some((second_index, second)) = characters.next() else {
        return Cow::Borrowed(value);
    };
    if first.is_uppercase() == second.is_uppercase() {
        return Cow::Borrowed(value);
    }

    let replacement = if first.is_uppercase() {
        first.to_lowercase().next().unwrap_or(first)
    } else {
        first.to_uppercase().next().unwrap_or(first)
    };
    let mut resolved = String::with_capacity(value.len());
    resolved.push(replacement);
    resolved.push_str(&value[second_index..]);
    Cow::Owned(resolved)
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 按 Java `String#trim` 语义移除两端不大于 U+0020 的字符。
///
/// Rust `str::trim` 会处理更多 Unicode 空白；Excel sheet 名、表头和兼容
/// 配置需要保持 Java `EasyExcel` 的原始行为。
#[must_use]
pub fn java_trim(value: &str) -> &str {
    value.trim_matches(|character| character <= '\u{20}')
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 按配置应用 Java trim 后比较两个字符串。
#[must_use]
pub fn equals_with_optional_java_trim(left: &str, right: &str, enabled: bool) -> bool {
    if enabled {
        java_trim(left) == java_trim(right)
    } else {
        left == right
    }
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 Mirrors `org.apache.commons.lang3.StringUtils#isEmpty`.
#[must_use]
pub fn is_empty(cs: Option<&str>) -> bool {
    match cs {
        Some(s) => s.is_empty(),
        None => true,
    }
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 Mirrors `org.apache.commons.lang3.StringUtils#isBlank`.
#[must_use]
pub fn is_blank(cs: Option<&str>) -> bool {
    match cs {
        Some(s) => s.trim().is_empty(),
        None => true,
    }
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 Mirrors `org.apache.commons.lang3.StringUtils#isNotBlank`.
#[must_use]
pub fn is_not_blank(cs: Option<&str>) -> bool {
    !is_blank(cs)
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 Mirrors `org.apache.commons.lang3.StringUtils#equals`.
#[must_use]
pub fn equals(cs1: Option<&str>, cs2: Option<&str>) -> bool {
    cs1 == cs2
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 Mirrors `java.lang.String#regionMatches(boolean, int, String, int, int)`.
#[must_use]
pub fn region_matches(
    ignore_case: bool,
    this_str: &str,
    this_offset: usize,
    other: &str,
    other_offset: usize,
    len: usize,
) -> bool {
    let this_chars: Vec<char> = this_str.chars().collect();
    let other_chars: Vec<char> = other.chars().collect();
    if this_offset + len > this_chars.len() || other_offset + len > other_chars.len() {
        return false;
    }
    for i in 0..len {
        let a = this_chars[this_offset + i];
        let b = other_chars[other_offset + i];
        if a == b {
            continue;
        }
        if !ignore_case {
            return false;
        }
        if !a.eq_ignore_ascii_case(&b) {
            return false;
        }
    }
    true
}

/// 对应 Java：com.alibaba.excel.util.StringUtils。 Mirrors `org.apache.commons.lang3.StringUtils#isNumeric`.
#[must_use]
pub fn is_numeric(cs: Option<&str>) -> bool {
    let s = match cs {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    s.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn is_empty_handles_null_and_empty() {
        // 对应 Java：StringUtils.isEmpty(null/""/"x")
        assert!(is_empty(None));
        assert!(is_empty(Some("")));
        assert!(!is_empty(Some("x")));
        assert!(!is_empty(Some(" ")));
    }

    #[test]
    fn is_blank_handles_null_blank_and_content() {
        // 对应 Java：StringUtils.isBlank(null/"  "/"x")
        assert!(is_blank(None));
        assert!(is_blank(Some("")));
        assert!(is_blank(Some(" \t\n")));
        assert!(!is_blank(Some("x")));
    }

    #[test]
    fn is_not_blank_inverts_is_blank() {
        // 对应 Java：StringUtils.isNotBlank
        assert!(!is_not_blank(None));
        assert!(!is_not_blank(Some("  ")));
        assert!(is_not_blank(Some("x")));
    }

    #[test]
    fn equals_compares_option_strings() {
        // 对应 Java：StringUtils.equals
        assert!(equals(Some("a"), Some("a")));
        assert!(!equals(Some("a"), Some("b")));
        assert!(equals(None, None));
        assert!(!equals(Some("a"), None));
        assert!(!equals(None, Some("a")));
    }

    #[test]
    fn region_matches_exact_and_ignore_case() {
        // 对应 Java：String.regionMatches
        assert!(region_matches(false, "hello", 0, "hell", 0, 4));
        assert!(region_matches(true, "Hello", 0, "hello", 0, 5));
        assert!(region_matches(false, "Hello", 0, "Hello", 0, 5));
        assert!(!region_matches(false, "Hello", 0, "hello", 0, 5));
    }

    #[test]
    fn region_matches_out_of_bounds_returns_false() {
        // 对应 Java：越界返回 false
        assert!(!region_matches(false, "abc", 2, "abc", 0, 2));
        assert!(!region_matches(false, "abc", 0, "abc", 2, 2));
        assert!(!region_matches(false, "abc", 0, "ab", 0, 3));
        // 空串与 len=0 的边界
        assert!(region_matches(false, "", 0, "", 0, 0));
    }

    #[test]
    fn region_matches_ascii_case_fold_only() {
        // 对应 Java：toLowerCase 比较，非 ASCII 大小写不折叠
        assert!(region_matches(true, "ABC", 1, "bc", 0, 2));
        assert!(!region_matches(true, "A", 0, "a", 1, 1));
    }

    #[test]
    fn is_numeric_accepts_digits_only() {
        // 对应 Java：StringUtils.isNumeric
        assert!(is_numeric(Some("123")));
        assert!(!is_numeric(Some("12a")));
        assert!(!is_numeric(Some("12.3")));
        assert!(!is_numeric(Some("-1")));
        assert!(!is_numeric(Some("")));
        assert!(!is_numeric(None));
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    #[test]
    fn region_matches_ignore_case_reports_lowercase_mismatch() {
        // 对应 Java：忽略大小写时仅 ASCII 大小写折叠，其余字符不同则返回 false
        assert!(region_matches(true, "AbC", 0, "aBc", 0, 3));
        assert!(!region_matches(true, "AbC", 0, "Axy", 0, 3));
        assert!(!region_matches(true, "Ä", 0, "ä", 0, 1));
    }
}
