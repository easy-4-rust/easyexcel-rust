//! 对应 Java：`com.alibaba.excel.metadata.FieldWrapper`.

/// Runtime field metadata for one annotated model field.
///
/// Java stores a reflective `Field`. Rust stores the field name and header
/// labels because `#[derive(ExcelRow)]` resolves reflection at compile time.
///
/// Rust port of Java `FieldWrapper`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
/// 对应 Java：com.alibaba.excel.metadata.FieldWrapper。
pub struct FieldWrapper {
    /// Java 反射 Field 的后端中立标识；Rust 无运行时 `java.lang.reflect.Field`。
    pub field: Option<String>,
    /// Rust field name. (Java `field` / `fieldName`)
    pub field_name: String,
    /// Sheet header labels from `@ExcelProperty`. (Java `heads`)
    pub heads: Vec<String>,
}

impl FieldWrapper {
    /// 对应 Java：com.alibaba.excel.metadata.FieldWrapper。 Creates a field wrapper. (Java all-args constructor)
    #[must_use]
    pub fn new(field_name: impl Into<String>, heads: Vec<String>) -> Self {
        let field_name = field_name.into();
        Self {
            field: Some(field_name.clone()),
            field_name,
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

    /// 同时设置 Java `field` 与 `fieldName` 的常用 Rust 构造。
    #[must_use]
    pub fn with_field_name(field_name: impl Into<String>, heads: Vec<String>) -> Self {
        let field_name = field_name.into();
        Self {
            field: Some(field_name.clone()),
            field_name,
            heads,
        }
    }

    /// Java `getField` 的后端中立视图。
    #[must_use]
    pub fn get_field(&self) -> Option<&str> {
        self.field.as_deref()
    }
    /// Java `setField` 的后端中立映射。
    pub fn set_field(&mut self, value: Option<String>) {
        self.field = value;
    }
    /// Java `getFieldName` 别名。
    #[must_use]
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }
    /// Java `setFieldName`。
    pub fn set_field_name(&mut self, value: impl Into<String>) {
        self.field_name = value.into();
    }
    /// Java `getHeads` 别名。
    #[must_use]
    pub fn get_heads(&self) -> &[String] {
        &self.heads
    }
    /// Java `setHeads`。
    pub fn set_heads(&mut self, value: Vec<String>) {
        self.heads = value;
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
