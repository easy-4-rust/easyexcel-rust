/// 对应 Java：无直接对应对象；Rust 架构扩展。 Backend sheet capable of creating a logical row.
pub trait RowCreator {
    /// Concrete row handle returned by this backend.
    type Row<'a>
    where
        Self: 'a;

    /// Creates a row at a zero-based index.
    ///
    /// # Errors
    ///
    /// Returns a format error when the row is outside the backend limit.
    fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>, ExcelError>;
}

