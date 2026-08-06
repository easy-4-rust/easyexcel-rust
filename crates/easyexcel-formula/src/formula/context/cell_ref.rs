/// 对应 Java：无直接对应对象；Rust 架构扩展。 The cell currently being evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRef {
    pub sheet: usize,
    pub row: u32,
    pub col: u32,
}

