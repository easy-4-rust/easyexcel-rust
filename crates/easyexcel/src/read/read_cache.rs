//! EasyExcel 共享字符串缓存兼容入口。
//!
//! 缓存协议、内存实现、Moka 活跃层和临时文件后备均由
//! `easyexcel-cache` 提供；本模块只维持 Java EasyExcel 风格的公开路径。

pub use easyexcel_cache::{
    ReadCacheMode, SharedStringCache, SharedStringCacheReader, SharedStringCacheWriter,
};

pub(crate) use easyexcel_cache::{
    DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES, create_cache, remove_thread_local_cache,
};

#[cfg(test)]
pub(crate) use easyexcel_cache::memory_cache;
