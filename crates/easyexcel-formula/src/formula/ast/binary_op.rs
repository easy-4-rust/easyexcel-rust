/// 对应 Java：无直接对应对象；Rust 架构扩展。 Binary operators, including the reference operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    /// Text concatenation `&`.
    Concat,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    /// Range operator `:` (forms a range from two references).
    Range,
    /// Intersection operator ` ` (space).
    Intersect,
    /// Union operator `,`.
    Union,
}

impl BinaryOp {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Binding precedence (higher binds tighter). Used by the Pratt parser.
    #[must_use]
    pub fn precedence(self) -> u8 {
        match self {
            BinaryOp::Range => 9,
            BinaryOp::Intersect => 8,
            BinaryOp::Union => 7,
            BinaryOp::Pow => 5,
            BinaryOp::Mul | BinaryOp::Div => 4,
            BinaryOp::Add | BinaryOp::Sub => 3,
            BinaryOp::Concat => 2,
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => 1,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 True if the operator is left-associative (all of ours except `^`).
    #[must_use]
    pub fn left_assoc(self) -> bool {
        !matches!(self, BinaryOp::Pow)
    }
}

