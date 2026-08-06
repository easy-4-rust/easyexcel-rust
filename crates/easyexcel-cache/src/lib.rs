//! `EasyExcel` 可复用缓存引擎。

pub mod cache;

pub use cache::{
    DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES, ReadCacheMode, SharedStringCache,
    SharedStringCacheHandle, SharedStringCachePolicy, SharedStringCacheReader,
    SharedStringCacheWriter, create_cache, create_file_cache, create_memory_cache,
    create_moka_cache, memory_cache, prebuilt_cache, remove_thread_local_cache,
};
