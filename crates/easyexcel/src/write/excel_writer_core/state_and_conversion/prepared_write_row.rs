/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct PreparedWriteRow {
    absent: bool,
    original_cells: Vec<CellValue>,
    cells: Vec<WriteCellData>,
}

