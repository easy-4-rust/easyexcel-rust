//! 共享字符串缓存协议与实现。

mod read_cache_mode;
mod shared_string_cache;
mod shared_string_cache_policy;

pub use read_cache_mode::ReadCacheMode;
pub use shared_string_cache::{
    DEFAULT_MAX_MEMORY_SHARED_STRINGS_BYTES, SharedStringCache, SharedStringCacheHandle,
    SharedStringCacheReader, SharedStringCacheWriter, create_cache, create_file_cache,
    create_memory_cache, create_moka_cache, memory_cache, prebuilt_cache,
    remove_thread_local_cache,
};
pub use shared_string_cache_policy::SharedStringCachePolicy;
