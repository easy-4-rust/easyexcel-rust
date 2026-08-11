//! Java CGLIB `BeanMap` 的 Rust 强类型替代。

use std::any::TypeId;
use std::collections::BTreeMap;

use crate::CellValue;

/// `ExcelRow` 的字段名视图。
///
/// 值来自实际 writer 使用的转换结果；声明类型独立保留，供 converter 与模板路径取得
/// Java `BeanMap.getPropertyType` 等价元数据。该类型是 Rust 惯用替代，不复制 CGLIB。
#[derive(Debug, Clone, PartialEq)]
pub struct BeanMap {
    values: BTreeMap<&'static str, CellValue>,
    field_types: BTreeMap<&'static str, Option<&'static str>>,
}

impl BeanMap {
    pub(crate) fn from_parts(
        values: BTreeMap<&'static str, CellValue>,
        field_types: BTreeMap<&'static str, Option<&'static str>>,
    ) -> Self {
        Self {
            values,
            field_types,
        }
    }

    /// 返回指定 Rust 字段名对应的转换后值。
    #[must_use]
    pub fn get(&self, field_name: &str) -> Option<&CellValue> {
        self.values.get(field_name)
    }

    /// 返回字段声明类型；手写 `ExcelRow` schema 可以不提供该信息。
    #[must_use]
    pub fn property_type(&self, field_name: &str) -> Option<&'static str> {
        self.field_types.get(field_name).copied().flatten()
    }

    /// 返回 Rust 可表达的字段声明类型身份。
    ///
    /// derive/schema 的稳定类型名继续用于诊断；内建标量在这里恢复为
    /// `TypeId`，供 Java `BeanMap.getPropertyType` 调用链选择 converter。
    #[must_use]
    pub fn property_type_id(&self, field_name: &str) -> Option<TypeId> {
        Some(match self.property_type(field_name)? {
            "String" | "std::string::String" | "alloc::string::String" => TypeId::of::<String>(),
            "bool" => TypeId::of::<bool>(),
            "i8" => TypeId::of::<i8>(),
            "i16" => TypeId::of::<i16>(),
            "i32" => TypeId::of::<i32>(),
            "i64" => TypeId::of::<i64>(),
            "u8" => TypeId::of::<u8>(),
            "u16" => TypeId::of::<u16>(),
            "u32" => TypeId::of::<u32>(),
            "u64" => TypeId::of::<u64>(),
            "f32" => TypeId::of::<f32>(),
            "f64" => TypeId::of::<f64>(),
            _ => return None,
        })
    }

    /// 按确定的字段名顺序遍历转换后值。
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &CellValue)> {
        self.values.iter().map(|(field, value)| (*field, value))
    }

    /// 返回已映射字段数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// 返回字段映射是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    fn sample_map() -> BeanMap {
        let mut values = BTreeMap::new();
        values.insert("name", CellValue::String("Alice".to_owned()));
        values.insert("age", CellValue::Int(30));
        let mut field_types = BTreeMap::new();
        field_types.insert("name", Some("String"));
        field_types.insert("age", Some("i32"));
        BeanMap::from_parts(values, field_types)
    }

    #[test]
    fn get_existing_and_missing() {
        let map = sample_map();
        assert!(map.get("name").is_some());
        assert!(map.get("missing").is_none());
    }

    #[test]
    fn property_type_known_and_unknown() {
        let map = sample_map();
        assert_eq!(map.property_type("name"), Some("String"));
        assert_eq!(map.property_type("missing"), None);
    }

    #[test]
    fn property_type_id_known_types() {
        let map = sample_map();
        assert_eq!(map.property_type_id("name"), Some(TypeId::of::<String>()));
        assert_eq!(map.property_type_id("age"), Some(TypeId::of::<i32>()));
        assert_eq!(map.property_type_id("missing"), None);
    }

    #[test]
    fn property_type_id_all_known_variants() {
        let mut values = BTreeMap::new();
        let mut field_types = BTreeMap::new();
        let types: &[(&str, &str)] = &[
            ("f_string", "String"),
            ("f_std_string", "std::string::String"),
            ("f_alloc_string", "alloc::string::String"),
            ("f_bool", "bool"),
            ("f_i8", "i8"),
            ("f_i16", "i16"),
            ("f_i32", "i32"),
            ("f_i64", "i64"),
            ("f_u8", "u8"),
            ("f_u16", "u16"),
            ("f_u32", "u32"),
            ("f_u64", "u64"),
            ("f_f32", "f32"),
            ("f_f64", "f64"),
            ("f_unknown", "CustomType"),
        ];
        for &(name, ty) in types {
            values.insert(name, CellValue::Empty);
            field_types.insert(name, Some(ty));
        }
        let map = BeanMap::from_parts(values, field_types);
        assert_eq!(
            map.property_type_id("f_string"),
            Some(TypeId::of::<String>())
        );
        assert_eq!(
            map.property_type_id("f_std_string"),
            Some(TypeId::of::<String>())
        );
        assert_eq!(
            map.property_type_id("f_alloc_string"),
            Some(TypeId::of::<String>())
        );
        assert_eq!(map.property_type_id("f_bool"), Some(TypeId::of::<bool>()));
        assert_eq!(map.property_type_id("f_i8"), Some(TypeId::of::<i8>()));
        assert_eq!(map.property_type_id("f_i16"), Some(TypeId::of::<i16>()));
        assert_eq!(map.property_type_id("f_i32"), Some(TypeId::of::<i32>()));
        assert_eq!(map.property_type_id("f_i64"), Some(TypeId::of::<i64>()));
        assert_eq!(map.property_type_id("f_u8"), Some(TypeId::of::<u8>()));
        assert_eq!(map.property_type_id("f_u16"), Some(TypeId::of::<u16>()));
        assert_eq!(map.property_type_id("f_u32"), Some(TypeId::of::<u32>()));
        assert_eq!(map.property_type_id("f_u64"), Some(TypeId::of::<u64>()));
        assert_eq!(map.property_type_id("f_f32"), Some(TypeId::of::<f32>()));
        assert_eq!(map.property_type_id("f_f64"), Some(TypeId::of::<f64>()));
        // 未知类型返回 None
        assert_eq!(map.property_type_id("f_unknown"), None);
    }

    #[test]
    fn property_type_with_none_field_type() {
        let mut values = BTreeMap::new();
        values.insert("x", CellValue::Empty);
        let mut field_types = BTreeMap::new();
        field_types.insert("x", None);
        let map = BeanMap::from_parts(values, field_types);
        assert_eq!(map.property_type("x"), None);
        assert_eq!(map.property_type_id("x"), None);
    }

    #[test]
    fn iter_returns_all_entries() {
        let map = sample_map();
        let entries: Vec<_> = map.iter().collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn len_and_is_empty() {
        let map = sample_map();
        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());
        let empty = BeanMap::from_parts(BTreeMap::new(), BTreeMap::new());
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn clone_and_eq() {
        let a = sample_map();
        let b = a.clone();
        assert_eq!(a, b);
    }
}
