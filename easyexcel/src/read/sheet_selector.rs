//! Sheet selection for workbook reads.

/// Selects a worksheet by index, name, or all sheets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SheetSelector {
    /// The first worksheet.
    #[default]
    First,
    /// A zero-based worksheet index.
    Index(usize),
    /// A worksheet name.
    Name(String),
    /// Every worksheet in workbook order.
    All,
}
