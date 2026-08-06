//! The formula abstract syntax tree, produced by the parser and consumed by the
//! evaluator. This is a frozen interface shared across the engine.

use easyexcel_model::addr::CellAddress;
use easyexcel_model::error::CellError;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 A parsed formula expression.
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

include!("ast/sheet_spec.rs");

include!("ast/reference.rs");

include!("ast/unary_op.rs");

include!("ast/binary_op.rs");
