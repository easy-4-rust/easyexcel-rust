/// 对应 Java：无直接对应对象；Rust 架构扩展。 Events synthesised by [`DummyRecordHandler`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DummyRecordEvent {
    /// Java `LastCellOfRowDummyRecord` — flush the current row.
    EndRow {
        /// Zero-based row index to emit.
        row: u32,
    },
    /// Java `MissingCellDummyRecord` — insert empty if absent.
    MissingCell(BlankCell),
}

