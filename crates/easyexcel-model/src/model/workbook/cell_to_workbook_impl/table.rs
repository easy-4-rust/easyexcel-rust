/// 对应 Java：无直接对应对象；Rust 架构扩展。 An Excel **table** object (the feature Excel calls a "Table" / ListObject): a
/// named, structured range with a header row and typed columns. Tables live on a
/// sheet and can be referenced from formulas with structured references like
/// `Sales[Amount]`.
#[derive(Debug, Clone)]
pub struct Table {
    /// The table's identifier (used in structured references).
    pub name: String,
    /// Display name shown in Excel (usually identical to `name`).
    pub display_name: String,
    /// Full range *including* the header and any totals row, sheet-relative.
    pub range: CellRange,
    /// Column header names, left to right.
    pub columns: Vec<String>,
    /// Number of header rows (0 or 1).
    pub header_rows: u32,
    /// Number of totals rows (0 or 1).
    pub totals_rows: u32,
    /// Numeric table id from the source file (0 = assign on write).
    pub id: u32,
    /// Original `<table>` XML, preserved verbatim so attributes we don't model
    /// (style, autofilter, calculated columns) round-trip losslessly. Empty for
    /// tables created in-memory — then the writer synthesizes minimal XML.
    pub raw_xml: Vec<u8>,
}

impl Table {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 The data-body range: the table range minus its header and totals rows.
    /// Returns `None` if the table has no body rows.
    #[must_use]
    pub fn data_range(&self) -> Option<CellRange> {
        let top = self.range.start.row + self.header_rows;
        let bottom = self.range.end.row.checked_sub(self.totals_rows)?;
        if top > bottom {
            return None;
        }
        Some(CellRange::new(
            CellAddress::new(top, self.range.start.col),
            CellAddress::new(bottom, self.range.end.col),
        ))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 The 0-based offset of a column (case-insensitive) within the table.
    #[must_use]
    pub fn column_index(&self, name: &str) -> Option<u32> {
        self.columns
            .iter()
            .position(|c| c.eq_ignore_ascii_case(name))
            .and_then(|index| u32::try_from(index).ok())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 The data-body range of a single column (case-insensitive header match).
    #[must_use]
    pub fn column_range(&self, name: &str) -> Option<CellRange> {
        let off = self.column_index(name)?;
        let body = self.data_range()?;
        let col = self.range.start.col + off;
        Some(CellRange::new(
            CellAddress::new(body.start.row, col),
            CellAddress::new(body.end.row, col),
        ))
    }
}

