/// 对应 Java：无直接对应对象；Rust 架构扩展。 工作表绝对合并区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateMergeRange {
    /// 首行，零基。
    pub first_row: u32,
    /// 末行，零基且包含。
    pub last_row: u32,
    /// 首列，零基。
    pub first_column: u16,
    /// 末列，零基且包含。
    pub last_column: u16,
}

