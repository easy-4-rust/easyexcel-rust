/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 单元格区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Biff8CellRange {
    /// 首行。
    pub first_row: u32,
    /// 尾行。
    pub last_row: u32,
    /// 首列。
    pub first_column: usize,
    /// 尾列。
    pub last_column: usize,
}

