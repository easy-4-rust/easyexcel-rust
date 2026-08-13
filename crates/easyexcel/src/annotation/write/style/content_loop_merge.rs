//! 对应 Java：`com.alibaba.excel.annotation.write.style.ContentLoopMerge`。

use crate::LoopMergeProperty;

/// 内容行循环合并声明。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentLoopMerge {
    each_row: i32,
    column_extend: i32,
}

impl Default for ContentLoopMerge {
    fn default() -> Self {
        Self {
            each_row: 1,
            column_extend: 1,
        }
    }
}

impl ContentLoopMerge {
    /// 创建 Java 默认参数对象。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            each_row: 1,
            column_extend: 1,
        }
    }
    /// 返回每组行数。
    #[must_use]
    pub const fn each_row(&self) -> i32 {
        self.each_row
    }
    /// 设置每组行数。
    pub const fn set_each_row(&mut self, value: i32) {
        self.each_row = value;
    }
    /// 返回横向扩展列数。
    #[must_use]
    pub const fn column_extend(&self) -> i32 {
        self.column_extend
    }
    /// 设置横向扩展列数。
    pub const fn set_column_extend(&mut self, value: i32) {
        self.column_extend = value;
    }
    /// 转换为运行期属性，拒绝 Java 中也无意义的负值和溢出。
    #[must_use]
    pub fn to_property(self) -> Option<LoopMergeProperty> {
        Some(LoopMergeProperty::new(
            u32::try_from(self.each_row).ok()?,
            u16::try_from(self.column_extend).ok()?,
        ))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_returns_defaults() {
        let m = ContentLoopMerge::new();
        assert_eq!(m.each_row(), 1);
        assert_eq!(m.column_extend(), 1);
    }

    #[test]
    fn default_trait_matches_new() {
        let m = ContentLoopMerge::default();
        assert_eq!(m, ContentLoopMerge::new());
    }

    #[test]
    fn setters_and_getters() {
        let mut m = ContentLoopMerge::new();
        m.set_each_row(5);
        assert_eq!(m.each_row(), 5);
        m.set_column_extend(3);
        assert_eq!(m.column_extend(), 3);
    }

    #[test]
    fn to_property_with_valid_values() {
        let mut m = ContentLoopMerge::new();
        m.set_each_row(10);
        m.set_column_extend(2);
        let prop = m.to_property();
        assert!(prop.is_some());
    }

    #[test]
    fn to_property_rejects_negative_each_row() {
        let mut m = ContentLoopMerge::new();
        m.set_each_row(-1);
        assert!(m.to_property().is_none());
    }

    #[test]
    fn to_property_rejects_negative_column_extend() {
        let mut m = ContentLoopMerge::new();
        m.set_column_extend(-1);
        assert!(m.to_property().is_none());
    }

    #[test]
    fn to_property_rejects_column_extend_overflow() {
        let mut m = ContentLoopMerge::new();
        m.set_column_extend(i32::MAX);
        // i32::MAX does not fit in u16
        assert!(m.to_property().is_none());
    }

    #[test]
    fn copy_clone_eq() {
        let a = ContentLoopMerge::new();
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_contains_struct_name() {
        let m = ContentLoopMerge::new();
        assert!(format!("{m:?}").contains("ContentLoopMerge"));
    }
}
