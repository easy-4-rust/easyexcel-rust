//! 1:1 包路径镜像：Java `com.alibaba.excel.enums.HolderEnum`。
//!
//! Rust 既有类型名为 [`crate::Holder`]（见 `enum_holder.rs`）。

#![allow(unused_imports)]
/// Java `HolderEnum` 的命名别名。
// Java 镜像 API 别名，保留以兼容 Java 命名。
#[allow(dead_code)]
pub type HolderEnum = crate::Holder;
