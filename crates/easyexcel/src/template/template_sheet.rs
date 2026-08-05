//! 模板填充与写入的目标工作表选择。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.WriteSheet`

/// Worksheet selected for Java-style template fill and write operations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TemplateSheet {
    /// Selects a worksheet by its zero-based workbook order.
    #[default]
    First,
    /// Selects a worksheet by its zero-based workbook order.
    Index(usize),
    /// Selects a worksheet by its exact workbook name.
    Name(String),
}

impl TemplateSheet {
    /// Selects the first worksheet, equivalent to Java `writerSheet().build()`.
    #[must_use]
    pub const fn first() -> Self {
        Self::First
    }

    /// Selects a worksheet by Java-style zero-based sheet number.
    #[must_use]
    pub const fn index(index: usize) -> Self {
        Self::Index(index)
    }

    /// Selects a worksheet by exact name.
    #[must_use]
    pub fn name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }
}
