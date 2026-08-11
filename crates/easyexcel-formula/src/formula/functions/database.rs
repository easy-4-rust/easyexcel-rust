//! Database functions (DSUM, DAVERAGE, DCOUNT, DCOUNTA, DMAX, DMIN, DGET,
//! DPRODUCT, DSTDEV, DSTDEVP, DVAR, DVARP).
//!
//! All functions take (database, field, criteria):
//!   - database : rectangular range; first row is headers.
//!   - field    : 1-based column index OR text matching a header (case-insensitive).
//!   - criteria : rectangular range; first row is headers, subsequent rows are
//!     conditions.  A record matches if it satisfies ALL conditions in
//!     at least ONE criteria row.

use super::{Criteria, Registry};
use crate::formula::coerce::{to_number, to_text};
use crate::formula::context::Context;
use crate::formula::value::{Array, Value};
use easyexcel_model::error::CellError;
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn register(r: &mut Registry) {
    r.add("DSUM", 3, 3, false, dsum);
    r.add("DAVERAGE", 3, 3, false, daverage);
    r.add("DCOUNT", 3, 3, false, dcount);
    r.add("DCOUNTA", 3, 3, false, dcounta);
    r.add("DMAX", 3, 3, false, dmax);
    r.add("DMIN", 3, 3, false, dmin);
    r.add("DGET", 3, 3, false, dget);
    r.add("DPRODUCT", 3, 3, false, dproduct);
    r.add("DSTDEV", 3, 3, false, dstdev);
    r.add("DSTDEVP", 3, 3, false, dstdevp);
    r.add("DVAR", 3, 3, false, dvar);
    r.add("DVARP", 3, 3, false, dvarp);
}

// ---------------------------------------------------------------------------
// Core helper
// ---------------------------------------------------------------------------

/// Materialise `database` into an Array via ctx.
fn to_array(ctx: &mut dyn Context, v: &Value) -> Array {
    match v {
        Value::Ref(r) => ctx.ref_to_array(*r),
        Value::Array(a) => a.clone(),
        other => Array {
            rows: 1,
            cols: 1,
            data: vec![other.clone()],
        },
    }
}

/// Resolve the `field` argument to a 0-based column index inside `db`.
/// Returns `None` if the field cannot be resolved (yields #VALUE!).
fn resolve_field(db: &Array, field: &Value) -> Result<usize, CellError> {
    // Numeric: 1-based column index
    if let Ok(n) = to_number(field) {
        let idx = n.floor() as i64 - 1;
        if idx < 0 || idx as usize >= db.cols {
            return Err(CellError::Value);
        }
        return Ok(idx as usize);
    }
    // Text: match against headers (first row), case-insensitive
    let name = to_text(field).map_err(|_| CellError::Value)?.to_lowercase();
    for c in 0..db.cols {
        if let Some(h) = db.get(0, c) {
            let header = to_text(h).unwrap_or_default().to_lowercase();
            if header == name {
                return Ok(c);
            }
        }
    }
    Err(CellError::Value)
}

