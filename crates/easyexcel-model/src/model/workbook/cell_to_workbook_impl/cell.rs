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

