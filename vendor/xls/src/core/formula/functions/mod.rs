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

/// A function implementation pointer.
pub type FnImpl = fn(&mut dyn Context, &[Value]) -> Value;

/// Sentinel for a variadic upper bound.
pub const VARIADIC: usize = usize::MAX;

/// Metadata + implementation for one worksheet function.
#[derive(Clone)]
pub struct FnEntry {
    pub name: &'static str,
    pub min_args: usize,
    pub max_args: usize,
    pub volatile: bool,
    pub func: FnImpl,
}

/// Case-insensitive function dispatch table.
pub struct Registry {
    map: HashMap<&'static str, FnEntry>,
}

impl Registry {
    fn new() -> Self {
        Registry {
            map: HashMap::new(),
        }
    }

    /// Register a function. Panics on duplicate names (a programming error).
    pub fn add(
        &mut self,
        name: &'static str,
        min: usize,
        max: usize,
        volatile: bool,
        func: FnImpl,
    ) {
        let entry = FnEntry {
            name,
            min_args: min,
            max_args: max,
            volatile,
            func,
        };
        if self.map.insert(name, entry).is_some() {
            panic!("duplicate function registration: {name}");
        }
    }

    /// Register `alias` as a thin synonym for an already-registered function.
    pub fn alias(&mut self, alias: &'static str, target: &'static str) {
        let mut entry = self
            .map
            .get(target)
            .unwrap_or_else(|| panic!("alias target not found: {target}"))
            .clone();
        entry.name = alias;
        if self.map.insert(alias, entry).is_some() {
            panic!("duplicate function registration: {alias}");
        }
    }

    pub fn get(&self, name: &str) -> Option<&FnEntry> {
        // Names are stored upper-case; look up with an upper-cased key. Strip the
        // OOXML `_xlfn.` future-function prefix if present.
        let n = name.trim_start_matches("_xlfn.").to_ascii_uppercase();
        self.map.get(n.as_str())
    }

    pub fn is_volatile(&self, name: &str) -> bool {
        self.get(name).map(|e| e.volatile).unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Build the standard library registry.
    pub fn standard() -> Self {
        let mut r = Registry::new();
        logical::register(&mut r);
        math::register(&mut r);
        text::register(&mut r);
        stats::register(&mut r);
        lookup::register(&mut r);
        datetime::register(&mut r);
        info::register(&mut r);
        financial::register(&mut r);
        engineering::register(&mut r);
        database::register(&mut r);
        dynamic::register(&mut r);
        stubs::register(&mut r);
        r
    }
}

impl Default for Registry {
    fn default() -> Self {
        Registry::standard()
    }
}

// ---------------------------------------------------------------------------
// Shared helpers for function implementations.
// ---------------------------------------------------------------------------

use crate::core::error::CellError;

/// Propagate the first error found in `args`, if any (used by strict numeric fns).
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
                            out.push(if b { 1.0 } else { 0.0 })
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

/// A parsed criterion used by the `*IF`/`*IFS` family.
///
/// Excel criteria are a string like `">5"`, `"<=10"`, `"<>x"`, `"apple"`, or a
/// bare value. Text comparisons are case-insensitive and support `*`/`?`
/// wildcards.
pub struct Criteria {
    op: CritOp,
    /// Numeric comparison target, if the criterion value parses as a number.
    num: Option<f64>,
    /// Text comparison target (lower-cased).
    text: String,
    /// Original (non-lowercased) text.
    raw: String,
}

#[derive(PartialEq)]
enum CritOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Criteria {
    /// Build a criterion from a criteria value (number, bool or text).
    pub fn parse(v: &Value) -> Criteria {
        let s = match v {
            Value::Number(n) => crate::core::value::format_number_general(*n),
            Value::Bool(b) => {
                if *b {
                    "TRUE".into()
                } else {
                    "FALSE".into()
                }
            }
            Value::Text(s) => s.clone(),
            _ => String::new(),
        };
        let (op, rest) = if let Some(r) = s.strip_prefix("<=") {
            (CritOp::Le, r)
        } else if let Some(r) = s.strip_prefix(">=") {
            (CritOp::Ge, r)
        } else if let Some(r) = s.strip_prefix("<>") {
            (CritOp::Ne, r)
        } else if let Some(r) = s.strip_prefix('<') {
            (CritOp::Lt, r)
        } else if let Some(r) = s.strip_prefix('>') {
            (CritOp::Gt, r)
        } else if let Some(r) = s.strip_prefix('=') {
            (CritOp::Eq, r)
        } else {
            (CritOp::Eq, s.as_str())
        };
        let num = super::coerce::parse_number_text(rest);
        Criteria {
            op,
            num,
            text: rest.to_lowercase(),
            raw: rest.to_string(),
        }
    }

    /// Test a candidate value against the criterion.
    pub fn matches(&self, v: &Value) -> bool {
        // Numeric comparison when both sides are numeric.
        if let Some(target) = self.num {
            if let Some(n) = numeric_of(v) {
                return match self.op {
                    CritOp::Eq => n == target,
                    CritOp::Ne => n != target,
                    CritOp::Lt => n < target,
                    CritOp::Le => n <= target,
                    CritOp::Gt => n > target,
                    CritOp::Ge => n >= target,
                };
            }
            // criterion numeric but cell not numeric: only <> matches
            return self.op == CritOp::Ne;
        }
        // Text comparison.
        let cell_text = match v {
            Value::Text(s) => s.to_lowercase(),
            Value::Bool(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Value::Empty => String::new(),
            Value::Number(n) => crate::core::value::format_number_general(*n).to_lowercase(),
            _ => return false,
        };
        match self.op {
            CritOp::Eq => {
                if self.raw.is_empty() {
                    matches!(v, Value::Empty)
                } else {
                    wildcard_match(&self.text, &cell_text)
                }
            }
            CritOp::Ne => !wildcard_match(&self.text, &cell_text),
            CritOp::Lt => cell_text < self.text,
            CritOp::Le => cell_text <= self.text,
            CritOp::Gt => cell_text > self.text,
            CritOp::Ge => cell_text >= self.text,
        }
    }
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
                } else {
                    return false;
                }
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
