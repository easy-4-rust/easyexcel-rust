#[derive(Debug, Clone)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct PendingCollectionFill {
    pub(crate) wrapper: FillWrapper,
    pub(crate) config: FillConfig,
    pub(crate) order: usize,
    pub(crate) column_styles: std::collections::BTreeMap<usize, u32>,
}
