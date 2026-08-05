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
use crate::read::read_cache::SharedStringCache;

/// Batch count used by Java `Ehcache.BATCH_COUNT`.
// 内部缓存 API 脚手架，暂未在 crate 内直接使用。
#[allow(dead_code)]
pub const BATCH_COUNT: usize = 100;

/// Default active batch count used by Java `SimpleReadCacheSelector`.
pub const DEFAULT_MAX_EHCACHE_ACTIVATE_BATCH_COUNT: i32 = 20;

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
        let active_entries = u64::try_from(batch_count)
            .unwrap_or(1)
            .saturating_mul(BATCH_COUNT as u64);
        Ok(Self::from_backend(easyexcel_cache::create_moka_cache(
            active_entries,
        )?))
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
        let active_bytes = u64::try_from(megabytes)
            .unwrap_or(1)
            .saturating_mul(1024 * 1024);
        Ok(Self::from_backend(
            easyexcel_cache::create_weighted_moka_cache(active_bytes)?,
        ))
    }

    /// Wraps an existing shared-string backend.
    #[must_use]
    pub fn from_backend(backend: Box<dyn SharedStringCache>) -> Self {
        Self {
            adapter: SharedStringCacheAdapter::new(backend),
        }
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
