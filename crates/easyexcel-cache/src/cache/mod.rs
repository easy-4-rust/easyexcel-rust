//! 共享字符串缓存协议与实现。

mod read_cache_mode;
mod shared_string_cache;

pub use read_cache_mode::ReadCacheMode;
pub use shared_string_cache::{
    DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES, DEFAULT_MOKA_ACTIVE_BATCHES,
    DEFAULT_MOKA_ACTIVE_ENTRIES, SHARED_STRING_CACHE_BATCH_SIZE, SharedStringCache,
    SharedStringCacheReader, SharedStringCacheWriter, create_cache, create_moka_cache,
    create_moka_cache_for_batches, create_weighted_moka_cache, create_weighted_moka_cache_mb,
    memory_cache, remove_thread_local_cache,
};
