/// 对应 Java：无直接对应对象；Rust 架构扩展。 共享字符串并发读取阶段。
pub trait SharedStringCacheReader: Send + Sync {
    /// 按零基下标读取共享字符串。
    ///
    /// # Errors
    ///
    /// 下标越界或后备存储读取失败时返回错误。
    fn get(&self, index: usize) -> Result<String>;

    /// 返回缓存中的共享字符串数量。
    fn len(&self) -> usize;

    /// 返回缓存是否为空。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl SharedStringCacheReader for MemorySharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        value_at(&self.values, index)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl SharedStringCacheReader for PrebuiltSharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        value_at(&self.values, index)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl SharedStringCacheReader for MemorySharedStringReader {
    fn get(&self, index: usize) -> Result<String> {
        value_at(&self.values, index)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl SharedStringCacheReader for MokaSharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        self.objects
            .get(&index)
            .map(|value| value.to_string())
            .ok_or_else(|| out_of_bounds(index))
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl SharedStringCacheReader for MokaSharedStringReader {
    fn get(&self, index: usize) -> Result<String> {
        self.objects
            .get(&index)
            .map(|value| value.to_string())
            .ok_or_else(|| out_of_bounds(index))
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl SharedStringCacheReader for FileSharedStringCache {
    fn get(&self, index: usize) -> Result<String> {
        read_file_entry(&self.path, &self.entries, index)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl SharedStringCacheReader for FileSharedStringReader {
    fn get(&self, index: usize) -> Result<String> {
        read_file_entry(&self.path, &self.entries, index)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

