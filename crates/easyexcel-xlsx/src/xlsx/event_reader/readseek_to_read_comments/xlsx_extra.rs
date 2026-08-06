/// 对应 Java：无直接对应对象；Rust 架构扩展。 中立的工作表附加信息事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxExtra {
    /// 附加信息种类。
    pub kind: XlsxExtraKind,
    /// 超链接目标或批注文本；合并区域为空。
    pub text: Option<String>,
    /// 起始行。
    pub first_row: u32,
    /// 结束行。
    pub last_row: u32,
    /// 起始列。
    pub first_column: usize,
    /// 结束列。
    pub last_column: usize,
}

