//! Moka 共享字符串缓存的 EasyExcel `ReadCache` 适配。
//!
//! 生命周期内不淘汰的对象存储由 `easyexcel-cache` 实现；本模块只保留
//! EasyExcel `ReadCache` 契约适配。

use crate::core::Result;

use super::read_cache::{ReadCache, SharedStringCacheAdapter};

/// 生命周期内完整保留共享字符串对象的 Moka `ReadCache` 适配器。
pub struct MokaCache {
    adapter: SharedStringCacheAdapter,
}

impl MokaCache {
    /// 创建无容量淘汰、无过期策略的 Moka 对象缓存。
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
