//! 对应 Java：`com.alibaba.excel.metadata.property.OnceAbsoluteMergeProperty`.

/// 对应 Java：`OnceAbsoluteMergeProperty`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnceAbsoluteMergeProperty {
    /// First row index. (Java `firstRowIndex`)
    pub first_row_index: i32,
    /// Last row index. (Java `lastRowIndex`)
    pub last_row_index: i32,
    /// First column index. (Java `firstColumnIndex`)
    pub first_column_index: i32,
    /// Last column index. (Java `lastColumnIndex`)
    pub last_column_index: i32,
}

impl OnceAbsoluteMergeProperty {
    /// Creates a `OnceAbsoluteMergeProperty`. (Java constructor)
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.OnceAbsoluteMergeProperty。
    pub const fn new(
        first_row_index: i32,
        last_row_index: i32,
        first_column_index: i32,
        last_column_index: i32,
    ) -> Self {
        Self {
            first_row_index,
            last_row_index,
            first_column_index,
            last_column_index,
        }
    }

    /// Java `getFirstRowIndex`。
    #[must_use]
    pub const fn get_first_row_index(&self) -> i32 { self.first_row_index }
    /// Java `setFirstRowIndex`。
    pub const fn set_first_row_index(&mut self, value: i32) { self.first_row_index = value; }
    /// Java `getLastRowIndex`。
    #[must_use]
    pub const fn get_last_row_index(&self) -> i32 { self.last_row_index }
    /// Java `setLastRowIndex`。
    pub const fn set_last_row_index(&mut self, value: i32) { self.last_row_index = value; }
    /// Java `getFirstColumnIndex`。
    #[must_use]
    pub const fn get_first_column_index(&self) -> i32 { self.first_column_index }
    /// Java `setFirstColumnIndex`。
    pub const fn set_first_column_index(&mut self, value: i32) { self.first_column_index = value; }
    /// Java `getLastColumnIndex`。
    #[must_use]
    pub const fn get_last_column_index(&self) -> i32 { self.last_column_index }
    /// Java `setLastColumnIndex`。
    pub const fn set_last_column_index(&mut self, value: i32) { self.last_column_index = value; }
}
