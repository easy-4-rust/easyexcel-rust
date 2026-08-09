//! 对应 Java：`com.alibaba.excel.metadata.FieldCache`.

use std::collections::BTreeMap;

use super::field_wrapper::FieldWrapper;

/// 对应 Java：com.alibaba.excel.metadata.FieldCache。 Cached, sorted model fields.
///
/// Rust port of Java `FieldCache`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct FieldCache {
    /// Fields sorted by column order, excluding ignored fields. (Java `sortedFieldMap`)
    pub sorted_field_map: BTreeMap<i32, FieldWrapper>,
    /// Fields that explicitly use `@ExcelProperty.index`. (Java `indexFieldMap`)
    pub index_field_map: BTreeMap<i32, FieldWrapper>,
}

impl FieldCache {
    /// 对应 Java：com.alibaba.excel.metadata.FieldCache。 Creates a field cache. (Java all-args constructor)
    #[must_use]
    pub fn new(
        sorted_field_map: BTreeMap<i32, FieldWrapper>,
        index_field_map: BTreeMap<i32, FieldWrapper>,
    ) -> Self {
        Self {
            sorted_field_map,
            index_field_map,
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.FieldCache。 Returns the sorted field map. (Java `getSortedFieldMap()`)
    #[must_use]
    pub fn sorted_field_map(&self) -> &BTreeMap<i32, FieldWrapper> {
        &self.sorted_field_map
    }

    /// 对应 Java：com.alibaba.excel.metadata.FieldCache。 Returns the index field map. (Java `getIndexFieldMap()`)
    #[must_use]
    pub fn index_field_map(&self) -> &BTreeMap<i32, FieldWrapper> {
        &self.index_field_map
    }

    /// Java `getSortedFieldMap` 别名。
    #[must_use] pub fn get_sorted_field_map(&self) -> &BTreeMap<i32, FieldWrapper> { &self.sorted_field_map }
    /// Java `setSortedFieldMap`。
    pub fn set_sorted_field_map(&mut self, value: BTreeMap<i32, FieldWrapper>) { self.sorted_field_map = value; }
    /// Java `getIndexFieldMap` 别名。
    #[must_use] pub fn get_index_field_map(&self) -> &BTreeMap<i32, FieldWrapper> { &self.index_field_map }
    /// Java `setIndexFieldMap`。
    pub fn set_index_field_map(&mut self, value: BTreeMap<i32, FieldWrapper>) { self.index_field_map = value; }
}
