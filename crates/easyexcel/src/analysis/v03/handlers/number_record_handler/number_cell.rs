/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decoded number cell produced by [`NumberRecordHandler`].
#[derive(Debug, Clone, PartialEq)]
pub struct NumberCell {
    /// Zero-based row. (Java `NumberRecord.getRow`)
    pub row: u32,
    /// Zero-based column. (Java `NumberRecord.getColumn`)
    pub column: usize,
    /// Raw IEEE value. (Java `NumberRecord.getValue`)
    pub value: f64,
    /// Format index from the format-tracking listener (may be 0 when unknown).
    pub format_index: u16,
}

