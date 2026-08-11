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
