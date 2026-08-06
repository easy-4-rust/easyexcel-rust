/// 对应 Java：无直接对应对象；Rust 架构扩展。 Row metadata (height / hidden state).
#[derive(Debug, Clone, Copy, Default)]
pub struct RowInfo {
    /// Height in points. `None` means default.
    pub height: Option<f64>,
    pub hidden: bool,
    pub style: Option<u32>,
}

