#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。 无法用普通十进制表示的 IEEE 754 数值类别。
pub enum NonFiniteNumber {
    /// 非数字值。
    Nan,
    /// 正无穷。
    PositiveInfinity,
    /// 负无穷。
    NegativeInfinity,
}

