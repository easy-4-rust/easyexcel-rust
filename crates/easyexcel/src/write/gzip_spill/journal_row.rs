use super::JournalCell;
use crate::write::merge_range::MergeRange;

/// Stateful journal 中一行的最终物理单元格、Handler 行高及实际合并结果。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JournalRow {
    pub(crate) cells: Vec<JournalCell>,
    pub(crate) row_height: Option<u16>,
    pub(crate) merge_ranges: Vec<MergeRange>,
}

impl JournalRow {
    pub(crate) fn empty() -> Self {
        Self {
            cells: Vec::new(),
            row_height: None,
            merge_ranges: Vec::new(),
        }
    }
}
