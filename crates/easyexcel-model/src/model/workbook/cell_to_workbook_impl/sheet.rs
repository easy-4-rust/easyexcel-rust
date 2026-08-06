/// 对应 Java：无直接对应对象；Rust 架构扩展。 One worksheet.
#[derive(Debug, Clone)]
pub struct Sheet {
    pub name: String,
    pub visibility: Visibility,
    /// Sparse cell storage, ordered for writer convenience.
    pub cells: BTreeMap<(u32, u32), Cell>,
    /// Sparse style indices, keyed by cell position (into the workbook's [`StyleTable`]).
    pub styles: BTreeMap<(u32, u32), u32>,
    pub columns: BTreeMap<u32, ColInfo>,
    pub rows: BTreeMap<u32, RowInfo>,
    pub merged: Vec<CellRange>,
    pub frozen: FrozenPanes,
    /// Default column width / row height (character units / points).
    pub default_col_width: f64,
    pub default_row_height: f64,
    /// Sheet-scoped opaque parts (drawings, etc.).
    pub opaque: Vec<OpaquePart>,
    /// Excel table objects defined on this sheet.
    pub tables: Vec<Table>,
    /// Live dynamic-array spill regions, keyed by anchor cell. Derived state,
    /// rebuilt by recalc; not saved.
    pub spills: BTreeMap<(u32, u32), Spill>,
}

