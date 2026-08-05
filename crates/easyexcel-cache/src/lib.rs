//! EasyExcel 可复用缓存引擎。

pub mod cache;

pub use cache::{
    DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES, DEFAULT_MOKA_ACTIVE_BATCHES,
    DEFAULT_MOKA_ACTIVE_ENTRIES, SHARED_STRING_CACHE_BATCH_SIZE, ReadCacheMode,
    SharedStringCache, SharedStringCacheHandle, SharedStringCacheReader, SharedStringCacheWriter,
    create_cache, create_moka_cache, create_moka_cache_for_batches, create_weighted_moka_cache,
    create_weighted_moka_cache_mb, memory_cache, prebuilt_cache, remove_thread_local_cache,
};
