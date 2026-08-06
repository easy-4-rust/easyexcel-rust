//! 文件共享字符串缓存的 `EasyExcel` `ReadCache` 适配。
//!
//! 临时文件创建、索引和生命周期由 `easyexcel-cache` 实现；本模块仅适配
//! `EasyExcel` 门面的错误与生命周期契约。

use crate::core::Result;

use super::read_cache::{ReadCache, SharedStringCacheAdapter};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 适用于大型 `sharedStrings.xml` SAX 读取的文件缓存。
pub struct FileCache {
    adapter: SharedStringCacheAdapter,
}

impl FileCache {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建临时文件共享字符串缓存。
    ///
    /// # Errors
    ///
    /// 临时文件无法创建时返回 I/O 错误。
    pub fn new() -> Result<Self> {
        Ok(Self {
            adapter: SharedStringCacheAdapter::new(easyexcel_cache::create_file_cache()?),
        })
    }
}

impl ReadCache for FileCache {
    fn put(&mut self, value: String) -> Result<()> {
        self.adapter.put(value)
    }

    fn get(&self, key: Option<usize>) -> Result<Option<String>> {
        self.adapter.get(key)
    }

    fn put_finished(&mut self) -> Result<()> {
        self.adapter.put_finished()
    }

    fn destroy(&mut self) {
        self.adapter = SharedStringCacheAdapter::new(easyexcel_cache::create_memory_cache());
    }
}
