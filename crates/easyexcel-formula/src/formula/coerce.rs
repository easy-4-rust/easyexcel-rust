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

    // ── to_number 扩展测试 ──────────────────────────────────────────────

    #[test]
    fn to_number_error_propagates() {
        assert_eq!(
            to_number(&Value::Error(CellError::NA)),
            Err(CellError::NA)
        );
        assert_eq!(
            to_number(&Value::Error(CellError::Ref)),
            Err(CellError::Ref)
        );
    }

    #[test]
    fn to_number_ref_and_lambda_errors() {
        // Ref 和 Lambda 应返回 Value 错误
        use crate::formula::value::RefRange;
        assert_eq!(
            to_number(&Value::Ref(RefRange {
                sheet: 0,
                start_row: 0,
                start_col: 0,
                end_row: 1,
                end_col: 1
            })),
            Err(CellError::Value)
        );
    }

    #[test]
    fn to_number_array_first_element() {
        use crate::formula::value::Array;
        // 空数组 → Value 错误
        assert_eq!(
            to_number(&Value::Array(Array::new(0, 0, vec![]))),
            Err(CellError::Value)
        );
        // 非空数组取第一个元素
        assert_eq!(
            to_number(&Value::Array(Array::new(
                1,
                2,
                vec![Value::Number(7.0), Value::Number(8.0)]
            ))),
            Ok(7.0)
        );
    }

    // ── to_text 测试 ────────────────────────────────────────────────────

    #[test]
    fn to_text_numbers_and_bools() {
        assert_eq!(to_text(&Value::Text("hi".into())), Ok("hi".into()));
        assert_eq!(to_text(&Value::Bool(true)), Ok("TRUE".into()));
        assert_eq!(to_text(&Value::Bool(false)), Ok("FALSE".into()));
        assert_eq!(to_text(&Value::Empty), Ok(String::new()));
    }

    #[test]
    fn to_text_error_propagates() {
        assert_eq!(to_text(&Value::Error(CellError::NA)), Err(CellError::NA));
    }

    #[test]
    fn to_text_array_first_element() {
        use crate::formula::value::Array;
        assert_eq!(
            to_text(&Value::Array(Array::new(
                1,
                1,
                vec![Value::Text("hello".into())]
            ))),
            Ok("hello".into())
        );
        assert_eq!(
            to_text(&Value::Array(Array::new(0, 0, vec![]))),
            Ok(String::new())
        );
    }

    #[test]
    fn to_text_ref_errors() {
        use crate::formula::value::RefRange;
        assert_eq!(
            to_text(&Value::Ref(RefRange {
                sheet: 0,
                start_row: 0,
                start_col: 0,
                end_row: 0,
                end_col: 0
            })),
            Err(CellError::Value)
        );
    }

    // ── to_bool 测试 ────────────────────────────────────────────────────

    #[test]
    fn to_bool_numbers() {
        assert_eq!(to_bool(&Value::Number(0.0)), Ok(false));
        assert_eq!(to_bool(&Value::Number(1.0)), Ok(true));
        assert_eq!(to_bool(&Value::Number(-1.0)), Ok(true));
        assert_eq!(to_bool(&Value::Number(0.001)), Ok(true));
    }

    #[test]
    fn to_bool_text_true_false() {
        assert_eq!(to_bool(&Value::Text("TRUE".into())), Ok(true));
        assert_eq!(to_bool(&Value::Text("true".into())), Ok(true));
        assert_eq!(to_bool(&Value::Text("FALSE".into())), Ok(false));
        assert_eq!(to_bool(&Value::Text("false".into())), Ok(false));
    }

    #[test]
    fn to_bool_text_numeric() {
        assert_eq!(to_bool(&Value::Text("1".into())), Ok(true));
        assert_eq!(to_bool(&Value::Text("0".into())), Ok(false));
        assert_eq!(to_bool(&Value::Text("42".into())), Ok(true));
    }

    #[test]
    fn to_bool_text_non_boolean() {
        assert_eq!(to_bool(&Value::Text("hello".into())), Err(CellError::Value));
    }

    #[test]
    fn to_bool_empty() {
        assert_eq!(to_bool(&Value::Empty), Ok(false));
    }

    #[test]
    fn to_bool_error_propagates() {
        assert_eq!(to_bool(&Value::Error(CellError::Num)), Err(CellError::Num));
    }

    #[test]
    fn to_bool_array_first_element() {
        use crate::formula::value::Array;
        assert_eq!(
            to_bool(&Value::Array(Array::new(1, 1, vec![Value::Bool(true)]))),
            Ok(true)
        );
        assert_eq!(
            to_bool(&Value::Array(Array::new(0, 0, vec![]))),
            Err(CellError::Value)
        );
    }

    #[test]
    fn to_bool_ref_and_lambda_errors() {
        use crate::formula::value::RefRange;
        assert_eq!(
            to_bool(&Value::Ref(RefRange {
                sheet: 0,
                start_row: 0,
                start_col: 0,
                end_row: 0,
                end_col: 0
            })),
            Err(CellError::Value)
        );
    }

    // ── parse_number_text 扩展测试 ──────────────────────────────────────

    #[test]
    fn parse_number_text_percentages() {
        assert_eq!(parse_number_text("50%"), Some(0.5));
        assert_eq!(parse_number_text("100%"), Some(1.0));
        assert_eq!(parse_number_text("0.5%"), Some(0.005));
        assert_eq!(parse_number_text("  50%  "), Some(0.5));
    }

    #[test]
    fn parse_number_text_scientific() {
        assert_eq!(parse_number_text("1e6"), Some(1_000_000.0));
        assert_eq!(parse_number_text("1.5E3"), Some(1500.0));
        assert_eq!(parse_number_text("-2.5e2"), Some(-250.0));
    }

    #[test]
    fn parse_number_text_comma_separated() {
        assert_eq!(parse_number_text("1,234"), Some(1234.0));
        assert_eq!(parse_number_text("1,000,000"), Some(1_000_000.0));
    }

    #[test]
    fn parse_number_text_whitespace() {
        assert_eq!(parse_number_text("  42  "), Some(42.0));
        assert_eq!(parse_number_text("  +3.5  "), Some(3.5));
    }

    #[test]
    fn parse_number_text_non_numeric() {
        assert_eq!(parse_number_text("hello"), None);
        assert_eq!(parse_number_text(""), None);
        assert_eq!(parse_number_text("   "), None);
    }

    // ── compare 扩展测试 ────────────────────────────────────────────────

    #[test]
    fn compare_equal_numbers() {
        assert_eq!(
            compare(&Value::Number(5.0), &Value::Number(5.0)),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn compare_booleans() {
        use std::cmp::Ordering::*;
        assert_eq!(compare(&Value::Bool(false), &Value::Bool(true)), Less);
        assert_eq!(compare(&Value::Bool(true), &Value::Bool(true)), Equal);
        assert_eq!(compare(&Value::Bool(true), &Value::Bool(false)), Greater);
    }

    #[test]
    fn compare_empty_as_number() {
        assert_eq!(
            compare(&Value::Empty, &Value::Number(0.0)),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn compare_empty_as_text() {
        assert_eq!(
            compare(&Value::Empty, &Value::Text("".into())),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn compare_empty_as_bool() {
        assert_eq!(
            compare(&Value::Empty, &Value::Bool(false)),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn compare_error_vs_anything() {
        // Error 类型 rank=3，高于 Bool(rank=2)
        use std::cmp::Ordering::*;
        assert_eq!(
            compare(&Value::Bool(true), &Value::Error(CellError::NA)),
            Less
        );
    }

    #[test]
    fn compare_nan_no_panic() {
        // Agent 68 回归：NaN 比较不应 panic
        let ord = compare(&Value::Number(f64::NAN), &Value::Number(1.0));
        assert_eq!(ord, std::cmp::Ordering::Equal); // partial_cmp(NaN, _) → None → Equal
    }

    // ── equal 测试 ──────────────────────────────────────────────────────

    #[test]
    fn equal_basic() {
        assert!(equal(&Value::Number(5.0), &Value::Number(5.0)));
        assert!(!equal(&Value::Number(5.0), &Value::Number(6.0)));
        assert!(equal(&Value::Text("abc".into()), &Value::Text("ABC".into())));
        assert!(!equal(&Value::Number(1.0), &Value::Text("1".into())));
    }
}
