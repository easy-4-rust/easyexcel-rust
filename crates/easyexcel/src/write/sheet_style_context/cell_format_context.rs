#[derive(Clone, Copy)]
/// 对应 Java：ignoreFillStyle。
pub(crate) struct CellFormatContext<'a> {
    pub(crate) explicit: Option<&'a CellStyle>,
    pub(crate) cell: Option<ExcelCellStyle>,
    pub(crate) font: Option<ExcelFontStyle>,
    /// Style contributed by registered `WriteHandler` strategies
    /// (Java `AbstractCellStyleStrategy` merge into `WriteCellData`).
    pub(crate) handler_cell: Option<ExcelCellStyle>,
    /// Style returned by `Converter::convert_to_excel_data`.
    pub(crate) converted_cell: Option<ExcelCellStyle>,
    /// Owned runtime format carried by `WriteCellData::DataFormatData`.
    pub(crate) converted_data_format: Option<&'a str>,
    pub(crate) global: WriteGlobalFlags,
}

impl<'a> CellFormatContext<'a> {
    /// Attaches a strategy-derived cell style (Java `WriteCellStyle.merge`).
    #[must_use]
    /// 对应 Java：ignoreFillStyle。
    pub(crate) const fn with_handler_cell(mut self, handler_cell: Option<ExcelCellStyle>) -> Self {
        self.handler_cell = handler_cell;
        self
    }

    /// 对应 Java：ignoreFillStyle。 Attaches converter-produced style metadata without flattening it into
    /// the scalar value.
    #[must_use]
    pub(crate) fn with_converted_cell(mut self, cell: &'a WriteCellData) -> Self {
        self.converted_cell = cell.write_cell_style().copied();
        self.converted_data_format = cell.data_format_data().and_then(|data| data.format());
        self
    }

    /// 对应 Java：`ignoreFillStyle`: retain non-style write flags while
    /// suppressing explicit, annotation and strategy style materialization.
    pub(crate) const fn without_fill_style(mut self) -> Self {
        self.explicit = None;
        self.cell = None;
        self.font = None;
        self.handler_cell = None;
        self.converted_cell = None;
        self.converted_data_format = None;
        self
    }
}

