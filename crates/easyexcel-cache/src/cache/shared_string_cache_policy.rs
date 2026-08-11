//! 共享字符串缓存选择与容量策略。

use easyexcel_io::Result;

use super::{ReadCacheMode, SharedStringCache, create_cache};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 根据 `sharedStrings.xml` 大小创建共享字符串缓存。
///
/// 该策略不依赖 `EasyExcel` 门面的 selector trait，可由事件读取器、CLI 或
/// 其他工作簿 API 直接复用。小于内存阈值时使用纯内存缓存；达到阈值后
/// 使用临时文件缓存，维持 SAX 大文件读取的内存边界。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedStringCachePolicy {
    max_memory_bytes: u64,
}

impl Default for SharedStringCachePolicy {
    fn default() -> Self {
        Self {
            max_memory_bytes: super::DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES,
        }
    }
}

impl SharedStringCachePolicy {
    /// 将 Java `maxUseMapCacheSize` 使用的十进制 MB 转换为字节。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn memory_megabytes_to_bytes(megabytes: u64) -> u64 {
        megabytes.saturating_mul(1_000_000)
    }

    /// 使用指定的纯内存缓存阈值创建策略。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(max_memory_bytes: u64) -> Self {
        Self { max_memory_bytes }
    }

    /// 返回纯内存缓存阈值（字节）。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn max_memory_bytes(self) -> u64 {
        self.max_memory_bytes
    }

    /// 根据共享字符串 XML 大小选择缓存模式。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn select_mode(self, shared_strings_xml_size: u64) -> ReadCacheMode {
        if shared_strings_xml_size < self.max_memory_bytes {
            ReadCacheMode::Memory
        } else {
            ReadCacheMode::File
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 根据当前策略创建缓存后端。
    ///
    /// # Errors
    ///
    /// 缓存创建失败时返回错误。
    pub fn create_cache(self, shared_strings_xml_size: u64) -> Result<Box<dyn SharedStringCache>> {
        if self.select_mode(shared_strings_xml_size) == ReadCacheMode::Memory {
            return create_cache(ReadCacheMode::Memory, shared_strings_xml_size);
        }
        create_cache(ReadCacheMode::File, shared_strings_xml_size)
    }
}

#[cfg(test)]
mod tests {
    use super::SharedStringCachePolicy;
    use crate::ReadCacheMode;

    #[test]
    fn boundary_selects_memory_below_threshold_and_file_at_threshold() {
        let policy = SharedStringCachePolicy::new(1_000_000);
        assert_eq!(policy.select_mode(999_999), ReadCacheMode::Memory);
        assert_eq!(policy.select_mode(1_000_000), ReadCacheMode::File);
    }

    #[test]
    fn default_has_default_memory_threshold() {
        let policy = SharedStringCachePolicy::default();
        assert_eq!(
            policy.max_memory_bytes(),
            super::super::DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES
        );
    }

    #[test]
    fn memory_megabytes_to_bytes_converts() {
        assert_eq!(SharedStringCachePolicy::memory_megabytes_to_bytes(0), 0);
        assert_eq!(
            SharedStringCachePolicy::memory_megabytes_to_bytes(1),
            1_000_000
        );
        assert_eq!(
            SharedStringCachePolicy::memory_megabytes_to_bytes(5),
            5_000_000
        );
        assert_eq!(
            SharedStringCachePolicy::memory_megabytes_to_bytes(u64::MAX),
            u64::MAX
        );
    }

    #[test]
    fn max_memory_bytes_returns_configured_value() {
        let policy = SharedStringCachePolicy::new(42);
        assert_eq!(policy.max_memory_bytes(), 42);
    }

    #[test]
    fn create_cache_returns_memory_cache_for_small_xml() {
        let policy = SharedStringCachePolicy::new(1_000_000);
        let cache = policy.create_cache(500_000).expect("create cache");
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn create_cache_returns_file_cache_for_large_xml() {
        let policy = SharedStringCachePolicy::new(1_000_000);
        let cache = policy.create_cache(2_000_000).expect("create cache");
        assert_eq!(cache.len(), 0);
    }
}
