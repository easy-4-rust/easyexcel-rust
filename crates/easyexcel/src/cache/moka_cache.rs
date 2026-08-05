//! Moka 共享字符串缓存的 EasyExcel `ReadCache` 适配。
//!
//! 活跃条目淘汰和临时文件后备由 `easyexcel-cache` 实现；本模块只保留
//! Java 构造参数到 Moka 容量参数的转换，以及门面 `ReadCache` 错误映射。

use crate::core::Result;

use super::read_cache::{ReadCache, SharedStringCacheAdapter};

/// Java `Ehcache.BATCH_COUNT` 对应的每批共享字符串数量。
#[allow(dead_code)]
pub const BATCH_COUNT: usize = easyexcel_cache::SHARED_STRING_CACHE_BATCH_SIZE as usize;

/// Java `SimpleReadCacheSelector` 默认保留的活跃批次数。
pub const DEFAULT_MAX_EHCACHE_ACTIVATE_BATCH_COUNT: i32 =
    easyexcel_cache::DEFAULT_MOKA_ACTIVE_BATCHES as i32;

/// Moka 活跃层与无损临时文件后备的 Java `ReadCache` 适配器。
pub struct MokaCache {
    adapter: SharedStringCacheAdapter,
}

impl MokaCache {
    /// 按 Java `maxCacheActivateBatchCount` 参数创建缓存。
    ///
    /// 每一批包含 [`BATCH_COUNT`] 条共享字符串，活跃层超出容量后由 Moka
    /// 淘汰；全部字符串仍保存在临时文件后备中。
    ///
    /// # Errors
    ///
    /// 临时后备文件无法创建时返回 I/O 错误。
    pub fn new(max_cache_activate_batch_count: Option<i32>) -> Result<Self> {
        let batch_count = max_cache_activate_batch_count
            .unwrap_or(DEFAULT_MAX_EHCACHE_ACTIVATE_BATCH_COUNT)
            .max(1);
        let active_batches = u64::try_from(batch_count).unwrap_or(1);
        Ok(Self {
            adapter: SharedStringCacheAdapter::new(
                easyexcel_cache::create_moka_cache_for_batches(active_batches)?,
            ),
        })
    }

    /// 按 Java 已废弃的 `maxCacheActivateSize` MB 参数创建加权缓存。
    ///
    /// # Errors
    ///
    /// 临时后备文件无法创建时返回 I/O 错误。
    pub fn with_max_cache_activate_size_mb(
        max_cache_activate_size_mb: Option<i32>,
    ) -> Result<Self> {
        let megabytes = max_cache_activate_size_mb.unwrap_or(16).max(1);
        let active_megabytes = u64::try_from(megabytes).unwrap_or(1);
        Ok(Self {
            adapter: SharedStringCacheAdapter::new(
                easyexcel_cache::create_weighted_moka_cache_mb(active_megabytes)?,
            ),
        })
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
}
