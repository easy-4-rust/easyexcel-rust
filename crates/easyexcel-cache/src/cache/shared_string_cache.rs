//! 共享字符串缓存的内存、临时文件和 Moka 分层实现。

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

/// Moka 活跃层默认最多保留的共享字符串条目数。
pub const DEFAULT_MOKA_ACTIVE_ENTRIES: u64 = 2_000;

/// Java `Ehcache` 每个共享字符串批次包含的条目数。
pub const SHARED_STRING_CACHE_BATCH_SIZE: u64 = 100;

/// Java `SimpleReadCacheSelector` 默认保留的活跃批次数。
pub const DEFAULT_MOKA_ACTIVE_BATCHES: u64 = 20;

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
/// 需要临时文件而文件创建失败时返回错误。
pub fn create_cache(mode: ReadCacheMode, xml_size: u64) -> Result<Box<dyn SharedStringCache>> {
    match mode {
        ReadCacheMode::Auto if xml_size < DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES => {
            Ok(create_memory_cache())
        }
        ReadCacheMode::Auto | ReadCacheMode::Disk => {
            create_moka_cache(DEFAULT_MOKA_ACTIVE_ENTRIES)
        }
        ReadCacheMode::Memory => Ok(create_memory_cache()),
    }
}

/// 创建按条目数量限制的 Moka 热缓存与临时文件后备。
///
/// # Errors
///
/// 临时后备文件创建失败时返回错误。
pub fn create_moka_cache(max_active_entries: u64) -> Result<Box<dyn SharedStringCache>> {
    let active = Cache::builder()
        .max_capacity(max_active_entries.max(1))
        .build();
    Ok(Box::new(MokaSharedStringCache::new(active)?))
}

/// 按 Java `Ehcache` 批次数创建 Moka 热缓存与临时文件后备。
///
/// # Errors
///
/// 临时后备文件创建失败时返回错误。
pub fn create_moka_cache_for_batches(
    max_active_batches: u64,
) -> Result<Box<dyn SharedStringCache>> {
    create_moka_cache(
        max_active_batches
            .max(1)
            .saturating_mul(SHARED_STRING_CACHE_BATCH_SIZE),
    )
}

/// 创建按 UTF-8 字节权重限制的 Moka 热缓存与临时文件后备。
///
/// # Errors
///
/// 临时后备文件创建失败时返回错误。
pub fn create_weighted_moka_cache(max_active_bytes: u64) -> Result<Box<dyn SharedStringCache>> {
    let active = Cache::builder()
        .max_capacity(max_active_bytes.max(1))
        .weigher(|_key: &usize, value: &Arc<str>| {
            u32::try_from(value.len()).unwrap_or(u32::MAX)
        })
        .build();
    Ok(Box::new(MokaSharedStringCache::new(active)?))
}

/// 按 Java 已废弃的 MB 容量参数创建加权 Moka 热缓存。
///
/// # Errors
///
/// 临时后备文件创建失败时返回错误。
pub fn create_weighted_moka_cache_mb(
    max_active_megabytes: u64,
) -> Result<Box<dyn SharedStringCache>> {
    create_weighted_moka_cache(max_active_megabytes.max(1).saturating_mul(1024 * 1024))
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
    active: Cache<usize, Arc<str>>,
    backing: DiskSharedStringCache,
}

impl MokaSharedStringCache {
    fn new(active: Cache<usize, Arc<str>>) -> Result<Self> {
        Ok(Self {
            active,
            backing: DiskSharedStringCache::new()?,
        })
    }
}

impl SharedStringCacheWriter for MokaSharedStringCache {
    fn put(&mut self, value: String) -> Result<()> {
        let index = self.backing.len();
        self.backing.put(value.clone())?;
        self.active.insert(index, Arc::<str>::from(value));
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        let Self { active, backing } = *self;
        Ok(Box::new(MokaSharedStringReader {
            active,
            backing: backing.into_reader()?,
        }))
    }
}

impl SharedStringCacheReader for MokaSharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        if let Some(value) = self.active.get(&index) {
            return Ok(value.to_string());
        }
        let value = self.backing.get(index)?;
        self.active.insert(index, Arc::<str>::from(value.as_str()));
        Ok(value)
    }

    fn len(&self) -> usize {
        self.backing.len()
    }
}

impl SharedStringCache for MokaSharedStringCache {}

struct DiskSharedStringCache {
    temporary_file: NamedTempFile,
    writer: File,
    path: PathBuf,
    entries: Vec<(u64, usize)>,
}

impl DiskSharedStringCache {
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

    fn put(&mut self, value: String) -> Result<()> {
        let offset = self.writer.seek(SeekFrom::End(0))?;
        let bytes = value.as_bytes();
        self.writer.write_all(bytes)?;
        self.entries.push((offset, bytes.len()));
        Ok(())
    }

    fn get(&self, index: usize) -> Result<String> {
        read_entry(&self.path, &self.entries, index)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn into_reader(mut self) -> Result<DiskSharedStringReader> {
        self.writer.flush()?;
        Ok(DiskSharedStringReader {
            temporary_file: self.temporary_file,
            path: self.path,
            entries: self.entries,
        })
    }
}

struct DiskSharedStringReader {
    temporary_file: NamedTempFile,
    path: PathBuf,
    entries: Vec<(u64, usize)>,
}

impl DiskSharedStringReader {
    fn get(&self, index: usize) -> Result<String> {
        let _lifetime_guard = &self.temporary_file;
        read_entry(&self.path, &self.entries, index)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

fn read_entry(path: &Path, entries: &[(u64, usize)], index: usize) -> Result<String> {
    let (offset, length) = entries.get(index).copied().ok_or_else(|| out_of_bounds(index))?;
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|error| Error::Other(error.to_string()))
}

fn out_of_bounds(index: usize) -> Error {
    Error::Other(format!("shared string index is out of bounds: {index}"))
}
