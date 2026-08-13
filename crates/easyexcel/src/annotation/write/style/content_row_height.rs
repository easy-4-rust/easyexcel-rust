//! 对应 Java：`com.alibaba.excel.annotation.write.style.ContentRowHeight`。

use crate::RowHeightProperty;

/// 内容行高声明；`-1` 表示自动高度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentRowHeight {
    value: i16,
}
impl Default for ContentRowHeight {
    fn default() -> Self {
        Self { value: -1 }
    }
}
impl ContentRowHeight {
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
        let h = ContentRowHeight::new();
        assert_eq!(h.value(), -1);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(ContentRowHeight::default(), ContentRowHeight::new());
    }

    #[test]
    fn setter_and_getter() {
        let mut h = ContentRowHeight::new();
        h.set_value(30);
        assert_eq!(h.value(), 30);
    }

    #[test]
    fn to_property_with_valid_value() {
        let mut h = ContentRowHeight::new();
        h.set_value(20);
        assert!(h.to_property().is_some());
    }

    #[test]
    fn to_property_negative_returns_none() {
        let h = ContentRowHeight::new();
        assert!(h.to_property().is_none());
    }

    #[test]
    fn copy_clone_eq() {
        let a = ContentRowHeight::new();
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
