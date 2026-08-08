use crate::core::CellValue;

use super::JournalCellStyle;

/// Stateful journal 中一个 Handler 执行后的最终物理单元格。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JournalCell {
    pub(crate) value: CellValue,
    pub(crate) style: Option<JournalCellStyle>,
}

impl JournalCell {
    pub(crate) const fn plain(value: CellValue) -> Self {
        Self { value, style: None }
    }
}
