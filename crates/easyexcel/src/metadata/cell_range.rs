//! 对应 Java：`com.alibaba.excel.metadata.CellRange`.

/// Inclusive rectangular cell range.
///
/// Rust port of Java `CellRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 对应 Java：com.alibaba.excel.metadata.CellRange。
pub struct CellRange {
    /// First row index. (Java `firstRow`)
    pub first_row: i32,
    /// Last row index. (Java `lastRow`)
    pub last_row: i32,
    /// First column index. (Java `firstCol`)
    pub first_col: i32,
    /// Last column index. (Java `lastCol`)
    pub last_col: i32,
}

impl CellRange {
    /// Creates a cell range. (Java constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellRange。
    pub const fn new(first_row: i32, last_row: i32, first_col: i32, last_col: i32) -> Self {
        Self {
            first_row,
            last_row,
            first_col,
            last_col,
        }
    }

    /// Returns the first row index. (Java `getFirstRow()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellRange。
    pub const fn first_row(&self) -> i32 {
        self.first_row
    }

    /// Returns the last row index. (Java `getLastRow()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellRange。
    pub const fn last_row(&self) -> i32 {
        self.last_row
    }

    /// Returns the first column index. (Java `getFirstCol()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellRange。
    pub const fn first_col(&self) -> i32 {
        self.first_col
    }

    /// Returns the last column index. (Java `getLastCol()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellRange。
    pub const fn last_col(&self) -> i32 {
        self.last_col
    }

    /// Java `getFirstRow` 别名。
    #[must_use] pub const fn get_first_row(&self) -> i32 { self.first_row }
    /// Java `setFirstRow`。
    pub const fn set_first_row(&mut self, value: i32) { self.first_row = value; }
    /// Java `getLastRow` 别名。
    #[must_use] pub const fn get_last_row(&self) -> i32 { self.last_row }
    /// Java `setLastRow`。
    pub const fn set_last_row(&mut self, value: i32) { self.last_row = value; }
    /// Java `getFirstCol` 别名。
    #[must_use] pub const fn get_first_col(&self) -> i32 { self.first_col }
    /// Java `setFirstCol`。
    pub const fn set_first_col(&mut self, value: i32) { self.first_col = value; }
    /// Java `getLastCol` 别名。
    #[must_use] pub const fn get_last_col(&self) -> i32 { self.last_col }
    /// Java `setLastCol`。
    pub const fn set_last_col(&mut self, value: i32) { self.last_col = value; }
}
