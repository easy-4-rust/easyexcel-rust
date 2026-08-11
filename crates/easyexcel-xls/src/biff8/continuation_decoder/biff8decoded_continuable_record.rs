/// 对应 Java：无直接对应对象；Rust 架构扩展。 已完整解码的 BIFF8 可续接逻辑记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biff8DecodedContinuableRecord {
    /// 按 BIFF 索引顺序排列的共享字符串。
    /// 当 `xls-lazy-sst` feature 启用时使用延迟解码容器；
    /// 否则使用立即解码的 `Vec`。
    #[cfg(feature = "xls-lazy-sst")]
    SharedStrings(crate::biff8::lazy_sst::LazySst),
    /// 按 BIFF 索引顺序排列的共享字符串（旧立即解码路径）。
    #[cfg(not(feature = "xls-lazy-sst"))]
    SharedStrings(Vec<crate::xls::Biff8SstString>),
    /// 一个完整的 Unicode 字符串。
    UnicodeString(String),
}
