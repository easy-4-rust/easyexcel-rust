//! 对应 Java：`com.alibaba.excel.metadata.format.DataFormatter`.
//!
//! Java's 874-line class formats Excel numbers and dates using POI's
//! internal format engine. Rust delegates to the `ssfmt` crate at the
//! reader call site (`easyexcel-reader`), then applies
//! [`java_compat_format_code`] + [`java_compat_display`] so STRING mode
//! matches `EasyExcel` / POI.

use ssfmt::{DateSystem, FormatOptions, Locale, NumberFormat, format};

/// Strip orphan decimal points left by optional `#` fraction digits.
///
/// ssfmt/SSF keeps a trailing `.` for `#.##` / `#.##%` when the fractional
/// part is empty (`9999.%`). POI / `EasyExcel` STRING mode drops it (`9999%`).
///
/// Does **not** trim whitespace: Excel format codes may emit intentional
/// trailing spaces (e.g. negative section `\-0.00\ ` → `-1.07 `).
///
/// Currency glyphs (`￥` U+FFE5 vs `¥` U+00A5) are left as emitted by the
/// format code / BIFF string — callers must decode FORMAT records as Latin-1
/// compressed Unicode, not UTF-8, so `0xA5` stays `¥`.
#[must_use]
pub fn java_compat_display(value: &str) -> String {
    let mut out = value.replace(".%", "%");
    if out.ends_with('.') {
        out.pop();
    }
    out
}

/// Rewrite Excel date format codes so ssfmt matches POI / `EasyExcel` STRING.
///
/// - CN literal `上午/下午` → `AM/PM` token (locale supplies `上午`/`下午`).
/// - `mmmmm` (first-letter month) → POI private-use wrap around short month:
///   `"\u{E001}"mmm"\u{E002}"` (e.g. `\u{E001}1月\u{E002}`).
///
/// Does not alter quoted literals beyond the explicit mappings above, and does
/// **not** trim or rewrite currency symbols.
#[must_use]
pub fn java_compat_date_format_code(format_str: &str) -> String {
    // CN AM/PM is a literal slash-pair in BuiltinFormats / custom codes; ssfmt
    // only treats the ASCII `AM/PM` token as a day-period field.
    let with_ampm = format_str.replace("上午/下午", "AM/PM");
    // Replace longest `mmmmm` runs first so we never leave a bare `mmm` behind
    // from a partial match. POI wraps the short month with U+E001 / U+E002.
    with_ampm.replace("mmmmm", "\"\u{E001}\"mmm\"\u{E002}\"")
}

/// Clean a numeric format code the way `EasyExcel`
/// `DataFormatter.cleanFormatForNumber` does before `DecimalFormat`.
///
/// - Drop `_X` / `*X` alignment pads (ssfmt would otherwise emit a space;
///   POI / `EasyExcel` do not for STRING).
/// - Unescape `\` / `"` so literal spaces like `\ ` survive as trailing
///   spaces on negative accounting formats (`-1.07 `).
///
/// Date formats should **not** go through this helper (`EasyExcel` keeps
/// them on the `CellFormat` path). Callers must gate on date vs number.
#[must_use]
pub fn java_compat_format_code(format_str: &str) -> String {
    let mut sb: Vec<char> = format_str.chars().collect();

    // Pass 1: remove `_` / `*` spacers and the following pad character.
    let mut i = 0usize;
    while i < sb.len() {
        let c = sb[i];
        if (c == '_' || c == '*') && !(i > 0 && sb[i - 1] == '\\') {
            if i + 1 < sb.len() {
                sb.remove(i + 1);
            }
            sb.remove(i);
            continue;
        }
        i += 1;
    }

    // Pass 2: drop quotes / backslashes; strip `+` after `E` (engineering).
    let mut i = 0usize;
    while i < sb.len() {
        let c = sb[i];
        if c == '\\' || c == '"' {
            sb.remove(i);
            continue;
        }
        if c == '+' && i > 0 && sb[i - 1] == 'E' {
            sb.remove(i);
            continue;
        }
        i += 1;
    }

    sb.into_iter().collect()
}

/// 使用 Excel 格式代码和 locale 渲染数字，并应用 POI/EasyExcel 兼容清理。
#[must_use]
pub fn format_with_code(
    value: f64,
    code: &str,
    date_1904: bool,
    locale: &Locale,
) -> Option<String> {
    let is_date = NumberFormat::parse(code)
        .ok()
        .is_some_and(|parsed| parsed.is_date_format());
    let resolved = if is_date {
        java_compat_date_format_code(code)
    } else {
        java_compat_format_code(code)
    };
    let options = FormatOptions {
        date_system: if date_1904 {
            DateSystem::Date1904
        } else {
            DateSystem::Date1900
        },
        locale: locale.clone(),
    };
    format(value, &resolved, &options)
        .ok()
        .map(|formatted| java_compat_display(&formatted))
}

