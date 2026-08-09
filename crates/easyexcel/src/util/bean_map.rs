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
