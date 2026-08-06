//! 对应 Java：`com.alibaba.excel.cache.ReadCache`.

use crate::core::Result;

use crate::cache::selector::ReadCacheSelector;
use crate::read::read_cache::{ReadCacheMode, SharedStringCache, SharedStringCacheReader};

/// Shared-string cache contract matching Java `ReadCache`.
///
/// 对应 Java：`com.alibaba.excel.cache.ReadCache`.
pub trait ReadCache: Send {
    /// Initializes the cache. (Java `init(AnalysisContext)`)
    ///
    /// Default implementation records initialization state so callers
    /// can verify the lifecycle fires. Concrete implementations should
    /// override to allocate resources.
    fn init(&mut self) {
        // Default: no resources to allocate (in-memory caches are lazy).
        // Concrete implementations may override when allocation is eager.
    }

    /// Stores the next shared string. (Java `put(String)`)
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when the value cannot be stored.
    fn put(&mut self, value: String) -> Result<()>;

    /// Reads a shared string by index. (Java `get(Integer)`)
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when the index is invalid.
    fn get(&self, key: Option<usize>) -> Result<Option<String>>;

    /// Marks the write phase complete. (Java `putFinished()`)
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when finalization fails.
    fn put_finished(&mut self) -> Result<()>;

    /// Releases cache resources. (Java `destroy()`)
    ///
    /// Default implementation is a no-op; owned cache objects release their
    /// entries when dropped.
    fn destroy(&mut self) {
        // Default: nothing to release for in-memory caches.
        // Concrete implementations release owned resources when dropped.
    }
}

/// 对应 Java：com.alibaba.excel.cache.ReadCache。 Creates an in-memory cache backend. (Java `new MapCache()`)
#[must_use]
pub fn new_map_cache() -> Box<dyn SharedStringCache> {
    easyexcel_cache::create_memory_cache()
}

/// 对应 Java：com.alibaba.excel.cache.ReadCache。 创建生命周期内不淘汰对象的 Moka 缓存。
#[must_use]
pub fn new_moka_cache() -> Box<dyn SharedStringCache> {
    easyexcel_cache::create_moka_cache()
}

/// 对应 Java：com.alibaba.excel.cache.ReadCache。 创建用于大文件 SAX 读取的临时文件缓存。
///
/// # Errors
///
/// 临时文件无法创建时返回错误。
pub fn new_file_cache() -> Result<Box<dyn SharedStringCache>> {
    Ok(easyexcel_cache::create_file_cache()?)
}

/// Resolves the effective [`ReadCacheMode`] for a shared-string table size.
///
/// 对应 Java：`ReadWorkbookHolder` selector wiring.
#[must_use]
pub fn resolve_read_cache_mode(
    mode: ReadCacheMode,
    selector: Option<&dyn ReadCacheSelector>,
    shared_strings_xml_size: u64,
) -> ReadCacheMode {
    selector.map_or(mode, |selector| {
        selector.select_mode(shared_strings_xml_size)
    })
}

include!("read_cache/shared_string_cache_adapter.rs");

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::cache::SimpleReadCacheSelector;

    #[test]
    fn default_read_cache_lifecycle_methods_are_noops() {
        // 对应 Java：ReadCache.init()/destroy() 默认空实现
        struct NoopCache;
        impl ReadCache for NoopCache {
            fn put(&mut self, _value: String) -> Result<()> {
                Ok(())
            }
            fn get(&self, _key: Option<usize>) -> Result<Option<String>> {
                Ok(None)
            }
            fn put_finished(&mut self) -> Result<()> {
                Ok(())
            }
        }
        let mut cache = NoopCache;
        cache.init();
        cache.destroy();
        assert!(cache.put("ignored".to_owned()).is_ok());
        // get/put_finished 同为默认空实现（对应 Java ReadCache 生命周期方法）
        assert!(cache.get(Some(0)).is_ok());
        assert!(cache.put_finished().is_ok());
    }

    #[test]
    fn resolve_read_cache_mode_prefers_selector_over_mode() {
        // 对应 Java：ReadWorkbookHolder 中 selector.selectMode 优先于 readCache
        let selector = SimpleReadCacheSelector::with_max_use_map_cache_size_mb(1);
        assert_eq!(
            resolve_read_cache_mode(ReadCacheMode::Moka, Some(&selector), 500),
            ReadCacheMode::Memory
        );
        assert_eq!(
            resolve_read_cache_mode(ReadCacheMode::Memory, Some(&selector), 2_000_000),
            ReadCacheMode::File
        );
        // 无 selector 时回退到配置的 mode
        assert_eq!(
            resolve_read_cache_mode(ReadCacheMode::Moka, None, 500),
            ReadCacheMode::Moka
        );
    }

    #[test]
    fn adapter_lifecycle_put_finish_get_and_into_reader() -> Result<()> {
        // 对应 Java：MapCache.putFinished() 后进入只读阶段
        let mut adapter = SharedStringCacheAdapter::new(new_map_cache());
        adapter.put("hello".to_owned())?;
        // put_finished 之前 get 走写入侧缓存
        assert_eq!(adapter.get(Some(0))?, Some("hello".to_owned()));
        adapter.put_finished()?;
        // 重复 put_finished 幂等（第二次直接返回）
        adapter.put_finished()?;
        assert_eq!(adapter.get(Some(0))?, Some("hello".to_owned()));
        // None key 直接返回 Ok(None)
        assert_eq!(adapter.get(None)?, None);
        let reader = adapter.into_reader();
        assert_eq!(reader.get(0)?, "hello".to_owned());
        assert_eq!(reader.len(), 1);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "ReadCache.put_finished must run before into_reader")]
    fn adapter_into_reader_before_put_finished_panics() {
        let adapter = SharedStringCacheAdapter::new(new_map_cache());
        let _ = adapter.into_reader();
    }
}
