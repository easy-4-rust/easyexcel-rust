//! 1:1 包路径镜像：Java `com.alibaba.excel.enums.NumericCellTypeEnum`。
//!
//! 既有实现：`enum_numeric_cell_type.rs` → [`crate::NumericCellType`]。

#![allow(unused_imports)]
/// Java `NumericCellTypeEnum` 命名别名。
// Java 镜像 API 别名，保留以兼容 Java 命名。
#[allow(dead_code)]
pub type NumericCellTypeEnum = crate::NumericCellType;
