//! 1:1 包路径镜像：Java `com.alibaba.excel.enums.HeadKindEnum`。
//!
//! 既有实现：`enum_head_kind.rs` → [`crate::HeadKind`]。

#![allow(unused_imports)]
/// Java `HeadKindEnum` 命名别名。
// Java 镜像 API 别名，保留以兼容 Java 命名。
#[allow(dead_code)]
pub type HeadKindEnum = crate::HeadKind;
