/// 对应 Java：无直接对应对象；Rust 架构扩展。 共享字符串顺序写入阶段。
pub trait SharedStringCacheWriter {
    /// 追加一条共享字符串。
    ///
    /// # Errors
    ///
    /// 后备存储写入失败时返回错误。
    fn put(&mut self, value: String) -> Result<()>;

    /// 结束写入并返回线程安全的只读视图。
    ///
    /// # Errors
    ///
    /// 后备存储无法完成落盘时返回错误。
    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>>;
}

impl SharedStringCacheWriter for MemorySharedStringCache {
    fn put(&mut self, value: String) -> Result<()> {
        self.values.push(value);
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        Ok(Box::new(MemorySharedStringReader {
            values: self.values,
        }))
    }
}

impl SharedStringCacheWriter for PrebuiltSharedStringCache {
    fn put(&mut self, _value: String) -> Result<()> {
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        Ok(Box::new(MemorySharedStringReader {
            values: self.values,
        }))
    }
}

impl SharedStringCacheWriter for MokaSharedStringCache {
    fn put(&mut self, value: String) -> Result<()> {
        let index = self.len;
        self.objects.insert(index, Arc::<str>::from(value));
        self.len = self.len.saturating_add(1);
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        let Self { objects, len } = *self;
        Ok(Box::new(MokaSharedStringReader { objects, len }))
    }
}

impl SharedStringCacheWriter for FileSharedStringCache {
    fn put(&mut self, value: String) -> Result<()> {
        let offset = self.writer.seek(SeekFrom::End(0))?;
        let bytes = value.as_bytes();
        self.writer.write_all(bytes)?;
        self.entries.push((offset, bytes.len()));
        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<Box<dyn SharedStringCacheReader>> {
        self.writer.flush()?;
        Ok(Box::new(FileSharedStringReader {
            _temporary_file: self.temporary_file,
            path: self.path,
            entries: self.entries,
        }))
    }
}

