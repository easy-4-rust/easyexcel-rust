//! 对应 Java：`com.alibaba.excel.util.ClassUtils.ContentPropertyKey`。

use std::any::TypeId;

/// 字段内容属性缓存键。
///
/// Rust 使用 `TypeId` 承载 Java `Class<?>` 的类型身份，并保留类、表头类和字段名共同
/// 参与相等性及哈希计算的语义。
/// 对应 Java：`ClassUtils.ContentPropertyKey`。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentPropertyKey {
    clazz: Option<TypeId>,
    head_class: Option<TypeId>,
    field_name: String,
}

impl ContentPropertyKey {
    /// 创建字段内容属性缓存键。
    ///
    /// `clazz` 是数据类型，`head_class` 是表头类型，`field_name` 是字段名。
    #[must_use]
    pub fn new(
        clazz: Option<TypeId>,
        head_class: Option<TypeId>,
        field_name: impl Into<String>,
    ) -> Self {
        Self {
            clazz,
            head_class,
            field_name: field_name.into(),
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

    /// 返回表头类型身份。
    #[must_use]
    pub const fn get_head_class(&self) -> Option<TypeId> {
        self.head_class
    }

    /// 设置表头类型身份。
    pub const fn set_head_class(&mut self, value: Option<TypeId>) {
        self.head_class = value;
    }

    /// 返回字段名。
    #[must_use]
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// 设置字段名。
    pub fn set_field_name(&mut self, value: impl Into<String>) {
        self.field_name = value.into();
    }
}
