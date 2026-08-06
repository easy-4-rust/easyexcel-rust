/// 对应 Java：无直接对应对象；Rust 架构扩展。 Backend workbook capable of creating a sheet.
pub trait SheetCreator {
    /// Concrete sheet handle returned by this backend.
    type Sheet<'a>
    where
        Self: 'a;

    /// Creates a sheet with the supplied name.
    ///
    /// # Errors
    ///
    /// Returns a format error for an invalid or duplicate sheet name.
    fn create_sheet(&mut self, sheet_name: &str) -> Result<Self::Sheet<'_>, ExcelError>;
}

