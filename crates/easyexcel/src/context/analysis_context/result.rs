/// Result alias used across the `easyexcel` crates.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type Result<T> = std::result::Result<T, ExcelError>;

