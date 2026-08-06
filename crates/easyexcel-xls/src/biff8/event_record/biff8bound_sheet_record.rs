/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 BOUNDSHEET 记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8BoundSheetRecord {
    /// 工作表名称。
    pub name: String,
    /// 工作表 BOF 的绝对偏移。
    pub bof_position: u32,
}

