//! 对应 Java：`com.alibaba.excel.cache.selector.SimpleReadCacheSelector`.
//!
//! Default workbook behaviour (`ReadCacheMode::Auto`) uses the same
//! [`DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES`] (`5_000_000`) boundary as Java
//! `EasyExcel`'s built-in selector: smaller `sharedStrings.xml` parts stay in
//! [`MapCache`](super::super::MapCache), larger parts use
//! [`FileCache`](super::super::FileCache) to preserve SAX bounded-memory reads.
//! [`MokaCache`](super::super::MokaCache) is an explicit lifecycle object-cache choice.

use super::read_cache_selector::ReadCacheSelector;
use crate::read::read_cache::{
    DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES, ReadCacheMode, SharedStringCache,
};

/// Simple cache selector matching Java's 5 MB map-cache boundary.
///
/// 对应 Java：`com.alibaba.excel.cache.selector.SimpleReadCacheSelector`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleReadCacheSelector {
    /// Maximum shared-string table size that keeps data in memory, in bytes.
    max_use_map_cache_size_bytes: u64,
}

impl Default for SimpleReadCacheSelector {
    fn default() -> Self {
        Self {
            max_use_map_cache_size_bytes: DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES,
        }
    }
}

impl SimpleReadCacheSelector {
    /// 对应 Java：com.alibaba.excel.cache.selector.SimpleReadCacheSelector。 Creates a selector with Java defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a selector with a custom map-cache size in megabytes.
    ///
    /// 对应 Java 的 `maxUseMapCacheSize` 阈值语义。
    #[must_use]
    pub fn with_max_use_map_cache_size_mb(max_use_map_cache_size_mb: u64) -> Self {
        Self {
            max_use_map_cache_size_bytes:
                easyexcel_cache::SharedStringCachePolicy::memory_megabytes_to_bytes(
                    max_use_map_cache_size_mb,
                ),
        }
    }

    /// 对应 Java：com.alibaba.excel.cache.selector.SimpleReadCacheSelector。 Sets the map-cache threshold in megabytes. (Java `setMaxUseMapCacheSize`)
    #[must_use]
    pub fn max_use_map_cache_size_mb(mut self, megabytes: u64) -> Self {
        self.max_use_map_cache_size_bytes =
            easyexcel_cache::SharedStringCachePolicy::memory_megabytes_to_bytes(megabytes);
        self
    }

    /// Returns the configured map-cache threshold in bytes.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.cache.selector.SimpleReadCacheSelector。
    pub const fn max_use_map_cache_size_bytes(&self) -> u64 {
        self.max_use_map_cache_size_bytes
    }
}

impl ReadCacheSelector for SimpleReadCacheSelector {
    fn select_mode(&self, shared_strings_xml_size: u64) -> ReadCacheMode {
        self.engine_policy().select_mode(shared_strings_xml_size)
    }

    fn create_cache(
        &self,
        shared_strings_xml_size: u64,
    ) -> easyexcel_io::Result<Box<dyn SharedStringCache>> {
        self.engine_policy().create_cache(shared_strings_xml_size)
    }
}

impl SimpleReadCacheSelector {
    /// 将 Java selector 参数映射为格式无关的缓存引擎策略。
    fn engine_policy(&self) -> easyexcel_cache::SharedStringCachePolicy {
        easyexcel_cache::SharedStringCachePolicy::new(self.max_use_map_cache_size_bytes)
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn selector_knobs_and_defaults_match_java() {
        // 对应 Java：SimpleReadCacheSelector 构造与 setter
        let selector = SimpleReadCacheSelector::new();
        assert_eq!(
            selector.max_use_map_cache_size_bytes(),
            DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES
        );

        let selector =
            SimpleReadCacheSelector::with_max_use_map_cache_size_mb(2).max_use_map_cache_size_mb(3);
        assert_eq!(selector.max_use_map_cache_size_bytes(), 3_000_000);
    }

    #[test]
    fn selector_mb_conversion_saturates_on_overflow() {
        // 对应 Java：Long 乘法溢出保护
        let selector = SimpleReadCacheSelector::new().max_use_map_cache_size_mb(u64::MAX);
        assert_eq!(selector.max_use_map_cache_size_bytes(), u64::MAX);
    }

    #[test]
    fn selector_boundary_uses_memory_below_threshold_and_file_at_or_above() {
        // 对应 Java：小于 maxUseMapCacheSize 用内存缓存，否则用磁盘缓存
        let selector = SimpleReadCacheSelector::with_max_use_map_cache_size_mb(1);
        assert_eq!(selector.select_mode(999_999), ReadCacheMode::Memory);
        assert_eq!(selector.select_mode(1_000_000), ReadCacheMode::File);
    }
}
