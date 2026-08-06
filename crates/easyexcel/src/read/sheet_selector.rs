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

impl SheetSelector {
    /// 映射为格式无关的引擎选择请求。
    #[must_use]
    pub(crate) fn as_engine_selection(&self) -> easyexcel_io::SheetSelection<'_> {
        match self {
            Self::First => easyexcel_io::SheetSelection::First,
            Self::Index(index) => easyexcel_io::SheetSelection::Index(*index),
            Self::Name(name) => easyexcel_io::SheetSelection::Name(name),
            Self::All => easyexcel_io::SheetSelection::All,
        }
    }
}
