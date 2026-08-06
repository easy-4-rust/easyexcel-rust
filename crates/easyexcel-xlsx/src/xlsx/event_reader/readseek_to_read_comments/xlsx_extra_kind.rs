/// 对应 Java：无直接对应对象；Rust 架构扩展。 工作表附加信息种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XlsxExtraKind {
    /// 合并区域。
    Merge,
    /// 超链接。
    Hyperlink,
    /// 批注。
    Comment,
}

