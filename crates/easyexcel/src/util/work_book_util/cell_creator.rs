/// 对应 Java：无直接对应对象；Rust 架构扩展。 Backend row capable of creating a logical cell.
pub trait CellCreator {
    /// Concrete cell handle returned by this backend.
    type Cell<'a>
    where
        Self: 'a;

    /// Creates a cell at a zero-based column index.
    ///
    /// # Errors
    ///
    /// Returns a format error when the column is outside the backend limit.
    fn create_cell(&mut self, column_index: u16) -> Result<Self::Cell<'_>, ExcelError>;
}

