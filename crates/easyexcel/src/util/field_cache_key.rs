//! 对应 Java：`com.alibaba.excel.util.ClassUtils.FieldCacheKey`。

use std::any::TypeId;

/// 解析字段表时使用的包含和排除缓存键。
///
/// Rust 使用 `TypeId` 替代 Java `Class<?>`，并完整保留字段名、列索引过滤条件参与
/// 相等性及哈希计算的语义。
/// 对应 Java：`ClassUtils.FieldCacheKey`。
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FieldCacheKey {
    clazz: Option<TypeId>,
    exclude_column_field_names: Vec<String>,
    exclude_column_indexes: Vec<usize>,
    include_column_field_names: Vec<String>,
    include_column_indexes: Vec<usize>,
}

impl FieldCacheKey {
    /// 创建指定数据类型的字段缓存键。
    #[must_use]
    pub fn new(clazz: Option<TypeId>) -> Self {
        Self {
            clazz,
            ..Self::default()
        }
    }

    /// 返回数据类型身份。
    #[must_use]
    pub const fn get_clazz(&self) -> Option<TypeId> {
        self.clazz
    }

    /// 设置数据类型身份。
    pub const fn set_clazz(&mut self, value: Option<TypeId>) {
        self.clazz = value;
    }

    /// 返回被排除的字段名。
    #[must_use]
    pub fn get_exclude_column_field_names(&self) -> &[String] {
        &self.exclude_column_field_names
    }

    /// 设置被排除的字段名。
    pub fn set_exclude_column_field_names(&mut self, value: Vec<String>) {
        self.exclude_column_field_names = value;
    }

    /// 返回被排除的列索引。
    #[must_use]
    pub fn get_exclude_column_indexes(&self) -> &[usize] {
        &self.exclude_column_indexes
    }

    /// 设置被排除的列索引。
    pub fn set_exclude_column_indexes(&mut self, value: Vec<usize>) {
        self.exclude_column_indexes = value;
    }

    /// 返回被包含的字段名。
    #[must_use]
    pub fn get_include_column_field_names(&self) -> &[String] {
        &self.include_column_field_names
    }

    /// 设置被包含的字段名。
    pub fn set_include_column_field_names(&mut self, value: Vec<String>) {
        self.include_column_field_names = value;
    }

    /// 返回被包含的列索引。
    #[must_use]
    pub fn get_include_column_indexes(&self) -> &[usize] {
        &self.include_column_indexes
    }

    /// 设置被包含的列索引。
    pub fn set_include_column_indexes(&mut self, value: Vec<usize>) {
        self.include_column_indexes = value;
    }
}
