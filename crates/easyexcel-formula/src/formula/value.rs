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

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_model::error::CellError;
    use easyexcel_model::value::CellValue;

    // ── Value::from_cell_value ──────────────────────────────────────────

    #[test]
    fn from_cell_value_empty() {
        assert_eq!(Value::from_cell_value(CellValue::Empty), Value::Empty);
    }

    #[test]
    fn from_cell_value_number() {
        assert_eq!(
            Value::from_cell_value(CellValue::Number(42.0)),
            Value::Number(42.0)
        );
    }

    #[test]
    fn from_cell_value_text() {
        assert_eq!(
            Value::from_cell_value(CellValue::Text("hi".into())),
            Value::Text("hi".into())
        );
    }

    #[test]
    fn from_cell_value_bool() {
        assert_eq!(
            Value::from_cell_value(CellValue::Bool(true)),
            Value::Bool(true)
        );
    }

    #[test]
    fn from_cell_value_error() {
        assert_eq!(
            Value::from_cell_value(CellValue::Error(CellError::NA)),
            Value::Error(CellError::NA)
        );
    }

    // ── Value::to_cell_value ────────────────────────────────────────────

    #[test]
    fn to_cell_value_scalars() {
        assert_eq!(Value::Empty.to_cell_value(), CellValue::Empty);
        assert_eq!(Value::Number(5.0).to_cell_value(), CellValue::Number(5.0));
        assert_eq!(
            Value::Text("x".into()).to_cell_value(),
            CellValue::Text("x".into())
        );
        assert_eq!(Value::Bool(false).to_cell_value(), CellValue::Bool(false));
        assert_eq!(
            Value::Error(CellError::Ref).to_cell_value(),
            CellValue::Error(CellError::Ref)
        );
    }

    #[test]
    fn to_cell_value_array_uses_first() {
        let arr = Array::new(2, 2, vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        assert_eq!(Value::Array(arr).to_cell_value(), CellValue::Number(1.0));
    }

    #[test]
    fn to_cell_value_empty_array() {
        let arr = Array::new(0, 0, vec![]);
        assert_eq!(Value::Array(arr).to_cell_value(), CellValue::Empty);
    }

    #[test]
    fn to_cell_value_ref_errors() {
        let r = RefRange {
            sheet: 0,
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 0,
        };
        assert_eq!(Value::Ref(r).to_cell_value(), CellValue::Error(CellError::Value));
    }

    #[test]
    fn to_cell_value_lambda_errors() {
        use std::rc::Rc;
        let lambda = Lambda {
            params: vec![],
            body: Expr::Number(1.0),
        };
        assert_eq!(
            Value::Lambda(Rc::new(lambda)).to_cell_value(),
            CellValue::Error(CellError::Calc)
        );
    }

    // ── Value::is_error / as_error ──────────────────────────────────────

    #[test]
    fn is_error_variants() {
        assert!(Value::Error(CellError::NA).is_error());
        assert!(!Value::Number(1.0).is_error());
        assert!(!Value::Text("x".into()).is_error());
        assert!(!Value::Empty.is_error());
    }

    #[test]
    fn as_error_variants() {
        assert_eq!(
            Value::Error(CellError::Ref).as_error(),
            Some(CellError::Ref)
        );
        assert_eq!(Value::Number(1.0).as_error(), None);
    }

    // ── Value::error ────────────────────────────────────────────────────

    #[test]
    fn error_constructor() {
        assert_eq!(
            Value::error(CellError::Div0),
            Value::Error(CellError::Div0)
        );
    }

    // ── From impls ──────────────────────────────────────────────────────

    #[test]
    fn from_impls() {
        assert_eq!(Value::from(42.0_f64), Value::Number(42.0));
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from(String::from("hi")), Value::Text("hi".into()));
        assert_eq!(Value::from("hi"), Value::Text("hi".into()));
        assert_eq!(Value::from(CellError::NA), Value::Error(CellError::NA));
    }

    // ── Array ───────────────────────────────────────────────────────────

    #[test]
    fn array_new() {
        let a = Array::new(2, 3, vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
            Value::Number(6.0),
        ]);
        assert_eq!(a.rows, 2);
        assert_eq!(a.cols, 3);
        assert_eq!(a.data.len(), 6);
    }

    #[test]
    fn array_scalar() {
        let a = Array::scalar(Value::Number(42.0));
        assert_eq!(a.rows, 1);
        assert_eq!(a.cols, 1);
        assert_eq!(a.data[0], Value::Number(42.0));
    }

    #[test]
    fn array_from_rows() {
        let a = Array::from_rows(vec![
            vec![Value::Number(1.0), Value::Number(2.0)],
            vec![Value::Number(3.0), Value::Number(4.0)],
        ]);
        assert_eq!(a.rows, 2);
        assert_eq!(a.cols, 2);
        assert_eq!(a.get(0, 1), Some(&Value::Number(2.0)));
        assert_eq!(a.get(1, 0), Some(&Value::Number(3.0)));
    }

    #[test]
    fn array_from_rows_empty() {
        let a = Array::from_rows(vec![]);
        assert_eq!(a.rows, 0);
        assert_eq!(a.cols, 0);
    }

    #[test]
    fn array_get_out_of_bounds() {
        let a = Array::new(1, 1, vec![Value::Number(1.0)]);
        assert_eq!(a.get(0, 0), Some(&Value::Number(1.0)));
        assert_eq!(a.get(1, 0), None);
        assert_eq!(a.get(0, 1), None);
    }

    // ── RefRange ────────────────────────────────────────────────────────

    #[test]
    fn ref_range_single() {
        let r = RefRange::single(0, 5, 3);
        assert_eq!(r.sheet, 0);
        assert_eq!(r.start_row, 5);
        assert_eq!(r.start_col, 3);
        assert_eq!(r.end_row, 5);
        assert_eq!(r.end_col, 3);
        assert!(r.is_single());
    }

    #[test]
    fn ref_range_rows_cols() {
        let r = RefRange {
            sheet: 0,
            start_row: 1,
            start_col: 2,
            end_row: 5,
            end_col: 4,
        };
        assert_eq!(r.rows(), 5);
        assert_eq!(r.cols(), 3);
        assert!(!r.is_single());
    }

    #[test]
    fn ref_range_iter() {
        let r = RefRange {
            sheet: 0,
            start_row: 0,
            start_col: 0,
            end_row: 1,
            end_col: 1,
        };
        let cells: Vec<(u32, u32)> = r.iter().collect();
        assert_eq!(cells, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
    }

    #[test]
    fn ref_range_iter_single() {
        let r = RefRange::single(0, 3, 2);
        let cells: Vec<(u32, u32)> = r.iter().collect();
        assert_eq!(cells, vec![(3, 2)]);
    }

    // ── Lambda ──────────────────────────────────────────────────────────

    #[test]
    fn lambda_debug_and_partialeq() {
        let l1 = Lambda {
            params: vec!["x".into()],
            body: Expr::Name("x".into()),
        };
        let l2 = Lambda {
            params: vec!["x".into()],
            body: Expr::Name("x".into()),
        };
        assert_eq!(l1, l2);
        assert!(format!("{l1:?}").contains("Lambda"));
    }
}
