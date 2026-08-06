/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decoded inline-label cell produced by [`LabelRecordHandler`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelCell {
    /// Zero-based row. (Java `LabelRecord.getRow`)
    pub row: u32,
    /// Zero-based column. (Java `LabelRecord.getColumn`)
    pub column: usize,
    /// Label text (already trimmed when `auto_trim` was set).
    pub value: String,
}

