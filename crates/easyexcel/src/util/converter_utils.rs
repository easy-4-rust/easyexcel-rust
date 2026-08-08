//! 对应 Java： com.alibaba.excel.util.ConverterUtils.

#![allow(dead_code)]

use std::any::{Any, TypeId};

use crate::core::excel_error::ExcelError;

/// 对应 Java：com.alibaba.excel.util.ConverterUtils。 Mirrors `com.alibaba.excel.util.ConverterUtils#convertToJavaObject`.
///
/// The Rust port performs cell-to-field conversion via the
/// `FromExcelCell` trait; this function is the Java-API-shaped anchor
/// returning an `Unsupported` error until wired in by the reader crate.
///
/// # Errors
///
/// 始终返回 [`ExcelError::Unsupported`]，提示改用 `FromExcelCell` trait。
pub fn convert_to_java_object(source: &str, target_type: TypeId) -> Result<String, ExcelError> {
    if target_type == TypeId::of::<String>() || target_type == TypeId::of::<&'static str>() {
        return Ok(source.to_owned());
    }
    if target_type == TypeId::of::<bool>() {
        return source
            .parse::<bool>()
            .map(|value| value.to_string())
            .map_err(|_| ExcelError::Format(format!("cannot convert '{source}' to bool")));
    }
    macro_rules! numeric {
        ($type:ty) => {
            if target_type == TypeId::of::<$type>() {
                return source
                    .parse::<$type>()
                    .map(|value| value.to_string())
                    .map_err(|_| ExcelError::Format(format!(
                        "cannot convert '{source}' to {}",
                        std::any::type_name::<$type>()
                    )));
            }
        };
    }
    numeric!(i8); numeric!(i16); numeric!(i32); numeric!(i64);
    numeric!(u8); numeric!(u16); numeric!(u32); numeric!(u64);
    numeric!(f32); numeric!(f64);
    Err(ExcelError::Unsupported(format!(
        "ConverterUtils.convertToJavaObject has no converter for target TypeId {target_type:?}"
    )))
}

/// Java `convertToJavaObject(ReadCellData, ..., Class, ...)` 的后端中立动态对象入口。
///
/// # Errors
///
/// 源类型与目标类型不兼容时返回带单元格值的转换错误。
pub fn convert_read_cell_to_java_object(
    source: &crate::ReadCellData,
    target_type: TypeId,
) -> Result<Box<dyn Any>, ExcelError> {
    let text = source.string_value();
    if target_type == TypeId::of::<String>() {
        return Ok(Box::new(text.to_owned()));
    }
    if target_type == TypeId::of::<bool>() {
        return source.boolean_value()
            .map(|value| Box::new(value) as Box<dyn Any>)
            .ok_or_else(|| ExcelError::Format(format!("cannot convert '{text}' to bool")));
    }
    let number = source.number_value();
    macro_rules! decimal_target {
        ($type:ty, $method:ident) => {
            if target_type == TypeId::of::<$type>() {
                use bigdecimal::ToPrimitive;
                return number
                    .as_ref()
                    .and_then(|value| value.$method())
                    .map(|value| Box::new(value) as Box<dyn Any>)
                    .ok_or_else(|| ExcelError::Format(format!(
                        "cannot convert '{text}' to {}",
                        std::any::type_name::<$type>()
                    )));
            }
        };
    }
    decimal_target!(i8, to_i8); decimal_target!(i16, to_i16);
    decimal_target!(i32, to_i32); decimal_target!(i64, to_i64);
    decimal_target!(u8, to_u8); decimal_target!(u16, to_u16);
    decimal_target!(u32, to_u32); decimal_target!(u64, to_u64);
    decimal_target!(f32, to_f32); decimal_target!(f64, to_f64);
    Err(ExcelError::Unsupported(format!(
        "ConverterUtils.convertToJavaObject has no converter for target TypeId {target_type:?}"
    )))
}

/// 对应 Java：com.alibaba.excel.util.ConverterUtils。 Mirrors `com.alibaba.excel.util.ConverterUtils#convertToStringMap`.
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
    easyexcel_utils::map_utils::to_string_map(entries)
}

/// 对应 Java：com.alibaba.excel.util.ConverterUtils。 Mirrors `com.alibaba.excel.util.ConverterUtils#defaultClassGeneric`.
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
