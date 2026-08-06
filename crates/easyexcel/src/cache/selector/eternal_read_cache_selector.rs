//! 对应 Java：`com.alibaba.excel.cache.selector.EternalReadCacheSelector`.

use super::read_cache_selector::ReadCacheSelector;
use crate::read::read_cache::ReadCacheMode;

/// Always returns the same cache mode regardless of shared-string table size.
///
/// 对应 Java：`com.alibaba.excel.cache.selector.EternalReadCacheSelector`.
///
/// Java `EasyExcel.readCache(ReadCache)` wraps the cache instance in this
/// selector so workbook reads skip the 5 MB Auto heuristic. Rust exposes the
/// same pinning through [`StoredReadCacheSelector::Eternal`](crate::StoredReadCacheSelector::Eternal)
/// or builder helpers on [`ExcelReader`](crate::ExcelReader).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EternalReadCacheSelector {
    mode: ReadCacheMode,
}

impl EternalReadCacheSelector {
    /// Creates a selector that always returns `mode`.
    ///
    /// 对应 Java：`new EternalReadCacheSelector(ReadCache readCache)`.
    #[must_use]
    pub const fn new(mode: ReadCacheMode) -> Self {
        Self { mode }
    }

    /// Creates a selector equivalent to Java `readCache(new MapCache())`.
    #[must_use]
    pub const fn map_cache() -> Self {
        Self::new(ReadCacheMode::Memory)
    }

    /// 创建固定使用 Moka 对象缓存的选择器。
    #[must_use]
    pub const fn moka() -> Self {
        Self::new(ReadCacheMode::Moka)
    }

    /// 创建固定使用临时文件缓存的选择器。
    #[must_use]
    pub const fn file_cache() -> Self {
        Self::new(ReadCacheMode::File)
    }
}

impl ReadCacheSelector for EternalReadCacheSelector {
    fn select_mode(&self, _shared_strings_xml_size: u64) -> ReadCacheMode {
        self.mode
    }
}
