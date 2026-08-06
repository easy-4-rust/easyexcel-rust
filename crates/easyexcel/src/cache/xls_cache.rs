//! 对应 Java：`com.alibaba.excel.cache.XlsCache`。
//!
//! 预构建 SST 的存储与索引实现由 `easyexcel-cache` 提供；本类型仅适配
//! Java `ReadCache` 生命周期。

use crate::core::Result;

use super::read_cache::{ReadCache, SharedStringCacheAdapter};

/// 对应 Java：com.alibaba.excel.cache.XlsCache。 BIFF SST 预构建共享字符串缓存的 Java 兼容门面。
pub struct XlsCache {
    adapter: SharedStringCacheAdapter,
}

impl XlsCache {
    /// 对应 Java：com.alibaba.excel.cache.XlsCache。 从 SST 索引顺序字符串创建缓存。
    #[must_use]
    pub fn new(values: Vec<String>) -> Self {
        Self {
            adapter: SharedStringCacheAdapter::new(easyexcel_cache::prebuilt_cache(values)),
        }
    }

    /// 对应 Java：com.alibaba.excel.cache.XlsCache。 创建空的 SST 缓存占位符。
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// 对应 Java：com.alibaba.excel.cache.XlsCache。 返回字符串数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.adapter.len()
    }

    /// 对应 Java：com.alibaba.excel.cache.XlsCache。 返回缓存是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.adapter.is_empty()
    }
}

impl ReadCache for XlsCache {
    fn put(&mut self, value: String) -> Result<()> {
        self.adapter.put(value)
    }

    fn get(&self, key: Option<usize>) -> Result<Option<String>> {
        let Some(index) = key else {
            return Ok(None);
        };
        if index >= self.adapter.len() {
            return Ok(None);
        }
        self.adapter.get(Some(index))
    }

    fn put_finished(&mut self) -> Result<()> {
        self.adapter.put_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_prebuilt_sst_and_ignores_puts() -> Result<()> {
        let mut cache = XlsCache::new(vec!["alpha".to_owned(), "beta".to_owned()]);
        cache.put("ignored".to_owned())?;
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(Some(0))?, Some("alpha".to_owned()));
        assert_eq!(cache.get(Some(2))?, None);
        cache.put_finished()?;
        assert_eq!(cache.get(Some(1))?, Some("beta".to_owned()));
        Ok(())
    }
}
