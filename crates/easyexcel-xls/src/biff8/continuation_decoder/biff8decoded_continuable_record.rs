/// 对应 Java：无直接对应对象；Rust 架构扩展。 已完整解码的 BIFF8 可续接逻辑记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biff8DecodedContinuableRecord {
    /// 按 BIFF 索引顺序排列的共享字符串。
    SharedStrings(Vec<String>),
    /// 一个完整的 Unicode 字符串。
    UnicodeString(String),
}

