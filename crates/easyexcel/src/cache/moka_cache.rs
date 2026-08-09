//! Moka 共享字符串缓存的 `EasyExcel` `ReadCache` 适配。
//!
//! 生命周期内不淘汰的对象存储由 `easyexcel-cache` 实现；本模块只保留
//! `EasyExcel` `ReadCache` 契约适配。

use crate::core::Result;

use super::read_cache::{ReadCache, SharedStringCacheAdapter};

/// Moka 共享字符串缓存的 Java `ReadCache` 生命周期适配器。
///
/// 实际对象存储由 `easyexcel-cache` 承载。本类型只提供显式选择 Moka
/// 后端时的适配，不代表已经退役的 Java Ehcache：后者的磁盘/活跃缓存语义
/// 由 `SharedStringCachePolicy` 与 Memory/File/Moka 后端组合替代。
pub struct MokaCache {
    adapter: SharedStringCacheAdapter,
}

impl MokaCache {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建无容量淘汰、无过期策略的 Moka 对象缓存。
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: SharedStringCacheAdapter::new(easyexcel_cache::create_moka_cache()),
        }
    }
}

impl Default for MokaCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadCache for MokaCache {
    fn put(&mut self, value: String) -> Result<()> {
        self.adapter.put(value)
    }

    fn get(&self, key: Option<usize>) -> Result<Option<String>> {
        self.adapter.get(key)
    }

    fn put_finished(&mut self) -> Result<()> {
        self.adapter.put_finished()
    }

    fn destroy(&mut self) {
        self.adapter = SharedStringCacheAdapter::new(easyexcel_cache::create_moka_cache());
    }
}
