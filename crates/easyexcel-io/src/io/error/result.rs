/// 表格 I/O 的统一结果类型。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type Result<T> = std::result::Result<T, Error>;

