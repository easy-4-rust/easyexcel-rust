/// 对应 Java：无直接对应对象；Rust 架构扩展。 The outcome of a recalculation pass.
#[derive(Debug, Default)]
pub struct RecalcReport {
    /// Formula cells that participate in a circular reference.
    pub circular: Vec<Coord>,
    /// Number of formula cells evaluated.
    pub evaluated: usize,
}