impl Sheet {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn new(name: impl Into<String>) -> Self {
        Sheet {
            name: name.into(),
            visibility: Visibility::Visible,
            cells: BTreeMap::new(),
            styles: BTreeMap::new(),
            columns: BTreeMap::new(),
            rows: BTreeMap::new(),
            merged: Vec::new(),
            frozen: FrozenPanes::default(),
            default_col_width: 8.43,
            default_row_height: 15.0,
            opaque: Vec::new(),
            tables: Vec::new(),
            spills: BTreeMap::new(),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 The spilled value at (row, col), if a spill region (other than its own
    /// anchor cell) covers it. The anchor cell itself is a real formula cell.
    #[must_use]
    pub fn spilled_at(&self, row: u32, col: u32) -> Option<&CellValue> {
        for (&(ar, ac), sp) in &self.spills {
            if row >= ar && row < ar + sp.rows && col >= ac && col < ac + sp.cols {
                let idx = ((row - ar) * sp.cols + (col - ac)) as usize;
                return sp.values.get(idx);
            }
        }
        None
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Remove all spill regions (recalc rebuilds them).
    pub fn clear_spills(&mut self) {
        self.spills.clear();
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn get(&self, row: u32, col: u32) -> Option<&Cell> {
        self.cells.get(&(row, col))
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn get_a1(&self, a1: &str) -> Option<&Cell> {
        let a = CellAddress::parse_a1(a1)?;
        self.get(a.row, a.col)
    }
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn set(&mut self, row: u32, col: u32, cell: Cell) {
        if cell.is_empty() && !self.styles.contains_key(&(row, col)) {
            self.cells.remove(&(row, col));
        } else {
            self.cells.insert((row, col), cell);
        }
    }
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn set_a1(&mut self, a1: &str, cell: Cell) -> bool {
        match CellAddress::parse_a1(a1) {
            Some(a) => {
                self.set(a.row, a.col, cell);
                true
            }
            None => false,
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn style_at(&self, row: u32, col: u32) -> Option<u32> {
        self.styles.get(&(row, col)).copied()
    }
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn set_style(&mut self, row: u32, col: u32, style: u32) {
        self.styles.insert((row, col), style);
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn value(&self, row: u32, col: u32) -> CellValue {
        // A real cell (incl. a spill anchor's formula) wins; otherwise a
        // dynamic-array spill region may cover this position.
        if let Some(cell) = self.get(row, col) {
            return cell.value();
        }
        if let Some(v) = self.spilled_at(row, col) {
            return v.clone();
        }
        CellValue::Empty
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回实际存储的单元格或样式覆盖区域。
    ///
    /// 该范围只描述工作表中持久化的稀疏 `cells` / `styles` 坐标，不包含
    /// 公式计算产生的临时 spill 区域。读取器可据此按物理行遍历工作簿模型，
    /// 无需在格式门面中重复实现边界扫描。
    #[must_use]
    pub fn stored_range(&self) -> Option<CellRange> {
        let mut coordinates = self.cells.keys().chain(self.styles.keys()).copied();
        let (first_row, first_column) = coordinates.next()?;
        let (mut min_row, mut min_column, mut max_row, mut max_column) =
            (first_row, first_column, first_row, first_column);
        for (row, column) in coordinates {
            min_row = min_row.min(row);
            min_column = min_column.min(column);
            max_row = max_row.max(row);
            max_column = max_column.max(column);
        }
        Some(CellRange::new(
            CellAddress::new(min_row, min_column),
            CellAddress::new(max_row, max_column),
        ))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 The used range as `(max_row, max_col)` exclusive bounds (0,0 if empty).
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        let (mut max_row, mut max_col) = self.stored_range().map_or((0, 0), |range| {
            (
                range.end.row.saturating_add(1),
                range.end.col.saturating_add(1),
            )
        });
        // Include spilled regions so they're visible to readers/exporters.
        for (&(ar, ac), sp) in &self.spills {
            max_row = max_row.max(ar.saturating_add(sp.rows));
            max_col = max_col.max(ac.saturating_add(sp.cols));
        }
        (max_row, max_col)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Is `(row, col)` covered by (but not the anchor of) a merged region?
    #[must_use]
    pub fn is_merged_continuation(&self, row: u32, col: u32) -> bool {
        self.merged
            .iter()
            .any(|m| m.contains(row, col) && (m.start.row != row || m.start.col != col))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Convert text cells in `range` that look like numbers (incl. thousands
    /// separators and `%`) into real [`Cell::Number`]s. Returns how many were
    /// converted. Handy for "numbers stored as text" exports that `SUM` ignores.
    pub fn coerce_text_to_numbers(&mut self, range: CellRange) -> usize {
        let mut converted = 0;
        for (r, c) in range.iter_cells() {
            let num = match self.get(r, c) {
                Some(Cell::Text(s)) => super::value::parse_number_text(s),
                _ => None,
            };
            if let Some(n) = num {
                self.set(r, c, Cell::Number(n));
                converted += 1;
            }
        }
        converted
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Clear every cell (and its style) in a rectangular range.
    pub fn clear_range(&mut self, range: CellRange) {
        for (r, c) in range.iter_cells() {
            self.cells.remove(&(r, c));
            self.styles.remove(&(r, c));
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Insert `count` blank rows at `at`, shifting rows `>= at` downward.
    pub fn insert_rows(&mut self, at: u32, count: u32) {
        if count == 0 {
            return;
        }
        self.remap(|r, c| Some((if r >= at { r + count } else { r }, c)));
        remap_axis(&mut self.rows, |r| {
            Some(if r >= at { r + count } else { r })
        });
        for m in &mut self.merged {
            if m.start.row >= at {
                m.start.row += count;
            }
            if m.end.row >= at {
                m.end.row += count;
            }
        }
        for t in &mut self.tables {
            if t.range.start.row >= at {
                t.range.start.row += count;
            }
            if t.range.end.row >= at {
                t.range.end.row += count;
            }
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Delete `count` rows starting at `at`, shifting rows below upward.
    pub fn delete_rows(&mut self, at: u32, count: u32) {
        if count == 0 {
            return;
        }
        let end = at.saturating_add(count);
        self.remap(|r, c| match r {
            r if r >= at && r < end => None,
            r if r >= end => Some((r - count, c)),
            _ => Some((r, c)),
        });
        remap_axis(&mut self.rows, |r| match r {
            r if r >= at && r < end => None,
            r if r >= end => Some(r - count),
            _ => Some(r),
        });
        shrink_merged(&mut self.merged, at, count, true);
        shrink_tables(&mut self.tables, at, count, true);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Insert `count` blank columns at `at`, shifting columns `>= at` rightward.
    pub fn insert_cols(&mut self, at: u32, count: u32) {
        if count == 0 {
            return;
        }
        self.remap(|r, c| Some((r, if c >= at { c + count } else { c })));
        remap_axis(&mut self.columns, |c| {
            Some(if c >= at { c + count } else { c })
        });
        for m in &mut self.merged {
            if m.start.col >= at {
                m.start.col += count;
            }
            if m.end.col >= at {
                m.end.col += count;
            }
        }
        for t in &mut self.tables {
            if t.range.start.col >= at {
                t.range.start.col += count;
            }
            if t.range.end.col >= at {
                t.range.end.col += count;
            }
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Delete `count` columns starting at `at`, shifting columns to the right
    /// leftward.
    pub fn delete_cols(&mut self, at: u32, count: u32) {
        if count == 0 {
            return;
        }
        let end = at.saturating_add(count);
        self.remap(|r, c| match c {
            c if c >= at && c < end => None,
            c if c >= end => Some((r, c - count)),
            _ => Some((r, c)),
        });
        remap_axis(&mut self.columns, |c| match c {
            c if c >= at && c < end => None,
            c if c >= end => Some(c - count),
            _ => Some(c),
        });
        shrink_merged(&mut self.merged, at, count, false);
        shrink_tables(&mut self.tables, at, count, false);
    }

    /// Rebuild the cell + style maps through a coordinate transform (`None`
    /// drops the entry). Shared by the row/column insert/delete operations.
    fn remap<F>(&mut self, f: F)
    where
        F: Fn(u32, u32) -> Option<(u32, u32)>,
    {
        let cells = std::mem::take(&mut self.cells);
        for ((r, c), v) in cells {
            if let Some(k) = f(r, c) {
                self.cells.insert(k, v);
            }
        }
        let styles = std::mem::take(&mut self.styles);
        for ((r, c), v) in styles {
            if let Some(k) = f(r, c) {
                self.styles.insert(k, v);
            }
        }
    }
}
