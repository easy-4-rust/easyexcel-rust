/// 对应 Java：无直接对应对象；Rust 架构扩展。 A single stored cell. Mirrors the variants Excel persists; the cell's *style*
/// is held separately on the [`Sheet`] (a sparse map keyed by position) so that
/// blank-but-formatted cells round-trip.
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    /// Empty (but possibly styled — see [`Sheet::style_at`]).
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
    Error(CellError),
    /// A formula cell: the source expression plus its last cached value.
    Formula {
        expr: String,
        cached: CellValue,
    },
}

impl Cell {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 The scalar value of this cell (formula → cached value).
    #[must_use]
    pub fn value(&self) -> CellValue {
        match self {
            Cell::Empty => CellValue::Empty,
            Cell::Number(n) => CellValue::Number(*n),
            Cell::Text(s) => CellValue::Text(s.clone()),
            Cell::Bool(b) => CellValue::Bool(*b),
            Cell::Error(e) => CellValue::Error(*e),
            Cell::Formula { cached, .. } => cached.clone(),
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn from_value(v: CellValue) -> Cell {
        match v {
            CellValue::Empty => Cell::Empty,
            CellValue::Number(n) => Cell::Number(n),
            CellValue::Text(s) => Cell::Text(s),
            CellValue::Bool(b) => Cell::Bool(b),
            CellValue::Error(e) => Cell::Error(e),
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_formula(&self) -> bool {
        matches!(self, Cell::Formula { .. })
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn formula_text(&self) -> Option<&str> {
        match self {
            Cell::Formula { expr, .. } => Some(expr),
            _ => None,
        }
    }
}

#[cfg(test)]
mod cell_tests {
    use super::*;

    #[test]
    fn cell_value_returns_scalar() {
        assert_eq!(Cell::Empty.value(), CellValue::Empty);
        assert_eq!(Cell::Number(42.0).value(), CellValue::Number(42.0));
        assert_eq!(
            Cell::Text("hello".into()).value(),
            CellValue::Text("hello".into())
        );
        assert_eq!(Cell::Bool(true).value(), CellValue::Bool(true));
        assert_eq!(
            Cell::Error(CellError::Value).value(),
            CellValue::Error(CellError::Value)
        );
    }

    #[test]
    fn cell_formula_value_returns_cached() {
        let cell = Cell::Formula {
            expr: "=A1+B1".into(),
            cached: CellValue::Number(42.0),
        };
        assert_eq!(cell.value(), CellValue::Number(42.0));
    }

    #[test]
    fn from_value_roundtrips() {
        assert_eq!(Cell::from_value(CellValue::Empty), Cell::Empty);
        assert_eq!(Cell::from_value(CellValue::Number(1.0)), Cell::Number(1.0));
        assert_eq!(
            Cell::from_value(CellValue::Text("x".into())),
            Cell::Text("x".into())
        );
        assert_eq!(Cell::from_value(CellValue::Bool(false)), Cell::Bool(false));
        assert_eq!(
            Cell::from_value(CellValue::Error(CellError::Value)),
            Cell::Error(CellError::Value)
        );
    }

    #[test]
    fn is_empty_checks_variant() {
        assert!(Cell::Empty.is_empty());
        assert!(!Cell::Number(0.0).is_empty());
        assert!(!Cell::Text("".into()).is_empty());
    }

    #[test]
    fn is_formula_checks_variant() {
        assert!(!Cell::Empty.is_formula());
        let formula = Cell::Formula {
            expr: "=1".into(),
            cached: CellValue::Number(1.0),
        };
        assert!(formula.is_formula());
    }

    #[test]
    fn formula_text_returns_expr() {
        let formula = Cell::Formula {
            expr: "=SUM(A1:A10)".into(),
            cached: CellValue::Number(55.0),
        };
        assert_eq!(formula.formula_text(), Some("=SUM(A1:A10)"));
        assert_eq!(Cell::Empty.formula_text(), None);
        assert_eq!(Cell::Number(1.0).formula_text(), None);
    }
}

