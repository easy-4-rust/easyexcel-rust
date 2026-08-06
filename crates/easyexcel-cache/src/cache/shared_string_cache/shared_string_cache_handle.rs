/// 对应 Java：无直接对应对象；Rust 架构扩展。 管理共享字符串缓存从顺序写入到并发只读的阶段切换。
///
/// 该状态机属于缓存引擎，不依赖 `EasyExcel` Java 门面的 `ReadCache` trait。
/// 门面只需把自身的可空索引和错误类型映射到这个句柄。
pub struct SharedStringCacheHandle {
    writer: Box<dyn SharedStringCache>,
    reader: Option<Box<dyn SharedStringCacheReader>>,
}

impl SharedStringCacheHandle {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 包装一个仍处于写入阶段的共享字符串缓存。
    #[must_use]
    pub fn new(writer: Box<dyn SharedStringCache>) -> Self {
        Self {
            writer,
            reader: None,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 追加一条共享字符串。
    ///
    /// # Errors
    ///
    /// 后备存储写入失败时返回错误。
    pub fn put(&mut self, value: String) -> Result<()> {
        self.writer.put(value)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按零基索引读取共享字符串。
    ///
    /// 完成前从写入缓存读取，完成后从不可变读取视图读取。
    ///
    /// # Errors
    ///
    /// 索引越界或后备存储读取失败时返回错误。
    pub fn get(&self, index: usize) -> Result<String> {
        self.reader
            .as_ref()
            .map_or_else(|| self.writer.get(index), |reader| reader.get(index))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 完成写入并切换到只读阶段；重复调用是幂等的。
    ///
    /// # Errors
    ///
    /// 后备存储无法完成落盘时返回错误。
    pub fn finish(&mut self) -> Result<()> {
        if self.reader.is_some() {
            return Ok(());
        }
        let writer = std::mem::replace(
            &mut self.writer,
            Box::new(MemorySharedStringCache::default()),
        );
        self.reader = Some(writer.finish()?);
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回当前缓存中的共享字符串数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.reader
            .as_ref()
            .map_or_else(|| self.writer.len(), |reader| reader.len())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回缓存是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 取出完成后的不可变读取视图。
    #[must_use]
    pub fn into_reader(self) -> Option<Box<dyn SharedStringCacheReader>> {
        self.reader
    }
}

