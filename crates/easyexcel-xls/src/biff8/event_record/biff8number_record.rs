/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 NUMBER 记录。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Biff8NumberRecord {
    /// 单元格公共头。
    pub header: Biff8CellHeader,
    /// IEEE-754 数值。
    pub value: f64,
}

