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
    /// Cell hyperlinks emitted as BIFF8 HLINK records.
    pub hyperlinks: Vec<Biff8Hyperlink>,
    /// 单元格批注；序列化为 MSODRAWING/OBJ/TXO/CONTINUE/NOTE 记录组。
    pub comments: Vec<Biff8Comment>,
    /// 内嵌 BIFF8 图表。
    pub charts: Vec<Biff8Chart>,
    /// 工作表保护的 16 位 Excel XOR verifier。
    pub protection_password_hash: Option<u16>,
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
            hyperlinks: Vec::new(),
            comments: Vec::new(),
            charts: Vec::new(),
            protection_password_hash: None,
            freeze: None,
            next_row: 0,
            next_data_index: 0,
        }
    }

    /// 使用 Excel 传统工作表密码 verifier 保护本工作表。
    pub fn protect_sheet(&mut self, password: &str) {
        self.protection_password_hash = Some(super::protection::legacy_password_hash(password));
    }

    /// 添加已经完成坐标与系列校验的内嵌图表。
    ///
    /// 对应 Java：`HSSFPatriarch#createChart(ClientAnchor)`。
    pub fn add_chart(&mut self, chart: Biff8Chart) {
        self.charts.push(chart);
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

    /// Adds a URL hyperlink attached to one cell.
    ///
    /// 对应 Java：`HSSFCell#setHyperlink(Hyperlink)` / `HyperlinkRecord`。
    ///
    /// # Errors
    ///
    /// Returns a BIFF8 format error for invalid coordinates, embedded NULs, or
    /// a link whose encoded HLINK payload exceeds the BIFF8 record limit.
    pub fn add_hyperlink(
        &mut self,
        row: u32,
        col: usize,
        url: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<()> {
        self.add_typed_hyperlink(
            row,
            row,
            col,
            col,
            url,
            label,
            Biff8HyperlinkKind::Url,
        )
    }

    /// 添加带类型和覆盖范围的 BIFF8 超链接。
    ///
    /// 对应 Java：`HSSFCell#setHyperlink(Hyperlink)` 以及 POI
    /// `Hyperlink#setFirstRow/setLastRow/setFirstColumn/setLastColumn`。
    ///
    /// # Errors
    ///
    /// 坐标越界、范围倒置、文本包含 NUL 或 HLINK 记录超过 BIFF8 上限时返回错误。
    #[allow(clippy::too_many_arguments)]
    pub fn add_typed_hyperlink(
        &mut self,
        first_row: u32,
        last_row: u32,
        first_col: usize,
        last_col: usize,
        address: impl Into<String>,
        label: impl Into<String>,
        kind: Biff8HyperlinkKind,
    ) -> Result<()> {
        let hyperlink = Biff8Hyperlink::new_range(
            checked_row_index(first_row)?,
            checked_row_index(last_row)?,
            checked_column_index(first_col)?,
            checked_column_index(last_col)?,
            address.into(),
            label.into(),
            kind,
        )?;
        self.hyperlinks.push(hyperlink);
        Ok(())
    }

    /// 添加单元格批注。
    ///
    /// 对应 Java：`HSSFCell#setCellComment(HSSFComment)`。
    ///
    /// # Errors
    ///
    /// 坐标越界、文本或作者包含 NUL、文本超过 BIFF8 TXO 长度时返回错误。
    pub fn add_comment(
        &mut self,
        row: u32,
        col: usize,
        text: impl Into<String>,
        author: impl Into<String>,
    ) -> Result<()> {
        let text = text.into();
        let author = author.into();
        if text.contains('\0') || author.contains('\0') {
            return Err(ExcelError::Xls(
                "BIFF8 comment text and author cannot contain NUL".to_owned(),
            ));
        }
        if text.encode_utf16().count() > usize::from(u16::MAX)
            || author.encode_utf16().count() > usize::from(u16::MAX)
        {
            return Err(ExcelError::Xls(
                "BIFF8 comment text or author exceeds 65535 UTF-16 units".to_owned(),
            ));
        }
        self.comments.push(Biff8Comment::new(
            checked_row_index(row)?,
            checked_column_index(col)?,
            text,
            author,
        ));
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
