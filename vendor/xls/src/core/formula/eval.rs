//! The tree-walking evaluator. Turns an [`Expr`] into a [`Value`] against a
//! workbook snapshot, handling operators, reference resolution, defined names,
//! and the lazy "special form" functions (IF, IFERROR, CHOOSE, …).

use std::rc::Rc;

use super::ast::{BinaryOp, Expr, Reference, SheetSpec, UnaryOp};
use super::coerce;
use super::context::{CellRef, Context};
use super::functions::Registry;
use super::value::{Array, Lambda, RefRange, Value};
use crate::core::error::CellError;
use crate::core::model::Workbook;

/// One name binding in a scope: `(name, value, was_omitted)`. `was_omitted`
/// marks a lambda parameter that the caller didn't supply (for `ISOMITTED`).
type Binding = (String, Value, bool);

/// A per-evaluation cursor over a workbook snapshot.
pub struct Evaluator<'a> {
    pub wb: &'a Workbook,
    pub registry: &'a Registry,
    pub current: CellRef,
    pub now: f64,
    pub today: f64,
    /// Recursion guard against pathological/mutually recursive named formulas.
    depth: u32,
    /// Lexical scopes for `LET` bindings and `LAMBDA` parameters (innermost last).
    scopes: Vec<Vec<Binding>>,
}

const MAX_DEPTH: u32 = 256;

/// Functions that must receive raw reference arguments because they inspect a
/// cell's location/identity/geometry rather than its value. Every other function
/// has single-cell references dereferenced to scalars before it runs.
fn wants_reference(name: &str) -> bool {
    matches!(
        name,
        "ROW"
            | "COLUMN"
            | "ROWS"
            | "COLUMNS"
            | "AREAS"
            | "OFFSET"
            | "INDEX"
            | "CELL"
            | "SHEET"
            | "SHEETS"
            | "ISREF"
            | "ISFORMULA"
            | "FORMULATEXT"
    )
}

impl<'a> Evaluator<'a> {
    pub fn new(
        wb: &'a Workbook,
        registry: &'a Registry,
        current: CellRef,
        now: f64,
        today: f64,
    ) -> Self {
        Evaluator {
            wb,
            registry,
            current,
            now,
            today,
            depth: 0,
            scopes: Vec::new(),
        }
    }

    /// Look up a lexically-scoped binding (LET / lambda param), innermost first.
    fn lookup_binding(&self, name: &str) -> Option<&Binding> {
        self.scopes.iter().rev().find_map(|frame| {
            frame
                .iter()
                .rev()
                .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
        })
    }