/// Collect the values in `field_col` for all database records (rows 1..) that
/// match at least one criteria row. Returns Err if an error propagates from the
/// database itself.
fn matching_field_values(ctx: &mut dyn Context, args: &[Value]) -> Result<Vec<Value>, CellError> {
    let db = to_array(ctx, &args[0]);
    let crit = to_array(ctx, &args[2]);

    if db.rows < 1 || crit.rows < 1 {
        return Ok(vec![]);
    }

    // Resolve field column
    let field_col = resolve_field(&db, &args[1])?;

    // Map criteria column headers → database column indices
    // A criteria column with a header not found in db is simply ignored.
    let mut crit_col_map: Vec<Option<usize>> = Vec::with_capacity(crit.cols);
    for cc in 0..crit.cols {
        if let Some(h) = crit.get(0, cc) {
            let hname = to_text(h).unwrap_or_default().to_lowercase();
            let db_col = (0..db.cols).find(|&dc| {
                db.get(0, dc)
                    .and_then(|v| to_text(v).ok())
                    .is_some_and(|s| s.to_lowercase() == hname)
            });
            crit_col_map.push(db_col);
        } else {
            crit_col_map.push(None);
        }
    }

    // Build criteria rows (skip the header row 0)
    // Each criteria row is a Vec of (db_col_index, Criteria)
    let crit_rows: Vec<Vec<(usize, Criteria)>> = (1..crit.rows)
        .map(|cr| {
            (0..crit.cols)
                .filter_map(|cc| {
                    let db_col = crit_col_map[cc]?;
                    let cell = crit.get(cr, cc)?;
                    // Blank criteria cell means "no constraint"
                    if matches!(cell, Value::Empty) {
                        return None;
                    }
                    Some((db_col, Criteria::parse(cell)))
                })
                .collect()
        })
        .collect();

    // Walk data rows (rows 1.. in database)
    let mut out = Vec::new();
    for dr in 1..db.rows {
        // A row matches if ANY criteria row matches (OR)
        let row_matches = crit_rows.is_empty()
            || crit_rows.iter().any(|conditions| {
                // ALL conditions in a criteria row must hold (AND)
                conditions.iter().all(|(dc, crit)| {
                    let cell = db.get(dr, *dc).unwrap_or(&Value::Empty);
                    crit.matches(cell)
                })
            });

        if row_matches {
            let fv = db.get(dr, field_col).cloned().unwrap_or(Value::Empty);
            out.push(fv);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Individual D-functions
// ---------------------------------------------------------------------------

fn dsum(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let mut total = 0.0;
            for v in &vals {
                if let Ok(n) = to_number(v) {
                    total += n; // non-numeric silently ignored
                }
            }
            Value::Number(total)
        }
    }
}

fn daverage(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let nums: Vec<f64> = vals.iter().filter_map(|v| to_number(v).ok()).collect();
            if nums.is_empty() {
                return Value::Error(CellError::Div0);
            }
            Value::Number(nums.iter().sum::<f64>() / nums.len() as f64)
        }
    }
}

fn dcount(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let count = vals.iter().filter(|v| to_number(v).is_ok()).count();
            Value::Number(count as f64)
        }
    }
}

fn dcounta(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let count = vals.iter().filter(|v| !matches!(v, Value::Empty)).count();
            Value::Number(count as f64)
        }
    }
}

fn dmax(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let nums: Vec<f64> = vals.iter().filter_map(|v| to_number(v).ok()).collect();
            if nums.is_empty() {
                return Value::Number(0.0);
            }
            Value::Number(nums.into_iter().fold(f64::NEG_INFINITY, f64::max))
        }
    }
}

fn dmin(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let nums: Vec<f64> = vals.iter().filter_map(|v| to_number(v).ok()).collect();
            if nums.is_empty() {
                return Value::Number(0.0);
            }
            Value::Number(nums.into_iter().fold(f64::INFINITY, f64::min))
        }
    }
}

fn dget(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => match vals.len() {
            0 => Value::Error(CellError::Value),
            1 => vals.into_iter().next().unwrap(),
            _ => Value::Error(CellError::Num),
        },
    }
}

fn dproduct(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let nums: Vec<f64> = vals.iter().filter_map(|v| to_number(v).ok()).collect();
            if nums.is_empty() {
                return Value::Number(0.0);
            }
            Value::Number(nums.into_iter().product())
        }
    }
}

fn dstdev(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let nums: Vec<f64> = vals.iter().filter_map(|v| to_number(v).ok()).collect();
            if nums.len() < 2 {
                return Value::Error(CellError::Div0);
            }
            Value::Number(sample_stdev(&nums))
        }
    }
}

fn dstdevp(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let nums: Vec<f64> = vals.iter().filter_map(|v| to_number(v).ok()).collect();
            if nums.is_empty() {
                return Value::Error(CellError::Div0);
            }
            Value::Number(pop_stdev(&nums))
        }
    }
}

fn dvar(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let nums: Vec<f64> = vals.iter().filter_map(|v| to_number(v).ok()).collect();
            if nums.len() < 2 {
                return Value::Error(CellError::Div0);
            }
            Value::Number(sample_var(&nums))
        }
    }
}

fn dvarp(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match matching_field_values(ctx, args) {
        Err(e) => Value::Error(e),
        Ok(vals) => {
            let nums: Vec<f64> = vals.iter().filter_map(|v| to_number(v).ok()).collect();
            if nums.is_empty() {
                return Value::Error(CellError::Div0);
            }
            Value::Number(pop_var(&nums))
        }
    }
}

// ---------------------------------------------------------------------------
// Statistics helpers
// ---------------------------------------------------------------------------

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn sample_var(xs: &[f64]) -> f64 {
    let m = mean(xs);
    let n = xs.len() as f64;
    xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0)
}

fn pop_var(xs: &[f64]) -> f64 {
    let m = mean(xs);
    let n = xs.len() as f64;
    xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n
}

