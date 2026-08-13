//! 对应 Java： com.alibaba.excel.util.ConverterUtils.

#![allow(dead_code)]

use std::any::{Any, TypeId};

use crate::core::excel_error::ExcelError;
use crate::{ConverterRegistry, ReadConverterContext};

/// 将已经格式化的文本转换为内建 Rust 标量的内部回退路径。
///
/// # Errors
///
/// 目标类型不是内建标量或文本内容无法解析时返回转换错误。
pub(crate) fn convert_text_to_rust_value(
    source: &str,
    target_type: TypeId,
) -> Result<String, ExcelError> {
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
                    .map_err(|_| {
                        ExcelError::Format(format!(
                            "cannot convert '{source}' to {}",
                            std::any::type_name::<$type>()
                        ))
                    });
            }
        };
    }
    numeric!(i8);
    numeric!(i16);
    numeric!(i32);
    numeric!(i64);
    numeric!(u8);
    numeric!(u16);
    numeric!(u32);
    numeric!(u64);
    numeric!(f32);
    numeric!(f64);
    Err(ExcelError::Unsupported(format!(
        "ConverterUtils.convertToJavaObject has no converter for target TypeId {target_type:?}"
    )))
}

/// Java 两个 `convertToJavaObject(...)` 重载的后端中立动态对象入口。
///
/// # Errors
///
/// 注册转换器失败，或默认转换无法表示目标类型时返回带单元格值的转换错误。
pub fn convert_to_java_object(
    source: &crate::ReadCellData,
    target_type: TypeId,
    converters: &ConverterRegistry,
    context: &ReadConverterContext<'_>,
) -> Result<Box<dyn Any>, ExcelError> {
    if let Some(value) = converters.convert_to_dynamic(target_type, context)? {
        return Ok(value);
    }
    convert_read_cell_without_registered_converter(source, target_type)
}

fn convert_read_cell_without_registered_converter(
    source: &crate::ReadCellData,
    target_type: TypeId,
) -> Result<Box<dyn Any>, ExcelError> {
    let text = source.string_value();
    if target_type == TypeId::of::<String>() {
        return Ok(Box::new(text.to_owned()));
    }
    if target_type == TypeId::of::<bool>() {
        return source
            .boolean_value()
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
                    .ok_or_else(|| {
                        ExcelError::Format(format!(
                            "cannot convert '{text}' to {}",
                            std::any::type_name::<$type>()
                        ))
                    });
            }
        };
    }
    decimal_target!(i8, to_i8);
    decimal_target!(i16, to_i16);
    decimal_target!(i32, to_i32);
    decimal_target!(i64, to_i64);
    decimal_target!(u8, to_u8);
    decimal_target!(u16, to_u16);
    decimal_target!(u32, to_u32);
    decimal_target!(u64, to_u64);
    decimal_target!(f32, to_f32);
    decimal_target!(f64, to_f64);
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

/// 返回 Java `defaultClassGeneric = String.class` 的后端中立类型身份。
#[must_use]
pub fn default_class_generic() -> TypeId {
    TypeId::of::<String>()
}

