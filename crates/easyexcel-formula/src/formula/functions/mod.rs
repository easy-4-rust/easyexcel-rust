//! The worksheet-function library and its dispatch registry.
//!
//! Each function is an eager implementation: by the time it runs, the evaluator
//! has already reduced its arguments to [`Value`]s (the few lazy "special forms"
//! — IF, IFERROR, IFNA, CHOOSE, IFS, SWITCH, AND, OR — are handled directly in
//! the evaluator). Functions reach the workbook through the [`Context`] trait.

use std::collections::HashMap;

use super::context::Context;
use super::value::Value;

pub mod database;
pub mod datetime;
pub mod dynamic;
pub mod engineering;
pub mod financial;
pub mod info;
pub mod logical;
pub mod lookup;
pub mod math;
pub mod stats;
pub mod stubs;
pub mod text;

#[cfg(test)]
pub mod testutil;

include!("mod/fn_impl.rs");

/// Sentinel for a variadic upper bound.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const VARIADIC: usize = usize::MAX;

include!("mod/fn_entry.rs");

include!("mod/registry.rs");

// ---------------------------------------------------------------------------
// Shared helpers for function implementations.
// ---------------------------------------------------------------------------

use easyexcel_model::error::CellError;

/// Propagate the first error found in `args`, if any (used by strict numeric fns).
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn first_error(args: &[Value]) -> Option<CellError> {
    for a in args {
        if let Value::Error(e) = a {
            return Some(*e);
        }
    }
    None
}

/// Collect all scalar numbers from the arguments, **ignoring** text/blank/bool
/// the way SUM/AVERAGE do for *range* contents, but coercing direct scalar
/// arguments. Errors short-circuit and are returned as `Err`.
///
/// `coerce_text` controls whether stray text that looks numeric is included
/// (true for literal arguments, false for range contents — matching Excel).
///
/// # Errors
///
/// 参数或展开后的范围中包含需要传播的 Excel 单元格错误时返回该错误。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn collect_numbers(
    ctx: &mut dyn Context,
    args: &[Value],
    include_bools_in_ranges: bool,
) -> Result<Vec<f64>, CellError> {
    let mut out = Vec::new();
    for arg in args {
        match arg {
            Value::Ref(_) | Value::Array(_) => {
                for v in ctx.flatten(arg) {
                    match v {
                        Value::Number(n) => out.push(n),
                        Value::Bool(b) if include_bools_in_ranges => {
                            out.push(if b { 1.0 } else { 0.0 });
                        }
                        Value::Error(e) => return Err(e),
                        // PARITY: Excel ignores *all* text in ranges. We coerce
                        // numeric-looking text (e.g. "6,000.00" stored as text by
                        // bank/CSV exports) so such cells still contribute to
                        // SUM/AVERAGE/… without mutating the stored data. Non-
                        // numeric text and blanks are still skipped.
                        Value::Text(s) => {
                            if let Some(n) = super::coerce::parse_number_text(&s) {
                                out.push(n);
                            }
                        }
                        _ => {} // blank / bool (when excluded) skipped
                    }
                }
            }
            Value::Number(n) => out.push(*n),
            Value::Bool(b) => out.push(if *b { 1.0 } else { 0.0 }),
            Value::Empty => {}
            Value::Error(e) => return Err(*e),
            Value::Text(s) => match super::coerce::parse_number_text(s) {
                Some(n) => out.push(n),
                None => return Err(CellError::Value),
            },
            Value::Lambda(_) => return Err(CellError::Value),
        }
    }
    Ok(out)
}

include!("mod/criteria.rs");

#[derive(PartialEq)]
enum CritOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

