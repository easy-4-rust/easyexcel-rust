/// 对应 Java：无直接对应对象；Rust 架构扩展。 Side-effects requested by [`BofRecordHandler`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BofAction {
    /// Reset workbook sheet cursor. (`TYPE_WORKBOOK`)
    ResetWorkbook,
    /// Ignore non-worksheet BOF.
    Ignore,
    /// Begin / skip a worksheet sheet.
    BeginWorksheet {
        /// Whether the matched sheet should be read (`ignoreRecord = false`).
        read_sheet: bool,
        /// Next `readSheetIndex` after this BOF.
        next_read_sheet_index: usize,
    },
}

