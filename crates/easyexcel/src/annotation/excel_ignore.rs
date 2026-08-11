//! 对应 Java：`com.alibaba.excel.annotation.ExcelIgnore`。

/// 忽略字段的零大小标记；derive 宏可把它作为运行期等价元数据保存。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExcelIgnore;

impl ExcelIgnore {
    /// 创建标记，对应 Java 无成员注解实例。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
