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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_none_clazz() {
        let key = FieldCacheKey::new(None);
        assert_eq!(key.get_clazz(), None);
        assert!(key.get_exclude_column_field_names().is_empty());
        assert!(key.get_exclude_column_indexes().is_empty());
        assert!(key.get_include_column_field_names().is_empty());
        assert!(key.get_include_column_indexes().is_empty());
    }

    #[test]
    fn new_with_type_id() {
        let tid = std::any::TypeId::of::<String>();
        let key = FieldCacheKey::new(Some(tid));
        assert_eq!(key.get_clazz(), Some(tid));
    }

    #[test]
    fn default_has_none_clazz() {
        let key = FieldCacheKey::default();
        assert_eq!(key.get_clazz(), None);
    }

    #[test]
    fn set_clazz_updates_value() {
        let tid = std::any::TypeId::of::<u32>();
        let mut key = FieldCacheKey::new(None);
        key.set_clazz(Some(tid));
        assert_eq!(key.get_clazz(), Some(tid));
        key.set_clazz(None);
        assert_eq!(key.get_clazz(), None);
    }

    #[test]
    fn exclude_column_field_names_roundtrip() {
        let mut key = FieldCacheKey::new(None);
        key.set_exclude_column_field_names(vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(key.get_exclude_column_field_names(), &["a", "b"]);
    }

    #[test]
    fn exclude_column_indexes_roundtrip() {
        let mut key = FieldCacheKey::new(None);
        key.set_exclude_column_indexes(vec![0, 2, 5]);
        assert_eq!(key.get_exclude_column_indexes(), &[0, 2, 5]);
    }

    #[test]
    fn include_column_field_names_roundtrip() {
        let mut key = FieldCacheKey::new(None);
        key.set_include_column_field_names(vec!["x".to_owned()]);
        assert_eq!(key.get_include_column_field_names(), &["x"]);
    }

    #[test]
    fn include_column_indexes_roundtrip() {
        let mut key = FieldCacheKey::new(None);
        key.set_include_column_indexes(vec![1, 3]);
        assert_eq!(key.get_include_column_indexes(), &[1, 3]);
    }

    #[test]
    fn partial_eq_reflexive() {
        let tid = std::any::TypeId::of::<i32>();
        let mut key = FieldCacheKey::new(Some(tid));
        key.set_include_column_indexes(vec![1]);
        assert_eq!(key, key.clone());
    }

    #[test]
    fn partial_eq_different_clazz() {
        let a = FieldCacheKey::new(Some(std::any::TypeId::of::<i32>()));
        let b = FieldCacheKey::new(Some(std::any::TypeId::of::<u32>()));
        assert_ne!(a, b);
    }

    #[test]
    fn partial_eq_different_include_fields() {
        let mut a = FieldCacheKey::new(None);
        let mut b = FieldCacheKey::new(None);
        a.set_include_column_field_names(vec!["a".to_owned()]);
        b.set_include_column_field_names(vec!["b".to_owned()]);
        assert_ne!(a, b);
    }

    #[test]
    fn debug_format_works() {
        let key = FieldCacheKey::new(None);
        let text = format!("{key:?}");
        assert!(text.contains("FieldCacheKey"));
    }

    #[test]
    fn hash_equal_keys_produce_same_hash() {
        use std::hash::{Hash, Hasher, DefaultHasher};
        let tid = std::any::TypeId::of::<String>();
        let mut a = FieldCacheKey::new(Some(tid));
        let mut b = FieldCacheKey::new(Some(tid));
        a.set_exclude_column_indexes(vec![0, 1]);
        b.set_exclude_column_indexes(vec![0, 1]);
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.hash(&mut ha);
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }
}
