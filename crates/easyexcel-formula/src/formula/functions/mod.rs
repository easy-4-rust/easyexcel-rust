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
}