/// 将浮点值约束为 Excel 显示使用的 14 位有效精度。
#[must_use]
pub fn excel_display_number(value: f64) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    format!("{value:.14e}").parse().unwrap_or(value)
}

/// 判断 General 格式是否进入 Java EasyExcel 的极值科学计数分支。
#[must_use]
pub fn is_scientific_magnitude(value: f64) -> bool {
    let absolute = value.abs();
    absolute >= 1E11 || (absolute <= 1E-10 && absolute > 0.0)
}

/// Java General 在禁用科学计数时对极值的整数显示。
#[must_use]
pub fn java_plain_extreme_format(value: f64) -> String {
    let rounded = value.round();
    if rounded == 0.0 {
        "0".to_owned()
    } else {
        format!("{rounded:.0}")
    }
}

/// Java General 科学计数显示，保留 locale 小数分隔符。
#[must_use]
pub fn java_scientific_format(value: f64, decimal_separator: char) -> String {
    let formatted = format!("{value:.5e}");
    let (mantissa, exponent) = formatted
        .split_once('e')
        .expect("Rust scientific formatting always contains an exponent");
    let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
    let exponent = exponent
        .parse::<i32>()
        .expect("Rust scientific formatting always emits a numeric exponent");
    let mantissa = if decimal_separator == '.' {
        mantissa.to_owned()
    } else {
        mantissa.replace('.', &decimal_separator.to_string())
    };
    format!("{mantissa}E{exponent}")
}

/// Formats a numeric value using a built-in or custom Excel format
/// code. (Java `DataFormatter.formatRawCellContents(...)`)
///
/// The real formatting happens in `easyexcel-reader` via `ssfmt`; this
/// stub exists for 1:1 Java API parity and applies only the POI-compatible
/// orphan-decimal cleanup when a pre-formatted string is supplied through
/// tests that call [`java_compat_display`] directly.
#[allow(dead_code)]
#[must_use]
pub fn format_raw_cell_contents(_value: f64, _format_code: &str) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_compat_display_strips_orphan_decimal_before_percent() {
        assert_eq!(java_compat_display("9999.%"), "9999%");
        assert_eq!(java_compat_display("9999."), "9999");
        // Intentional trailing space from format codes must be preserved.
        assert_eq!(java_compat_display("-1.07 "), "-1.07 ");
    }

    #[test]
    fn java_compat_format_code_strips_pads_keeps_literal_space() {
        // Positive `_ ` pad removed; negative `\ ` becomes trailing space.
        let cleaned = java_compat_format_code(r"0.00_ ;[Red]\-0.00\ ");
        assert_eq!(cleaned, "0.00;[Red]-0.00 ");
        // Accounting `_)` pad removed; `\(` / `\)` unescaped.
        let acct = java_compat_format_code(r"0.00_);[Red]\(0.00\)");
        assert_eq!(acct, "0.00;[Red](0.00)");
    }

    #[test]
    fn java_compat_date_format_code_rewrites_cn_ampm_and_mmmmm() {
        assert_eq!(
            java_compat_date_format_code(r#"[DBNum1]上午/下午h"时"mm"分""#),
            r#"[DBNum1]AM/PMh"时"mm"分""#
        );
        assert_eq!(
            java_compat_date_format_code("mmmmm/yy"),
            "\"\u{E001}\"mmm\"\u{E002}\"/yy"
        );
        // Trailing spaces in unrelated date codes must be preserved by callers;
        // this helper only rewrites the two known tokens.
        assert_eq!(
            java_compat_date_format_code(r#"yyyy"年"m"月" "#),
            r#"yyyy"年"m"月" "#
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn format_code_strips_star_pad_and_engineering_plus() {
        // 对应 Java：DataFormatter 数字格式清理
        assert_eq!(java_compat_format_code(r"* #,##0"), "#,##0");
        assert_eq!(java_compat_format_code(r"0.00E+00"), "0.00E00");
        // 转义的下划线不作为填充符移除（仅去除反斜杠）
        assert_eq!(java_compat_format_code(r"\_0"), "_0");
    }

    #[test]
    fn format_raw_cell_contents_stub_returns_none() {
        // 对应 Java：格式化由 easyexcel-reader 的 ssfmt 完成
        assert_eq!(format_raw_cell_contents(1.5, "0.00"), None);
    }
}
