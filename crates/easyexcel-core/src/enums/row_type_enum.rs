//! 1:1 包路径镜像：Java `com.alibaba.excel.enums.RowTypeEnum`。
//!
//! 既有实现：`enum_row_type.rs` → [`crate::RowType`]。

#![allow(unused_imports)]
/// Java `RowTypeEnum` 命名别名。
// Java 镜像 API 别名，保留以兼容 Java 命名。
#[allow(dead_code)]
pub type RowTypeEnum = crate::RowType;
