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
