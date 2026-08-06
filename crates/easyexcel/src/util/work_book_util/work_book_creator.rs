/// 对应 Java：无直接对应对象；Rust 架构扩展。 Backend factory used by [`create_work_book`].
pub trait WorkBookCreator {
    /// Concrete workbook produced by this backend.
    type WorkBook;

    /// Creates or opens the workbook.
    ///
    /// # Errors
    ///
    /// Returns an I/O, format or unsupported-operation error from the backend.
    fn create_work_book(self) -> Result<Self::WorkBook, ExcelError>;
}

