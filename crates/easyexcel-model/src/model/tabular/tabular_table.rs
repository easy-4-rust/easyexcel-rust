use crate::CellRange;

use super::TabularCell;

/// 对应 Java：无直接对应对象；Rust 架构扩展。一个可映射为工作表的二维表格。
#[derive(Debug, Clone, PartialEq)]
pub struct TabularTable {
    name: String,
    rows: Vec<Vec<TabularCell>>,
    merges: Vec<CellRange>,
}

impl TabularTable {
    /// 创建空表格。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rows: Vec::new(),
            merges: Vec::new(),
        }
    }

    /// 返回表格名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回所有行。
    #[must_use]
    pub fn rows(&self) -> &[Vec<TabularCell>] {
        &self.rows
    }

    /// 返回合并区域。
    #[must_use]
    pub fn merges(&self) -> &[CellRange] {
        &self.merges
    }

    /// 追加一行。
    pub fn push_row(&mut self, row: Vec<TabularCell>) {
        self.rows.push(row);
    }

    /// 追加一个合并区域。
    pub fn push_merge(&mut self, range: CellRange) {
        self.merges.push(range);
    }
}
