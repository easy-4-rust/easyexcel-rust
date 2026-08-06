/// 对应 Java：无直接对应对象；Rust 架构扩展。 TxO/CONTINUE 记录解码片段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biff8TextObjectFragment {
    /// `TxO` 起始记录。
    Start {
        /// 形状对象编号。
        object_id: u32,
        /// 记录内携带的可选文本。
        text: Option<String>,
    },
    /// 后续 CONTINUE 文本。
    Continue(String),
}

