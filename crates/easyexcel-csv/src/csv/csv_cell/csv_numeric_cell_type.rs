/// 对应 Java：无直接对应对象；Rust 架构扩展。 CSV 数字负载的基础分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvNumericCellType {
    /// 普通数字。
    Number,
    /// Excel 日期序列。
    Date,
}
