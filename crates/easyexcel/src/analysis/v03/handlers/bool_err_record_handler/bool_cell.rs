/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decoded boolean cell produced by [`BoolErrRecordHandler`].
///
/// Java's handler only materialises the boolean branch via
/// `BoolErrRecord.getBooleanValue()` (error branch is not exposed separately).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolCell {
    /// Zero-based row.
    pub row: u32,
    /// Zero-based column.
    pub column: usize,
    /// Boolean value. (Java `getBooleanValue`)
    pub value: bool,
}

