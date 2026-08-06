//! The evaluator's internal value type. Richer than [`easyexcel_model::value::CellValue`]
//! because it also models arrays and live range references.

use std::rc::Rc;

use super::ast::Expr;
use easyexcel_model::error::CellError;
use easyexcel_model::value::CellValue;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 A value flowing through formula evaluation.
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

include!("value/lambda.rs");

include!("value/ref_range.rs");

include!("value/array.rs");

impl Value {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn from_cell_value(v: CellValue) -> Value {
        match v {
            CellValue::Empty => Value::Empty,
            CellValue::Number(n) => Value::Number(n),
            CellValue::Text(s) => Value::Text(s),
            CellValue::Bool(b) => Value::Bool(b),
            CellValue::Error(e) => Value::Error(e),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Collapse to a scalar [`CellValue`] for storage as a cached result. Arrays
    /// reduce to their top-left element; refs are not expected here.
    #[must_use]
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
                .map_or(CellValue::Empty, Value::to_cell_value),
            Value::Ref(_) => CellValue::Error(CellError::Value),
            // A bare lambda can't live in a cell.
            Value::Lambda(_) => CellValue::Error(CellError::Calc),
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn error(e: CellError) -> Value {
        Value::Error(e)
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 If this value is an error, return it.
    #[must_use]
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
