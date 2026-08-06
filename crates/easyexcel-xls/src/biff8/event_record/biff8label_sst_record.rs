/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 LABELSST 记录。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Biff8LabelSstRecord {
    /// 单元格公共头。
    pub header: Biff8CellHeader,
    /// 共享字符串表索引。
    pub sst_index: usize,
}

