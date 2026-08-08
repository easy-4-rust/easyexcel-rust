use super::JournalCell;

/// Stateful journal 中一行的最终物理单元格与 Handler 行高结果。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JournalRow {
    pub(crate) cells: Vec<JournalCell>,
    pub(crate) row_height: Option<u16>,
}

impl JournalRow {
    pub(crate) fn empty() -> Self {
        Self {
            cells: Vec::new(),
            row_height: None,
        }
    }
}
