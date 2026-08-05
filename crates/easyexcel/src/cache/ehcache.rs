//! 对应 Java：`com.alibaba.excel.cache.Ehcache`。
//!
//! Rust 不保留 Ehcache 引擎实现。该名称仅作为 Java API 兼容入口，实际
//! `ReadCache` 适配类型是 [`MokaCache`](super::MokaCache)，活跃层与临时文件
//! 后备由 `easyexcel-cache` 唯一实现。

pub use super::moka_cache::{
    BATCH_COUNT, DEFAULT_MAX_EHCACHE_ACTIVATE_BATCH_COUNT, MokaCache as Ehcache,
};
