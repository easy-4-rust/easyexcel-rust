/// 对应 Java：无直接对应对象；Rust 架构扩展。 Tag → handler routing table, mirroring Java `XlsxRowHandler.XLSX_CELL_HANDLER_MAP`.
pub enum RoutedHandler {
    /// `<c>`
    Cell(CellTagHandler),
    /// `<row>`
    Row(RowTagHandler),
    /// `<v>`
    CellValue(CellValueTagHandler),
    /// inline `<t>`
    InlineString(CellInlineStringValueTagHandler),
    /// `<f>`
    Formula(CellFormulaTagHandler),
    /// `<dimension>`
    Count(CountTagHandler),
    /// `<mergeCell>`
    Merge(MergeCellTagHandler),
    /// `<hyperlink>`
    Hyperlink(HyperlinkTagHandler),
}

impl RoutedHandler {
    fn as_mut(&mut self) -> &mut dyn XlsxTagHandler {
        match self {
            Self::Cell(h) => h,
            Self::Row(h) => h,
            Self::CellValue(h) => h,
            Self::InlineString(h) => h,
            Self::Formula(h) => h,
            Self::Count(h) => h,
            Self::Merge(h) => h,
            Self::Hyperlink(h) => h,
        }
    }
}

