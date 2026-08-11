//! The recalculation engine: parse formulas, build a dependency graph among
//! formula cells, order them with a topological sort, evaluate, and write back
//! cached values. Detects circular references and always re-evaluates volatile
//! formulas.

use std::collections::{HashMap, HashSet, VecDeque};

use super::ast::{Expr, SheetSpec};
use super::context::CellRef;
use super::eval::Evaluator;
use super::functions::Registry;
use super::parse;
use super::value::Value;
use easyexcel_model::error::CellError;
use easyexcel_model::model::{Cell, Spill, Workbook};
use easyexcel_model::value::CellValue;

include!("engine/coord.rs");

/// A within-sheet (row, col) coordinate.
type Coord2 = (u32, u32);

/// A resolved range of cells (for dependency overlap tests).
#[derive(Clone, Copy)]
struct RangeDep {
    sheet: usize,
    r0: u32,
    c0: u32,
    r1: u32,
    c1: u32,
}

impl RangeDep {
    fn contains(&self, sheet: usize, row: u32, col: u32) -> bool {
        sheet == self.sheet && row >= self.r0 && row <= self.r1 && col >= self.c0 && col <= self.c1
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 The persistent calculation engine for a workbook. Holds the function
/// registry and a cache of parsed ASTs keyed by formula text.
pub struct Engine {
    pub registry: Registry,
    ast_cache: HashMap<String, std::rc::Rc<Expr>>,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

include!("engine/recalc_report.rs");

impl Engine {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn new() -> Self {
        Engine {
            registry: Registry::standard(),
            ast_cache: HashMap::new(),
        }
    }

    fn parse_cached(&mut self, text: &str) -> Result<std::rc::Rc<Expr>, String> {
        if let Some(e) = self.ast_cache.get(text) {
            return Ok(e.clone());
        }
        let expr = parse::parse(text).map_err(|e| e.to_string())?;
        let rc = std::rc::Rc::new(expr);
        self.ast_cache.insert(text.to_string(), rc.clone());
        Ok(rc)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Evaluate a single standalone formula string against the workbook, as if it
    /// were entered at `at`. Used by the `eval` CLI subcommand.
    pub fn eval_formula(&mut self, wb: &Workbook, at: CellRef, formula: &str) -> Value {
        let text = formula.strip_prefix('=').unwrap_or(formula);
        let expr = match self.parse_cached(text) {
            Ok(e) => e,
            Err(_) => return Value::Error(CellError::Name),
        };
        let (now, today) = now_today(wb);
        let mut ev = Evaluator::new(wb, &self.registry, at, now, today);
        ev.eval(&expr)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Full recalculation: re-evaluate every formula cell in dependency order and
    /// store cached values back into the workbook.
    pub fn recalc(&mut self, wb: &mut Workbook) -> RecalcReport {
        // 1. Collect all formula cells with parsed ASTs.
        let mut formulas: Vec<(Coord, std::rc::Rc<Expr>)> = Vec::new();
        for (si, sheet) in wb.sheets.iter().enumerate() {
            for (&(row, col), cell) in &sheet.cells {
                if let Cell::Formula { expr, .. } = cell {
                    let text = expr.strip_prefix('=').unwrap_or(expr).to_string();
                    if let Ok(ast) = self.parse_cached(&text) {
                        formulas.push(((si, row, col), ast));
                    }
                }
            }
        }

        // Index from coord → node id.
        let index: HashMap<Coord, usize> = formulas
            .iter()
            .enumerate()
            .map(|(i, (c, _))| (*c, i))
            .collect();

        // 2. Extract referenced ranges per formula.
        let mut ranges: Vec<Vec<RangeDep>> = Vec::with_capacity(formulas.len());
        for ((si, _, _), ast) in &formulas {
            let mut acc = Vec::new();
            collect_ranges(ast, *si, wb, &mut acc);
            ranges.push(acc);
        }

        // 3. Build edges precedent → dependent among formula cells.
        //
        // For each dependent's referenced ranges we find the *formula* cells that
        // fall inside them. Small ranges (the overwhelmingly common case — a cell
        // referencing a neighbour or a modest block) are resolved by enumerating
        // their cells and probing the coord→node index, which is O(area). Only
        // genuinely huge ranges (e.g. whole columns) fall back to scanning the
        // formula set. This keeps recalc near-linear instead of O(F²).
        const ENUM_THRESHOLD: u64 = 4096;
        let n = formulas.len();
        let mut deps: Vec<Vec<usize>> = vec![Vec::new(); n]; // node -> dependents
        let mut indeg = vec![0usize; n];
        for (dependent, rngs) in ranges.iter().enumerate() {
            let mut precedents: HashSet<usize> = HashSet::new();
            for rg in rngs {
                let area = u64::from(rg.r1 - rg.r0 + 1) * u64::from(rg.c1 - rg.c0 + 1);
                if area <= ENUM_THRESHOLD {
                    for r in rg.r0..=rg.r1 {
                        for c in rg.c0..=rg.c1 {
                            if let Some(&prec_id) = index.get(&(rg.sheet, r, c))
                                && prec_id != dependent
                            {
                                precedents.insert(prec_id);
                            }
                        }
                    }
                } else {
                    // Large range: scan the (sparse) formula set instead.
                    for (coord, &prec_id) in &index {
                        if prec_id != dependent && rg.contains(coord.0, coord.1, coord.2) {
                            precedents.insert(prec_id);
                        }
                    }
                }
            }
            for p in precedents {
                deps[p].push(dependent);
                indeg[dependent] += 1;
            }
        }

        // 4. Kahn topological sort.
        let mut queue: VecDeque<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
        let mut order = Vec::with_capacity(n);
        let mut indeg_work = indeg.clone();
        while let Some(node) = queue.pop_front() {
            order.push(node);
            for &d in &deps[node] {
                indeg_work[d] -= 1;
                if indeg_work[d] == 0 {
                    queue.push_back(d);
                }
            }
        }

        let mut report = RecalcReport::default();
        // Any node not emitted by Kahn's algorithm participates in a cycle. Use a
        // boolean marker (O(n)) rather than `order.contains` (which would be O(n²)).
        let in_cycle: HashSet<usize> = if order.len() < n {
            let mut placed = vec![false; n];
            for &node in &order {
                placed[node] = true;
            }
            (0..n).filter(|&i| !placed[i]).collect()
        } else {
            HashSet::new()
        };

        // 5. Evaluate in topo order, writing cached values immediately so that
        //    dependents observe fresh precedents. Array / multi-cell-range
        //    results *spill* into neighboring cells.
        //
        //    Spilled cells are not formula cells, so a formula that reads one is
        //    not linked to the spill's anchor in the dependency graph. We resolve
        //    that by iterating: each pass reads the previous pass's spills and
        //    rebuilds them, converging (usually in 1–2 passes) once the spill set
        //    is stable. Reads during a pass see the prior pass's spills, so the
        //    fixpoint doesn't depend on intra-pass evaluation order.
        let (now, today) = now_today(wb);
        const MAX_PASSES: usize = 12;
        for _ in 0..MAX_PASSES {
            let mut new_spills: HashMap<usize, std::collections::BTreeMap<Coord2, Spill>> =
                HashMap::new();
            report.evaluated = 0;
            for &node in &order {
                let (coord, ast) = &formulas[node];
                let (si, row, col) = *coord;
                let value = {
                    let mut ev = Evaluator::new(
                        wb,
                        &self.registry,
                        CellRef {
                            sheet: si,
                            row,
                            col,
                        },
                        now,
                        today,
                    );
                    ev.eval(ast)
                };
                report.evaluated += 1;

                // Decide whether this result is a dynamic array that spills.
                let spill = as_spill_payload(&value);
                match spill {
                    Some((rows, cols)) if rows * cols > 1 => {
                        let sheet_spills = new_spills.entry(si).or_default();
                        if spill_fits(wb, si, row, col, rows, cols, sheet_spills) {
                            let values = spill_values(wb, &value, rows, cols);
                            let top_left = values.first().cloned().unwrap_or_default();
                            write_cached(wb, si, row, col, top_left);
                            sheet_spills.insert((row, col), Spill { rows, cols, values });
                        } else {
                            write_cached(wb, si, row, col, CellValue::Error(CellError::Spill));
                        }
                    }
                    _ => {
                        // A bare single-cell reference result (`=A1`) derefs to
                        // the referenced cell's value; everything else is scalar.
                        let scalar = match &value {
                            Value::Ref(r) if r.is_single() => wb
                                .sheets
                                .get(r.sheet)
                                .map(|s| s.value(r.start_row, r.start_col))
                                .unwrap_or_default(),
                            other => other.to_cell_value(),
                        };
                        write_cached(wb, si, row, col, scalar);
                    }
                }
            }

            // Commit the new spill regions; stop when they stop changing.
            let mut changed = false;
            for si in 0..wb.sheets.len() {
                let ns = new_spills.remove(&si).unwrap_or_default();
                if wb.sheets[si].spills != ns {
                    changed = true;
                    wb.sheets[si].spills = ns;
                }
            }
            if !changed {
                break;
            }
        }

        // 6. Circular cells: store a circular error so it is visible.
        for node in in_cycle {
            let (coord, _) = &formulas[node];
            let (si, row, col) = *coord;
            report.circular.push((si, row, col));
            write_cached(
                wb,
                si,
                row,
                col,
                easyexcel_model::value::CellValue::Error(CellError::Ref),
            );
        }

        report
    }
}

fn write_cached(wb: &mut Workbook, si: usize, row: u32, col: u32, value: CellValue) {
    if let Some(sheet) = wb.sheets.get_mut(si)
        && let Some(Cell::Formula { cached, .. }) = sheet.cells.get_mut(&(row, col))
    {
        *cached = value;
    }
}

/// If `value` is a dynamic array (an array, or a multi-cell range reference),
/// return its `(rows, cols)`; otherwise `None` (it's a scalar).
fn as_spill_payload(value: &Value) -> Option<(u32, u32)> {
    match value {
        Value::Array(a) => Some((a.rows as u32, a.cols as u32)),
        Value::Ref(r) if !r.is_single() => Some((r.rows(), r.cols())),
        _ => None,
    }
}

/// Materialize a dynamic-array result into row-major scalar cell values.
fn spill_values(wb: &Workbook, value: &Value, rows: u32, cols: u32) -> Vec<CellValue> {
    match value {
        Value::Array(a) => a
            .data
            .iter()
            .map(super::value::Value::to_cell_value)
            .collect(),
        Value::Ref(r) => {
            let mut out = Vec::with_capacity((rows as usize) * (cols as usize));
            for (rr, cc) in r.iter() {
                out.push(
                    wb.sheets
                        .get(r.sheet)
                        .map(|s| s.value(rr, cc))
                        .unwrap_or_default(),
                );
            }
            out
        }
        _ => Vec::new(),
    }
}

/// True if a `rows×cols` spill anchored at (row, col) can be placed: every
/// non-anchor target cell is empty and not already claimed by another spill in
/// this pass. Oversized spills are rejected to bound memory.
fn spill_fits(
    wb: &Workbook,
    si: usize,
    row: u32,
    col: u32,
    rows: u32,
    cols: u32,
    pass_spills: &std::collections::BTreeMap<Coord2, Spill>,
) -> bool {
    if u64::from(rows) * u64::from(cols) > 1_048_576 {
        return false;
    }
    let Some(sheet) = wb.sheets.get(si) else {
        return false;
    };
    for dr in 0..rows {
        for dc in 0..cols {
            if dr == 0 && dc == 0 {
                continue; // the anchor holds the formula itself
            }
            let (Some(tr), Some(tc)) = (row.checked_add(dr), col.checked_add(dc)) else {
                return false;
            };
            if let Some(cell) = sheet.cells.get(&(tr, tc))
                && !cell.is_empty()
            {
                return false; // would overwrite real data
            }
            for (&(ar, ac), sp) in pass_spills {
                if tr >= ar && tr < ar + sp.rows && tc >= ac && tc < ac + sp.cols {
                    return false; // collides with another spill this pass
                }
            }
        }
    }
    true
}

/// Walk an AST collecting all referenced ranges, resolving sheet names against
/// the workbook. Unknown sheet names are skipped (they'll evaluate to errors).
fn collect_ranges(expr: &Expr, current_sheet: usize, wb: &Workbook, out: &mut Vec<RangeDep>) {
    match expr {
        Expr::Ref(r) => {
            let sheets: Vec<usize> = match &r.sheet {
                SheetSpec::Current => vec![current_sheet],
                SheetSpec::Name(n) => wb.sheet_index(n).into_iter().collect(),
                SheetSpec::Span(a, b) => match (wb.sheet_index(a), wb.sheet_index(b)) {
                    (Some(i), Some(j)) => (i.min(j)..=i.max(j)).collect(),
                    _ => vec![],
                },
            };
            let start = r.start;
            let end = r.end.unwrap_or(start);
            for s in sheets {
                out.push(RangeDep {
                    sheet: s,
                    r0: start.row.min(end.row),
                    c0: start.col.min(end.col),
                    r1: start.row.max(end.row),
                    c1: start.col.max(end.col),
                });
            }
        }
        Expr::Unary { expr, .. } => collect_ranges(expr, current_sheet, wb, out),
        Expr::Binary { lhs, rhs, .. } => {
            collect_ranges(lhs, current_sheet, wb, out);
            collect_ranges(rhs, current_sheet, wb, out);
        }
        Expr::Func { args, .. } => {
            for a in args {
                collect_ranges(a, current_sheet, wb, out);
            }
        }
        Expr::Array(rows) => {
            for row in rows {
                for e in row {
                    collect_ranges(e, current_sheet, wb, out);
                }
            }
        }
        _ => {}
    }
}

/// Compute NOW and TODAY serials for the workbook's date system.
fn now_today(wb: &Workbook) -> (f64, f64) {
    use chrono::Local;
    let now = Local::now().naive_local();
    let now_serial = wb.date_system.datetime_to_serial(now);
    let today_serial = wb.date_system.date_to_serial(now.date()) as f64;
    (now_serial, today_serial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_model::model::Cell;

    fn set_formula(wb: &mut Workbook, a1: &str, f: &str) {
        wb.sheet_mut(0).unwrap().set_a1(
            a1,
            Cell::Formula {
                expr: f.to_string(),
                cached: easyexcel_model::value::CellValue::Empty,
            },
        );
    }

    #[test]
    fn chained_recalc() {
        let mut wb = Workbook::new();
        wb.sheet_mut(0).unwrap().set_a1("A1", Cell::Number(10.0));
        set_formula(&mut wb, "A2", "=A1*2");
        set_formula(&mut wb, "A3", "=A2+5");
        let mut eng = Engine::new();
        eng.recalc(&mut wb);
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(1, 0),
            easyexcel_model::value::CellValue::Number(20.0)
        );
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(2, 0),
            easyexcel_model::value::CellValue::Number(25.0)
        );
    }

    #[test]
    fn scalar_functions_deref_single_cell_refs() {
        // Regression: ABS/ISNUMBER/ISTEXT/N over a single-cell reference must see
        // the cell's value, not the reference. Reference functions still see the
        // reference. (See the "SUM gave zero / formula doesn't work" report — the
        // data was text, but the IS*/scalar-fn deref bug was real.)
        use easyexcel_model::value::CellValue;
        let mut wb = Workbook::new();
        wb.sheet_mut(0).unwrap().set_a1("A1", Cell::Number(-5.0));
        wb.sheet_mut(0)
            .unwrap()
            .set_a1("B1", Cell::Text("hi".into()));
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 9,
            col: 9,
        };
        let v = |eng: &mut Engine, f: &str| eng.eval_formula(&wb, at, f).to_cell_value();

        assert_eq!(v(&mut eng, "=ABS(A1)"), CellValue::Number(5.0));
        assert_eq!(v(&mut eng, "=ISNUMBER(A1)"), CellValue::Bool(true));
        assert_eq!(v(&mut eng, "=ISTEXT(B1)"), CellValue::Bool(true));
        assert_eq!(v(&mut eng, "=ISTEXT(A1)"), CellValue::Bool(false));
        assert_eq!(v(&mut eng, "=N(A1)"), CellValue::Number(-5.0));
        assert_eq!(v(&mut eng, "=SUM(A1)"), CellValue::Number(-5.0));
        // Reference functions still receive the raw reference.
        assert_eq!(v(&mut eng, "=ROW(A1)"), CellValue::Number(1.0));
        assert_eq!(v(&mut eng, "=COLUMN(B1)"), CellValue::Number(2.0));
        assert_eq!(v(&mut eng, "=ISREF(A1)"), CellValue::Bool(true));
    }

    #[test]
    fn dynamic_array_spills() {
        use easyexcel_model::value::CellValue;
        let mut wb = Workbook::new();
        {
            let s = wb.sheet_mut(0).unwrap();
            s.set_a1("A1", Cell::Number(3.0));
            s.set_a1("A2", Cell::Number(1.0));
            s.set_a1("A3", Cell::Number(2.0));
        }
        set_formula(&mut wb, "C1", "=SORT(A1:A3)");
        let mut eng = Engine::new();
        eng.recalc(&mut wb);
        let s = wb.sheet_mut(0).unwrap();
        // Anchor holds the top-left; C2/C3 are spilled (not real cells).
        assert_eq!(s.value(0, 2), CellValue::Number(1.0));
        assert_eq!(s.value(1, 2), CellValue::Number(2.0));
        assert_eq!(s.value(2, 2), CellValue::Number(3.0));
        assert!(s.get(1, 2).is_none(), "spilled cell is not a real cell");
        // A formula reading a spilled cell sees its value (converges over passes).
        // (set after the spill exists, then recalc again)
        set_formula(&mut wb, "E1", "=C3");
        eng.recalc(&mut wb);
        assert_eq!(wb.sheet_mut(0).unwrap().value(0, 4), CellValue::Number(3.0));
    }

    #[test]
    fn spill_blocked_by_obstruction() {
        use easyexcel_model::value::CellValue;
        let mut wb = Workbook::new();
        {
            let s = wb.sheet_mut(0).unwrap();
            s.set_a1("A1", Cell::Number(3.0));
            s.set_a1("A2", Cell::Number(1.0));
            s.set_a1("C2", Cell::Text("blocker".into())); // sits in the spill path
        }
        set_formula(&mut wb, "C1", "=SORT(A1:A2)");
        let mut eng = Engine::new();
        eng.recalc(&mut wb);
        // The anchor reports #SPILL! and nothing spills over the blocker.
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(0, 2),
            CellValue::Error(CellError::Spill)
        );
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(1, 2),
            CellValue::Text("blocker".into())
        );
    }

    #[test]
    fn detects_cycle() {
        let mut wb = Workbook::new();
        set_formula(&mut wb, "A1", "=A2+1");
        set_formula(&mut wb, "A2", "=A1+1");
        let mut eng = Engine::new();
        let report = eng.recalc(&mut wb);
        assert_eq!(report.circular.len(), 2);
    }

    // ── Agent 68 panic 回归测试：NaN 比较不 panic ──────────────────────────

    #[test]
    fn recalc_nan_comparison_no_panic() {
        // 将 NaN 写入单元格，然后用公式引用它触发 partial_cmp NaN 路径
        // 关键验证：不 panic（无论返回什么值）
        let mut wb = Workbook::new();
        wb.sheet_mut(0)
            .unwrap()
            .set_a1("A1", Cell::Number(f64::NAN));
        set_formula(&mut wb, "B1", "=A1+1");
        set_formula(&mut wb, "B2", "=A1>0");
        set_formula(&mut wb, "B3", "=A1<0");
        set_formula(&mut wb, "B4", "=A1=0");
        let mut eng = Engine::new();
        eng.recalc(&mut wb);
        // NaN 运算不 panic 即为通过；结果可能是 Number(NaN) 或 Error
        // 不对具体值做断言，仅验证执行完成
    }

    #[test]
    fn eval_formula_strips_equals_prefix() {
        use easyexcel_model::value::CellValue;
        let mut wb = Workbook::new();
        wb.sheet_mut(0).unwrap().set_a1("A1", Cell::Number(42.0));
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 5,
            col: 5,
        };
        // 带 = 前缀，引用 A1 → 得到 Ref → to_cell_value 变 Error(Value)
        // 但 SUM(A1) 会解引用 → 得到 42.0
        let v1 = eng.eval_formula(&wb, at, "=SUM(A1)").to_cell_value();
        assert_eq!(v1, CellValue::Number(42.0));
        // 不带 = 前缀也能解析（parse 内部会 strip_prefix）
        let v2 = eng.eval_formula(&wb, at, "SUM(A1)").to_cell_value();
        assert_eq!(v2, CellValue::Number(42.0));
    }