fn sample_stdev(xs: &[f64]) -> f64 {
    sample_var(xs).sqrt()
}

fn pop_stdev(xs: &[f64]) -> f64 {
    pop_var(xs).sqrt()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::functions::testutil::{TestCtx, rng};

    /// Build a small database table:
    ///
    ///   row 0 (headers): Name  | Age | Salary
    ///   row 1:           Alice |  25 | 50000
    ///   row 2:           Bob   |  35 | 60000
    ///   row 3:           Carol |  45 | 70000
    ///   row 4:           Dave  |  35 | 55000
    ///
    /// Criteria table (cols 0-1 = Age / Salary):
    ///   row 0 (headers): Age
    ///   row 1:           >30
    fn make_ctx() -> TestCtx {
        TestCtx::with_cells(&[
            // database: rows 0-4, cols 0-2
            (0, 0, Value::Text("Name".into())),
            (0, 1, Value::Text("Age".into())),
            (0, 2, Value::Text("Salary".into())),
            (1, 0, Value::Text("Alice".into())),
            (1, 1, Value::Number(25.0)),
            (1, 2, Value::Number(50000.0)),
            (2, 0, Value::Text("Bob".into())),
            (2, 1, Value::Number(35.0)),
            (2, 2, Value::Number(60000.0)),
            (3, 0, Value::Text("Carol".into())),
            (3, 1, Value::Number(45.0)),
            (3, 2, Value::Number(70000.0)),
            (4, 0, Value::Text("Dave".into())),
            (4, 1, Value::Number(35.0)),
            (4, 2, Value::Number(55000.0)),
            // criteria: rows 6-7, col 5
            (6, 5, Value::Text("Age".into())),
            (7, 5, Value::Text(">30".into())),
        ])
    }

    /// database ref: rows 0-4, cols 0-2
    fn db_ref() -> Value {
        rng(0, 0, 4, 2)
    }

    /// criteria ref: rows 6-7, col 5 (just Age > 30)
    fn crit_ref() -> Value {
        rng(6, 5, 7, 5)
    }

    // DSUM ---------------------------------------------------------------

    #[test]
    fn dsum_age_gt30() {
        let mut ctx = make_ctx();
        // DSUM(db, "Salary", criteria) with Age > 30: Bob+Carol+Dave = 185000
        let result = dsum(
            &mut ctx,
            &[db_ref(), Value::Text("Salary".into()), crit_ref()],
        );
        assert_eq!(result, Value::Number(185_000.0));
    }

    #[test]
    fn dsum_by_column_index() {
        let mut ctx = make_ctx();
        // Field = 2 means Age; age of >30 rows: 35+45+35 = 115
        let result = dsum(&mut ctx, &[db_ref(), Value::Number(2.0), crit_ref()]);
        assert_eq!(result, Value::Number(115.0));
    }

    #[test]
    fn dsum_all_rows_no_filter() {
        // criteria that matches everything: single blank row after header
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Text("Age".into())),
            (1, 0, Value::Number(10.0)),
            (2, 0, Value::Number(20.0)),
            // criteria: header "Age", blank row → all match
            (4, 3, Value::Text("Age".into())),
            // row 5 col 3 left empty → blank criterion
        ]);
        let result = dsum(
            &mut ctx,
            &[rng(0, 0, 2, 0), Value::Text("Age".into()), rng(4, 3, 5, 3)],
        );
        assert_eq!(result, Value::Number(30.0));
    }

    // DAVERAGE -----------------------------------------------------------

    #[test]
    fn daverage_salary_age_gt30() {
        let mut ctx = make_ctx();
        // Average salary of >30: (60000+70000+55000)/3
        let result = daverage(
            &mut ctx,
            &[db_ref(), Value::Text("Salary".into()), crit_ref()],
        );
        assert_eq!(result, Value::Number(185_000.0 / 3.0));
    }

    // DCOUNT / DCOUNTA ---------------------------------------------------

    #[test]
    fn dcount_numeric_only() {
        let mut ctx = make_ctx();
        // DCOUNT on Age col with Age>30 → 3 numeric values
        let result = dcount(&mut ctx, &[db_ref(), Value::Text("Age".into()), crit_ref()]);
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn dcounta_includes_text() {
        let mut ctx = make_ctx();
        // DCOUNTA on Name col with Age>30 → 3 non-blank values (text)
        let result = dcounta(
            &mut ctx,
            &[db_ref(), Value::Text("Name".into()), crit_ref()],
        );
        assert_eq!(result, Value::Number(3.0));
    }

    // DMAX / DMIN --------------------------------------------------------

    #[test]
    fn dmax_salary() {
        let mut ctx = make_ctx();
        let result = dmax(
            &mut ctx,
            &[db_ref(), Value::Text("Salary".into()), crit_ref()],
        );
        assert_eq!(result, Value::Number(70000.0));
    }

    #[test]
    fn dmin_salary() {
        let mut ctx = make_ctx();
        let result = dmin(
            &mut ctx,
            &[db_ref(), Value::Text("Salary".into()), crit_ref()],
        );
        assert_eq!(result, Value::Number(55000.0));
    }

    // DGET ---------------------------------------------------------------

    #[test]
    fn dget_single_match() {
        // criteria: Age = 45 → exactly one record (Carol)
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Text("Name".into())),
            (0, 1, Value::Text("Age".into())),
            (1, 0, Value::Text("Alice".into())),
            (1, 1, Value::Number(25.0)),
            (2, 0, Value::Text("Carol".into())),
            (2, 1, Value::Number(45.0)),
            // criteria
            (4, 0, Value::Text("Age".into())),
            (5, 0, Value::Number(45.0)),
        ]);
        let result = dget(
            &mut ctx,
            &[rng(0, 0, 2, 1), Value::Text("Name".into()), rng(4, 0, 5, 0)],
        );
        assert_eq!(result, Value::Text("Carol".into()));
    }

    #[test]
    fn dget_multiple_matches_returns_num_error() {
        let mut ctx = make_ctx();
        // Age>30 matches three records → #NUM!
        let result = dget(
            &mut ctx,
            &[db_ref(), Value::Text("Name".into()), crit_ref()],
        );
        assert_eq!(result, Value::Error(CellError::Num));
    }

    #[test]
    fn dget_no_match_returns_value_error() {
        // criteria: Age = 99 → no match
        let mut ctx2 = TestCtx::with_cells(&[
            (0, 0, Value::Text("Age".into())),
            (1, 0, Value::Number(10.0)),
            (3, 0, Value::Text("Age".into())),
            (4, 0, Value::Number(99.0)),
        ]);
        let result = dget(
            &mut ctx2,
            &[rng(0, 0, 1, 0), Value::Text("Age".into()), rng(3, 0, 4, 0)],
        );
        assert_eq!(result, Value::Error(CellError::Value));
    }

    // DPRODUCT -----------------------------------------------------------

    #[test]
    fn dproduct_age_gt30() {
        let mut ctx = make_ctx();
        // Product of age for >30: 35 * 45 * 35 = 55125
        let result = dproduct(&mut ctx, &[db_ref(), Value::Text("Age".into()), crit_ref()]);
        assert_eq!(result, Value::Number(35.0 * 45.0 * 35.0));
    }

    // DSTDEV / DSTDEVP / DVAR / DVARP ------------------------------------

    #[test]
    fn dstdev_sample() {
        let mut ctx = make_ctx();
        // ages of >30: [35, 45, 35], sample stdev
        let result = dstdev(&mut ctx, &[db_ref(), Value::Text("Age".into()), crit_ref()]);
        let ages = vec![35.0_f64, 45.0, 35.0];
        let expected = sample_stdev(&ages);
        assert!(
            matches!(result, Value::Number(n) if (n - expected).abs() < 1e-9),
            "expected ~{expected}, got {result:?}"
        );
    }

    #[test]
    fn dstdevp_population() {
        let mut ctx = make_ctx();
        let result = dstdevp(&mut ctx, &[db_ref(), Value::Text("Age".into()), crit_ref()]);
        let ages = vec![35.0_f64, 45.0, 35.0];
        let expected = pop_stdev(&ages);
        assert!(
            matches!(result, Value::Number(n) if (n - expected).abs() < 1e-9),
            "expected ~{expected}, got {result:?}"
        );
    }

    #[test]
    fn dvar_sample() {
        let mut ctx = make_ctx();
        let result = dvar(&mut ctx, &[db_ref(), Value::Text("Age".into()), crit_ref()]);
        let ages = vec![35.0_f64, 45.0, 35.0];
        let expected = sample_var(&ages);
        assert!(
            matches!(result, Value::Number(n) if (n - expected).abs() < 1e-9),
            "got {result:?}"
        );
    }

    #[test]
    fn dvarp_population() {
        let mut ctx = make_ctx();
        let result = dvarp(&mut ctx, &[db_ref(), Value::Text("Age".into()), crit_ref()]);
        let ages = vec![35.0_f64, 45.0, 35.0];
        let expected = pop_var(&ages);
        assert!(
            matches!(result, Value::Number(n) if (n - expected).abs() < 1e-9),
            "got {result:?}"
        );
    }

    // registry smoke-test: build the full standard registry; database functions
    // must register without panicking (panics on duplicate names).
    #[test]
    fn register_no_dupes() {
        let r = crate::formula::functions::Registry::standard();
        assert!(r.get("DSUM").is_some());
        assert!(r.get("DAVERAGE").is_some());
        assert!(r.get("DCOUNT").is_some());
        assert!(r.get("DCOUNTA").is_some());
        assert!(r.get("DMAX").is_some());
        assert!(r.get("DMIN").is_some());
        assert!(r.get("DGET").is_some());
        assert!(r.get("DPRODUCT").is_some());
        assert!(r.get("DSTDEV").is_some());
        assert!(r.get("DSTDEVP").is_some());
        assert!(r.get("DVAR").is_some());
        assert!(r.get("DVARP").is_some());
    }

    // ── 补充数据库函数测试 ──────────────────────────────────────────────

    #[test]
    fn dsum_empty_result() {
        // criteria 匹配不到任何行 → 返回 0
        let mut ctx2 = TestCtx::with_cells(&[
            (0, 0, Value::Text("Age".into())),
            (1, 0, Value::Number(10.0)),
            (2, 0, Value::Number(20.0)),
            (3, 0, Value::Text("Age".into())),
            (4, 0, Value::Number(99.0)),
        ]);
        let result = dsum(
            &mut ctx2,
            &[rng(0, 0, 2, 0), Value::Text("Age".into()), rng(3, 0, 4, 0)],
        );
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn dcount_empty_result() {
        let mut ctx2 = TestCtx::with_cells(&[
            (0, 0, Value::Text("Age".into())),
            (1, 0, Value::Number(10.0)),
            (3, 0, Value::Text("Age".into())),
            (4, 0, Value::Number(99.0)),
        ]);
        let result = dcount(
            &mut ctx2,
            &[rng(0, 0, 1, 0), Value::Text("Age".into()), rng(3, 0, 4, 0)],
        );
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn dmax_empty_result() {
        let mut ctx2 = TestCtx::with_cells(&[
            (0, 0, Value::Text("Age".into())),
            (1, 0, Value::Number(10.0)),
            (3, 0, Value::Text("Age".into())),
            (4, 0, Value::Number(99.0)),
        ]);
        let result = dmax(
            &mut ctx2,
            &[rng(0, 0, 1, 0), Value::Text("Age".into()), rng(3, 0, 4, 0)],
        );
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn dmin_empty_result() {
        let mut ctx2 = TestCtx::with_cells(&[
            (0, 0, Value::Text("Age".into())),
            (1, 0, Value::Number(10.0)),
            (3, 0, Value::Text("Age".into())),
            (4, 0, Value::Number(99.0)),
        ]);
        let result = dmin(
            &mut ctx2,
            &[rng(0, 0, 1, 0), Value::Text("Age".into()), rng(3, 0, 4, 0)],
        );
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn dstdev_single_value() {
        // 只有一个匹配值 → #DIV/0!
        let mut ctx2 = TestCtx::with_cells(&[
            (0, 0, Value::Text("Age".into())),
            (1, 0, Value::Number(25.0)),
            (3, 0, Value::Text("Age".into())),
            (4, 0, Value::Number(25.0)),
        ]);
        let result = dstdev(
            &mut ctx2,
            &[rng(0, 0, 1, 0), Value::Text("Age".into()), rng(3, 0, 4, 0)],
        );
        assert_eq!(result, Value::Error(CellError::Div0));
    }

    #[test]
    fn dvarp_single_value() {
        let mut ctx2 = TestCtx::with_cells(&[
            (0, 0, Value::Text("Age".into())),
            (1, 0, Value::Number(25.0)),
            (3, 0, Value::Text("Age".into())),
            (4, 0, Value::Number(25.0)),
        ]);
        let result = dvarp(
            &mut ctx2,
            &[rng(0, 0, 1, 0), Value::Text("Age".into()), rng(3, 0, 4, 0)],
        );
        // DVARP 只匹配到一行 → n=1 → 方差为 0.0
        assert_eq!(result, Value::Number(0.0));
    }
}
