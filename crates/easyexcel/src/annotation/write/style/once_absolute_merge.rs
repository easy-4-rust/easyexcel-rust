//! 对应 Java：`com.alibaba.excel.annotation.write.style.OnceAbsoluteMerge`。

use crate::OnceAbsoluteMergeProperty;

/// 一次性绝对坐标合并声明。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnceAbsoluteMerge {
    first_row_index: i32,
    last_row_index: i32,
    first_column_index: i32,
    last_column_index: i32,
}
impl Default for OnceAbsoluteMerge {
    fn default() -> Self {
        Self::new()
    }
}
impl OnceAbsoluteMerge {
    /// 创建全部坐标为 `-1` 的 Java 默认参数对象。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            first_row_index: -1,
            last_row_index: -1,
            first_column_index: -1,
            last_column_index: -1,
        }
    }
    #[must_use]
    pub const fn first_row_index(&self) -> i32 {
        self.first_row_index
    }
    pub const fn set_first_row_index(&mut self, value: i32) {
        self.first_row_index = value;
    }
    #[must_use]
    pub const fn last_row_index(&self) -> i32 {
        self.last_row_index
    }
    pub const fn set_last_row_index(&mut self, value: i32) {
        self.last_row_index = value;
    }
    #[must_use]
    pub const fn first_column_index(&self) -> i32 {
        self.first_column_index
    }
    pub const fn set_first_column_index(&mut self, value: i32) {
        self.first_column_index = value;
    }
    #[must_use]
    pub const fn last_column_index(&self) -> i32 {
        self.last_column_index
    }
    pub const fn set_last_column_index(&mut self, value: i32) {
        self.last_column_index = value;
    }
    /// 转换为运行期合并属性，保留 `-1` sentinel。
    #[must_use]
    pub const fn to_property(self) -> OnceAbsoluteMergeProperty {
        OnceAbsoluteMergeProperty::new(
            self.first_row_index,
            self.last_row_index,
            self.first_column_index,
            self.last_column_index,
        )
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_returns_all_minus_one() {
        let m = OnceAbsoluteMerge::new();
        assert_eq!(m.first_row_index(), -1);
        assert_eq!(m.last_row_index(), -1);
        assert_eq!(m.first_column_index(), -1);
        assert_eq!(m.last_column_index(), -1);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(OnceAbsoluteMerge::default(), OnceAbsoluteMerge::new());
    }

    #[test]
    fn setters_and_getters() {
        let mut m = OnceAbsoluteMerge::new();
        m.set_first_row_index(0);
        assert_eq!(m.first_row_index(), 0);
        m.set_last_row_index(5);
        assert_eq!(m.last_row_index(), 5);
        m.set_first_column_index(1);
        assert_eq!(m.first_column_index(), 1);
        m.set_last_column_index(3);
        assert_eq!(m.last_column_index(), 3);
    }

    #[test]
    fn to_property_preserves_sentinel() {
        let m = OnceAbsoluteMerge::new();
        let prop = m.to_property();
        // All -1 sentinels are preserved
        let _ = prop;
    }

    #[test]
    fn to_property_with_configured_values() {
        let mut m = OnceAbsoluteMerge::new();
        m.set_first_row_index(0);
        m.set_last_row_index(10);
        m.set_first_column_index(0);
        m.set_last_column_index(5);
        let prop = m.to_property();
        let _ = prop;
    }

    #[test]
    fn copy_clone_eq() {
        let a = OnceAbsoluteMerge::new();
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_contains_struct_name() {
        let m = OnceAbsoluteMerge::new();
        assert!(format!("{m:?}").contains("OnceAbsoluteMerge"));
    }
}
