/// 对应 Java：无直接对应对象；Rust 架构扩展。 Collected bound-sheet entry (name + BOF position).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundSheetEntry {
    /// Sheet display name. (Java `BoundSheetRecord.getSheetname`)
    pub name: String,
    /// Absolute BOF file position used for ordering.
    pub bof_position: u32,
}

