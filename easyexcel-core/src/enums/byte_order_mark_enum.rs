//! 1:1 包路径镜像：Java `com.alibaba.excel.enums.ByteOrderMarkEnum`。
//!
//! 既有实现：`enum_byte_order_mark.rs` → [`crate::ByteOrderMark`]。

#![allow(unused_imports)]
/// Java `ByteOrderMarkEnum` 命名别名。
// Java 镜像 API 别名，保留以兼容 Java 命名。
#[allow(dead_code)]
pub type ByteOrderMarkEnum = crate::ByteOrderMark;