    #[test]
    fn eval_formula_parse_error_returns_name() {
        use easyexcel_model::value::CellValue;
        let wb = Workbook::new();
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 0,
            col: 0,
        };
        let v = eng.eval_formula(&wb, at, "!!!invalid").to_cell_value();
        assert_eq!(v, CellValue::Error(CellError::Name));
    }

    #[test]
    fn recalc_empty_workbook() {
        let mut wb = Workbook::new();
        let mut eng = Engine::new();
        let report = eng.recalc(&mut wb);
        assert_eq!(report.evaluated, 0);
        assert!(report.circular.is_empty());
    }

    #[test]
    fn recalc_with_parse_errors_skips_cells() {
        let mut wb = Workbook::new();
        set_formula(&mut wb, "A1", "=INVALID_FORMULA_TOO_MANY_CHARS!!!+!!!"); // parse error
        set_formula(&mut wb, "A2", "=1+1"); // valid
        let mut eng = Engine::new();
        let report = eng.recalc(&mut wb);
        assert_eq!(report.evaluated, 1);
    }

    // ── RangeDep::contains ──────────────────────────────────────────────

    #[test]
    fn range_dep_contains() {
        let rd = RangeDep {
            sheet: 0,
            r0: 1,
            c0: 1,
            r1: 5,
            c1: 5,
        };
        assert!(rd.contains(0, 1, 1));
        assert!(rd.contains(0, 5, 5));
        assert!(rd.contains(0, 3, 3));
        assert!(!rd.contains(1, 3, 3)); // 不同 sheet
        assert!(!rd.contains(0, 0, 3)); // row 越界
        assert!(!rd.contains(0, 3, 0)); // col 越界
        assert!(!rd.contains(0, 6, 3)); // row 越界
        assert!(!rd.contains(0, 3, 6)); // col 越界
    }

    // ── as_spill_payload ────────────────────────────────────────────────

    #[test]
    fn as_spill_payload_scalar_is_none() {
        assert!(as_spill_payload(&Value::Number(1.0)).is_none());
        assert!(as_spill_payload(&Value::Text("x".into())).is_none());
        assert!(as_spill_payload(&Value::Bool(true)).is_none());
        assert!(as_spill_payload(&Value::Empty).is_none());
    }

    #[test]
    fn as_spill_payload_array() {
        let arr = Value::Array(super::super::value::Array::new(
            2,
            3,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
                Value::Number(6.0),
            ],
        ));
        assert_eq!(as_spill_payload(&arr), Some((2, 3)));
    }

    #[test]
    fn as_spill_payload_single_ref_is_none() {
        let r = super::super::value::RefRange {
            sheet: 0,
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 0,
        };
        assert!(as_spill_payload(&Value::Ref(r)).is_none());
    }

    #[test]
    fn as_spill_payload_multi_ref() {
        let r = super::super::value::RefRange {
            sheet: 0,
            start_row: 0,
            start_col: 0,
            end_row: 2,
            end_col: 1,
        };
        assert_eq!(as_spill_payload(&Value::Ref(r)), Some((3, 2)));
    }

    // ── spill_fits 边界条件 ─────────────────────────────────────────────

    #[test]
    fn spill_fits_oversized_rejected() {
        let wb = Workbook::new();
        let pass_spills = std::collections::BTreeMap::new();
        // 超过 1_048_576 限制
        assert!(!spill_fits(&wb, 0, 0, 0, 2000, 600, &pass_spills));
    }

    #[test]
    fn spill_fits_invalid_sheet() {
        let wb = Workbook::new();
        let pass_spills = std::collections::BTreeMap::new();
        assert!(!spill_fits(&wb, 99, 0, 0, 2, 2, &pass_spills));
    }

    #[test]
    fn spill_fits_with_collision() {
        let mut wb = Workbook::new();
        wb.sheet_mut(0)
            .unwrap()
            .set_a1("B2", Cell::Text("block".into()));
        let pass_spills = std::collections::BTreeMap::new();
        // A1:B2 会碰撞 B2
        assert!(!spill_fits(&wb, 0, 0, 0, 2, 2, &pass_spills));
    }

    #[test]
    fn spill_fits_clear() {
        let wb = Workbook::new();
        let pass_spills = std::collections::BTreeMap::new();
        assert!(spill_fits(&wb, 0, 0, 0, 3, 3, &pass_spills));
    }

    // ── 多公式依赖排序 ──────────────────────────────────────────────────

    #[test]
    fn recalc_diamond_dependency() {
        // A1=1, A2=A1+1, A3=A1+2, A4=A2+A3
        let mut wb = Workbook::new();
        wb.sheet_mut(0).unwrap().set_a1("A1", Cell::Number(10.0));
        set_formula(&mut wb, "A2", "=A1+1");
        set_formula(&mut wb, "A3", "=A1+2");
        set_formula(&mut wb, "A4", "=A2+A3");
        let mut eng = Engine::new();
        eng.recalc(&mut wb);
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(3, 0),
            easyexcel_model::value::CellValue::Number(23.0)
        );
    }

    #[test]
    fn recalc_independent_formulas() {
        let mut wb = Workbook::new();
        wb.sheet_mut(0).unwrap().set_a1("A1", Cell::Number(5.0));
        set_formula(&mut wb, "B1", "=A1*2");
        set_formula(&mut wb, "C1", "=A1*3");
        set_formula(&mut wb, "D1", "=100");
        let mut eng = Engine::new();
        let report = eng.recalc(&mut wb);
        assert_eq!(report.evaluated, 3);
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(0, 1),
            easyexcel_model::value::CellValue::Number(10.0)
        );
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(0, 2),
            easyexcel_model::value::CellValue::Number(15.0)
        );
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(0, 3),
            easyexcel_model::value::CellValue::Number(100.0)
        );
    }

    // ── eval_formula 函数调用 ───────────────────────────────────────────

    #[test]
    fn eval_formula_function_calls() {
        use easyexcel_model::value::CellValue;
        let mut wb = Workbook::new();
        wb.sheet_mut(0).unwrap().set_a1("A1", Cell::Number(10.0));
        wb.sheet_mut(0).unwrap().set_a1("A2", Cell::Number(20.0));
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 0,
            col: 0,
        };
        assert_eq!(
            eng.eval_formula(&wb, at, "=SUM(A1:A2)").to_cell_value(),
            CellValue::Number(30.0)
        );
        assert_eq!(
            eng.eval_formula(&wb, at, "=MAX(A1:A2)").to_cell_value(),
            CellValue::Number(20.0)
        );
        assert_eq!(
            eng.eval_formula(&wb, at, "=MIN(A1:A2)").to_cell_value(),
            CellValue::Number(10.0)
        );
        assert_eq!(
            eng.eval_formula(&wb, at, "=COUNT(A1:A2)").to_cell_value(),
            CellValue::Number(2.0)
        );
    }

    #[test]
    fn eval_formula_string_concat() {
        use easyexcel_model::value::CellValue;
        let mut wb = Workbook::new();
        wb.sheet_mut(0)
            .unwrap()
            .set_a1("A1", Cell::Text("Hello".into()));
        wb.sheet_mut(0)
            .unwrap()
            .set_a1("A2", Cell::Text(" World".into()));
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 0,
            col: 0,
        };
        assert_eq!(
            eng.eval_formula(&wb, at, "=A1&A2").to_cell_value(),
            CellValue::Text("Hello World".into())
        );
    }

    #[test]
    fn eval_formula_comparison() {
        use easyexcel_model::value::CellValue;
        let mut wb = Workbook::new();
        wb.sheet_mut(0).unwrap().set_a1("A1", Cell::Number(5.0));
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 0,
            col: 0,
        };
        assert_eq!(
            eng.eval_formula(&wb, at, "=A1>3").to_cell_value(),
            CellValue::Bool(true)
        );
        assert_eq!(
            eng.eval_formula(&wb, at, "=A1<3").to_cell_value(),
            CellValue::Bool(false)
        );
        assert_eq!(
            eng.eval_formula(&wb, at, "=A1=5").to_cell_value(),
            CellValue::Bool(true)
        );
    }

    // ── 缓存行为验证 ────────────────────────────────────────────────────

    #[test]
    fn ast_cache_hit() {
        let wb = Workbook::new();
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 0,
            col: 0,
        };
        // 同一公式两次求值，第二次应命中缓存
        let v1 = eng.eval_formula(&wb, at, "=1+2").to_cell_value();
        let v2 = eng.eval_formula(&wb, at, "=1+2").to_cell_value();
        assert_eq!(v1, v2);
        // 缓存大小应为 1
        assert_eq!(eng.ast_cache.len(), 1);
    }

    // ── Engine::default ─────────────────────────────────────────────────

    #[test]
    fn engine_default_trait() {
        let eng = Engine::default();
        assert!(eng.registry.len() >= 80);
    }

    // ── write_cached 对非公式单元格的 no-op ─────────────────────────────

    #[test]
    fn write_cached_non_formula_cell_is_noop() {
        let mut wb = Workbook::new();
        wb.sheet_mut(0).unwrap().set_a1("A1", Cell::Number(42.0));
        write_cached(&mut wb, 0, 0, 0, easyexcel_model::value::CellValue::Number(999.0));
        // A1 是 Number 不是 Formula，write_cached 应无效果
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(0, 0),
            easyexcel_model::value::CellValue::Number(42.0)
        );
    }

    #[test]
    fn write_cached_invalid_sheet() {
        let mut wb = Workbook::new();
        // sheet 99 不存在，write_cached 不应 panic
        write_cached(&mut wb, 99, 0, 0, easyexcel_model::value::CellValue::Number(1.0));
    }
}
