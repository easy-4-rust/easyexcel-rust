/// 对应 Java：无直接对应对象；Rust 架构扩展。 Outcome of [`LabelSstRecordHandler::process_label_sst`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelSstCell {
    /// Empty cell when the cache is missing or the index is absent.
    Empty {
        /// Zero-based row.
        row: u32,
        /// Zero-based column.
        column: usize,
    },
    /// Resolved shared-string cell.
    String {
        /// Zero-based row.
        row: u32,
        /// Zero-based column.
        column: usize,
        /// Resolved text (already trimmed when `auto_trim` was set).
        value: String,
    },
}

