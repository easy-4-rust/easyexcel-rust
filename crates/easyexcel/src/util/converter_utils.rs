//! 对应 Java： com.alibaba.excel.util.ConverterUtils.

#![allow(dead_code)]

use std::any::TypeId;

use crate::core::excel_error::ExcelError;

/// Mirrors `com.alibaba.excel.util.ConverterUtils#convertToJavaObject`.
///
/// The Rust port performs cell-to-field conversion via the
/// `FromExcelCell` trait; this function is the Java-API-shaped anchor
/// returning an `Unsupported` error until wired in by the reader crate.
///
/// # Errors
///
/// 始终返回 [`ExcelError::Unsupported`]，提示改用 `FromExcelCell` trait。
pub fn convert_to_java_object(_source: &str, _target_type: TypeId) -> Result<String, ExcelError> {
    Err(ExcelError::Unsupported(
        "ConverterUtils.convertToJavaObject: use the FromExcelCell trait instead".to_owned(),
    ))
}

/// Mirrors `com.alibaba.excel.util.ConverterUtils#convertToStringMap`.
///
/// Converts a flat `(key, value)` iterator into a `HashMap<String, String>`,
/// the Rust analogue of the Java `Map<String, String>` produced by the
/// original helper.
#[must_use]
pub fn convert_to_string_map<'a, K, V, I>(entries: I) -> std::collections::HashMap<String, String>
where
    K: AsRef<str> + 'a,
    V: ToString + 'a,
    I: IntoIterator<Item = (&'a K, &'a V)>,
{
    entries
        .into_iter()
        .map(|(k, v)| (k.as_ref().to_owned(), v.to_string()))
        .collect()
}

/// Mirrors `com.alibaba.excel.util.ConverterUtils#defaultClassGeneric`.
#[must_use]
pub fn default_class_generic(_type_id: TypeId) -> Option<TypeId> {
    None
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn convert_to_java_object_reports_unsupported() {
        // 对应 Java：ConverterUtils.convertToJavaObject 尚未接入时返回明确错误
        let error = convert_to_java_object("x", TypeId::of::<String>()).expect_err("unsupported");
        assert!(error.to_string().contains("FromExcelCell"));
    }

    #[test]
    fn convert_to_string_map_flattens_entries() {
        // 对应 Java：ConverterUtils.convertToStringMap
        let entries = [("name".to_string(), 1_u32), ("age".to_string(), 2_u32)];
        let map = convert_to_string_map(entries.iter().map(|(k, v)| (k, v)));
        assert_eq!(map.get("name").map(String::as_str), Some("1"));
        assert_eq!(map.get("age").map(String::as_str), Some("2"));

        // 空迭代器
        let empty: std::iter::Empty<(&String, &u32)> = std::iter::empty();
        assert!(convert_to_string_map(empty).is_empty());
    }

    #[test]
    fn default_class_generic_returns_none() {
        // 对应 Java：defaultClassGeneric 默认实现
        assert_eq!(default_class_generic(TypeId::of::<String>()), None);
    }
}
