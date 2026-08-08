//! 对应 Java：`com.alibaba.excel.converters.ConverterKeyBuild`.

use std::any::TypeId;

use crate::core::enum_cell_data_type::CellDataType;

/// 对应 Java：com.alibaba.excel.converters.ConverterKeyBuild。 Strongly typed converter dispatch key.
///
/// Java stores `(Class<?>, CellDataTypeEnum)` and normalizes primitive classes
/// to their boxed equivalents. Rust has one `TypeId` for each scalar type, so
/// no primitive/boxed normalization step is necessary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConverterKey {
    rust_type: TypeId,
    cell_data_type: Option<CellDataType>,
}

impl ConverterKey {
    /// Builds a key from an already erased Rust type.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ConverterKeyBuild。
    pub const fn new(rust_type: TypeId, cell_data_type: Option<CellDataType>) -> Self {
        Self {
            rust_type,
            cell_data_type,
        }
    }

    /// 对应 Java：com.alibaba.excel.converters.ConverterKeyBuild。 Builds a key for `T` and an optional Excel cell type.
    #[must_use]
    pub fn of<T: 'static>(cell_data_type: Option<CellDataType>) -> Self {
        Self::new(TypeId::of::<T>(), cell_data_type)
    }

    /// Returns the Rust target type component.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ConverterKeyBuild。
    pub const fn rust_type(&self) -> TypeId {
        self.rust_type
    }

    /// Returns the optional Excel cell type component.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ConverterKeyBuild。
    pub const fn cell_data_type(&self) -> Option<CellDataType> {
        self.cell_data_type
    }
    /// Java Lombok `getCellDataTypeEnum`。
    #[must_use] pub const fn get_cell_data_type_enum(&self) -> Option<CellDataType> { self.cell_data_type }
    /// Java Lombok `setCellDataTypeEnum`。
    pub const fn set_cell_data_type_enum(&mut self, value: Option<CellDataType>) { self.cell_data_type = value; }
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
