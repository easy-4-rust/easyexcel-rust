/// 对应 Java：无直接对应对象；Rust 架构扩展。 One inclusive merge region in BIFF coordinates (Java HSSF `CellRangeAddress`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Biff8Merge {
    /// First row (0-based).
    pub first_row: u16,
    /// Last row (0-based, inclusive).
    pub last_row: u16,
    /// First column (0-based).
    pub first_col: u8,
    /// Last column (0-based, inclusive).
    pub last_col: u8,
}

impl Biff8Merge {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Converts format-neutral inclusive bounds into BIFF8 coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when a row exceeds the BIFF8 65,536-row
    /// limit or a column exceeds the 256-column limit.
    pub fn try_from_bounds(
        first_row: u32,
        last_row: u32,
        first_col: u16,
        last_col: u16,
    ) -> Result<Self> {
        Ok(Self {
            first_row: checked_row_index(first_row)?,
            last_row: checked_row_index(last_row)?,
            first_col: checked_column_index(usize::from(first_col))?,
            last_col: checked_column_index(usize::from(last_col))?,
        })
    }
}

