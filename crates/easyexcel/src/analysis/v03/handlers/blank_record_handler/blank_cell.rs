/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decoded blank-cell placement produced by [`BlankRecordHandler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlankCell {
    /// Zero-based row. (Java `BlankRecord.getRow`)
    pub row: u32,
    /// Zero-based column. (Java `BlankRecord.getColumn`)
    pub column: usize,
}

