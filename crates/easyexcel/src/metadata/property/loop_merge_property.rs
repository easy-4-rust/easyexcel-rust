//! 对应 Java：`com.alibaba.excel.metadata.property.LoopMergeProperty`.

/// 对应 Java：`LoopMergeProperty`. (Java `eachRow: int`, `columnExtend: int`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LoopMergeProperty {
    /// Each row. (Java `eachRow`)
    pub each_row: u32,
    /// Extend column. (Java `columnExtend`)
    pub column_extend: u16,
}

impl LoopMergeProperty {
    /// Creates a `LoopMergeProperty`. (Java constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.LoopMergeProperty。
    pub const fn new(each_row: u32, column_extend: u16) -> Self {
        Self {
            each_row,
            column_extend,
        }
    }
    /// Returns `eachRow`. (Java `getEachRow()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.LoopMergeProperty。
    pub const fn each_row(&self) -> u32 {
        self.each_row
    }
    #[must_use] pub const fn get_each_row(&self) -> u32 { self.each_row() }
    /// Java `setEachRow` 的非负运行期映射。
    pub const fn set_each_row(&mut self, value: u32) { self.each_row = value; }
    /// Returns `columnExtend`. (Java `getColumnExtend()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.LoopMergeProperty。
    pub const fn column_extend(&self) -> u16 {
        self.column_extend
    }
    #[must_use] pub const fn get_column_extend(&self) -> u16 { self.column_extend() }
    /// Java `setColumnExtend` 的已校验运行期映射。
    pub const fn set_column_extend(&mut self, value: u16) { self.column_extend = value; }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_and_accessors() {
        // 对应 Java：LoopMergeProperty 构造与 getter
        let property = LoopMergeProperty::new(2, 3);
        assert_eq!(property.each_row, 2);
        assert_eq!(property.column_extend, 3);
        assert_eq!(property.each_row(), 2);
        assert_eq!(property.column_extend(), 3);
    }
}
