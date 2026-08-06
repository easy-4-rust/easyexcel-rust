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
            use std::cmp::Ordering::{Equal, Greater, Less};
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
    fn date_system(&self) -> easyexcel_model::dates::DateSystem {
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
