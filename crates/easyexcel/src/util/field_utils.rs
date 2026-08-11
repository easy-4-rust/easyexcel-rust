//! 对应 Java： com.alibaba.excel.util.FieldUtils.
//!
//! Java uses reflection to resolve fields. Rust delegates the field-name algorithm to
//! `easyexcel-utils` and resolves fields from derive-generated [`ExcelRow`] metadata.

use std::any::{Any, TypeId};
use std::borrow::Cow;

use crate::core::{ExcelColumn, ExcelRow};
use crate::metadata::NullObject;
use crate::util::bean_map::BeanMap;

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

/// 返回 Java `FieldUtils.nullObjectClass` 的后端中立类型键。
#[must_use]
pub fn null_object_class() -> TypeId {
    TypeId::of::<NullObject>()
}

/// 返回动态字段值的 Rust 类型键；空值回退到 Java `NullObject.class` 的 Rust 载体。
#[must_use]
pub fn get_field_class(value: Option<&dyn Any>) -> TypeId {
    value.map_or_else(null_object_class, Any::type_id)
}

/// 返回字段声明类型；声明缺失时回退到运行时值类型。
///
/// 对应 Java：`FieldUtils#getFieldClass(Map, String, Object)`。Java 仅在
/// `Map` 实际为 CGLIB `BeanMap` 时读取属性类型；Rust 直接接收其强类型替代。
#[must_use]
pub fn get_field_class_from_map(
    data_map: Option<&BeanMap>,
    field_name: &str,
    value: Option<&dyn Any>,
) -> TypeId {
    data_map
        .and_then(|map| map.property_type_id(field_name))
        .unwrap_or_else(|| get_field_class(value))
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

    #[test]
    fn null_object_class_returns_consistent_type_id() {
        let a = null_object_class();
        let b = null_object_class();
        assert_eq!(a, b);
    }

    #[test]
    fn get_field_class_with_some_and_none() {
        let string_val = "hello".to_owned();
        let with_some = get_field_class(Some(&string_val));
        assert_eq!(with_some, TypeId::of::<String>());
        let with_none = get_field_class(None);
        assert_eq!(with_none, null_object_class());
    }

    #[test]
    fn get_field_class_from_map_with_none_map() {
        let val = 42_i32;
        let result = get_field_class_from_map(None, "field", Some(&val));
        assert_eq!(result, TypeId::of::<i32>());
    }

    #[test]
    fn get_field_class_from_map_with_none_value() {
        let result = get_field_class_from_map(None, "field", None);
        assert_eq!(result, null_object_class());
    }
}
