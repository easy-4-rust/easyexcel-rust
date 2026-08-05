//! 对应 Java：`com.alibaba.excel.cache.Ehcache`.
//!
//! Java binds two org.ehcache managers: a persistent disk manager
//! (`FILE_CACHE_MANAGER`, 20 GB pool) and a heap active cache
//! (`ACTIVE_CACHE_MANAGER`, sized by `maxCacheActivateBatchCount` entries or
//! deprecated `maxCacheActivateSize` MB). Strings are batched in groups of
//! [`BATCH_COUNT`] before spilling to disk.
//!
//! Rust keeps the same [`ReadCache`] surface while delegating to
//! `easyexcel-cache`'s Moka active tier and lossless temporary-file backing
//! store. Moka supplies concurrent admission/eviction; the backing store keeps
//! every shared string addressable even after an active entry is evicted.

use crate::core::Result;

use super::read_cache::{ReadCache, SharedStringCacheAdapter};

/// Batch count used by Java `Ehcache.BATCH_COUNT`.
// 内部缓存 API 脚手架，暂未在 crate 内直接使用。
#[allow(dead_code)]
pub const BATCH_COUNT: usize = easyexcel_cache::SHARED_STRING_CACHE_BATCH_SIZE as usize;

/// Default active batch count used by Java `SimpleReadCacheSelector`.
pub const DEFAULT_MAX_EHCACHE_ACTIVATE_BATCH_COUNT: i32 =
    easyexcel_cache::DEFAULT_MOKA_ACTIVE_BATCHES as i32;

/// Disk-backed shared-string cache matching Java `Ehcache`.
///
/// 对应 Java：`com.alibaba.excel.cache.Ehcache`.
///
/// Use [`ReadCacheMode::Disk`](crate::ReadCacheMode::Disk) or
/// [`EternalReadCacheSelector::ehcache`] at the workbook level; this type exists
/// for API parity and direct [`ReadCache`] tests.
pub struct Ehcache {
    adapter: SharedStringCacheAdapter,
}

impl Ehcache {
    /// Creates a disk-backed cache with Java default batch sizing.
    ///
    /// 对应 Java：`new Ehcache(null, maxCacheActivateBatchCount)`.
    ///
    /// `max_cache_activate_batch_count` controls Moka's active entry capacity;
    /// each Java batch contains [`BATCH_COUNT`] strings.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the temporary cache file cannot be created.
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

    /// Creates a cache with the deprecated Java `maxCacheActivateSize` MB knob.
    ///
    /// 对应 Java：`new Ehcache(maxCacheActivateSize)`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the temporary cache file cannot be created.
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

impl ReadCache for Ehcache {
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
