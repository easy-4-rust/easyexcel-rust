//! 1:1 包路径镜像：Java `com.alibaba.excel.enums.CacheLocationEnum`。
//!
//! 既有实现：`enum_cache_location.rs` → [`crate::CacheLocation`]。

#![allow(unused_imports)]
/// Java `CacheLocationEnum` 命名别名。
// Java 镜像 API 别名，保留以兼容 Java 命名。
#[allow(dead_code)]
pub type CacheLocationEnum = crate::CacheLocation;
