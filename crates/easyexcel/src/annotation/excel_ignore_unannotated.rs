//! 对应 Java：`com.alibaba.excel.annotation.ExcelIgnoreUnannotated`。

/// 忽略类型中所有未声明 Excel 元数据字段的零大小标记。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExcelIgnoreUnannotated;

impl ExcelIgnoreUnannotated {
    /// 创建标记，对应 Java 无成员注解实例。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
