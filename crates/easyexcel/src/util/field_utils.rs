//! 对应 Java： com.alibaba.excel.util.FieldUtils.
//!
//! Java uses reflection to resolve fields. Rust delegates the field-name algorithm to
//! `easyexcel-utils` and resolves fields from derive-generated [`ExcelRow`] metadata.

use std::borrow::Cow;

use crate::core::{ExcelColumn, ExcelRow};

/// 对应 Java：com.alibaba.excel.util.FieldUtils。 Mirrors `com.alibaba.excel.util.FieldUtils#resolveCglibFieldName`.
///
/// Java compares the first two characters and switches the first character's case when only
/// one is uppercase. The reusable string rule is implemented by `easyexcel-utils`.
#[must_use]
pub fn resolve_cglib_field_name(name: &str) -> Cow<'_, str> {
    easyexcel_utils::string_utils::resolve_cglib_field_name(name)
}

/// 对应 Java：com.alibaba.excel.util.FieldUtils。 Mirrors `com.alibaba.excel.util.FieldUtils#getField`.
///
/// Rust field access is resolved at compile time via `derive(ExcelRow)`.
#[must_use]
pub fn get_field<T: ExcelRow>(field_name: &str) -> Option<&'static ExcelColumn> {
    T::schema().iter().find(|column| column.field == field_name)
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[allow(dead_code)]
    #[derive(crate::ExcelRow)]
    struct MetadataRow {
        #[excel(name = "Name", index = 0)]
        name: String,
    }

    #[test]
    fn resolve_cglib_field_name_matches_java_cases() {
        // 对应 Java `FieldUtils.resolveCglibFieldName` Javadoc 示例。
        assert_eq!(resolve_cglib_field_name("name"), "name");
        assert_eq!(resolve_cglib_field_name("String2"), "string2");
        assert_eq!(resolve_cglib_field_name("sTring3"), "STring3");
        assert_eq!(resolve_cglib_field_name("STring4"), "STring4");
        assert_eq!(resolve_cglib_field_name("STRING5"), "STRING5");
    }

    #[test]
    fn get_field_uses_derive_schema() {
        // 对应 Java：字段查询改由 derive(ExcelRow) 的静态 schema 完成。
        assert_eq!(
            get_field::<MetadataRow>("name").map(|column| column.name),
            Some("Name")
        );
        assert!(get_field::<MetadataRow>("missing").is_none());
    }
}
