//! Scalar cell values — the result type for evaluated cells and formula caches.

use super::error::CellError;

/// A scalar value that can live in a cell or be the cached result of a formula.
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
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            CellValue::Empty => Some(0.0),
            _ => None,
        }
    }

    /// Plain, unformatted text rendering (no number-format applied).
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

/// Render a number the way Excel's "General" format does: shortest round-trip
/// representation, no trailing zeros, integers without a decimal point.
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

/// 按 Excel 标量强制转换规则解析数字文本。
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
        assert_eq!(format_number_general(1000000.0), "1000000");
    }

    #[test]
    fn coercion() {
        assert_eq!(CellValue::Bool(true).as_number(), Some(1.0));
        assert_eq!(CellValue::Empty.as_number(), Some(0.0));
        assert_eq!(CellValue::Text("x".into()).as_number(), None);
    }
}
