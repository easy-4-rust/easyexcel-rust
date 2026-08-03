//! 1:1 包路径镜像：Java `com.alibaba.excel.enums.CellExtraTypeEnum`。
//!
//! 既有实现：`enum_cell_extra_type.rs` → [`crate::CellExtraType`]。

#![allow(unused_imports)]
/// Java `CellExtraTypeEnum` 命名别名。
// Java 镜像 API 别名，保留以兼容 Java 命名。
#[allow(dead_code)]
pub type CellExtraTypeEnum = crate::CellExtraType;
