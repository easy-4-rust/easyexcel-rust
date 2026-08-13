//! 对应 Java：`com.alibaba.excel.annotation.write.style.HeadRowHeight`。

use crate::RowHeightProperty;

/// 表头行高声明；`-1` 表示自动高度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeadRowHeight {
    value: i16,
}
impl Default for HeadRowHeight {
    fn default() -> Self {
        Self { value: -1 }
    }
}
impl HeadRowHeight {
    /// 创建 Java 默认参数对象。
    #[must_use]
    pub const fn new() -> Self {
        Self { value: -1 }
    }
    /// 返回高度。
    #[must_use]
    pub const fn value(&self) -> i16 {
        self.value
    }
    /// 设置高度。
    pub const fn set_value(&mut self, value: i16) {
        self.value = value;
    }
    /// 转换为运行期属性。
    #[must_use]
    pub fn to_property(self) -> Option<RowHeightProperty> {
        u16::try_from(self.value).ok().map(RowHeightProperty::new)
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_returns_default_minus_one() {
        let h = HeadRowHeight::new();
        assert_eq!(h.value(), -1);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(HeadRowHeight::default(), HeadRowHeight::new());
    }

    #[test]
    fn setter_and_getter() {
        let mut h = HeadRowHeight::new();
        h.set_value(25);
        assert_eq!(h.value(), 25);
    }

    #[test]
    fn to_property_with_valid_value() {
        let mut h = HeadRowHeight::new();
        h.set_value(20);
        assert!(h.to_property().is_some());
    }

    #[test]
    fn to_property_negative_returns_none() {
        let h = HeadRowHeight::new();
        assert!(h.to_property().is_none());
    }

    #[test]
    fn copy_clone_eq() {
        let a = HeadRowHeight::new();
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
