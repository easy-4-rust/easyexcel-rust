//! EasyExcel 可复用缓存引擎。

pub mod cache;

pub use cache::{
    DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES, DEFAULT_MOKA_ACTIVE_ENTRIES, ReadCacheMode,
    SharedStringCache, SharedStringCacheReader, SharedStringCacheWriter, create_cache,
    create_moka_cache, create_weighted_moka_cache, memory_cache, remove_thread_local_cache,
};
