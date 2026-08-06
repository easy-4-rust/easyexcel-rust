/// 对应 Java：无直接对应对象；Rust 架构扩展。 Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation `-x`.
    Neg,
    /// Unary plus `+x` (a no-op kept for round-trip).
    Plus,
    /// Postfix percent `x%` (divides by 100).
    Percent,
}

