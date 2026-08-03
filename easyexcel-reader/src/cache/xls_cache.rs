//! 对应 Java：`com.alibaba.excel.cache.XlsCache`.
//!
//! Java builds the cache from a POI `SSTRecord` during BIFF event parsing.
//! Rust calamine resolves SST internally; this type remains for API parity and
//! for callers that already materialized a string table out-of-band.

use easyexcel_core::Result;

use super::read_cache::ReadCache;

/// XLS shared-string cache backed by a pre-built string table.
///
/// 对应 Java：`com.alibaba.excel.cache.XlsCache`.
///
/// [`put`](ReadCache::put) is a no-op because the SST is immutable after
/// construction, matching Java usage after `SstRecordHandler` finishes.
pub struct XlsCache {
    values: Vec<String>,
}

impl XlsCache {
    /// Creates a cache from an SST string table. (Java `new XlsCache(SSTRecord)`)
    #[must_use]
    pub fn new(values: Vec<String>) -> Self {
        Self { values }
    }

    /// Creates an empty cache placeholder.
    #[must_use]
    pub fn empty() -> Self {
        Self { values: Vec::new() }
    }

    /// Returns the number of indexed strings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether the cache contains no strings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl ReadCache for XlsCache {
    fn put(&mut self, _value: String) -> Result<()> {
        Ok(())
    }

    fn get(&self, key: Option<usize>) -> Result<Option<String>> {
        Ok(match key {
            Some(index) if index < self.values.len() => Some(self.values[index].clone()),
            _ => None,
        })
    }

    fn put_finished(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn xls_cache_reads_the_prebuilt_string_table() -> Result<()> {
        // 对应 Java：XlsCache(SSTRecord) 只读查询
        let cache = XlsCache::new(vec!["alpha".to_owned(), "beta".to_owned()]);
        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());
        assert_eq!(cache.get(Some(0))?, Some("alpha".to_owned()));
        assert_eq!(cache.get(Some(1))?, Some("beta".to_owned()));
        // 越界与空 key 均返回 None（对应 Java：get 未命中返回 null）
        assert_eq!(cache.get(Some(2))?, None);
        assert_eq!(cache.get(None)?, None);
        Ok(())
    }

    #[test]
    fn xls_cache_empty_placeholder_is_noop() -> Result<()> {
        // 对应 Java：new XlsCache() 空缓存占位
        let mut cache = XlsCache::empty();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        cache.put("ignored".to_owned())?;
        cache.put_finished()?;
        assert_eq!(cache.get(Some(0))?, None);
        Ok(())
    }
}
