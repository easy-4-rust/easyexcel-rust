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
    /// Java nullable `maxUseMapCacheSize`，单位 MB。
    max_use_map_cache_size_mb: Option<i64>,
    /// 已弃用的 Java 活跃缓存 MB 上限，保留以兼容旧配置。
    max_cache_activate_size_mb: Option<i32>,
    /// Java 旧磁盘缓存实现的活跃批次数兼容配置。
    max_cache_activate_batch_count: Option<i32>,
}

impl Default for SimpleReadCacheSelector {
    fn default() -> Self {
        Self {
            max_use_map_cache_size_mb: None,
            max_cache_activate_size_mb: None,
            max_cache_activate_batch_count: None,
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
            max_use_map_cache_size_mb: Some(
                i64::try_from(max_use_map_cache_size_mb).unwrap_or(i64::MAX),
            ),
            max_cache_activate_size_mb: None,
            max_cache_activate_batch_count: None,
        }
    }

    /// 对应 Java 已弃用双参数构造器。
    #[must_use]
    pub const fn with_limits(
        max_use_map_cache_size_mb: Option<i64>,
        max_cache_activate_size_mb: Option<i32>,
    ) -> Self {
        Self {
            max_use_map_cache_size_mb,
            max_cache_activate_size_mb,
            max_cache_activate_batch_count: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.cache.selector.SimpleReadCacheSelector。 Sets the map-cache threshold in megabytes. (Java `setMaxUseMapCacheSize`)
    #[must_use]
    pub fn max_use_map_cache_size_mb(mut self, megabytes: u64) -> Self {
        self.max_use_map_cache_size_mb = Some(i64::try_from(megabytes).unwrap_or(i64::MAX));
        self
    }

    /// Returns the configured map-cache threshold in bytes.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.cache.selector.SimpleReadCacheSelector。
    pub fn max_use_map_cache_size_bytes(&self) -> u64 {
        let megabytes = match self.max_use_map_cache_size_mb {
            Some(value) if value > 0 => u64::try_from(value).unwrap_or_default(),
            Some(_) => return 0,
            None => DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES / 1_000_000,
        };
        easyexcel_cache::SharedStringCachePolicy::memory_megabytes_to_bytes(megabytes)
    }

    /// Java `getMaxUseMapCacheSize`，保留首次选择缓存前的 nullable 状态。
    #[must_use]
    pub const fn get_max_use_map_cache_size(&self) -> Option<i64> { self.max_use_map_cache_size_mb }
    /// Java `setMaxUseMapCacheSize`，单位 MB。
    pub const fn set_max_use_map_cache_size(&mut self, value: Option<i64>) { self.max_use_map_cache_size_mb = value; }
    /// Java `getMaxCacheActivateSize`。
    #[must_use]
    pub const fn get_max_cache_activate_size(&self) -> Option<i32> { self.max_cache_activate_size_mb }
    /// Java `setMaxCacheActivateSize`。
    pub const fn set_max_cache_activate_size(&mut self, value: Option<i32>) { self.max_cache_activate_size_mb = value; }
    /// Java `getMaxCacheActivateBatchCount`。
    #[must_use]
    pub const fn get_max_cache_activate_batch_count(&self) -> Option<i32> { self.max_cache_activate_batch_count }
    /// Java `setMaxCacheActivateBatchCount`。
    pub const fn set_max_cache_activate_batch_count(&mut self, value: Option<i32>) { self.max_cache_activate_batch_count = value; }
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
        easyexcel_cache::SharedStringCachePolicy::new(self.max_use_map_cache_size_bytes())
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
