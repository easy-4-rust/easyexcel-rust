/// 对应 Java：无直接对应对象；Rust 架构扩展。 可跨 `CONTINUE` 记录保存的 BIFF8 逻辑记录类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8ContinuableRecordKind {
    /// 共享字符串表（SST）。
    SharedStringTable,
    /// 公式缓存结果后的 Unicode STRING 记录。
    UnicodeString,
}

