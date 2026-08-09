//! 对应 Java：`com.alibaba.excel.cache.*` and `cache.selector.*`.
//!
//! ## Java ↔ Rust mapping
//!
//! | Java | Rust | Notes |
//! |------|------|-------|
//! | `MapCache` | [`MapCache`] | In-memory `HashMap`-style backend |
//! | `MokaCache` | [`MokaCache`] | Lifecycle-scoped object cache without entry eviction |
//! | legacy `Ehcache` | `easyexcel-cache::SharedStringCachePolicy` + [`ReadCache`] / [`ReadCacheSelector`] | Memory/File/Moka 组合替代；不保留 Ehcache 依赖或同名空壳 |
//! | file cache | [`FileCache`] | Temporary-file backend for bounded-memory SAX reads |
//! | `XlsCache` | [`XlsCache`] | Pre-built SST table for BIFF reads |
//! | `SimpleReadCacheSelector` | [`SimpleReadCacheSelector`] | 5 MB (`5_000_000` byte) Auto boundary |
//! | `EternalReadCacheSelector` | [`EternalReadCacheSelector`] | Pins Memory, Moka or File regardless of size |
//! | `ReadCache` | [`ReadCache`] | Shared-string put/get contract |
//!
//! XLSX SAX uses [`crate::read::read_cache::ReadCacheMode`] (`Auto` / `Memory` / `Moka` / `File`) wired
//! through `ReadOptions::read_cache` and optional `ReadOptions::read_cache_selector`.
//! Legacy XLS reads use the `easyexcel-xls` BIFF engine and do not consult these selectors.

mod file_cache;
mod map_cache;
mod moka_cache;
mod read_cache;
pub mod selector;
mod xls_cache;

pub use file_cache::FileCache;
pub use map_cache::MapCache;
pub use moka_cache::MokaCache;
pub use read_cache::ReadCache;
pub use selector::{EternalReadCacheSelector, ReadCacheSelector, SimpleReadCacheSelector};
pub use xls_cache::XlsCache;

pub use read_cache::{new_file_cache, new_map_cache, new_moka_cache, resolve_read_cache_mode};

#[cfg(test)]
mod tests;
