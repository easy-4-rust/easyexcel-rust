/// `easyexcel` 门面的统一结果类型。
///
/// 对应 Java：无直接对应对象；Rust 惯用错误返回载体。
pub type Result<T> = std::result::Result<T, crate::support::excel_error::ExcelError>;
