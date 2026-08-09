/// 后端中立的绝对合并范围（行区间 × 列区间）。
///
/// 对应 Java：无直接对应对象；Rust 模型扩展。facade 的 POI
/// `CellRangeAddress` 兼容路径重导出本类型，XLS/XLSX 引擎直接消费它。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MergeRange {
    /// 起始行（零基）。
    pub first_row: u32,
    /// 结束行（零基，包含）。
    pub last_row: u32,
    /// 起始列（零基）。
    pub first_column: u16,
    /// 结束列（零基，包含）。
    pub last_column: u16,
}

impl MergeRange {
    /// 创建绝对合并范围。
    #[must_use]
    pub const fn new(first_row: u32, last_row: u32, first_column: u16, last_column: u16) -> Self {
        Self {
            first_row,
            last_row,
            first_column,
            last_column,
        }
    }
}
