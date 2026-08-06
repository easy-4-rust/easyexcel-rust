/// 对应 Java：无直接对应对象；Rust 架构扩展。 Raw fields carried by a BIFF `LabelSST` record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelSstReference {
    /// Zero-based row.
    pub row: u32,
    /// Zero-based column.
    pub column: usize,
    /// Shared-string table index.
    pub sst_index: usize,
}