    /// Evaluate an expression to a value.
    pub fn eval(&mut self, expr: &Expr) -> Value {
        if self.depth > MAX_DEPTH {
            return Value::Error(CellError::Num);
        }
        match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::Text(s) => Value::Text(s.clone()),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Error(e) => Value::Error(*e),
            Expr::Ref(r) => self.eval_ref(r),
            Expr::Name(name) => self.eval_name(name),
            Expr::Unary { op, expr } => self.eval_unary(*op, expr),
            Expr::Binary { op, lhs, rhs } => self.eval_binary(*op, lhs, rhs),
            Expr::Array(rows) => self.eval_array_const(rows),
            Expr::Func { name, args } => self.eval_func(name, args),
        }
    }

    fn resolve_sheet(&self, spec: &SheetSpec) -> Option<usize> {
        match spec {
            SheetSpec::Current => Some(self.current.sheet),
            SheetSpec::Name(n) => self.wb.sheet_index(n),
            // 3D spans collapse to their first sheet for scalar resolution; the
            // aggregate functions that support 3D handle the span themselves.
            SheetSpec::Span(a, _) => self.wb.sheet_index(a),
        }
    }

    fn eval_ref(&mut self, r: &Reference) -> Value {
        let Some(sheet) = self.resolve_sheet(&r.sheet) else {
            return Value::Error(CellError::Ref);
        };
        let start = r.start;
        let end = r.end.unwrap_or(start);
        Value::Ref(RefRange {
            sheet,
            start_row: start.row.min(end.row),
            start_col: start.col.min(end.col),
            end_row: start.row.max(end.row),
            end_col: start.col.max(end.col),
        })
    }

    fn eval_name(&mut self, name: &str) -> Value {
        // A LET/lambda binding shadows workbook defined names.
        if let Some((_, v, _)) = self.lookup_binding(name) {
            return v.clone();
        }
        // Structured table reference, e.g. `Sales[Amount]` or `Sales[#All]`.
        if name.contains('[') {
            return match self.wb.resolve_structured(name) {
                Some((sheet, r)) => Value::Ref(RefRange {
                    sheet,
                    start_row: r.start.row,
                    start_col: r.start.col,
                    end_row: r.end.row,
                    end_col: r.end.col,
                }),
                None => Value::Error(CellError::Name),
            };
        }
        // Look for a sheet-local name first, then workbook scope.
        let def = self
            .wb
            .defined_names
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(name) && d.scope == Some(self.current.sheet))
            .or_else(|| {
                self.wb
                    .defined_names
                    .iter()
                    .find(|d| d.name.eq_ignore_ascii_case(name) && d.scope.is_none())
            });
        let Some(def) = def else {
            // A bare table name resolves to its data-body range.
            if let Some((sheet, r)) = self.wb.resolve_structured(name) {
                return Value::Ref(RefRange {
                    sheet,
                    start_row: r.start.row,
                    start_col: r.start.col,
                    end_row: r.end.row,
                    end_col: r.end.col,
                });
            }
            return Value::Error(CellError::Name);
        };
        match super::parse::parse(&def.refers_to) {
            Ok(expr) => {
                self.depth += 1;
                let v = self.eval(&expr);
                self.depth -= 1;
                v
            }
            Err(_) => Value::Error(CellError::Name),
        }
    }

    fn eval_unary(&mut self, op: UnaryOp, expr: &Expr) -> Value {
        let v = self.eval(expr);
        // Array context: apply element-wise.
        if is_arrayish(&v) {
            let arr = self.materialize_bcast(v);
            let data = arr.data.into_iter().map(|e| scalar_unop(op, e)).collect();
            return Value::Array(Array::new(arr.rows, arr.cols, data));
        }
        let v = self.deref_scalar(v);
        scalar_unop(op, v)
    }

    fn eval_binary(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr) -> Value {
        // Reference operators are handled structurally before value coercion.
        match op {
            BinaryOp::Range => return self.eval_range_op(lhs, rhs),
            BinaryOp::Union => return self.eval_union_op(lhs, rhs),
            BinaryOp::Intersect => return self.eval_intersect_op(lhs, rhs),
            _ => {}
        }

        let l = self.eval(lhs);
        let r = self.eval(rhs);

        // Array context: if either side is an array or a multi-cell range, apply
        // the operator element-wise (broadcasting scalars / 1×n / m×1), yielding
        // an array. This is what makes `A1:A10>5` a boolean array — the input
        // FILTER/SORTBY need — and what lets `range*2` spill.
        if is_arrayish(&l) || is_arrayish(&r) {
            return self.broadcast_binary(op, l, r);
        }

        let l = self.deref_scalar(l);
        let r = self.deref_scalar(r);
        scalar_binop(op, l, r)
    }

    /// Apply a binary operator element-wise over array/range operands.
    fn broadcast_binary(&mut self, op: BinaryOp, l: Value, r: Value) -> Value {
        let la = self.materialize_bcast(l);
        let ra = self.materialize_bcast(r);
        let Some(rows) = bcast_dim(la.rows, ra.rows) else {
            return Value::Error(CellError::NA);
        };
        let Some(cols) = bcast_dim(la.cols, ra.cols) else {
            return Value::Error(CellError::NA);
        };
        let mut data = Vec::with_capacity(rows * cols);
        for i in 0..rows {
            for j in 0..cols {
                let a = la.data[bcast_idx(i, j, la.rows, la.cols)].clone();
                let b = ra.data[bcast_idx(i, j, ra.rows, ra.cols)].clone();
                data.push(scalar_binop(op, a, b));
            }
        }
        Value::Array(Array::new(rows, cols, data))
    }

    /// Materialize a value to a dense array for broadcasting (scalar → 1×1).
    fn materialize_bcast(&mut self, v: Value) -> Array {
        match v {
            Value::Array(a) => a,
            Value::Ref(r) => self.ref_to_array(r),
            other => Array::scalar(other),
        }
    }

    /// `A1:B2` style range building from two reference operands.
    fn eval_range_op(&mut self, lhs: &Expr, rhs: &Expr) -> Value {
        let l = self.eval(lhs);
        let r = self.eval(rhs);
        match (l, r) {
            (Value::Ref(a), Value::Ref(b)) if a.sheet == b.sheet => Value::Ref(RefRange {
                sheet: a.sheet,
                start_row: a.start_row.min(b.start_row),
                start_col: a.start_col.min(b.start_col),
                end_row: a.end_row.max(b.end_row),
                end_col: a.end_col.max(b.end_col),
            }),
            _ => Value::Error(CellError::Ref),
        }
    }

    fn eval_union_op(&mut self, lhs: &Expr, rhs: &Expr) -> Value {
        // A union of ranges; we materialize into a flat single-row array for the
        // common aggregate use-case (SUM((A1:A2,B1:B2))).
        let mut data = Vec::new();
        for e in [lhs, rhs] {
            let v = self.eval(e);
            data.extend(self.flatten(&v));
        }
        let len = data.len();
        Value::Array(Array::new(1, len, data))
    }

    fn eval_intersect_op(&mut self, lhs: &Expr, rhs: &Expr) -> Value {
        let l = self.eval(lhs);
        let r = self.eval(rhs);
        match (l, r) {
            (Value::Ref(a), Value::Ref(b)) if a.sheet == b.sheet => {
                let sr = a.start_row.max(b.start_row);
                let sc = a.start_col.max(b.start_col);
                let er = a.end_row.min(b.end_row);
                let ec = a.end_col.min(b.end_col);
                if sr > er || sc > ec {
                    Value::Error(CellError::Null)
                } else {
                    Value::Ref(RefRange {
                        sheet: a.sheet,
                        start_row: sr,
                        start_col: sc,
                        end_row: er,
                        end_col: ec,
                    })
                }
            }
            _ => Value::Error(CellError::Null),
        }
    }

    fn eval_array_const(&mut self, rows: &[Vec<Expr>]) -> Value {
        let nrows = rows.len();
        let ncols = rows.first().map(|r| r.len()).unwrap_or(0);
        let mut data = Vec::with_capacity(nrows * ncols);
        for row in rows {
            for e in row {
                let v = self.eval(e);
                data.push(self.deref_scalar(v));
            }
        }
        Value::Array(Array::new(nrows, ncols, data))
    }

    fn eval_func(&mut self, name: &str, args: &[Expr]) -> Value {
        let upper = name.trim_start_matches("_xlfn.").to_ascii_uppercase();
        // Lazy special forms first.
        match upper.as_str() {
            "IF" => return self.sf_if(args),
            "IFERROR" => return self.sf_iferror(args, false),
            "IFNA" => return self.sf_iferror(args, true),
            "CHOOSE" => return self.sf_choose(args),
            "IFS" => return self.sf_ifs(args),
            "SWITCH" => return self.sf_switch(args),
            "AND" => return self.sf_and_or(args, true),
            "OR" => return self.sf_and_or(args, false),
            "LAMBDA" => return self.sf_lambda(args),
            "LET" => return self.sf_let(args),
            "MAP" => return self.sf_map(args),
            "REDUCE" => return self.sf_reduce(args),
            "SCAN" => return self.sf_scan(args),
            "BYROW" => return self.sf_byrow(args, true),
            "BYCOL" => return self.sf_byrow(args, false),
            "MAKEARRAY" => return self.sf_makearray(args),
            "ISOMITTED" => return self.sf_isomitted(args),
            _ => {}
        }

        // A LET/lambda-bound name used as a function call: `f(x)`.
        if let Some((_, Value::Lambda(l), _)) = self.lookup_binding(&upper) {
            let lambda = l.clone();
            let argv: Vec<Value> = args.iter().map(|a| self.eval(a)).collect();
            return self.call_lambda(&lambda, argv);
        }

        let Some(entry) = self.registry.get(&upper) else {
            return Value::Error(CellError::Name);
        };
        if args.len() < entry.min_args || args.len() > entry.max_args {
            return Value::Error(CellError::Value);
        }
        let mut argv = Vec::with_capacity(args.len());
        for a in args {
            argv.push(self.eval(a));
        }
        // Most functions operate on values: a single-cell reference argument
        // (e.g. ABS(A1), ISNUMBER(A1)) must be dereferenced to the cell's scalar
        // value before the function runs. Range/aggregate functions are
        // unaffected — they receive multi-cell refs (left intact) and `flatten`
        // already accepts scalars. Functions that inspect a reference's location
        // or identity (ROW, OFFSET, …) opt out via `wants_reference`.
        if !wants_reference(&upper) {
            for v in &mut argv {
                if let Value::Ref(r) = v
                    && r.is_single()
                {
                    *v = self.cell(r.sheet, r.start_row, r.start_col);
                }
            }
        }
        (entry.func)(self, &argv)
    }

    // --- Lazy special forms ------------------------------------------------

    fn sf_if(&mut self, args: &[Expr]) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(CellError::Value);
        }
        let cond = self.eval(&args[0]);
        let cond = self.deref_scalar(cond);
        match coerce::to_bool(&cond) {
            Ok(true) => self.eval(&args[1]),
            Ok(false) => {
                if args.len() == 3 {
                    self.eval(&args[2])
                } else {
                    Value::Bool(false)
                }
            }
            Err(e) => Value::Error(e),
        }
    }

    fn sf_iferror(&mut self, args: &[Expr], only_na: bool) -> Value {
        if args.len() != 2 {
            return Value::Error(CellError::Value);
        }
        let v = self.eval(&args[0]);
        let v = self.deref_scalar(v);
        match &v {
            Value::Error(e) if !only_na || *e == CellError::NA => self.eval(&args[1]),
            _ => v,
        }
    }

    fn sf_choose(&mut self, args: &[Expr]) -> Value {
        if args.len() < 2 {
            return Value::Error(CellError::Value);
        }
        let idx = self.eval(&args[0]);
        let idx = self.deref_scalar(idx);
        let i = match coerce::to_number(&idx) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        };
        if i < 1 || (i as usize) >= args.len() {
            return Value::Error(CellError::Value);
        }
        self.eval(&args[i as usize])
    }

    fn sf_ifs(&mut self, args: &[Expr]) -> Value {
        if args.is_empty() || !args.len().is_multiple_of(2) {
            return Value::Error(CellError::Value);
        }
        let mut i = 0;
        while i < args.len() {
            let cond = self.eval(&args[i]);
            let cond = self.deref_scalar(cond);
            match coerce::to_bool(&cond) {
                Ok(true) => return self.eval(&args[i + 1]),
                Ok(false) => {}
                Err(e) => return Value::Error(e),
            }
            i += 2;
        }
        Value::Error(CellError::NA)
    }

    fn sf_switch(&mut self, args: &[Expr]) -> Value {
        if args.len() < 3 {
            return Value::Error(CellError::Value);
        }
        let subject = self.eval(&args[0]);
        let subject = self.deref_scalar(subject);
        let mut i = 1;
        while i + 1 < args.len() {
            let case = self.eval(&args[i]);
            let case = self.deref_scalar(case);
            if coerce::equal(&subject, &case) {
                return self.eval(&args[i + 1]);
            }
            i += 2;
        }
        // Trailing odd arg is the default.
        if i < args.len() {
            self.eval(&args[i])
        } else {
            Value::Error(CellError::NA)
        }
    }

    fn sf_and_or(&mut self, args: &[Expr], is_and: bool) -> Value {
        if args.is_empty() {
            return Value::Error(CellError::Value);
        }
        let mut seen = false;
        for a in args {
            let v = self.eval(a);
            for scalar in self.flatten(&v) {
                match scalar {
                    Value::Empty => continue,
                    Value::Text(_) => continue, // text in ranges ignored
                    Value::Error(e) => return Value::Error(e),
                    other => match coerce::to_bool(&other) {
                        Ok(b) => {
                            seen = true;
                            if is_and && !b {
                                return Value::Bool(false);
                            }
                            if !is_and && b {
                                return Value::Bool(true);
                            }
                        }
                        Err(e) => return Value::Error(e),
                    },
                }
            }
        }
        if !seen {
            return Value::Error(CellError::Value);
        }
        Value::Bool(is_and)
    }

    // --- LAMBDA & higher-order functions -----------------------------------

    /// `LAMBDA(param1, …, paramN, body)` → a callable function value.
    fn sf_lambda(&mut self, args: &[Expr]) -> Value {
        if args.is_empty() {
            return Value::Error(CellError::Value);
        }
        let (param_exprs, body) = args.split_at(args.len() - 1);
        let mut params = Vec::with_capacity(param_exprs.len());
        for p in param_exprs {
            match p {
                Expr::Name(n) => params.push(n.clone()),
                _ => return Value::Error(CellError::Value),
            }
        }
        Value::Lambda(Rc::new(Lambda {
            params,
            body: body[0].clone(),
        }))
    }

    /// Call a lambda with positional args (missing trailing args are "omitted").
    fn call_lambda(&mut self, lambda: &Lambda, mut args: Vec<Value>) -> Value {
        if args.len() > lambda.params.len() {
            return Value::Error(CellError::Value);
        }
        if self.depth > MAX_DEPTH {
            return Value::Error(CellError::Num);
        }
        let mut frame: Vec<Binding> = Vec::with_capacity(lambda.params.len());
        for (i, p) in lambda.params.iter().enumerate() {
            match args.get_mut(i) {
                Some(v) => frame.push((p.clone(), std::mem::replace(v, Value::Empty), false)),
                None => frame.push((p.clone(), Value::Empty, true)),
            }
        }
        self.scopes.push(frame);
        self.depth += 1;
        let r = self.eval(&lambda.body);
        self.depth -= 1;
        self.scopes.pop();
        r
    }

    /// `LET(name1, value1, …, calculation)` — bind names then evaluate the body.
    fn sf_let(&mut self, args: &[Expr]) -> Value {
        if args.len() < 3 || args.len().is_multiple_of(2) {
            return Value::Error(CellError::Value);
        }
        self.scopes.push(Vec::new());
        let pairs = args.len() - 1;
        let mut i = 0;
        let mut err: Option<CellError> = None;
        while i < pairs {
            let name = match &args[i] {
                Expr::Name(n) => n.clone(),
                _ => {
                    err = Some(CellError::Value);
                    break;
                }
            };
            let val = self.eval(&args[i + 1]);
            self.scopes.last_mut().unwrap().push((name, val, false));
            i += 2;
        }
        let result = match err {
            Some(e) => Value::Error(e),
            None => self.eval(&args[args.len() - 1]),
        };
        self.scopes.pop();
        result
    }

    /// Evaluate an argument expected to be a lambda value.
    fn eval_lambda_arg(&mut self, e: &Expr) -> Result<Rc<Lambda>, Value> {
        match self.eval(e) {
            Value::Lambda(l) => Ok(l),
            Value::Error(err) => Err(Value::Error(err)),
            _ => Err(Value::Error(CellError::Value)),
        }
    }

    /// `MAP(array1, …, lambda)` — apply the lambda element-wise across arrays.
    fn sf_map(&mut self, args: &[Expr]) -> Value {
        if args.len() < 2 {
            return Value::Error(CellError::Value);
        }
        let lambda = match self.eval_lambda_arg(&args[args.len() - 1]) {
            Ok(l) => l,
            Err(e) => return e,
        };
        let arrays: Vec<Array> = args[..args.len() - 1]
            .iter()
            .map(|a| {
                let v = self.eval(a);
                self.materialize_bcast(v)
            })
            .collect();
        let (rows, cols) = (arrays[0].rows, arrays[0].cols);
        if arrays.iter().any(|a| a.rows != rows || a.cols != cols) {
            return Value::Error(CellError::Value);
        }
        let mut data = Vec::with_capacity(rows * cols);
        for idx in 0..rows * cols {
            let call_args: Vec<Value> = arrays.iter().map(|a| a.data[idx].clone()).collect();
            let v = self.call_lambda(&lambda, call_args);
            data.push(self.deref_scalar(v));
        }
        Value::Array(Array::new(rows, cols, data))
    }

    /// `REDUCE(initial, array, lambda(acc, value))` — fold to a single value.
    fn sf_reduce(&mut self, args: &[Expr]) -> Value {
        if args.len() != 3 {
            return Value::Error(CellError::Value);
        }
        let init = self.eval(&args[0]);
        let mut acc = self.deref_scalar(init);
        let av = self.eval(&args[1]);
        let arr = self.materialize_bcast(av);
        let lambda = match self.eval_lambda_arg(&args[2]) {
            Ok(l) => l,
            Err(e) => return e,
        };
        for v in arr.data {
            let r = self.call_lambda(&lambda, vec![acc, v]);
            acc = self.deref_scalar(r);
        }
        acc
    }

    /// `SCAN(initial, array, lambda(acc, value))` — running fold, same shape.
    fn sf_scan(&mut self, args: &[Expr]) -> Value {
        if args.len() != 3 {
            return Value::Error(CellError::Value);
        }
        let init = self.eval(&args[0]);
        let mut acc = self.deref_scalar(init);
        let av = self.eval(&args[1]);
        let arr = self.materialize_bcast(av);
        let lambda = match self.eval_lambda_arg(&args[2]) {
            Ok(l) => l,
            Err(e) => return e,
        };
        let mut data = Vec::with_capacity(arr.data.len());
        for v in &arr.data {
            let r = self.call_lambda(&lambda, vec![acc.clone(), v.clone()]);
            acc = self.deref_scalar(r);
            data.push(acc.clone());
        }
        Value::Array(Array::new(arr.rows, arr.cols, data))
    }

    /// `BYROW(array, lambda(row))` (or BYCOL) — apply per row/column.
    fn sf_byrow(&mut self, args: &[Expr], by_row: bool) -> Value {
        if args.len() != 2 {
            return Value::Error(CellError::Value);
        }
        let av = self.eval(&args[0]);
        let arr = self.materialize_bcast(av);
        let lambda = match self.eval_lambda_arg(&args[1]) {
            Ok(l) => l,
            Err(e) => return e,
        };
        if by_row {
            let mut data = Vec::with_capacity(arr.rows);
            for r in 0..arr.rows {
                let row: Vec<Value> = (0..arr.cols)
                    .map(|c| arr.data[r * arr.cols + c].clone())
                    .collect();
                let v = self.call_lambda(&lambda, vec![Value::Array(Array::new(1, arr.cols, row))]);
                data.push(self.deref_scalar(v));
            }
            Value::Array(Array::new(arr.rows, 1, data))
        } else {
            let mut data = Vec::with_capacity(arr.cols);
            for c in 0..arr.cols {
                let col: Vec<Value> = (0..arr.rows)
                    .map(|r| arr.data[r * arr.cols + c].clone())
                    .collect();
                let v = self.call_lambda(&lambda, vec![Value::Array(Array::new(arr.rows, 1, col))]);
                data.push(self.deref_scalar(v));
            }
            Value::Array(Array::new(1, arr.cols, data))
        }
    }

    /// `MAKEARRAY(rows, cols, lambda(r, c))` — build an array from a lambda.
    fn sf_makearray(&mut self, args: &[Expr]) -> Value {
        if args.len() != 3 {
            return Value::Error(CellError::Value);
        }
        let rv = self.eval(&args[0]);
        let cv = self.eval(&args[1]);
        let rows = match coerce::to_number(&self.deref_scalar(rv)) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        };
        let cols = match coerce::to_number(&self.deref_scalar(cv)) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        };
        if rows < 1 || cols < 1 {
            return Value::Error(CellError::Value);
        }
        let lambda = match self.eval_lambda_arg(&args[2]) {
            Ok(l) => l,
            Err(e) => return e,
        };
        let mut data = Vec::with_capacity((rows * cols) as usize);
        for r in 1..=rows {
            for c in 1..=cols {
                let v = self.call_lambda(
                    &lambda,
                    vec![Value::Number(r as f64), Value::Number(c as f64)],
                );
                data.push(self.deref_scalar(v));
            }
        }
        Value::Array(Array::new(rows as usize, cols as usize, data))
    }

    /// `ISOMITTED(param)` — TRUE when a lambda parameter was not supplied.
    fn sf_isomitted(&mut self, args: &[Expr]) -> Value {
        if args.len() != 1 {
            return Value::Error(CellError::Value);
        }
        if let Expr::Name(n) = &args[0]
            && let Some((_, _, omitted)) = self.lookup_binding(n)
        {
            return Value::Bool(*omitted);
        }
        Value::Bool(false)
    }

    /// Reduce a value to a scalar for arithmetic/logical context: a single-cell
    /// reference reads its cell; a range applies implicit intersection against
    /// the current row/column, falling back to `#VALUE!` when it can't.
    pub fn deref_scalar(&mut self, v: Value) -> Value {
        match v {
            Value::Ref(r) => {
                if r.is_single() {
                    self.cell(r.sheet, r.start_row, r.start_col)
                } else {
                    // Implicit intersection with the formula's position.
                    self.implicit_intersection(r)
                }
            }
            Value::Array(a) => a.data.into_iter().next().unwrap_or(Value::Empty),
            other => other,
        }
    }

    fn implicit_intersection(&mut self, r: RefRange) -> Value {
        let cur = self.current;
        // Single row spanning current column.
        if r.start_row == r.end_row && cur.col >= r.start_col && cur.col <= r.end_col {
            return self.cell(r.sheet, r.start_row, cur.col);
        }
        // Single column spanning current row.
        if r.start_col == r.end_col && cur.row >= r.start_row && cur.row <= r.end_row {
            return self.cell(r.sheet, cur.row, r.start_col);
        }
        Value::Error(CellError::Value)
    }
}

