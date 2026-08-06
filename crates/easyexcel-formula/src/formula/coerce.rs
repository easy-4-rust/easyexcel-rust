//! Type-coercion helpers shared by the operators and the function library.
//! These implement Excel's coercion rules: booleans ↔ numbers, numeric text ↔
//! numbers, empty → 0/"", and the cross-type comparison ordering.

use super::value::Value;
use easyexcel_model::error::CellError;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Coerce a value to a number following Excel's rules for scalar contexts:
/// numbers pass through, booleans → 1/0, empty → 0, numeric text parses, other
/// text → `#VALUE!`, errors propagate.
///
/// # Errors
///
/// 文本无法解析为数字，或输入本身是错误、引用、lambda 时返回对应单元格错误。
pub fn to_number(v: &Value) -> Result<f64, CellError> {
    match v {
        Value::Number(n) => Ok(*n),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::Empty => Ok(0.0),
        Value::Text(s) => parse_number_text(s).ok_or(CellError::Value),
        Value::Error(e) => Err(*e),
        Value::Array(a) => match a.data.first() {
            Some(first) => to_number(first),
            None => Err(CellError::Value),
        },
        Value::Ref(_) | Value::Lambda(_) => Err(CellError::Value),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse text the way Excel coerces a string operand to a number: decimal,
/// scientific, leading/trailing whitespace, leading `+`/`-`, and percentages.
#[must_use]
pub fn parse_number_text(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if let Some(stripped) = t.strip_suffix('%') {
        return stripped.trim().parse::<f64>().ok().map(|n| n / 100.0);
    }
    // Plain float (handles 1e6, -3.5, +2, etc.)
    if let Ok(n) = t.parse::<f64>() {
        return Some(n);
    }
    // Thousands-separated integers like "1,234"
    if t.contains(',') {
        let cleaned: String = t.chars().filter(|c| *c != ',').collect();
        if let Ok(n) = cleaned.parse::<f64>() {
            return Some(n);
        }
    }
    None
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Coerce to display text (numbers via General format, booleans uppercase).
///
/// # Errors
///
/// 输入是错误、引用或 lambda 时返回对应单元格错误。
pub fn to_text(v: &Value) -> Result<String, CellError> {
    match v {
        Value::Text(s) => Ok(s.clone()),
        Value::Number(n) => Ok(easyexcel_model::value::format_number_general(*n)),
        Value::Bool(b) => Ok(if *b { "TRUE".into() } else { "FALSE".into() }),
        Value::Empty => Ok(String::new()),
        Value::Error(e) => Err(*e),
        Value::Array(a) => match a.data.first() {
            Some(first) => to_text(first),
            None => Ok(String::new()),
        },
        Value::Ref(_) | Value::Lambda(_) => Err(CellError::Value),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Coerce to a boolean: booleans pass, numbers → nonzero, "TRUE"/"FALSE" text,
/// empty → false, other text → `#VALUE!`.
///
/// # Errors
///
/// 文本无法按布尔或数字规则解析，或输入本身是错误、引用、lambda 时返回错误。
pub fn to_bool(v: &Value) -> Result<bool, CellError> {
    match v {
        Value::Bool(b) => Ok(*b),
        Value::Number(n) => Ok(*n != 0.0),
        Value::Empty => Ok(false),
        Value::Text(s) => {
            if s.eq_ignore_ascii_case("true") {
                Ok(true)
            } else if s.eq_ignore_ascii_case("false") {
                Ok(false)
            } else if let Some(n) = parse_number_text(s) {
                Ok(n != 0.0)
            } else {
                Err(CellError::Value)
            }
        }
        Value::Error(e) => Err(*e),
        Value::Array(a) => match a.data.first() {
            Some(first) => to_bool(first),
            None => Err(CellError::Value),
        },
        Value::Ref(_) | Value::Lambda(_) => Err(CellError::Value),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Excel comparison ordering across mixed types. Returns `Less`/`Equal`/`Greater`.
///
/// Within a type: numbers compare numerically, text compares
/// case-insensitively, booleans FALSE < TRUE. Across types the hierarchy is
/// number < text < boolean (empty is treated as 0 or "" depending on the other
/// operand).
#[must_use]
pub fn compare(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    fn rank(v: &Value) -> u8 {
        match v {
            Value::Number(_) | Value::Empty => 0,
            Value::Text(_) => 1,
            Value::Bool(_) => 2,
            _ => 3,
        }
    }
    // Normalize empty against the other operand's type.
    let (a, b) = (normalize_empty(a, b), normalize_empty(b, a));
    let (ra, rb) = (rank(&a), rank(&b));
    if ra != rb {
        return ra.cmp(&rb);
    }
    match (&a, &b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
        (Value::Text(x), Value::Text(y)) => x.to_lowercase().cmp(&y.to_lowercase()),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => Ordering::Equal,
    }
}

fn normalize_empty(v: &Value, other: &Value) -> Value {
    match v {
        Value::Empty => match other {
            Value::Text(_) => Value::Text(String::new()),
            Value::Bool(_) => Value::Bool(false),
            _ => Value::Number(0.0),
        },
        _ => v.clone(),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Excel equality (used by `=`/`<>` and by criteria matching). Numeric-text is
/// NOT coerced here — Excel treats `"1"=1` as FALSE.
#[must_use]
pub fn equal(a: &Value, b: &Value) -> bool {
    compare(a, b) == std::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_text() {
        assert_eq!(to_number(&Value::Bool(true)), Ok(1.0));
        assert_eq!(to_number(&Value::Empty), Ok(0.0));
        assert_eq!(to_number(&Value::Text("3.5".into())), Ok(3.5));
        assert_eq!(to_number(&Value::Text("50%".into())), Ok(0.5));
        assert_eq!(to_number(&Value::Text("x".into())), Err(CellError::Value));
    }

    #[test]
    fn comparisons() {
        use std::cmp::Ordering::*;
        assert_eq!(compare(&Value::Number(1.0), &Value::Number(2.0)), Less);
        assert_eq!(
            compare(&Value::Text("abc".into()), &Value::Text("ABC".into())),
            Equal
        );
        // number < text < bool
        assert_eq!(compare(&Value::Number(9.0), &Value::Text("a".into())), Less);
        assert_eq!(compare(&Value::Text("z".into()), &Value::Bool(false)), Less);
    }
}