fn numeric_of(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => Some(*n),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Case-insensitive wildcard match (`*` = any run, `?` = one char). Both inputs
/// should already be lower-cased. `~` escapes a literal `*`/`?`.
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    fn rec(p: &[char], t: &[char]) -> bool {
        let mut pi = 0;
        let mut ti = 0;
        let mut star: Option<(usize, usize)> = None;
        while ti < t.len() {
            if pi < p.len() && p[pi] == '~' && pi + 1 < p.len() {
                // escaped literal
                if t[ti] == p[pi + 1] {
                    pi += 2;
                    ti += 1;
                    continue;
                } else if let Some((sp, st)) = star {
                    pi = sp + 1;
                    ti = st + 1;
                    star = Some((sp, st + 1));
                    continue;
                }
                return false;
            }
            if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
                pi += 1;
                ti += 1;
            } else if pi < p.len() && p[pi] == '*' {
                star = Some((pi, ti));
                pi += 1;
            } else if let Some((sp, st)) = star {
                pi = sp + 1;
                ti = st + 1;
                star = Some((sp, st + 1));
            } else {
                return false;
            }
        }
        while pi < p.len() && p[pi] == '*' {
            pi += 1;
        }
        pi == p.len()
    }
    rec(&p, &t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::value::Value;

    #[test]
    fn registry_builds_without_dupes() {
        let r = Registry::standard();
        assert!(
            r.len() >= 80,
            "expected a substantial function library, got {}",
            r.len()
        );
        assert!(r.get("sum").is_some());
        assert!(r.get("SUM").is_some());
        assert!(r.get("not").is_some());
        assert!(r.is_volatile("RAND"));
    }

    // ── Registry::get 边界测试 ──────────────────────────────────────────

    #[test]
    fn registry_get_xlfn_prefix() {
        let r = Registry::standard();
        // _xlfn. 前缀应被自动剥离
        assert!(r.get("_xlfn.SUM").is_some());
        assert!(r.get("_xlfn.VLOOKUP").is_some());
        // IF 是特殊形式（lazy），不在注册表中，get 返回 None 也正常
        assert!(r.get("_xlfn.ABS").is_some());
    }

    #[test]
    fn registry_get_unknown_returns_none() {
        let r = Registry::standard();
        assert!(r.get("NONEXISTENT_FUNCTION").is_none());
        assert!(r.get("").is_none());
    }

    #[test]
    fn registry_is_volatile_nonvolatile() {
        let r = Registry::standard();
        assert!(!r.is_volatile("SUM"));
        assert!(!r.is_volatile("IF"));
        assert!(!r.is_volatile("NONEXISTENT"));
    }

    #[test]
    fn registry_is_empty_false() {
        let r = Registry::standard();
        assert!(!r.is_empty());
    }

    #[test]
    fn registry_default_trait() {
        let r = Registry::default();
        assert!(r.len() >= 80);
    }

    // ── first_error 测试 ────────────────────────────────────────────────

    #[test]
    fn first_error_no_errors() {
        assert!(first_error(&[Value::Number(1.0), Value::Text("x".into())]).is_none());
    }

    #[test]
    fn first_error_returns_first() {
        assert_eq!(
            first_error(&[Value::Number(1.0), Value::Error(CellError::NA)]),
            Some(CellError::NA)
        );
    }

    #[test]
    fn first_error_empty_args() {
        assert!(first_error(&[]).is_none());
    }

    // ── collect_numbers 测试 ────────────────────────────────────────────

    #[test]
    fn collect_numbers_scalar_args() {
        use crate::formula::functions::testutil::TestCtx;
        let mut ctx = TestCtx::new();
        let args = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let result = collect_numbers(&mut ctx, &args, false).unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn collect_numbers_bool_coerced() {
        use crate::formula::functions::testutil::TestCtx;
        let mut ctx = TestCtx::new();
        let args = vec![Value::Bool(true), Value::Bool(false)];
        let result = collect_numbers(&mut ctx, &args, false).unwrap();
        assert_eq!(result, vec![1.0, 0.0]);
    }

    #[test]
    fn collect_numbers_empty_skipped() {
        use crate::formula::functions::testutil::TestCtx;
        let mut ctx = TestCtx::new();
        let args = vec![Value::Number(1.0), Value::Empty, Value::Number(2.0)];
        let result = collect_numbers(&mut ctx, &args, false).unwrap();
        assert_eq!(result, vec![1.0, 2.0]);
    }

    #[test]
    fn collect_numbers_error_short_circuits() {
        use crate::formula::functions::testutil::TestCtx;
        let mut ctx = TestCtx::new();
        let args = vec![Value::Number(1.0), Value::Error(CellError::NA)];
        assert_eq!(
            collect_numbers(&mut ctx, &args, false),
            Err(CellError::NA)
        );
    }

    #[test]
    fn collect_numbers_text_scalar_errors() {
        use crate::formula::functions::testutil::TestCtx;
        let mut ctx = TestCtx::new();
        let args = vec![Value::Text("hello".into())];
        assert_eq!(
            collect_numbers(&mut ctx, &args, false),
            Err(CellError::Value)
        );
    }

    #[test]
    fn collect_numbers_numeric_text_scalar_coerced() {
        use crate::formula::functions::testutil::TestCtx;
        let mut ctx = TestCtx::new();
        let args = vec![Value::Text("42".into())];
        let result = collect_numbers(&mut ctx, &args, false).unwrap();
        assert_eq!(result, vec![42.0]);
    }

    #[test]
    fn collect_numbers_lambda_errors() {
        use crate::formula::functions::testutil::TestCtx;
        use std::rc::Rc;
        use crate::formula::value::Lambda;
        use crate::formula::ast::Expr;
        let mut ctx = TestCtx::new();
        let args = vec![Value::Lambda(Rc::new(Lambda {
            params: vec!["x".into()],
            body: Expr::Name("x".into()),
        }))];
        assert_eq!(
            collect_numbers(&mut ctx, &args, false),
            Err(CellError::Value)
        );
    }

    // ── wildcard_match 测试 ─────────────────────────────────────────────

    #[test]
    fn wildcard_exact() {
        assert!(wildcard_match("hello", "hello"));
        assert!(!wildcard_match("hello", "world"));
    }

    #[test]
    fn wildcard_star() {
        assert!(wildcard_match("he*", "hello"));
        assert!(wildcard_match("*llo", "hello"));
        assert!(wildcard_match("h*o", "hello"));
        assert!(wildcard_match("*", "anything"));
        assert!(wildcard_match("*", ""));
    }

    #[test]
    fn wildcard_question() {
        assert!(wildcard_match("he?lo", "hello"));
        assert!(!wildcard_match("he?o", "hello"));
        assert!(wildcard_match("???", "abc"));
        assert!(!wildcard_match("??", "abc"));
    }

    #[test]
    fn wildcard_escape() {
        assert!(wildcard_match("~*", "*"));
        assert!(wildcard_match("~?", "?"));
        assert!(!wildcard_match("~*", "x"));
    }

    #[test]
    fn wildcard_combined() {
        assert!(wildcard_match("h*?o", "hello"));
        assert!(wildcard_match("*?", "ab"));
    }

    #[test]
    fn wildcard_no_match() {
        assert!(!wildcard_match("abc", "xyz"));
        assert!(!wildcard_match("abc*", "xyz"));
    }

    // ── numeric_of 测试 ────────────────────────────────────────────────

    #[test]
    fn numeric_of_numbers() {
        assert_eq!(numeric_of(&Value::Number(5.0)), Some(5.0));
        assert_eq!(numeric_of(&Value::Bool(true)), Some(1.0));
        assert_eq!(numeric_of(&Value::Bool(false)), Some(0.0));
        assert_eq!(numeric_of(&Value::Text("x".into())), None);
        assert_eq!(numeric_of(&Value::Empty), None);
    }

    // ── VARIADIC 常量 ──────────────────────────────────────────────────

    #[test]
    fn variadic_is_max() {
        assert_eq!(VARIADIC, usize::MAX);
    }

    // ── Criteria::parse 测试 ───────────────────────────────────────────

    #[test]
    fn criteria_parse_number() {
        let c = Criteria::parse(&Value::Number(5.0));
        // Numeric criterion: Eq 5.0
        assert!(c.matches(&Value::Number(5.0)));
        assert!(!c.matches(&Value::Number(4.0)));
    }

    #[test]
    fn criteria_parse_gt() {
        let c = Criteria::parse(&Value::Text(">10".into()));
        assert!(c.matches(&Value::Number(11.0)));
        assert!(!c.matches(&Value::Number(10.0)));
        assert!(!c.matches(&Value::Number(9.0)));
    }

    #[test]
    fn criteria_parse_lt() {
        let c = Criteria::parse(&Value::Text("<10".into()));
        assert!(c.matches(&Value::Number(9.0)));
        assert!(!c.matches(&Value::Number(10.0)));
    }

    #[test]
    fn criteria_parse_ge() {
        let c = Criteria::parse(&Value::Text(">=10".into()));
        assert!(c.matches(&Value::Number(10.0)));
        assert!(c.matches(&Value::Number(11.0)));
        assert!(!c.matches(&Value::Number(9.0)));
    }

    #[test]
    fn criteria_parse_le() {
        let c = Criteria::parse(&Value::Text("<=10".into()));
        assert!(c.matches(&Value::Number(10.0)));
        assert!(!c.matches(&Value::Number(11.0)));
    }

    #[test]
    fn criteria_parse_ne() {
        let c = Criteria::parse(&Value::Text("<>10".into()));
        assert!(c.matches(&Value::Number(9.0)));
        assert!(!c.matches(&Value::Number(10.0)));
    }

    #[test]
    fn criteria_parse_eq_prefix() {
        let c = Criteria::parse(&Value::Text("=hello".into()));
        assert!(c.matches(&Value::Text("hello".into())));
        assert!(!c.matches(&Value::Text("world".into())));
    }

    #[test]
    fn criteria_parse_ne_text() {
        let c = Criteria::parse(&Value::Text("<>hello".into()));
        assert!(!c.matches(&Value::Text("hello".into())));
        assert!(c.matches(&Value::Text("world".into())));
    }

    #[test]
    fn criteria_parse_bool_true() {
        let c = Criteria::parse(&Value::Bool(true));
        // TRUE → "TRUE" → matches text "true"
        assert!(c.matches(&Value::Bool(true)));
        assert!(!c.matches(&Value::Bool(false)));
    }

    #[test]
    fn criteria_parse_bool_false() {
        let c = Criteria::parse(&Value::Bool(false));
        assert!(c.matches(&Value::Bool(false)));
        assert!(!c.matches(&Value::Bool(true)));
    }

    #[test]
    fn criteria_parse_empty_criterion() {
        let c = Criteria::parse(&Value::Text("".into()));
        // Empty criterion matches empty cells
        assert!(c.matches(&Value::Empty));
        assert!(!c.matches(&Value::Number(1.0)));
    }

    #[test]
    fn criteria_parse_wildcard() {
        let c = Criteria::parse(&Value::Text("he*o".into()));
        assert!(c.matches(&Value::Text("hello".into())));
        assert!(c.matches(&Value::Text("hero".into())));
        assert!(!c.matches(&Value::Text("hi".into())));
    }

    #[test]
    fn criteria_parse_question_wildcard() {
        let c = Criteria::parse(&Value::Text("he?lo".into()));
        assert!(c.matches(&Value::Text("hello".into())));
        assert!(!c.matches(&Value::Text("heo".into())));
    }

    #[test]
    fn criteria_parse_non_numeric_gt() {
        // ">abc" is text comparison, not numeric
        let c = Criteria::parse(&Value::Text(">abc".into()));
        assert!(c.matches(&Value::Text("bcd".into())));
        assert!(!c.matches(&Value::Text("aab".into())));
    }

    #[test]
    fn criteria_parse_ne_num_vs_text() {
        // Criterion is numeric "<>5", cell is text → only <> matches
        let c = Criteria::parse(&Value::Text("<>5".into()));
        assert!(c.matches(&Value::Text("hello".into())));
        assert!(!c.matches(&Value::Number(5.0)));
    }

    #[test]
    fn criteria_parse_text_match_case_insensitive() {
        let c = Criteria::parse(&Value::Text("Hello".into()));
        assert!(c.matches(&Value::Text("hello".into())));
        assert!(c.matches(&Value::Text("HELLO".into())));
    }

    #[test]
    fn criteria_parse_number_cell_text_criterion() {
        // Criterion is numeric 42, cell is number 42
        let c = Criteria::parse(&Value::Number(42.0));
        assert!(c.matches(&Value::Number(42.0)));
        assert!(!c.matches(&Value::Number(43.0)));
    }

    #[test]
    fn criteria_parse_error_value() {
        let c = Criteria::parse(&Value::Error(CellError::NA));
        // Error → empty string → Eq empty → only matches Empty
        assert!(c.matches(&Value::Empty));
        assert!(!c.matches(&Value::Number(1.0)));
    }

    #[test]
    fn criteria_parse_ge_text() {
        let c = Criteria::parse(&Value::Text(">=apple".into()));
        assert!(c.matches(&Value::Text("apple".into())));
        assert!(c.matches(&Value::Text("banana".into())));
        assert!(!c.matches(&Value::Text("aaa".into())));
    }

    #[test]
    fn criteria_parse_le_text() {
        let c = Criteria::parse(&Value::Text("<=m".into()));
        assert!(c.matches(&Value::Text("a".into())));
        assert!(c.matches(&Value::Text("m".into())));
        assert!(!c.matches(&Value::Text("z".into())));
    }
}
