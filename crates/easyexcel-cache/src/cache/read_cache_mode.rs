//! 共享字符串缓存选择模式。

/// 共享字符串缓存选择模式，对应 Java `EasyExcel` 的 `ReadCacheSelector` 行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadCacheMode {
    /// 小型 `sharedStrings.xml` 使用顺序内存缓存，超过阈值后使用文件缓存。
    #[default]
    Auto,
    /// 将全部共享字符串保存在内存中，对应 Java `MapCache`。
    Memory,
    /// 使用生命周期内不淘汰的 Moka 对象缓存。
    Moka,
    /// 使用临时文件保存共享字符串，适合大文件 SAX 读取。
    File,
}
