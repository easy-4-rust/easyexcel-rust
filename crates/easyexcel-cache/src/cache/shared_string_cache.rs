//! 共享字符串缓存的内存、Moka 对象与临时文件实现。

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use easyexcel_io::{Error, Result};
use moka::sync::Cache;
use tempfile::NamedTempFile;

use super::ReadCacheMode;

/// Java `SimpleReadCacheSelector` 使用的默认内存阈值。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES: u64 = 5_000_000;

include!("shared_string_cache/shared_string_cache_writer.rs");

include!("shared_string_cache/shared_string_cache_reader.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 同时支持写入阶段和读取阶段的共享字符串缓存。
pub trait SharedStringCache: SharedStringCacheWriter + SharedStringCacheReader {
    /// 结束当前缓存的写入阶段。
    ///
    /// # Errors
    ///
    /// 后备存储无法完成落盘时返回错误。
    fn put_and_finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>>
    where
        Self: Sized,
    {
        self.finish()
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 创建空的只读内存缓存。
#[must_use]
pub fn memory_cache() -> Box<dyn SharedStringCacheReader> {
    Box::new(MemorySharedStringReader::default())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 创建处于顺序写入阶段的纯内存共享字符串缓存。
#[must_use]
pub fn create_memory_cache() -> Box<dyn SharedStringCache> {
    Box::new(MemorySharedStringCache::default())
}

/// 从已经解码的 BIFF SST 创建不可变共享字符串缓存。
///
/// 该后端对应 Java `XlsCache(SSTRecord)`：写入是空操作，所有字符串在
/// 构造时已经按 SST 索引固定。
#[must_use]
pub fn prebuilt_cache(values: Vec<String>) -> Box<dyn SharedStringCache> {
    Box::new(PrebuiltSharedStringCache { values })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按模式和 XML 大小创建共享字符串缓存。
///
/// # Errors
///
/// 缓存创建失败时返回错误。
pub fn create_cache(mode: ReadCacheMode, xml_size: u64) -> Result<Box<dyn SharedStringCache>> {
    match mode {
        ReadCacheMode::Auto if xml_size < DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES => {
            Ok(create_memory_cache())
        }
        ReadCacheMode::Auto | ReadCacheMode::File => create_file_cache(),
        ReadCacheMode::Moka => Ok(create_moka_cache()),
        ReadCacheMode::Memory => Ok(create_memory_cache()),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 创建由临时文件持有全部共享字符串的缓存。
///
/// # Errors
///
/// 临时文件无法创建时返回错误。
pub fn create_file_cache() -> Result<Box<dyn SharedStringCache>> {
    Ok(Box::new(FileSharedStringCache::new()?))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 创建生命周期内不淘汰对象的 Moka 共享字符串缓存。
///
/// 不设置容量、TTL 或 TTI；条目只在缓存对象销毁时整体释放。
#[must_use]
pub fn create_moka_cache() -> Box<dyn SharedStringCache> {
    Box::new(MokaSharedStringCache::new())
}

/// 兼容旧调用点；Moka 后端不持有线程局部文件句柄，因此无需显式清理。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const fn remove_thread_local_cache() {
    // Moka/Vec/File 后端由所有权管理，不持有 Java ThreadLocal 生命周期状态。
}

include!("shared_string_cache/shared_string_cache_handle.rs");

#[derive(Default)]
struct MemorySharedStringCache {
    values: Vec<String>,
}

impl SharedStringCache for MemorySharedStringCache {}

struct PrebuiltSharedStringCache {
    values: Vec<String>,
}

impl SharedStringCache for PrebuiltSharedStringCache {}

#[derive(Default)]
struct MemorySharedStringReader {
    values: Vec<String>,
}

fn value_at(values: &[String], index: usize) -> Result<String> {
    values
        .get(index)
        .cloned()
        .ok_or_else(|| out_of_bounds(index))
}

struct MokaSharedStringCache {
    objects: Cache<usize, Arc<str>>,
    len: usize,
}

impl MokaSharedStringCache {
    fn new() -> Self {
        Self {
            objects: Cache::builder().build(),
            len: 0,
        }
    }
}

impl SharedStringCache for MokaSharedStringCache {}

/// 完成写入后的 Moka 对象缓存只读视图。
struct MokaSharedStringReader {
    objects: Cache<usize, Arc<str>>,
    len: usize,
}

/// 顺序写入、按索引读取的临时文件共享字符串缓存。
struct FileSharedStringCache {
    temporary_file: NamedTempFile,
    writer: File,
    path: PathBuf,
    entries: Vec<(u64, usize)>,
}

impl FileSharedStringCache {
    fn new() -> Result<Self> {
        let temporary_file = NamedTempFile::new()?;
        let path = temporary_file.path().to_path_buf();
        let writer = temporary_file.reopen()?;
        Ok(Self {
            temporary_file,
            writer,
            path,
            entries: Vec::new(),
        })
    }
}

impl SharedStringCache for FileSharedStringCache {}

/// 完成写入后的文件缓存只读视图。
struct FileSharedStringReader {
    _temporary_file: NamedTempFile,
    path: PathBuf,
    entries: Vec<(u64, usize)>,
}

fn read_file_entry(path: &Path, entries: &[(u64, usize)], index: usize) -> Result<String> {
    let (offset, length) = entries
        .get(index)
        .copied()
        .ok_or_else(|| out_of_bounds(index))?;
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|error| Error::Other(error.to_string()))
}

fn out_of_bounds(index: usize) -> Error {
    Error::Other(format!("shared string index is out of bounds: {index}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moka_object_cache_keeps_every_value_before_and_after_finish() {
        let mut cache = create_moka_cache();
        for index in 0..128 {
            cache
                .put(format!("shared-{index}"))
                .expect("append shared string");
        }
        assert_eq!(cache.len(), 128);
        assert_eq!(cache.get(0).expect("read first object"), "shared-0");
        assert_eq!(cache.get(127).expect("read latest object"), "shared-127");

        let reader = cache.finish().expect("finish Moka cache");
        assert_eq!(reader.len(), 128);
        assert_eq!(reader.get(1).expect("read after finish"), "shared-1");
        assert!(reader.get(128).is_err());
    }

    #[test]
    fn moka_object_cache_accepts_multibyte_values_without_eviction() {
        let mut cache = create_moka_cache();
        cache.put("中文".to_owned()).expect("append UTF-8 value");
        cache.put("second".to_owned()).expect("append second value");
        assert_eq!(cache.get(0).expect("read cached UTF-8"), "中文");
        assert_eq!(cache.get(1).expect("read second value"), "second");
    }

    #[test]
    fn file_cache_round_trips_values_before_and_after_finish() {
        let mut cache = create_file_cache().expect("create file cache");
        cache.put("first".to_owned()).expect("append first");
        cache.put("中文".to_owned()).expect("append UTF-8 value");
        assert_eq!(cache.get(0).expect("read before finish"), "first");

        let reader = cache.finish().expect("finish file cache");
        assert_eq!(reader.len(), 2);
        assert_eq!(reader.get(1).expect("read after finish"), "中文");
        assert!(reader.get(2).is_err());
    }

    #[test]
    fn memory_cache_is_empty_on_creation() {
        let reader = memory_cache();
        assert_eq!(reader.len(), 0);
        assert!(reader.get(0).is_err());
    }

    #[test]
    fn create_memory_cache_writes_and_reads() {
        let mut cache = create_memory_cache();
        assert_eq!(cache.len(), 0);
        cache.put("hello".to_owned()).expect("put");
        cache.put("world".to_owned()).expect("put");
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(0).expect("get"), "hello");
        assert_eq!(cache.get(1).expect("get"), "world");
        assert!(cache.get(2).is_err());
    }

    #[test]
    fn prebuilt_cache_contains_all_values() {
        let values = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
        let cache = prebuilt_cache(values);
        // Prebuilt cache: put is a no-op, values are already there
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(0).expect("get"), "a");
        assert_eq!(cache.get(1).expect("get"), "b");
        assert_eq!(cache.get(2).expect("get"), "c");
        assert!(cache.get(3).is_err());
    }

    #[test]
    fn create_cache_auto_mode_selects_memory_for_small() {
        let cache = create_cache(ReadCacheMode::Auto, 100).expect("create");
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn create_cache_auto_mode_selects_file_for_large() {
        let cache = create_cache(ReadCacheMode::Auto, DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES + 1)
            .expect("create");
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn create_cache_memory_mode() {
        let cache = create_cache(ReadCacheMode::Memory, 0).expect("create");
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn create_cache_file_mode() {
        let cache = create_cache(ReadCacheMode::File, 0).expect("create");
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn create_cache_moka_mode() {
        let cache = create_cache(ReadCacheMode::Moka, 0).expect("create");
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn remove_thread_local_cache_is_noop() {
        remove_thread_local_cache();
        // Just verify it doesn't panic
    }

    #[test]
    fn memory_cache_put_and_finish_roundtrip() {
        let mut cache = create_memory_cache();
        cache.put("first".to_owned()).expect("put");
        cache.put("second".to_owned()).expect("put");
        let reader = cache.finish().expect("finish");
        assert_eq!(reader.len(), 2);
        assert_eq!(reader.get(0).expect("get"), "first");
        assert_eq!(reader.get(1).expect("get"), "second");
    }

    #[test]
    fn prebuilt_cache_finish_preserves_values() {
        let values = vec!["x".to_owned(), "y".to_owned()];
        let cache = prebuilt_cache(values);
        let reader = cache.finish().expect("finish");
        assert_eq!(reader.len(), 2);
        assert_eq!(reader.get(0).expect("get"), "x");
        assert_eq!(reader.get(1).expect("get"), "y");
    }

    #[test]
    fn shared_string_cache_handle_put_and_finish() {
        let cache = create_memory_cache();
        let mut handle = SharedStringCacheHandle::new(cache);
        handle.put("test".to_owned()).expect("put");
        assert_eq!(handle.len(), 1);
        assert_eq!(handle.get(0).expect("get"), "test");
        handle.finish().expect("finish");
        assert_eq!(handle.len(), 1);
        assert_eq!(handle.get(0).expect("get"), "test");
        assert!(handle.get(1).is_err());
    }

    #[test]
    fn shared_string_cache_handle_is_empty() {
        let cache = create_memory_cache();
        let handle = SharedStringCacheHandle::new(cache);
        assert!(handle.is_empty());
    }

    #[test]
    fn shared_string_cache_handle_finish_is_idempotent() {
        let cache = create_memory_cache();
        let mut handle = SharedStringCacheHandle::new(cache);
        handle.finish().expect("first finish");
        handle.finish().expect("second finish");
    }

    #[test]
    fn shared_string_cache_handle_into_reader() {
        let cache = create_memory_cache();
        let mut handle = SharedStringCacheHandle::new(cache);
        handle.finish().expect("finish");
        let reader = handle.into_reader();
        assert!(reader.is_some());
    }
}
