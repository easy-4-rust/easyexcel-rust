/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 单元格记录公共头。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Biff8CellHeader {
    /// 零基行号。
    pub row: u32,
    /// 零基列号。
    pub column: usize,
    /// XF 样式索引。
    pub xf_index: u16,
}

