/// 对应 Java：无直接对应对象；Rust 架构扩展。 Sparse worksheet buffer accumulated by the stateful / one-shot writers.
#[derive(Debug, Clone, Default)]
pub struct Biff8Sheet {
    /// Worksheet name (BOUNDSHEET short Unicode string).
    pub name: String,
    /// Sparse cells keyed by `(row, column)` in 0-based BIFF coordinates.
    pub cells: BTreeMap<(u16, u8), Biff8Cell>,
    /// Column widths in Excel character units (Java `sheet.setColumnWidth`).
    pub column_widths: BTreeMap<u8, u16>,
    /// Row heights in points (Java `row.setHeightInPoints`).
    pub row_heights: BTreeMap<u16, u16>,
    /// Merged regions (Java `addMergedRegion` / `MergedCellsTable`).
    pub merges: Vec<Biff8Merge>,
    /// Frozen panes as `(rows, cols)` — Java `Sheet.createFreezePane(row, col)`,
    /// emitted as a `PANE` record + `WINDOW2` fFrozen flags.
    pub freeze: Option<(u16, u16)>,
    /// Next free row index (includes any header rows already written).
    pub next_row: u32,
    /// Next data-row index used for content-style cycling parity with XLSX.
    pub next_data_index: usize,
}

impl Biff8Sheet {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates an empty sheet with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cells: BTreeMap::new(),
            column_widths: BTreeMap::new(),
            row_heights: BTreeMap::new(),
            merges: Vec::new(),
            freeze: None,
            next_row: 0,
            next_data_index: 0,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Validates a format-neutral row index against BIFF8 limits.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when `row` is outside the BIFF8 range.
    pub fn validate_row_index(row: u32) -> Result<()> {
        checked_row_index(row).map(|_| ())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Narrows a format-neutral column index after applying BIFF8 limits.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when `col` is outside the BIFF8 range.
    pub fn column_index(col: usize) -> Result<u16> {
        checked_column_index(col).map(u16::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes a cell at `(row, col)`, enforcing BIFF8 row/column limits.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when the coordinate exceeds BIFF8 limits
    /// (`65_536` rows × 256 columns).
    pub fn set(&mut self, row: u32, col: usize, cell: Biff8Cell) -> Result<()> {
        let row = checked_row_index(row)?;
        let col = checked_column_index(col)?;
        self.cells.insert((row, col), cell);
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Validates and sets a column width using a format-neutral column index.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when `col` exceeds the BIFF8 column limit.
    pub fn set_column_width_at(&mut self, col: usize, width_chars: u16) -> Result<()> {
        self.set_column_width(checked_column_index(col)?, width_chars);
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Sets column width in character units (POI `setColumnWidth(col, chars*256)`).
    pub fn set_column_width(&mut self, col: u8, width_chars: u16) {
        self.column_widths.insert(col, width_chars);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Sets row height in points (POI `setHeightInPoints`).
    pub fn set_row_height(&mut self, row: u16, height_points: u16) {
        self.row_heights.insert(row, height_points);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Validates and sets a row height using a format-neutral row index.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when `row` exceeds the BIFF8 row limit.
    pub fn set_row_height_at(&mut self, row: u32, height_points: u16) -> Result<()> {
        self.set_row_height(checked_row_index(row)?, height_points);
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Configures frozen panes while enforcing BIFF8 row and column bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when the frozen row or column count is
    /// outside the BIFF8 coordinate range.
    pub fn set_freeze_panes(&mut self, rows: u32, cols: u16) -> Result<()> {
        let rows = checked_row_index(rows)?;
        checked_column_index(usize::from(cols))?;
        self.freeze = Some((rows, cols));
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends a merge region when it spans more than one cell.
    ///
    /// # Errors
    ///
    /// Returns a format error when `last_row`/`last_col` precede
    /// `first_row`/`first_col` (reversed range).
    pub fn add_merge(&mut self, merge: Biff8Merge) -> Result<()> {
        if merge.last_row < merge.first_row || merge.last_col < merge.first_col {
            return Err(ExcelError::Xls(
                "BIFF8 merge last row/col must be >= first".to_owned(),
            ));
        }
        if merge.first_row == merge.last_row && merge.first_col == merge.last_col {
            return Ok(());
        }
        self.merges.push(merge);
        Ok(())
    }

    /// Returns exclusive `(max_row, max_col)` for the DIMENSION record.
    fn dimensions(&self) -> (u32, u16) {
        let mut max_row = 0u32;
        let mut max_col = 0u16;
        for &(row, col) in self.cells.keys() {
            max_row = max_row.max(u32::from(row).saturating_add(1));
            max_col = max_col.max(u16::from(col).saturating_add(1));
        }
        for merge in &self.merges {
            max_row = max_row.max(u32::from(merge.last_row).saturating_add(1));
            max_col = max_col.max(u16::from(merge.last_col).saturating_add(1));
        }
        (max_row, max_col)
    }
}

