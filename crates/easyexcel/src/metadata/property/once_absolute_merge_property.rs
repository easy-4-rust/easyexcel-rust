//! 对应 Java：`com.alibaba.excel.metadata.property.OnceAbsoluteMergeProperty`.

/// 对应 Java：`OnceAbsoluteMergeProperty`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub const fn get_first_row_index(&self) -> i32 {
        self.first_row_index
    }
    /// Java `setFirstRowIndex`。
    pub const fn set_first_row_index(&mut self, value: i32) {
        self.first_row_index = value;
    }
    /// Java `getLastRowIndex`。
    #[must_use]
    pub const fn get_last_row_index(&self) -> i32 {
        self.last_row_index
    }
    /// Java `setLastRowIndex`。
    pub const fn set_last_row_index(&mut self, value: i32) {
        self.last_row_index = value;
    }
    /// Java `getFirstColumnIndex`。
    #[must_use]
    pub const fn get_first_column_index(&self) -> i32 {
        self.first_column_index
    }
    /// Java `setFirstColumnIndex`。
    pub const fn set_first_column_index(&mut self, value: i32) {
        self.first_column_index = value;
    }
    /// Java `getLastColumnIndex`。
    #[must_use]
    pub const fn get_last_column_index(&self) -> i32 {
        self.last_column_index
    }
    /// Java `setLastColumnIndex`。
    pub const fn set_last_column_index(&mut self, value: i32) {
        self.last_column_index = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_and_getters() {
        let prop = OnceAbsoluteMergeProperty::new(1, 5, 2, 8);
        assert_eq!(prop.get_first_row_index(), 1);
        assert_eq!(prop.get_last_row_index(), 5);
        assert_eq!(prop.get_first_column_index(), 2);
        assert_eq!(prop.get_last_column_index(), 8);
    }

    #[test]
    fn setters() {
        let mut prop = OnceAbsoluteMergeProperty::new(0, 0, 0, 0);
        prop.set_first_row_index(10);
        assert_eq!(prop.get_first_row_index(), 10);
        prop.set_last_row_index(20);
        assert_eq!(prop.get_last_row_index(), 20);
        prop.set_first_column_index(3);
        assert_eq!(prop.get_first_column_index(), 3);
        prop.set_last_column_index(7);
        assert_eq!(prop.get_last_column_index(), 7);
    }

    #[test]
    fn equality_and_hash() {
        let a = OnceAbsoluteMergeProperty::new(1, 2, 3, 4);
        let b = OnceAbsoluteMergeProperty::new(1, 2, 3, 4);
        assert_eq!(a, b);
        let c = OnceAbsoluteMergeProperty::new(0, 0, 0, 0);
        assert_ne!(a, c);
    }

    #[test]
    fn clone_and_debug() {
        let prop = OnceAbsoluteMergeProperty::new(1, 2, 3, 4);
        let cloned = prop;
        assert_eq!(prop, cloned);
        let text = format!("{:?}", prop);
        assert!(text.contains("OnceAbsoluteMergeProperty"));
    }
}
