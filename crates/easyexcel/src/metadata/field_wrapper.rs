//! 对应 Java：`com.alibaba.excel.metadata.FieldWrapper`.

/// Runtime field metadata for one annotated model field.
///
/// Java stores a reflective `Field`. Rust stores the field name and header
/// labels because `#[derive(ExcelRow)]` resolves reflection at compile time.
///
/// Rust port of Java `FieldWrapper`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// 对应 Java：com.alibaba.excel.metadata.FieldWrapper。
pub struct FieldWrapper {
    /// Rust field name. (Java `field` / `fieldName`)
    pub field_name: String,
    /// Sheet header labels from `@ExcelProperty`. (Java `heads`)
    pub heads: Vec<String>,
}

impl FieldWrapper {
    /// 对应 Java：com.alibaba.excel.metadata.FieldWrapper。 Creates a field wrapper. (Java all-args constructor)
    #[must_use]
    pub fn new(field_name: impl Into<String>, heads: Vec<String>) -> Self {
        Self {
            field_name: field_name.into(),
            heads,
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.FieldWrapper。 Returns the field name. (Java `getFieldName()`)
    #[must_use]
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// 对应 Java：com.alibaba.excel.metadata.FieldWrapper。 Returns the configured header labels. (Java `getHeads()`)
    #[must_use]
    pub fn heads(&self) -> &[String] {
        &self.heads
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_accessors_and_default() {
        // 对应 Java：FieldWrapper 构造与 getter
        let wrapper = FieldWrapper::new("name", vec!["姓名".to_owned(), "英文名".to_owned()]);
        assert_eq!(wrapper.field_name(), "name");
        assert_eq!(wrapper.field_name, "name");
        assert_eq!(
            wrapper.heads(),
            &["姓名".to_owned(), "英文名".to_owned()][..]
        );
        assert_eq!(wrapper.heads.len(), 2);

        let empty = FieldWrapper::default();
        assert_eq!(empty.field_name(), "");
        assert!(empty.heads().is_empty());
    }
}
