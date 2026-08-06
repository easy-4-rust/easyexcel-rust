//! 共享字符串缓存选择与容量策略。

use easyexcel_io::Result;

use super::{
    DEFAULT_MOKA_ACTIVE_BATCHES, ReadCacheMode, SharedStringCache, create_cache,
    create_moka_cache_for_batches, create_weighted_moka_cache_mb,
};

/// 根据 `sharedStrings.xml` 大小和活跃缓存容量创建共享字符串缓存。
///
/// 该策略不依赖 EasyExcel 门面的 selector trait，可由事件读取器、CLI 或
/// 其他工作簿 API 直接复用。小于内存阈值时使用纯内存缓存；达到阈值后
/// 使用 Moka 活跃层与临时文件无损后备。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedStringCachePolicy {
    max_memory_bytes: u64,
    max_active_megabytes: Option<u64>,
    max_active_batches: Option<u64>,
}

impl Default for SharedStringCachePolicy {
    fn default() -> Self {
        Self {
            max_memory_bytes: super::DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES,
            max_active_megabytes: None,
            max_active_batches: None,
        }
    }
}

impl SharedStringCachePolicy {
    /// 将 Java `maxUseMapCacheSize` 使用的十进制 MB 转换为字节。
    #[must_use]
    pub const fn memory_megabytes_to_bytes(megabytes: u64) -> u64 {
        megabytes.saturating_mul(1_000_000)
    }

    /// 使用指定的纯内存缓存阈值创建策略。
    #[must_use]
    pub const fn new(max_memory_bytes: u64) -> Self {
        Self {
            max_memory_bytes,
            max_active_megabytes: None,
            max_active_batches: None,
        }
    }

    /// 设置 Moka 活跃层的 UTF-8 字节权重上限（MB）。
    ///
    /// 该配置优先于批次数，兼容 Java 已废弃的
    /// `maxCacheActivateSize` 选择顺序。
    #[must_use]
    pub const fn with_max_active_megabytes(mut self, megabytes: Option<u64>) -> Self {
        self.max_active_megabytes = megabytes;
        self
    }

    /// 设置 Moka 活跃层最多保留的共享字符串批次数。
    #[must_use]
    pub const fn with_max_active_batches(mut self, batches: Option<u64>) -> Self {
        self.max_active_batches = batches;
        self
    }

    /// 返回纯内存缓存阈值（字节）。
    #[must_use]
    pub const fn max_memory_bytes(self) -> u64 {
        self.max_memory_bytes
    }

    /// 根据共享字符串 XML 大小选择缓存模式。
    #[must_use]
    pub const fn select_mode(self, shared_strings_xml_size: u64) -> ReadCacheMode {
        if shared_strings_xml_size < self.max_memory_bytes {
            ReadCacheMode::Memory
        } else {
            ReadCacheMode::Disk
        }
    }

    /// 根据当前策略创建缓存后端。
    ///
    /// # Errors
    ///
    /// 磁盘后备所需的临时文件无法创建时返回错误。
    pub fn create_cache(self, shared_strings_xml_size: u64) -> Result<Box<dyn SharedStringCache>> {
        if self.select_mode(shared_strings_xml_size) == ReadCacheMode::Memory {
            return create_cache(ReadCacheMode::Memory, shared_strings_xml_size);
        }
        if let Some(megabytes) = self.max_active_megabytes {
            return create_weighted_moka_cache_mb(megabytes.max(1));
        }
        create_moka_cache_for_batches(
            self.max_active_batches
                .unwrap_or(DEFAULT_MOKA_ACTIVE_BATCHES)
                .max(1),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SharedStringCachePolicy;
    use crate::ReadCacheMode;

    #[test]
    fn boundary_selects_memory_below_threshold_and_disk_at_threshold() {
        let policy = SharedStringCachePolicy::new(1_000_000);
        assert_eq!(policy.select_mode(999_999), ReadCacheMode::Memory);
        assert_eq!(policy.select_mode(1_000_000), ReadCacheMode::Disk);
    }
}
