//! 命名或未命名集合填充包装。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.fill.FillWrapper`

use crate::TemplateData;

/// 对应 Java：com.alibaba.excel.write.metadata.fill.FillWrapper。 Named or unnamed collection data corresponding to Java `EasyExcel`'s `FillWrapper`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FillWrapper {
    pub(crate) name: Option<String>,
    rows: Vec<TemplateData>,
}

impl FillWrapper {
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillWrapper。 Creates an unnamed collection for `{.field}` placeholders.
    #[must_use]
    pub fn new(rows: impl IntoIterator<Item = TemplateData>) -> Self {
        Self {
            name: None,
            rows: rows.into_iter().collect(),
        }
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillWrapper。 Creates a named collection for `{name.field}` placeholders.
    #[must_use]
    pub fn named(name: impl Into<String>, rows: impl IntoIterator<Item = TemplateData>) -> Self {
        Self {
            name: Some(name.into()),
            rows: rows.into_iter().collect(),
        }
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillWrapper。 Returns the optional collection prefix.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillWrapper。 Returns collection rows in fill order.
    #[must_use]
    pub fn rows(&self) -> &[TemplateData] {
        &self.rows
    }
}
