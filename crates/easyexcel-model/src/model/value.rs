//! Scalar cell values — the result type for evaluated cells and formula caches.

use super::error::CellError;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 A scalar value that can live in a cell or be the cached result of a formula.
///
/// This is deliberately *scalar*: the formula engine has a richer internal
/// `Value` type (in `core::formula`) that additionally models arrays and range
/// references. Anything stored on a [`crate::model::Cell`] reduces to one
/// of these.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CellValue {
    /// A genuinely empty cell.
    #[default]
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
    Error(CellError),
}

impl CellValue {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn as_number(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            CellValue::Empty => Some(0.0),
            _ => None,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Plain, unformatted text rendering (no number-format applied).
    #[must_use]
    pub fn to_display_string(&self) -> String {
        match self {
            CellValue::Empty => String::new(),
            CellValue::Number(n) => format_number_general(*n),
            CellValue::Text(s) => s.clone(),
            CellValue::Bool(b) => {
                if *b {
                    "TRUE".into()
                } else {
                    "FALSE".into()
                }
            }
            CellValue::Error(e) => e.as_str().to_string(),
        }
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Render a number the way Excel's "General" format does: shortest round-trip
/// representation, no trailing zeros, integers without a decimal point.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn format_number_general(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    if !n.is_finite() {
        return if n.is_nan() {
            "NaN".into()
        } else if n > 0.0 {
            "Inf".into()
        } else {
            "-Inf".into()
        };
    }
    if n.fract() == 0.0 && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    // Use Rust's shortest float formatting, which round-trips.
    let s = format!("{n}");
    s
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按 Excel 标量强制转换规则解析数字文本。
///
/// 支持空白、正负号、科学计数法、百分比和千位分隔符。
#[must_use]
pub fn parse_number_text(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(percent) = trimmed.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f64>()
            .ok()
            .map(|number| number / 100.0);
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        return Some(number);
    }
    if trimmed.contains(',') {
        let normalized: String = trimmed
            .chars()
            .filter(|character| *character != ',')
            .collect();
        return normalized.parse::<f64>().ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_number_format() {
        assert_eq!(format_number_general(0.0), "0");
        assert_eq!(format_number_general(1.0), "1");
        assert_eq!(format_number_general(1.5), "1.5");
        assert_eq!(format_number_general(-42.0), "-42");
        assert_eq!(format_number_general(1_000_000.0), "1000000");
    }

    #[test]
    fn coercion() {
        assert_eq!(CellValue::Bool(true).as_number(), Some(1.0));
        assert_eq!(CellValue::Empty.as_number(), Some(0.0));
        assert_eq!(CellValue::Text("x".into()).as_number(), None);
    }

    #[test]
    fn format_number_general_non_finite() {
        assert_eq!(format_number_general(f64::NAN), "NaN");
        assert_eq!(format_number_general(f64::INFINITY), "Inf");
        assert_eq!(format_number_general(f64::NEG_INFINITY), "-Inf");
    }

    #[test]
    fn format_number_general_large_integer() {
        // Numbers >= 1e15 don't use i64 formatting
        let result = format_number_general(1e15);
        assert!(!result.is_empty());
    }

    #[test]
    fn cell_value_is_empty() {
        assert!(CellValue::Empty.is_empty());
        assert!(!CellValue::Number(0.0).is_empty());
        assert!(!CellValue::Text("".into()).is_empty());
        assert!(!CellValue::Bool(false).is_empty());
    }

    #[test]
    fn cell_value_as_number_for_number() {
        assert_eq!(CellValue::Number(42.5).as_number(), Some(42.5));
    }

    #[test]
    fn cell_value_to_display_string() {
        assert_eq!(CellValue::Empty.to_display_string(), "");
        assert_eq!(CellValue::Number(42.0).to_display_string(), "42");
        assert_eq!(CellValue::Text("hello".into()).to_display_string(), "hello");
        assert_eq!(CellValue::Bool(true).to_display_string(), "TRUE");
        assert_eq!(CellValue::Bool(false).to_display_string(), "FALSE");
    }

    #[test]
    fn cell_value_to_display_string_for_error() {
        let result = CellValue::Error(super::super::error::CellError::Value).to_display_string();
        assert!(!result.is_empty());
    }

    #[test]
    fn cell_value_default_is_empty() {
        assert_eq!(CellValue::default(), CellValue::Empty);
    }

    #[test]
    fn parse_number_text_basic() {
        assert_eq!(parse_number_text("42"), Some(42.0));
        assert_eq!(parse_number_text("3.14"), Some(3.14));
        assert_eq!(parse_number_text("-5"), Some(-5.0));
        assert_eq!(parse_number_text("1e3"), Some(1000.0));
    }

    #[test]
    fn parse_number_text_percent() {
        assert_eq!(parse_number_text("50%"), Some(0.5));
        assert_eq!(parse_number_text("100%"), Some(1.0));
        assert_eq!(parse_number_text(" 50% "), Some(0.5));
    }

    #[test]
    fn parse_number_text_with_commas() {
        assert_eq!(parse_number_text("1,234.5"), Some(1234.5));
        assert_eq!(parse_number_text("1,000,000"), Some(1000000.0));
    }

    #[test]
    fn parse_number_text_empty_and_whitespace() {
        assert_eq!(parse_number_text(""), None);
        assert_eq!(parse_number_text("   "), None);
    }

    #[test]
    fn parse_number_text_trim() {
        assert_eq!(parse_number_text("  42  "), Some(42.0));
    }

    #[test]
    fn parse_number_text_invalid() {
        assert_eq!(parse_number_text("abc"), None);
    }
}
