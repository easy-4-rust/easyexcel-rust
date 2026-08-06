/// 对应 Java：无直接对应对象；Rust 架构扩展。 Actions requested by [`EofRecordHandler`] at sheet EOF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EofAction {
    /// Ignore — sheet was skipped or stop was requested without stop-sheet.
    Ignore,
    /// Call `endSheet` because the user stopped the current sheet.
    EndSheetOnly,
    /// Forge a final row flush (non-empty cellMap) then `endSheet`.
    FlushRowThenEndSheet,
    /// Just `endSheet`.
    EndSheet,
}