/// Java `convertToStringMap(Map<Integer, ReadCellData<?>>, AnalysisContext)` 的稀疏列语义。
///
/// 缺失列和 `EMPTY` 单元格显式写入 `None`，其余单元格使用已解析显示值。
pub fn convert_read_cells_to_string_map(
    cells: &std::collections::BTreeMap<usize, crate::ReadCellData>,
) -> Result<std::collections::BTreeMap<usize, Option<String>>, ExcelError> {
    let mut result = std::collections::BTreeMap::new();
    let mut index = 0usize;
    for (column, cell) in cells {
        while index < *column {
            result.insert(index, None);
            index = index.saturating_add(1);
        }
        let value = if cell.cell_type() == crate::CellDataType::Empty {
            None
        } else {
            Some(convert_text_to_rust_value(
                cell.string_value(),
                TypeId::of::<String>(),
            )?)
        };
        result.insert(*column, value);
        index = column.saturating_add(1);
    }
    Ok(result)
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn text_fallback_converts_supported_types_and_rejects_unknown_types() {
        assert_eq!(
            convert_text_to_rust_value("x", TypeId::of::<String>()),
            Ok("x".to_owned())
        );
        let error = convert_text_to_rust_value("x", TypeId::of::<()>())
            .expect_err("unknown target must fail");
        assert!(error.to_string().contains("no converter"));
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
    fn default_class_generic_is_string() {
        // 对应 Java：defaultClassGeneric 默认实现
        assert_eq!(default_class_generic(), TypeId::of::<String>());
    }

    #[test]
    fn text_fallback_converts_numeric_types() {
        // 对应 Java：convertTextToRustValue 各数值类型分支
        assert_eq!(
            convert_text_to_rust_value("42", TypeId::of::<i8>()).unwrap(),
            "42"
        );
        assert_eq!(
            convert_text_to_rust_value("1000", TypeId::of::<i16>()).unwrap(),
            "1000"
        );
        assert_eq!(
            convert_text_to_rust_value("100000", TypeId::of::<i32>()).unwrap(),
            "100000"
        );
        assert_eq!(
            convert_text_to_rust_value("9999999", TypeId::of::<i64>()).unwrap(),
            "9999999"
        );
        assert_eq!(
            convert_text_to_rust_value("255", TypeId::of::<u8>()).unwrap(),
            "255"
        );
        assert_eq!(
            convert_text_to_rust_value("65535", TypeId::of::<u16>()).unwrap(),
            "65535"
        );
        assert_eq!(
            convert_text_to_rust_value("100000", TypeId::of::<u32>()).unwrap(),
            "100000"
        );
        assert_eq!(
            convert_text_to_rust_value("9999999", TypeId::of::<u64>()).unwrap(),
            "9999999"
        );
        assert_eq!(
            convert_text_to_rust_value("3.14", TypeId::of::<f32>()).unwrap(),
            "3.14"
        );
        assert_eq!(
            convert_text_to_rust_value("3.14", TypeId::of::<f64>()).unwrap(),
            "3.14"
        );
    }

    #[test]
    fn text_fallback_converts_bool() {
        assert_eq!(
            convert_text_to_rust_value("true", TypeId::of::<bool>()).unwrap(),
            "true"
        );
        assert_eq!(
            convert_text_to_rust_value("false", TypeId::of::<bool>()).unwrap(),
            "false"
        );
        assert!(convert_text_to_rust_value("notbool", TypeId::of::<bool>()).is_err());
    }

    #[test]
    fn text_fallback_rejects_invalid_numeric() {
        assert!(convert_text_to_rust_value("abc", TypeId::of::<i32>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<i8>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<i16>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<i64>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<u8>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<u16>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<u32>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<u64>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<f32>()).is_err());
        assert!(convert_text_to_rust_value("abc", TypeId::of::<f64>()).is_err());
    }

    #[test]
    fn text_fallback_converts_str_type() {
        assert_eq!(
            convert_text_to_rust_value("hello", TypeId::of::<&'static str>()).unwrap(),
            "hello"
        );
    }

    #[test]
    fn convert_read_cells_to_string_map_empty() {
        let cells = std::collections::BTreeMap::new();
        let result = convert_read_cells_to_string_map(&cells).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn convert_read_cells_to_string_map_consecutive_columns() {
        let mut cells = std::collections::BTreeMap::new();
        cells.insert(0, crate::ReadCellData::from_string("Alice"));
        cells.insert(1, crate::ReadCellData::from_string("Bob"));
        cells.insert(2, crate::ReadCellData::from_string("Charlie"));
        let result = convert_read_cells_to_string_map(&cells).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[&0], Some("Alice".to_owned()));
        assert_eq!(result[&1], Some("Bob".to_owned()));
        assert_eq!(result[&2], Some("Charlie".to_owned()));
    }

    #[test]
    fn convert_read_cells_to_string_map_sparse_columns_fills_none() {
        let mut cells = std::collections::BTreeMap::new();
        cells.insert(0, crate::ReadCellData::from_string("first"));
        // Column 1 is missing
        cells.insert(2, crate::ReadCellData::from_string("third"));
        let result = convert_read_cells_to_string_map(&cells).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[&0], Some("first".to_owned()));
        assert_eq!(result[&1], None); // sparse column
        assert_eq!(result[&2], Some("third".to_owned()));
    }

    #[test]
    fn convert_read_cells_to_string_map_empty_cell() {
        let mut cells = std::collections::BTreeMap::new();
        cells.insert(0, crate::ReadCellData::from_string("val"));
        cells.insert(1, crate::ReadCellData::new_empty_instance(None, None));
        let result = convert_read_cells_to_string_map(&cells).unwrap();
        assert_eq!(result[&0], Some("val".to_owned()));
        // Empty cell type -> None
        assert_eq!(result[&1], None);
    }

    #[test]
    fn convert_read_cells_to_string_map_multiple_sparse_gaps() {
        let mut cells = std::collections::BTreeMap::new();
        cells.insert(0, crate::ReadCellData::from_string("a"));
        // 1, 2 missing
        cells.insert(3, crate::ReadCellData::from_string("d"));
        // 4 missing
        cells.insert(5, crate::ReadCellData::from_string("f"));
        let result = convert_read_cells_to_string_map(&cells).unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[&0], Some("a".to_owned()));
        assert_eq!(result[&1], None);
        assert_eq!(result[&2], None);
        assert_eq!(result[&3], Some("d".to_owned()));
        assert_eq!(result[&4], None);
        assert_eq!(result[&5], Some("f".to_owned()));
    }
}
