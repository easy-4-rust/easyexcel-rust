//! 对应 Java：`com.alibaba.excel.annotation.write.style.ColumnWidth`。

use crate::ColumnWidthProperty;

/// 列宽声明；`-1` 表示使用默认列宽。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnWidth {
    value: i32,
}

impl Default for ColumnWidth {
    fn default() -> Self {
        Self { value: -1 }
    }
}

impl ColumnWidth {
    /// 创建 Java 默认参数对象。
    #[must_use]
    pub const fn new() -> Self {
        Self { value: -1 }
    }
    /// 返回列宽。
    #[must_use]
    pub const fn value(&self) -> i32 {
        self.value
    }
    /// 设置列宽。
    pub const fn set_value(&mut self, value: i32) {
        self.value = value;
    }
    /// 有效列宽转换为运行期属性；`-1` 返回 `None`。
    #[must_use]
    pub fn to_property(self) -> Option<ColumnWidthProperty> {
        u16::try_from(self.value).ok().map(ColumnWidthProperty::new)
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_returns_default_minus_one() {
        let w = ColumnWidth::new();
        assert_eq!(w.value(), -1);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(ColumnWidth::default(), ColumnWidth::new());
    }

    #[test]
    fn setter_and_getter() {
        let mut w = ColumnWidth::new();
        w.set_value(25);
        assert_eq!(w.value(), 25);
    }

    #[test]
    fn to_property_with_valid_value() {
        let mut w = ColumnWidth::new();
        w.set_value(20);
        assert!(w.to_property().is_some());
    }

    #[test]
    fn to_property_negative_returns_none() {
        let w = ColumnWidth::new();
        assert!(w.to_property().is_none());
    }

    #[test]
    fn to_property_overflow_returns_none() {
        let mut w = ColumnWidth::new();
        w.set_value(i32::MAX);
        assert!(w.to_property().is_none());
    }

    #[test]
    fn copy_clone_eq() {
        let a = ColumnWidth::new();
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