/// True if a value should drive element-wise (array) operator behavior: an
/// array, or a multi-cell range reference (a single cell is treated as scalar).
fn is_arrayish(v: &Value) -> bool {
    match v {
        Value::Array(_) => true,
        Value::Ref(r) => !r.is_single(),
        _ => false,
    }
}

/// Broadcast two dimension sizes: equal, or one is 1 (repeated). `None` if
/// incompatible.
fn bcast_dim(a: usize, b: usize) -> Option<usize> {
    if a == b {
        Some(a)
    } else if a == 1 {
        Some(b)
    } else if b == 1 {
        Some(a)
    } else {
        None
    }
}

/// Index into a (possibly 1-sized, broadcast) array dimension.
fn bcast_idx(i: usize, j: usize, rows: usize, cols: usize) -> usize {
    let r = if rows == 1 { 0 } else { i };
    let c = if cols == 1 { 0 } else { j };
    r * cols + c
}

/// Apply a unary operator to one already-dereferenced scalar.
fn scalar_unop(op: UnaryOp, v: Value) -> Value {
    if let Value::Error(e) = v {
        return Value::Error(e);
    }
    match op {
        UnaryOp::Plus => match coerce::to_number(&v) {
            Ok(n) => Value::Number(n),
            Err(e) => Value::Error(e),
        },
        UnaryOp::Neg => match coerce::to_number(&v) {
            Ok(n) => Value::Number(-n),
            Err(e) => Value::Error(e),
        },
        UnaryOp::Percent => match coerce::to_number(&v) {
            Ok(n) => Value::Number(n / 100.0),
            Err(e) => Value::Error(e),
        },
    }
}

