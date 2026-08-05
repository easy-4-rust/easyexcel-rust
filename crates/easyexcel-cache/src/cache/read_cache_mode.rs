//! 共享字符串缓存选择模式。

/// 共享字符串缓存选择模式，对应 Java EasyExcel 的 `ReadCacheSelector` 行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadCacheMode {
    /// 小型 `sharedStrings.xml` 使用内存，超过阈值后使用 Moka 热缓存与临时文件后备。
    #[default]
    Auto,
    /// 将全部共享字符串保存在内存中，对应 Java `MapCache`。
    Memory,
    /// 使用 Moka 热缓存与临时文件持久后备，对应 Java `Ehcache`。
    Disk,
}
