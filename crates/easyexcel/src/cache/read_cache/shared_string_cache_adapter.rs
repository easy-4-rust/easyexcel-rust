/// 对应 Java：无直接对应对象；Rust 架构扩展。 Adapts the internal SAX cache writer to the Java `ReadCache` surface.
pub(crate) struct SharedStringCacheAdapter {
    inner: easyexcel_cache::SharedStringCacheHandle,
}

impl SharedStringCacheAdapter {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Wraps a live shared-string cache writer.
    #[must_use]
    pub fn new(inner: Box<dyn SharedStringCache>) -> Self {
        Self {
            inner: easyexcel_cache::SharedStringCacheHandle::new(inner),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回写入侧或已完成读取侧的字符串数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回缓存是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the read-only cache produced by [`ReadCache::put_finished`].
    ///
    /// # Panics
    ///
    /// Panics when called before [`ReadCache::put_finished`].
    // 内部缓存 API 脚手架，暂未在 crate 内直接调用。
    #[must_use]
    #[allow(dead_code)]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn into_reader(self) -> Box<dyn SharedStringCacheReader> {
        self.inner
            .into_reader()
            .expect("ReadCache.put_finished must run before into_reader")
    }
}

impl ReadCache for SharedStringCacheAdapter {
    fn put(&mut self, value: String) -> Result<()> {
        self.inner.put(value)?;
        Ok(())
    }

    fn get(&self, key: Option<usize>) -> Result<Option<String>> {
        let Some(index) = key else {
            return Ok(None);
        };
        Ok(Some(self.inner.get(index)?))
    }

    fn put_finished(&mut self) -> Result<()> {
        self.inner.finish()?;
        Ok(())
    }
}

