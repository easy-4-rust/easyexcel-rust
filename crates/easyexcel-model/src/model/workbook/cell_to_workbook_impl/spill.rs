/// 对应 Java：无直接对应对象；Rust 架构扩展。 A spilled dynamic-array result, owned by the anchor cell (top-left). The
/// anchor is a real formula cell whose cached value is the top-left element;
/// the remaining region cells are *derived* (not stored in `cells`) and read
/// from here. Spills are recomputed on every recalc, never persisted.
#[derive(Debug, Clone, PartialEq)]
pub struct Spill {
    pub rows: u32,
    pub cols: u32,
    /// Row-major values, length `rows * cols` (index 0 is the anchor).
    pub values: Vec<CellValue>,
}

