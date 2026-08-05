//! The evaluator's internal value type. Richer than [`easyexcel_model::value::CellValue`]
//! because it also models arrays and live range references.

use std::rc::Rc;

use super::ast::Expr;
use easyexcel_model::error::CellError;
use easyexcel_model::value::CellValue;

/// A value flowing through formula evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
    Error(CellError),
    /// A 2D array (from array constants, range materialization, or array results).
    Array(Array),
    /// A live reference to a rectangular range on a concrete sheet. Functions
    /// that need reference semantics (ROW, COLUMN, OFFSET, INDEX, …) consume this
    /// before it is coerced to a scalar/array.
    Ref(RefRange),
    /// A first-class function value (from `LAMBDA`), callable by LET-bound name
    /// or by the higher-order functions (MAP, REDUCE, BYROW, …).
    Lambda(Rc<Lambda>),
}

/// A `LAMBDA(param1, …, body)` closure: parameter names and the body expression.
#[derive(Debug, PartialEq)]
pub struct Lambda {
    pub params: Vec<String>,
    pub body: Expr,
}

/// A concrete, resolved range reference (sheet index already looked up).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefRange {
    pub sheet: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

impl RefRange {
    pub fn single(sheet: usize, row: u32, col: u32) -> Self {
        RefRange {
            sheet,
            start_row: row,
            start_col: col,
            end_row: row,
            end_col: col,
        }
    }
    pub fn rows(&self) -> u32 {
        self.end_row - self.start_row + 1
    }
    pub fn cols(&self) -> u32 {
        self.end_col - self.start_col + 1
    }
    pub fn is_single(&self) -> bool {
        self.start_row == self.end_row && self.start_col == self.end_col
    }
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        let (r0, r1, c0, c1) = (self.start_row, self.end_row, self.start_col, self.end_col);
        (r0..=r1).flat_map(move |r| (c0..=c1).map(move |c| (r, c)))
    }
}

/// A dense 2D array of scalar values (row-major). Used for array constants and
/// CSE array results. Elements are never themselves arrays or refs.
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Value>,
}

impl Array {
    pub fn new(rows: usize, cols: usize, data: Vec<Value>) -> Self {
        debug_assert_eq!(rows * cols, data.len());
        Array { rows, cols, data }
    }

    pub fn scalar(v: Value) -> Self {
        Array {
            rows: 1,
            cols: 1,
            data: vec![v],
        }
    }

    pub fn from_rows(rows: Vec<Vec<Value>>) -> Self {
        let nrows = rows.len();
        let ncols = rows.first().map(|r| r.len()).unwrap_or(0);
        let data = rows.into_iter().flatten().collect();
        Array {
            rows: nrows,
            cols: ncols,
            data,
        }
    }

    pub fn get(&self, r: usize, c: usize) -> Option<&Value> {
        if r < self.rows && c < self.cols {
            self.data.get(r * self.cols + c)
        } else {
            None
        }
    }
}

impl Value {
    pub fn from_cell_value(v: CellValue) -> Value {
        match v {
            CellValue::Empty => Value::Empty,
            CellValue::Number(n) => Value::Number(n),
            CellValue::Text(s) => Value::Text(s),
            CellValue::Bool(b) => Value::Bool(b),
            CellValue::Error(e) => Value::Error(e),
        }
    }

    /// Collapse to a scalar [`CellValue`] for storage as a cached result. Arrays
    /// reduce to their top-left element; refs are not expected here.
    pub fn to_cell_value(&self) -> CellValue {
        match self {
            Value::Empty => CellValue::Empty,
            Value::Number(n) => CellValue::Number(*n),
            Value::Text(s) => CellValue::Text(s.clone()),
            Value::Bool(b) => CellValue::Bool(*b),
            Value::Error(e) => CellValue::Error(*e),
            Value::Array(a) => a
                .data
                .first()
                .map(|v| v.to_cell_value())
                .unwrap_or(CellValue::Empty),
            Value::Ref(_) => CellValue::Error(CellError::Value),
            // A bare lambda can't live in a cell.
            Value::Lambda(_) => CellValue::Error(CellError::Calc),
        }
    }

    pub fn error(e: CellError) -> Value {
        Value::Error(e)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }

    /// If this value is an error, return it.
    pub fn as_error(&self) -> Option<CellError> {
        match self {
            Value::Error(e) => Some(*e),
            _ => None,
        }
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}
impl From<CellError> for Value {
    fn from(e: CellError) -> Self {
        Value::Error(e)
    }
}
