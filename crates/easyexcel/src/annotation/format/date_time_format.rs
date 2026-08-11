//! 对应 Java：`com.alibaba.excel.annotation.format.DateTimeFormat`。

use crate::{BooleanEnum, DateTimeFormatProperty};

/// 日期格式注解的运行期等价对象。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DateTimeFormat {
    value: String,
    use_1904windowing: BooleanEnum,
}

impl DateTimeFormat {
    /// 创建 Java 默认参数对象。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// 返回格式模式。
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
    /// 设置格式模式。
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
    /// 返回日期窗口三态开关。
    #[must_use]
    pub const fn use_1904windowing(&self) -> BooleanEnum {
        self.use_1904windowing
    }
    /// 设置日期窗口三态开关。
    pub const fn set_use_1904windowing(&mut self, value: BooleanEnum) {
        self.use_1904windowing = value;
    }
    /// 转换为写入/读取引擎属性。
    #[must_use]
    pub fn to_property(&self) -> DateTimeFormatProperty {
        DateTimeFormatProperty::new(&self.value, self.use_1904windowing.value().unwrap_or(false))
    }
}
