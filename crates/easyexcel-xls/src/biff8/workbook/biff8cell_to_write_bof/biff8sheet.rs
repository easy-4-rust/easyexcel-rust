/// 对应 Java：无直接对应对象；Rust 架构扩展。 Sparse worksheet buffer accumulated by the stateful / one-shot writers.
#[derive(Debug, Clone, Default)]
pub struct Biff8Sheet {
    /// Worksheet name (BOUNDSHEET short Unicode string).
    pub name: String,
    /// BOUNDSHEET 可见性。
    pub visibility: easyexcel_model::Visibility,
    /// Sparse cells keyed by `(row, column)` in 0-based BIFF coordinates.
    pub cells: BTreeMap<(u16, u8), Biff8Cell>,
    /// Column widths in Excel character units (Java `sheet.setColumnWidth`).
    pub column_widths: BTreeMap<u8, u16>,
    /// 精确 BIFF8 `1/256` 字符列宽；存在时优先于整数兼容视图。
    column_width_units: BTreeMap<u8, u16>,
    /// 工作表默认列宽，使用 BIFF8 `1/256` 字符单位。
    default_column_width_units: Option<u16>,
    /// 列级默认 XF。
    column_xfs: BTreeMap<u8, u16>,
    /// 隐藏列集合。
    hidden_columns: BTreeSet<u8>,
    /// 宽度由调用者显式设置的列集合（COLINFO fUserSet）。
    column_user_set_widths: BTreeSet<u8>,
    /// Row heights in points (Java `row.setHeightInPoints`).
    pub row_heights: BTreeMap<u16, u16>,
    /// 精确 BIFF8 twips 行高；存在时优先于整数兼容视图。
    row_height_twips: BTreeMap<u16, u16>,
    /// 工作表默认行高，使用 BIFF8 twips。
    default_row_height_twips: Option<u16>,
    /// 行级默认 XF。
    row_xfs: BTreeMap<u16, u16>,
    /// 隐藏行集合。
    hidden_rows: BTreeSet<u16>,
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
            visibility: easyexcel_model::Visibility::Visible,
            cells: BTreeMap::new(),
            column_widths: BTreeMap::new(),
            column_width_units: BTreeMap::new(),
            default_column_width_units: None,
            column_xfs: BTreeMap::new(),
            hidden_columns: BTreeSet::new(),
            column_user_set_widths: BTreeSet::new(),
            row_heights: BTreeMap::new(),
            row_height_twips: BTreeMap::new(),
            default_row_height_twips: None,
            row_xfs: BTreeMap::new(),
            hidden_rows: BTreeSet::new(),
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
        self.column_width_units
            .insert(col, width_chars.saturating_mul(256));
        self.column_user_set_widths.insert(col);
    }

    /// 使用 BIFF8 原始 `1/256` 字符单位设置列宽。
    pub fn set_column_width_units_at(&mut self, col: usize, width_units: u16) -> Result<()> {
        let col = checked_column_index(col)?;
        self.column_width_units.insert(col, width_units);
        self.column_user_set_widths.insert(col);
        Ok(())
    }

    /// 设置工作表默认列宽，单位为 BIFF8 `1/256` 字符。
    pub fn set_default_column_width_units(&mut self, width_units: u16) {
        self.default_column_width_units = Some(width_units);
    }

    /// 设置列宽、列级 XF 与隐藏状态。
    ///
    /// 对应 Java：`HSSFSheet#setColumnWidth`、`setColumnHidden`、`setDefaultColumnStyle`。
    pub fn set_column_metadata_at(
        &mut self,
        col: usize,
        width_units: u16,
        xf_index: u16,
        hidden: bool,
        user_set_width: bool,
    ) -> Result<()> {
        let col = checked_column_index(col)?;
        self.column_width_units.insert(col, width_units);
        self.column_xfs.insert(col, xf_index);
        if user_set_width {
            self.column_user_set_widths.insert(col);
        } else {
            self.column_user_set_widths.remove(&col);
        }
        if hidden {
            self.hidden_columns.insert(col);
        } else {
            self.hidden_columns.remove(&col);
        }
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Sets row height in points (POI `setHeightInPoints`).
    pub fn set_row_height(&mut self, row: u16, height_points: u16) {
        self.row_heights.insert(row, height_points);
        self.row_height_twips
            .insert(row, height_points.saturating_mul(20));
    }

    /// 使用 BIFF8 原始 twips（`1/20` point）设置行高。
    pub fn set_row_height_twips_at(&mut self, row: u32, height_twips: u16) -> Result<()> {
        if !(2..=8_192).contains(&height_twips) {
            return Err(ExcelError::Xls(format!(
                "BIFF8 row height must be 2..=8192 twips, got {height_twips}"
            )));
        }
        let row = checked_row_index(row)?;
        self.row_height_twips.insert(row, height_twips);
        Ok(())
    }

    /// 设置工作表默认行高，单位为 BIFF8 twips。
    pub fn set_default_row_height_twips(&mut self, height_twips: u16) -> Result<()> {
        if !(1..=8_179).contains(&height_twips) {
            return Err(ExcelError::Xls(format!(
                "BIFF8 default row height must be 1..=8179 twips, got {height_twips}"
            )));
        }
        self.default_row_height_twips = Some(height_twips);
        Ok(())
    }

    /// 设置行高、行级 XF 与隐藏状态。
    ///
    /// 对应 Java：`HSSFRow#setHeightInPoints`、`setZeroHeight`、`setRowStyle`。
    pub fn set_row_metadata_at(
        &mut self,
        row: u32,
        height_twips: u16,
        custom_height: bool,
        xf_index: Option<u16>,
        hidden: bool,
    ) -> Result<()> {
        if !(2..=8_192).contains(&height_twips) {
            return Err(ExcelError::Xls(format!(
                "BIFF8 row height must be 2..=8192 twips, got {height_twips}"
            )));
        }
        let row = checked_row_index(row)?;
        if custom_height {
            self.row_height_twips.insert(row, height_twips);
        }
        if let Some(xf_index) = xf_index {
            self.row_xfs.insert(row, xf_index);
        } else {
            self.row_xfs.remove(&row);
        }
        if hidden {
            self.hidden_rows.insert(row);
        } else {
            self.hidden_rows.remove(&row);
        }
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Validates and sets a row height using a format-neutral row index.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when `row` exceeds the BIFF8 row limit.
    pub fn set_row_height_at(&mut self, row: u32, height_points: u16) -> Result<()> {
        let height_twips = height_points.checked_mul(20).ok_or_else(|| {
            ExcelError::Xls(format!("BIFF8 row height overflows twips: {height_points}pt"))
        })?;
        if !(2..=8_192).contains(&height_twips) {
            return Err(ExcelError::Xls(format!(
                "BIFF8 row height must be 2..=8192 twips, got {height_twips}"
            )));
        }
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

    /// 将后端中立合并范围转换并写入 BIFF8 工作表。
    pub fn add_merge_range(&mut self, range: easyexcel_model::MergeRange) -> Result<()> {
        self.add_merge(Biff8Merge::try_from_bounds(
            range.first_row,
            range.last_row,
            range.first_column,
            range.last_column,
        )?)
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
        self.set_comment(Biff8Comment::new(
            checked_row_index(row)?,
            checked_column_index(col)?,
            text,
            author,
        ));
        Ok(())
    }

    /// 新增或替换指定坐标的完整 BIFF8 批注。
    ///
    /// 对应 Java：`HSSFCell#setCellComment`；同一单元格只能关联一个 NOTE，
    /// 后写入的批注覆盖此前对象，避免输出重复 NOTE/OBJ/TXO 链。
    pub fn set_comment(&mut self, comment: Biff8Comment) {
        self.comments
            .retain(|existing| existing.row != comment.row || existing.col != comment.col);
        self.comments.push(comment);
    }

    /// 删除指定坐标的批注并返回是否实际删除。
    ///
    /// 对应 Java：`HSSFCell#removeCellComment()`。
    pub fn remove_comment(&mut self, row: u32, col: usize) -> Result<bool> {
        let row = checked_row_index(row)?;
        let col = checked_column_index(col)?;
        let before = self.comments.len();
        self.comments
            .retain(|comment| comment.row != row || comment.col != col);
        Ok(self.comments.len() != before)
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

#[cfg(test)]
mod biff8sheet_tests {
    use super::*;

    #[test]
    fn new_sheet_has_correct_name() {
        let sheet = Biff8Sheet::new("Sheet1");
        assert_eq!(sheet.name, "Sheet1");
        assert_eq!(sheet.visibility, easyexcel_model::Visibility::Visible);
        assert!(sheet.cells.is_empty());
        assert!(sheet.merges.is_empty());
        assert!(sheet.hyperlinks.is_empty());
        assert!(sheet.comments.is_empty());
        assert!(sheet.charts.is_empty());
        assert!(sheet.freeze.is_none());
        assert_eq!(sheet.next_row, 0);
    }

    #[test]
    fn new_sheet_with_string() {
        let sheet = Biff8Sheet::new("Test".to_owned());
        assert_eq!(sheet.name, "Test");
    }

    #[test]
    fn set_cell_valid_coordinates() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let cell = Biff8Cell::general(Biff8Value::Text("hello".to_owned()));
        sheet.set(0, 0, cell).expect("should succeed");
        assert!(sheet.cells.contains_key(&(0, 0)));
    }

    #[test]
    fn set_cell_max_row() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let cell = Biff8Cell::general(Biff8Value::Number(1.0));
        sheet.set(65535, 0, cell).expect("max row should be valid");
    }

    #[test]
    fn set_cell_out_of_range_row() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let cell = Biff8Cell::general(Biff8Value::Number(1.0));
        assert!(sheet.set(65536, 0, cell).is_err());
    }

    #[test]
    fn set_cell_out_of_range_column() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let cell = Biff8Cell::general(Biff8Value::Number(1.0));
        assert!(sheet.set(0, 256, cell).is_err());
    }

    #[test]
    fn validate_row_index_valid() {
        assert!(Biff8Sheet::validate_row_index(0).is_ok());
        assert!(Biff8Sheet::validate_row_index(65535).is_ok());
    }

    #[test]
    fn validate_row_index_invalid() {
        assert!(Biff8Sheet::validate_row_index(65536).is_err());
    }

    #[test]
    fn column_index_valid() {
        assert_eq!(Biff8Sheet::column_index(0).unwrap(), 0);
        assert_eq!(Biff8Sheet::column_index(255).unwrap(), 255);
    }

    #[test]
    fn column_index_invalid() {
        assert!(Biff8Sheet::column_index(256).is_err());
    }

    #[test]
    fn set_column_width() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet.set_column_width(1, 100);
        assert_eq!(sheet.column_widths.get(&1), Some(&100));
        assert_eq!(sheet.column_width_units.get(&1), Some(&25600));
        assert!(sheet.column_user_set_widths.contains(&1));
    }

    #[test]
    fn set_column_width_at_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet.set_column_width_at(5, 80).expect("should succeed");
        assert_eq!(sheet.column_widths.get(&5), Some(&80));
    }

    #[test]
    fn set_column_width_at_out_of_range() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_column_width_at(256, 80).is_err());
    }

    #[test]
    fn set_column_width_units_at_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .set_column_width_units_at(3, 5120)
            .expect("should succeed");
        assert_eq!(sheet.column_width_units.get(&3), Some(&5120));
        assert!(sheet.column_user_set_widths.contains(&3));
    }

    #[test]
    fn set_column_width_units_at_out_of_range() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_column_width_units_at(256, 5120).is_err());
    }

    #[test]
    fn set_default_column_width_units() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet.set_default_column_width_units(8000);
        assert_eq!(sheet.default_column_width_units, Some(8000));
    }

    #[test]
    fn set_column_metadata_at_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .set_column_metadata_at(2, 4800, 15, true, true)
            .expect("should succeed");
        assert_eq!(sheet.column_width_units.get(&2), Some(&4800));
        assert_eq!(sheet.column_xfs.get(&2), Some(&15));
        assert!(sheet.hidden_columns.contains(&2));
        assert!(sheet.column_user_set_widths.contains(&2));
    }

    #[test]
    fn set_column_metadata_at_not_hidden() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .set_column_metadata_at(2, 4800, 15, false, false)
            .expect("should succeed");
        assert!(!sheet.hidden_columns.contains(&2));
        assert!(!sheet.column_user_set_widths.contains(&2));
    }

    #[test]
    fn set_column_metadata_at_out_of_range() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_column_metadata_at(256, 4800, 15, false, false).is_err());
    }

    #[test]
    fn set_row_height() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet.set_row_height(5, 30);
        assert_eq!(sheet.row_heights.get(&5), Some(&30));
        assert_eq!(sheet.row_height_twips.get(&5), Some(&600));
    }

    #[test]
    fn set_row_height_at_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet.set_row_height_at(10, 20).expect("should succeed");
        assert_eq!(sheet.row_heights.get(&10), Some(&20));
        assert_eq!(sheet.row_height_twips.get(&10), Some(&400));
    }

    #[test]
    fn set_row_height_at_out_of_range_row() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_row_height_at(65536, 20).is_err());
    }

    #[test]
    fn set_row_height_at_overflow_twips() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        // u16::MAX * 20 would overflow
        assert!(sheet.set_row_height_at(0, u16::MAX).is_err());
    }

    #[test]
    fn set_row_height_twips_at_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .set_row_height_twips_at(5, 400)
            .expect("should succeed");
        assert_eq!(sheet.row_height_twips.get(&5), Some(&400));
    }

    #[test]
    fn set_row_height_twips_at_too_low() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_row_height_twips_at(0, 1).is_err());
    }

    #[test]
    fn set_row_height_twips_at_too_high() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_row_height_twips_at(0, 8193).is_err());
    }

    #[test]
    fn set_row_height_twips_at_out_of_range_row() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_row_height_twips_at(65536, 400).is_err());
    }

    #[test]
    fn set_default_row_height_twips_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .set_default_row_height_twips(300)
            .expect("should succeed");
        assert_eq!(sheet.default_row_height_twips, Some(300));
    }

    #[test]
    fn set_default_row_height_twips_too_low() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_default_row_height_twips(0).is_err());
    }

    #[test]
    fn set_default_row_height_twips_too_high() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_default_row_height_twips(8180).is_err());
    }

    #[test]
    fn set_row_metadata_at_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .set_row_metadata_at(5, 400, true, Some(15), true)
            .expect("should succeed");
        assert_eq!(sheet.row_height_twips.get(&5), Some(&400));
        assert_eq!(sheet.row_xfs.get(&5), Some(&15));
        assert!(sheet.hidden_rows.contains(&5));
    }

    #[test]
    fn set_row_metadata_at_no_custom_height() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .set_row_metadata_at(5, 400, false, None, false)
            .expect("should succeed");
        // When custom_height is false, twips should not be set
        assert!(sheet.row_height_twips.get(&5).is_none());
        // When xf_index is None, row_xfs should not have entry
        assert!(sheet.row_xfs.get(&5).is_none());
        assert!(!sheet.hidden_rows.contains(&5));
    }

    #[test]
    fn set_row_metadata_at_remove_xf() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        // First set an XF
        sheet
            .set_row_metadata_at(5, 400, true, Some(15), false)
            .expect("should succeed");
        assert_eq!(sheet.row_xfs.get(&5), Some(&15));
        // Then remove it
        sheet
            .set_row_metadata_at(5, 400, true, None, false)
            .expect("should succeed");
        assert!(sheet.row_xfs.get(&5).is_none());
    }

    #[test]
    fn set_row_metadata_at_invalid_height() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_row_metadata_at(0, 1, true, None, false).is_err());
        assert!(sheet.set_row_metadata_at(0, 8193, true, None, false).is_err());
    }

    #[test]
    fn set_freeze_panes_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet.set_freeze_panes(3, 2).expect("should succeed");
        assert_eq!(sheet.freeze, Some((3, 2)));
    }

    #[test]
    fn set_freeze_panes_out_of_range_row() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_freeze_panes(65536, 0).is_err());
    }

    #[test]
    fn set_freeze_panes_out_of_range_col() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.set_freeze_panes(0, 256).is_err());
    }

    #[test]
    fn add_merge_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let merge = Biff8Merge {
            first_row: 0,
            last_row: 1,
            first_col: 0,
            last_col: 1,
        };
        sheet.add_merge(merge).expect("should succeed");
        assert_eq!(sheet.merges.len(), 1);
    }

    #[test]
    fn add_merge_single_cell_is_noop() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let merge = Biff8Merge {
            first_row: 0,
            last_row: 0,
            first_col: 0,
            last_col: 0,
        };
        sheet.add_merge(merge).expect("should succeed");
        assert!(sheet.merges.is_empty());
    }

    #[test]
    fn add_merge_reversed_rows_fails() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let merge = Biff8Merge {
            first_row: 5,
            last_row: 3,
            first_col: 0,
            last_col: 1,
        };
        assert!(sheet.add_merge(merge).is_err());
    }

    #[test]
    fn add_merge_reversed_cols_fails() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let merge = Biff8Merge {
            first_row: 0,
            last_row: 1,
            first_col: 5,
            last_col: 3,
        };
        assert!(sheet.add_merge(merge).is_err());
    }

    #[test]
    fn protect_sheet_sets_hash() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.protection_password_hash.is_none());
        sheet.protect_sheet("test");
        assert!(sheet.protection_password_hash.is_some());
    }

    #[test]
    fn add_comment_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .add_comment(0, 0, "note text", "author")
            .expect("should succeed");
        assert_eq!(sheet.comments.len(), 1);
    }

    #[test]
    fn add_comment_with_nul_text_fails() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.add_comment(0, 0, "text\0with\0nul", "author").is_err());
    }

    #[test]
    fn add_comment_with_nul_author_fails() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.add_comment(0, 0, "text", "author\0nul").is_err());
    }

    #[test]
    fn add_comment_out_of_range_row() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.add_comment(65536, 0, "text", "author").is_err());
    }

    #[test]
    fn add_comment_out_of_range_col() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.add_comment(0, 256, "text", "author").is_err());
    }

    #[test]
    fn set_comment_replaces_existing() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let c1 = Biff8Comment::new(0, 0, "first".to_owned(), "a".to_owned());
        let c2 = Biff8Comment::new(0, 0, "second".to_owned(), "b".to_owned());
        sheet.set_comment(c1);
        assert_eq!(sheet.comments.len(), 1);
        sheet.set_comment(c2);
        assert_eq!(sheet.comments.len(), 1);
        assert_eq!(sheet.comments[0].text, "second");
    }

    #[test]
    fn set_comment_different_coords() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let c1 = Biff8Comment::new(0, 0, "first".to_owned(), "a".to_owned());
        let c2 = Biff8Comment::new(1, 0, "second".to_owned(), "b".to_owned());
        sheet.set_comment(c1);
        sheet.set_comment(c2);
        assert_eq!(sheet.comments.len(), 2);
    }

    #[test]
    fn remove_comment_existing() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .add_comment(0, 0, "text", "author")
            .expect("should succeed");
        let removed = sheet.remove_comment(0, 0).expect("should succeed");
        assert!(removed);
        assert!(sheet.comments.is_empty());
    }

    #[test]
    fn remove_comment_nonexistent() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let removed = sheet.remove_comment(0, 0).expect("should succeed");
        assert!(!removed);
    }

    #[test]
    fn remove_comment_out_of_range_row() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.remove_comment(65536, 0).is_err());
    }

    #[test]
    fn remove_comment_out_of_range_col() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        assert!(sheet.remove_comment(0, 256).is_err());
    }

    #[test]
    fn add_hyperlink_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .add_hyperlink(0, 0, "https://example.com", "Example")
            .expect("should succeed");
        assert_eq!(sheet.hyperlinks.len(), 1);
    }

    #[test]
    fn add_typed_hyperlink_valid() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet
            .add_typed_hyperlink(
                0,
                1,
                0,
                1,
                "https://example.com",
                "Example",
                Biff8HyperlinkKind::Url,
            )
            .expect("should succeed");
        assert_eq!(sheet.hyperlinks.len(), 1);
    }

    #[test]
    fn add_chart() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let chart = Biff8Chart::new(Biff8ChartKind::Bar, 0, 0, 10, 5);
        sheet.add_chart(chart);
        assert_eq!(sheet.charts.len(), 1);
    }

    #[test]
    fn dimensions_empty_sheet() {
        let sheet = Biff8Sheet::new("Sheet1");
        let (max_row, max_col) = sheet.dimensions();
        assert_eq!(max_row, 0);
        assert_eq!(max_col, 0);
    }

    #[test]
    fn dimensions_with_cells() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let cell = Biff8Cell::general(Biff8Value::Number(1.0));
        sheet.set(5, 3, cell).expect("should succeed");
        let (max_row, max_col) = sheet.dimensions();
        assert_eq!(max_row, 6);
        assert_eq!(max_col, 4);
    }

    #[test]
    fn dimensions_with_merges() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        let merge = Biff8Merge {
            first_row: 0,
            last_row: 10,
            first_col: 0,
            last_col: 5,
        };
        sheet.merges.push(merge);
        let (max_row, max_col) = sheet.dimensions();
        assert_eq!(max_row, 11);
        assert_eq!(max_col, 6);
    }

    #[test]
    fn set_row_height_zero_points() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        // 0 * 20 = 0 twips, which is below minimum of 2
        assert!(sheet.set_row_height_at(0, 0).is_err());
    }

    #[test]
    fn column_width_saturating_mul() {
        let mut sheet = Biff8Sheet::new("Sheet1");
        sheet.set_column_width(0, u16::MAX);
        // Should saturate, not overflow
        assert!(sheet.column_width_units.get(&0).is_some());
    }
}
