/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 BOF 记录声明的子流类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8BofType {
    /// 工作簿全局子流。
    Workbook,
    /// 工作表子流。
    Worksheet,
    /// 其他 BIFF 子流类型。
    Other(u16),
}

