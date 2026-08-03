//! 对应 Java：`com.alibaba.excel.read.metadata.holder.ReadSheetHolder`.

/// 对应 Java：`ReadSheetHolder extends AbstractReadHolder`.
#[derive(Debug, Clone)]
pub struct ReadSheetHolder {
    /// Mirrors `ReadSheetHolder.sheetNo`.
    pub sheet_no: i32,
    /// Mirrors `ReadSheetHolder.sheetName`.
    pub sheet_name: String,
    /// Mirrors `ReadSheetHolder.rowIndex`.
    pub row_index: i32,
    /// Mirrors `ReadSheetHolder.ended`.
    pub ended: bool,
}

impl ReadSheetHolder {
    /// 对应 Java：`ReadSheetHolder(ReadSheet, ReadWorkbookHolder)`.
    pub fn new(sheet_no: i32, sheet_name: impl Into<String>) -> Self {
        Self {
            sheet_no,
            sheet_name: sheet_name.into(),
            row_index: -1,
            ended: false,
        }
    }
}
