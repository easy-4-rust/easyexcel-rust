//! 对应 Java：`com.alibaba.excel.metadata.AbstractCell`.

use super::cell::Cell;

/// 对应 Java：com.alibaba.excel.metadata.AbstractCell。 Base cell coordinate holder.
///
/// Rust port of Java `AbstractCell implements Cell`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AbstractCell {
    /// Row index. (Java `rowIndex`)
    pub row_index: Option<i32>,
    /// Column index. (Java `columnIndex`)
    pub column_index: Option<i32>,
}

impl AbstractCell {
    /// Creates an empty cell coordinate. (Java default constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.AbstractCell。
    pub const fn new() -> Self {
        Self {
            row_index: None,
            column_index: None,
        }
    }

    /// Creates a cell coordinate with explicit indices. (Java setter chain)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.AbstractCell。
    pub const fn with_indices(row_index: i32, column_index: i32) -> Self {
        Self {
            row_index: Some(row_index),
            column_index: Some(column_index),
        }
    }

    /// 返回行下标。对应 Java：`AbstractCell#getRowIndex`。
    #[must_use]
    pub const fn get_row_index(&self) -> Option<i32> {
        self.row_index
    }

    /// 设置行下标。对应 Java：`AbstractCell#setRowIndex`。
    pub const fn set_row_index(&mut self, row_index: Option<i32>) {
        self.row_index = row_index;
    }

    /// 返回列下标。对应 Java：`AbstractCell#getColumnIndex`。
    #[must_use]
    pub const fn get_column_index(&self) -> Option<i32> {
        self.column_index
    }

    /// 设置列下标。对应 Java：`AbstractCell#setColumnIndex`。
    pub const fn set_column_index(&mut self, column_index: Option<i32>) {
        self.column_index = column_index;
    }
}

impl Cell for AbstractCell {
    fn row_index(&self) -> Option<i32> {
        self.row_index
    }

    fn column_index(&self) -> Option<i32> {
        self.column_index
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_with_indices_and_cell_trait() {
        // 对应 Java：AbstractCell 构造与 Cell 接口
        let empty = AbstractCell::new();
        assert_eq!(empty.row_index(), None);
        assert_eq!(empty.column_index(), None);
        assert_eq!(AbstractCell::default(), empty);

        let positioned = AbstractCell::with_indices(3, 7);
        assert_eq!(positioned.row_index(), Some(3));
        assert_eq!(positioned.column_index(), Some(7));
        assert_eq!(positioned.row_index, Some(3));
        assert_eq!(positioned.column_index, Some(7));
    }
}
