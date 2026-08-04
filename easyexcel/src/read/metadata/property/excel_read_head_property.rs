//! 对应 Java：`com.alibaba.excel.read.metadata.property.ExcelReadHeadProperty`.

use crate::metadata::configuration_holder::ConfigurationHolder;
use crate::metadata::property::excel_head_property::ExcelHeadProperty;

/// Read-side header metadata.
///
/// Rust port of Java `ExcelReadHeadProperty extends ExcelHeadProperty`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExcelReadHeadProperty(ExcelHeadProperty);

impl ExcelReadHeadProperty {
    /// Creates read-side header metadata. (Java constructor)
    #[must_use]
    pub fn new(
        configuration_holder: Option<&dyn ConfigurationHolder>,
        head_clazz: Option<String>,
        head: Option<Vec<Vec<String>>>,
    ) -> Self {
        let property = if let Some(head_clazz) = head_clazz {
            ExcelHeadProperty::for_class(configuration_holder, head_clazz, head)
        } else {
            ExcelHeadProperty::new(configuration_holder, head)
        };
        Self(property)
    }

    /// Returns the underlying header property. (Java inherited getters)
    #[must_use]
    pub fn inner(&self) -> &ExcelHeadProperty {
        &self.0
    }

    /// Returns whether any header is configured. (Java `hasHead()`)
    #[must_use]
    pub fn has_head(&self) -> bool {
        self.0.has_head()
    }

    /// Returns the header map. (Java `getHeadMap()`)
    #[must_use]
    pub fn head_map(&self) -> &std::collections::BTreeMap<i32, crate::metadata::head::Head> {
        self.0.head_map()
    }
}

impl std::ops::Deref for ExcelReadHeadProperty {
    type Target = ExcelHeadProperty;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::HeadKind;

    #[test]
    fn new_without_class_and_inner_accessor() {
        // 对应 Java：ExcelReadHeadProperty 无类名构造与 inner
        let property = ExcelReadHeadProperty::new(None, None, Some(vec![vec!["Name".to_owned()]]));
        assert!(property.has_head());
        assert_eq!(property.inner().head_map().len(), 1);
        assert_eq!(property.head_map().len(), 1);
        // Deref 到 ExcelHeadProperty
        let inner: &ExcelHeadProperty = &property;
        assert_eq!(inner.head_map().len(), 1);
    }

    #[test]
    fn new_with_class_uses_for_class() {
        // 对应 Java：指定类名时按类解析表头
        let property = ExcelReadHeadProperty::new(None, Some("Model".to_owned()), None);
        // Java：类字段元数据应用后 headKind 恒为 CLASS
        assert!(property.has_head());
        assert!(property.inner().head_map().is_empty());
        assert_eq!(property.inner().head_kind(), HeadKind::Class);
    }
}
