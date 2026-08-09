//! 对应 Java：`com.alibaba.excel.converters.ConverterKeyBuild`.

use std::any::TypeId;

use crate::core::enum_cell_data_type::CellDataType;

// 保留历史公开路径，嵌套 public 类型的真实实现由独立对象文件承载。
pub use super::converter_key::ConverterKey;

/// 使用 Java `buildKey(Class<?>)` 的后端中立形状构建 key。
#[must_use]
pub const fn build_key_for_type(rust_type: TypeId) -> ConverterKey {
    ConverterKey::new(rust_type, None)
}

/// 使用 Java `buildKey(Class<?>, CellDataTypeEnum)` 的后端中立形状构建 key。
#[must_use]
pub const fn build_key_for_type_and_cell_data(
    rust_type: TypeId,
    cell_data_type: Option<CellDataType>,
) -> ConverterKey {
    ConverterKey::new(rust_type, cell_data_type)
}

/// 对应 Java：com.alibaba.excel.converters.ConverterKeyBuild。 Builds Java's `(Class, CellDataTypeEnum)` key for Rust type `T`.
#[must_use]
pub fn build_key<T: 'static>(cell_data_type: Option<CellDataType>) -> ConverterKey {
    ConverterKey::of::<T>(cell_data_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converter_key_uses_both_rust_and_excel_types() {
        assert_eq!(
            build_key::<i32>(Some(CellDataType::Number)),
            ConverterKey::of::<i32>(Some(CellDataType::Number))
        );
        assert_ne!(
            build_key::<i32>(Some(CellDataType::Number)),
            build_key::<i32>(Some(CellDataType::String))
        );
        assert_ne!(
            build_key::<i32>(Some(CellDataType::Number)),
            build_key::<i64>(Some(CellDataType::Number))
        );
        assert_eq!(
            build_key::<i32>(None).cell_data_type(),
            None,
            "unqualified keys mirror Java's default-write converter key"
        );
    }
}
