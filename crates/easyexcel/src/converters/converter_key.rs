//! 对应 Java：`com.alibaba.excel.converters.ConverterKeyBuild.ConverterKey`。

use std::any::TypeId;

use crate::core::enum_cell_data_type::CellDataType;

/// 强类型 converter 分派键。
///
/// Java 保存 `(Class<?>, CellDataTypeEnum)` 并规范化 primitive/boxed 类型；Rust 以唯一
/// `TypeId` 表示目标类型，无需再做装箱归一化。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConverterKey {
    rust_type: TypeId,
    cell_data_type: Option<CellDataType>,
}

impl ConverterKey {
    /// 从已擦除 Rust 类型和可选 Excel 单元格类型创建分派键。
    #[must_use]
    pub const fn new(rust_type: TypeId, cell_data_type: Option<CellDataType>) -> Self {
        Self {
            rust_type,
            cell_data_type,
        }
    }

    /// 为泛型 `T` 和可选 Excel 单元格类型创建分派键。
    #[must_use]
    pub fn of<T: 'static>(cell_data_type: Option<CellDataType>) -> Self {
        Self::new(TypeId::of::<T>(), cell_data_type)
    }

    /// 返回 Rust 目标类型身份。
    #[must_use]
    pub const fn rust_type(&self) -> TypeId {
        self.rust_type
    }

    /// 返回 Java `clazz` 的后端中立 `TypeId`。
    #[must_use]
    pub const fn get_clazz(&self) -> TypeId {
        self.rust_type
    }

    /// 设置 Java `clazz` 的后端中立 `TypeId`。
    pub const fn set_clazz(&mut self, value: TypeId) {
        self.rust_type = value;
    }

    /// 返回可选 Excel 单元格类型。
    #[must_use]
    pub const fn cell_data_type(&self) -> Option<CellDataType> {
        self.cell_data_type
    }

    /// 返回 Java `cellDataTypeEnum`。
    #[must_use]
    pub const fn get_cell_data_type_enum(&self) -> Option<CellDataType> {
        self.cell_data_type
    }

    /// 设置 Java `cellDataTypeEnum`。
    pub const fn set_cell_data_type_enum(&mut self, value: Option<CellDataType>) {
        self.cell_data_type = value;
    }
}
