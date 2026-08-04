//! The formula abstract syntax tree, produced by the parser and consumed by the
//! evaluator. This is a frozen interface shared across the engine.

use crate::core::addr::CellAddress;
use crate::core::error::CellError;

/// A parsed formula expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal.
    Number(f64),
    /// A string literal.
    Text(String),
    /// A boolean literal (`TRUE`/`FALSE`).
    Bool(bool),
    /// An error literal (`#REF!`, …).
    Error(CellError),
    /// A cell or range reference (possibly cross-sheet / 3D).
    Ref(Reference),
    /// A defined name (named range / constant).
    Name(String),
    /// Unary prefix/postfix operator applied to an expression.
    Unary { op: UnaryOp, expr: Box<Expr> },
    /// Binary operator.
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// A function call: name (upper-cased by the parser) + arguments.
    Func { name: String, args: Vec<Expr> },
    /// An array constant `{1,2;3,4}` as rows of expressions.
    Array(Vec<Vec<Expr>>),
}

/// How the referenced sheet(s) are specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SheetSpec {
    /// No sheet qualifier — resolves to the formula's own sheet.
    Current,
    /// `SheetName!` qualifier.
    Name(String),
    /// 3D span `First:Last!` qualifier.
    Span(String, String),
}

/// A cell or rectangular range reference.
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub sheet: SheetSpec,
    pub start: CellAddress,
    /// `None` for a single cell; `Some` for an `A1:B2` range.
    pub end: Option<CellAddress>,
}

impl Reference {
    pub fn cell(sheet: SheetSpec, addr: CellAddress) -> Reference {
        Reference {
            sheet,
            start: addr,
            end: None,
        }
    }
    pub fn range(sheet: SheetSpec, start: CellAddress, end: CellAddress) -> Reference {
        Reference {
            sheet,
            start,
            end: Some(end),
        }
    }
    pub fn is_range(&self) -> bool {
        self.end.is_some()
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation `-x`.
    Neg,
    /// Unary plus `+x` (a no-op kept for round-trip).
    Plus,
    /// Postfix percent `x%` (divides by 100).
    Percent,
}

/// Binary operators, including the reference operators.
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
    /// Binding precedence (higher binds tighter). Used by the Pratt parser.
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

    /// True if the operator is left-associative (all of ours except `^`).
    pub fn left_assoc(self) -> bool {
        !matches!(self, BinaryOp::Pow)
    }
}
