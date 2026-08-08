//! 对应 Java：`com.alibaba.excel.cache.Ehcache`。

use crate::core::{ExcelError, Result};
use super::{FileCache, ReadCache};

/// Java Ehcache 共享字符串缓存的 Rust 磁盘后端等价实现。
///
/// Java 以 100 条为一批写入 20GB 磁盘池并保留有限活跃批次；Rust 的
/// `easyexcel-cache` 文件索引逐条写入，具有相同的有界内存和随机读取语义。
pub struct Ehcache {
    backend: Option<FileCache>,
    max_cache_activate_size_mb: Option<u64>,
    max_cache_activate_batch_count: Option<usize>,
}

impl Ehcache {
    /// Java 每批共享字符串数量。
    pub const BATCH_COUNT: usize = 100;
    /// Java 调试缓存 miss 采样周期。
    pub const DEBUG_CACHE_MISS_SIZE: usize = 1_000;
    /// Java 调试写入采样周期。
    pub const DEBUG_WRITE_SIZE: usize = 1_000_000;

    /// 对应已弃用的 `Ehcache(Integer maxCacheActivateSize)`。
    #[must_use]
    pub const fn with_max_cache_activate_size(max_cache_activate_size_mb: Option<u64>) -> Self {
        Self { backend: None, max_cache_activate_size_mb, max_cache_activate_batch_count: None }
    }

    /// 对应 Java 双参数构造器。
    #[must_use]
    pub const fn new(
        max_cache_activate_size_mb: Option<u64>,
        max_cache_activate_batch_count: Option<usize>,
    ) -> Self {
        Self { backend: None, max_cache_activate_size_mb, max_cache_activate_batch_count }
    }

    /// 返回兼容配置的活跃缓存 MB 上限。
    #[must_use]
    pub const fn max_cache_activate_size_mb(&self) -> Option<u64> { self.max_cache_activate_size_mb }
    /// 返回兼容配置的活跃批次数上限。
    #[must_use]
    pub const fn max_cache_activate_batch_count(&self) -> Option<usize> { self.max_cache_activate_batch_count }

    fn ensure_backend(&mut self) -> Result<&mut FileCache> {
        if self.backend.is_none() { self.backend = Some(FileCache::new()?); }
        self.backend.as_mut().ok_or_else(|| ExcelError::Format("Ehcache backend was not initialized".to_owned()))
    }
}

impl ReadCache for Ehcache {
    fn init(&mut self) { self.backend = FileCache::new().ok(); }
    fn put(&mut self, value: String) -> Result<()> { self.ensure_backend()?.put(value) }
    fn get(&self, key: Option<usize>) -> Result<Option<String>> {
        match &self.backend {
            Some(backend) => backend.get(key),
            None if key.is_none() => Ok(None),
            None => Err(ExcelError::Format("Ehcache must be initialized before get".to_owned())),
        }
    }
    fn put_finished(&mut self) -> Result<()> { self.ensure_backend()?.put_finished() }
    fn destroy(&mut self) {
        if let Some(backend) = &mut self.backend { backend.destroy(); }
        self.backend = None;
    }
}
