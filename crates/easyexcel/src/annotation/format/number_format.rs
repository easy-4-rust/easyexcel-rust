//! 对应 Java：`com.alibaba.excel.annotation.format.NumberFormat`。

use crate::{NumberFormatProperty, NumberRoundingMode};

/// 数字格式注解的运行期等价对象。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NumberFormat {
    value: String,
    rounding_mode: NumberRoundingMode,
}

impl NumberFormat {
    /// 创建 Java 默认参数对象（`HALF_UP`）。
    #[must_use]
    pub fn new() -> Self { Self::default() }
    /// 返回格式模式。
    #[must_use]
    pub fn value(&self) -> &str { &self.value }
    /// 设置格式模式。
    pub fn set_value(&mut self, value: impl Into<String>) { self.value = value.into(); }
    /// 返回舍入模式。
    #[must_use]
    pub const fn rounding_mode(&self) -> NumberRoundingMode { self.rounding_mode }
    /// 设置舍入模式。
    pub const fn set_rounding_mode(&mut self, value: NumberRoundingMode) { self.rounding_mode = value; }
    /// 转换为引擎属性。
    #[must_use]
    pub fn to_property(&self) -> NumberFormatProperty {
        NumberFormatProperty::new(&self.value, self.rounding_mode)
    }
}