/// Apply a (non-reference) binary operator to two already-dereferenced scalars.
fn scalar_binop(op: BinaryOp, l: Value, r: Value) -> Value {
    if let Value::Error(e) = l {
        return Value::Error(e);
    }
    if let Value::Error(e) = r {
        return Value::Error(e);
    }
    match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Pow => {
            let (a, b) = match (coerce::to_number(&l), coerce::to_number(&r)) {
                (Ok(a), Ok(b)) => (a, b),
                (Err(e), _) | (_, Err(e)) => return Value::Error(e),
            };
            let result = match op {
                BinaryOp::Add => a + b,
                BinaryOp::Sub => a - b,
                BinaryOp::Mul => a * b,
                BinaryOp::Div => {
                    if b == 0.0 {
                        return Value::Error(CellError::Div0);
                    }
                    a / b
                }
                BinaryOp::Pow => {
                    let p = a.powf(b);
                    if p.is_nan() {
                        return Value::Error(CellError::Num);
                    }
                    p
                }
                _ => unreachable!(),
            };
            if !result.is_finite() {
                return Value::Error(CellError::Num);
            }
            Value::Number(result)
        }
        BinaryOp::Concat => {
            let a = match coerce::to_text(&l) {
                Ok(s) => s,
                Err(e) => return Value::Error(e),
            };
            let b = match coerce::to_text(&r) {
                Ok(s) => s,
                Err(e) => return Value::Error(e),
            };
            Value::Text(a + &b)
        }
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            let ord = coerce::compare(&l, &r);
            use std::cmp::Ordering::*;
            let res = match op {
                BinaryOp::Eq => ord == Equal,
                BinaryOp::Ne => ord != Equal,
                BinaryOp::Lt => ord == Less,
                BinaryOp::Le => ord != Greater,
                BinaryOp::Gt => ord == Greater,
                BinaryOp::Ge => ord != Less,
                _ => unreachable!(),
            };
            Value::Bool(res)
        }
        // Reference operators are handled before scalar coercion.
        BinaryOp::Range | BinaryOp::Union | BinaryOp::Intersect => Value::Error(CellError::Value),
    }
}

impl Context for Evaluator<'_> {
    fn date_system(&self) -> crate::core::dates::DateSystem {
        self.wb.date_system
    }
    fn current(&self) -> CellRef {
        self.current
    }
    fn sheet_count(&self) -> usize {
        self.wb.sheets.len()
    }
    fn sheet_index(&self, name: &str) -> Option<usize> {
        self.wb.sheet_index(name)
    }
    fn sheet_name(&self, idx: usize) -> Option<String> {
        self.wb.sheets.get(idx).map(|s| s.name.clone())
    }
    fn cell(&mut self, sheet: usize, row: u32, col: u32) -> Value {
        Value::from_cell_value(
            self.wb
                .sheets
                .get(sheet)
                .map(|s| s.value(row, col))
                .unwrap_or_default(),
        )
    }
    fn now_serial(&self) -> f64 {
        self.now
    }
    fn today_serial(&self) -> f64 {
        self.today
    }
}
