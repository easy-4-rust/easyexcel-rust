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
pub const DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES: u64 = 5_000_000;

/// 共享字符串顺序写入阶段。
pub trait SharedStringCacheWriter {
    /// 追加一条共享字符串。
    ///
    /// # Errors
    ///
    /// 后备存储写入失败时返回错误。
    fn put(&mut self, value: String) -> Result<()>;

    /// 结束写入并返回线程安全的只读视图。
    ///
    /// # Errors
    ///
    /// 后备存储无法完成落盘时返回错误。
    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>>;
}

/// 共享字符串并发读取阶段。
pub trait SharedStringCacheReader: Send + Sync {
    /// 按零基下标读取共享字符串。
    ///
    /// # Errors
    ///
    /// 下标越界或后备存储读取失败时返回错误。
    fn get(&self, index: usize) -> Result<String>;

    /// 返回缓存中的共享字符串数量。
    fn len(&self) -> usize;

    /// 返回缓存是否为空。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 同时支持写入阶段和读取阶段的共享字符串缓存。
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

/// 创建空的只读内存缓存。
#[must_use]
pub fn memory_cache() -> Box<dyn SharedStringCacheReader> {
    Box::new(MemorySharedStringReader::default())
}

/// 创建处于顺序写入阶段的纯内存共享字符串缓存。
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

/// 按模式和 XML 大小创建共享字符串缓存。
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

/// 创建由临时文件持有全部共享字符串的缓存。
///
/// # Errors
///
/// 临时文件无法创建时返回错误。
pub fn create_file_cache() -> Result<Box<dyn SharedStringCache>> {
    Ok(Box::new(FileSharedStringCache::new()?))
}

/// 创建生命周期内不淘汰对象的 Moka 共享字符串缓存。
///
/// 不设置容量、TTL 或 TTI；条目只在缓存对象销毁时整体释放。
#[must_use]
pub fn create_moka_cache() -> Box<dyn SharedStringCache> {
    Box::new(MokaSharedStringCache::new())
}

/// 兼容旧调用点；Moka 后端不持有线程局部文件句柄，因此无需显式清理。
pub const fn remove_thread_local_cache() {}

/// 管理共享字符串缓存从顺序写入到并发只读的阶段切换。
///
/// 该状态机属于缓存引擎，不依赖 EasyExcel Java 门面的 `ReadCache` trait。
/// 门面只需把自身的可空索引和错误类型映射到这个句柄。
pub struct SharedStringCacheHandle {
    writer: Box<dyn SharedStringCache>,
    reader: Option<Box<dyn SharedStringCacheReader>>,
}

impl SharedStringCacheHandle {
    /// 包装一个仍处于写入阶段的共享字符串缓存。
    #[must_use]
    pub fn new(writer: Box<dyn SharedStringCache>) -> Self {
        Self {
            writer,
            reader: None,
        }
    }

    /// 追加一条共享字符串。
    ///
    /// # Errors
    ///
    /// 后备存储写入失败时返回错误。
    pub fn put(&mut self, value: String) -> Result<()> {
        self.writer.put(value)
    }

    /// 按零基索引读取共享字符串。
    ///
    /// 完成前从写入缓存读取，完成后从不可变读取视图读取。
    ///
    /// # Errors
    ///
    /// 索引越界或后备存储读取失败时返回错误。
    pub fn get(&self, index: usize) -> Result<String> {
        self.reader
            .as_ref()
            .map_or_else(|| self.writer.get(index), |reader| reader.get(index))
    }

    /// 完成写入并切换到只读阶段；重复调用是幂等的。
    ///
    /// # Errors
    ///
    /// 后备存储无法完成落盘时返回错误。
    pub fn finish(&mut self) -> Result<()> {
        if self.reader.is_some() {
            return Ok(());
        }
        let writer = std::mem::replace(
            &mut self.writer,
            Box::new(MemorySharedStringCache::default()),
        );
        self.reader = Some(writer.finish()?);
        Ok(())
    }

    /// 返回当前缓存中的共享字符串数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.reader
            .as_ref()
            .map_or_else(|| self.writer.len(), |reader| reader.len())
    }

    /// 返回缓存是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 取出完成后的不可变读取视图。
    #[must_use]
    pub fn into_reader(self) -> Option<Box<dyn SharedStringCacheReader>> {
        self.reader
    }
}

#[derive(Default)]
struct MemorySharedStringCache {
    values: Vec<String>,
}

impl SharedStringCacheWriter for MemorySharedStringCache {
    fn put(&mut self, value: String) -> Result<()> {
        self.values.push(value);
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        Ok(Box::new(MemorySharedStringReader {
            values: self.values,
        }))
    }
}

impl SharedStringCacheReader for MemorySharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        value_at(&self.values, index)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl SharedStringCache for MemorySharedStringCache {}

struct PrebuiltSharedStringCache {
    values: Vec<String>,
}

impl SharedStringCacheWriter for PrebuiltSharedStringCache {
    fn put(&mut self, _value: String) -> Result<()> {
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        Ok(Box::new(MemorySharedStringReader {
            values: self.values,
        }))
    }
}

impl SharedStringCacheReader for PrebuiltSharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        value_at(&self.values, index)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl SharedStringCache for PrebuiltSharedStringCache {}

#[derive(Default)]
struct MemorySharedStringReader {
    values: Vec<String>,
}

impl SharedStringCacheReader for MemorySharedStringReader {
    fn get(&self, index: usize) -> Result<String> {
        value_at(&self.values, index)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
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

impl SharedStringCacheWriter for MokaSharedStringCache {
    fn put(&mut self, value: String) -> Result<()> {
        let index = self.len;
        self.objects.insert(index, Arc::<str>::from(value));
        self.len = self.len.saturating_add(1);
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        let Self { objects, len } = *self;
        Ok(Box::new(MokaSharedStringReader { objects, len }))
    }
}

impl SharedStringCacheReader for MokaSharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        self.objects
            .get(&index)
            .map(|value| value.to_string())
            .ok_or_else(|| out_of_bounds(index))
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl SharedStringCache for MokaSharedStringCache {}

/// 完成写入后的 Moka 对象缓存只读视图。
struct MokaSharedStringReader {
    objects: Cache<usize, Arc<str>>,
    len: usize,
}

impl SharedStringCacheReader for MokaSharedStringReader {
    fn get(&self, index: usize) -> Result<String> {
        self.objects
            .get(&index)
            .map(|value| value.to_string())
            .ok_or_else(|| out_of_bounds(index))
    }

    fn len(&self) -> usize {
        self.len
    }
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

impl SharedStringCacheWriter for FileSharedStringCache {
    fn put(&mut self, value: String) -> Result<()> {
        let offset = self.writer.seek(SeekFrom::End(0))?;
        let bytes = value.as_bytes();
        self.writer.write_all(bytes)?;
        self.entries.push((offset, bytes.len()));
        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        self.writer.flush()?;
        Ok(Box::new(FileSharedStringReader {
            temporary_file: self.temporary_file,
            path: self.path,
            entries: self.entries,
        }))
    }
}

impl SharedStringCacheReader for FileSharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        read_file_entry(&self.path, &self.entries, index)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl SharedStringCache for FileSharedStringCache {}

/// 完成写入后的文件缓存只读视图。
struct FileSharedStringReader {
    temporary_file: NamedTempFile,
    path: PathBuf,
    entries: Vec<(u64, usize)>,
}

impl SharedStringCacheReader for FileSharedStringReader {
    fn get(&self, index: usize) -> Result<String> {
        let _lifetime_guard = &self.temporary_file;
        read_file_entry(&self.path, &self.entries, index)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
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
}
