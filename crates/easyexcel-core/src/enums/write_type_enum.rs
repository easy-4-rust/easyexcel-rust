//! 1:1 包路径镜像：Java `com.alibaba.excel.enums.WriteTypeEnum`。
//!
//! 既有实现：`enum_write_type.rs` → [`crate::WriteType`]。

#![allow(unused_imports)]
/// Java `WriteTypeEnum` 命名别名。
// Java 镜像 API 别名，保留以兼容 Java 命名。
#[allow(dead_code)]
pub type WriteTypeEnum = crate::WriteType;
