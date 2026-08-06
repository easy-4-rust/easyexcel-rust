/// 对应 Java：无直接对应对象；Rust 架构扩展。 Column metadata (width / hidden state).
#[derive(Debug, Clone, Copy, Default)]
pub struct ColInfo {
    /// Width in character units (Excel's measure). `None` means default.
    pub width: Option<f64>,
    pub hidden: bool,
    pub style: Option<u32>,
}

