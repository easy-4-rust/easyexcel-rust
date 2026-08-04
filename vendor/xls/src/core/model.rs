//! The in-memory spreadsheet data model: [`Workbook`], [`Sheet`], [`Cell`].

use std::collections::BTreeMap;

use super::addr::{CellAddress, CellRange};
use super::dates::DateSystem;
use super::error::CellError;
use super::numfmt;
use super::styles::StyleTable;
use super::value::CellValue;

/// A single stored cell. Mirrors the variants Excel persists; the cell's *style*
/// is held separately on the [`Sheet`] (a sparse map keyed by position) so that
/// blank-but-formatted cells round-trip.
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    /// Empty (but possibly styled — see [`Sheet::style_at`]).
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
    Error(CellError),
    /// A formula cell: the source expression plus its last cached value.
    Formula {
        expr: String,
        cached: CellValue,
    },
}

impl Cell {
    /// The scalar value of this cell (formula → cached value).
    pub fn value(&self) -> CellValue {
        match self {
            Cell::Empty => CellValue::Empty,
            Cell::Number(n) => CellValue::Number(*n),
            Cell::Text(s) => CellValue::Text(s.clone()),
            Cell::Bool(b) => CellValue::Bool(*b),
            Cell::Error(e) => CellValue::Error(*e),
            Cell::Formula { cached, .. } => cached.clone(),
        }
    }

    pub fn from_value(v: CellValue) -> Cell {
        match v {
            CellValue::Empty => Cell::Empty,
            CellValue::Number(n) => Cell::Number(n),
            CellValue::Text(s) => Cell::Text(s),
            CellValue::Bool(b) => Cell::Bool(b),
            CellValue::Error(e) => Cell::Error(e),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }

    pub fn is_formula(&self) -> bool {
        matches!(self, Cell::Formula { .. })
    }

    pub fn formula_text(&self) -> Option<&str> {
        match self {
            Cell::Formula { expr, .. } => Some(expr),
            _ => None,
        }
    }
}

/// Column metadata (width / hidden state).
#[derive(Debug, Clone, Copy, Default)]
pub struct ColInfo {
    /// Width in character units (Excel's measure). `None` means default.
    pub width: Option<f64>,
    pub hidden: bool,
    pub style: Option<u32>,
}

/// Row metadata (height / hidden state).
#[derive(Debug, Clone, Copy, Default)]
pub struct RowInfo {
    /// Height in points. `None` means default.
    pub height: Option<f64>,
    pub hidden: bool,
    pub style: Option<u32>,
}

/// Frozen-pane configuration: number of frozen rows/columns at the top-left.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrozenPanes {
    pub rows: u32,
    pub cols: u32,
}

/// Sheet visibility state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    VeryHidden,
}

/// An opaque part preserved verbatim for round-trip (charts, drawings, pivots,
/// VBA, unknown XML parts or BIFF records).
#[derive(Debug, Clone)]
pub struct OpaquePart {
    /// Logical name/path (zip part name, or OLE stream name).
    pub name: String,
    pub data: Vec<u8>,
}

/// A spilled dynamic-array result, owned by the anchor cell (top-left). The
/// anchor is a real formula cell whose cached value is the top-left element;
/// the remaining region cells are *derived* (not stored in `cells`) and read
/// from here. Spills are recomputed on every recalc, never persisted.
#[derive(Debug, Clone, PartialEq)]
pub struct Spill {
    pub rows: u32,
    pub cols: u32,
    /// Row-major values, length `rows * cols` (index 0 is the anchor).
    pub values: Vec<CellValue>,
}

/// An Excel **table** object (the feature Excel calls a "Table" / ListObject): a
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
    /// The data-body range: the table range minus its header and totals rows.
    /// Returns `None` if the table has no body rows.
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

    /// The 0-based offset of a column (case-insensitive) within the table.
    pub fn column_index(&self, name: &str) -> Option<u32> {
        self.columns
            .iter()
            .position(|c| c.eq_ignore_ascii_case(name))
            .map(|i| i as u32)
    }

    /// The data-body range of a single column (case-insensitive header match).
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

/// One worksheet.
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

    /// The spilled value at (row, col), if a spill region (other than its own
    /// anchor cell) covers it. The anchor cell itself is a real formula cell.
    pub fn spilled_at(&self, row: u32, col: u32) -> Option<&CellValue> {
        for (&(ar, ac), sp) in &self.spills {
            if row >= ar && row < ar + sp.rows && col >= ac && col < ac + sp.cols {
                let idx = ((row - ar) * sp.cols + (col - ac)) as usize;
                return sp.values.get(idx);
            }
        }
        None
    }

    /// Remove all spill regions (recalc rebuilds them).
    pub fn clear_spills(&mut self) {
        self.spills.clear();
    }

    pub fn get(&self, row: u32, col: u32) -> Option<&Cell> {
        self.cells.get(&(row, col))
    }

    pub fn get_a1(&self, a1: &str) -> Option<&Cell> {
        let a = CellAddress::parse_a1(a1)?;
        self.get(a.row, a.col)
    }

    pub fn set(&mut self, row: u32, col: u32, cell: Cell) {
        if cell.is_empty() && !self.styles.contains_key(&(row, col)) {
            self.cells.remove(&(row, col));
        } else {
            self.cells.insert((row, col), cell);
        }
    }

    pub fn set_a1(&mut self, a1: &str, cell: Cell) -> bool {
        match CellAddress::parse_a1(a1) {
            Some(a) => {
                self.set(a.row, a.col, cell);
                true
            }
            None => false,
        }
    }

    pub fn style_at(&self, row: u32, col: u32) -> Option<u32> {
        self.styles.get(&(row, col)).copied()
    }

    pub fn set_style(&mut self, row: u32, col: u32, style: u32) {
        self.styles.insert((row, col), style);
    }

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

    /// The used range as `(max_row, max_col)` exclusive bounds (0,0 if empty).
    pub fn dimensions(&self) -> (u32, u32) {
        let mut max_row = 0;
        let mut max_col = 0;
        for &(r, c) in self.cells.keys().chain(self.styles.keys()) {
            max_row = max_row.max(r + 1);
            max_col = max_col.max(c + 1);
        }
        // Include spilled regions so they're visible to readers/exporters.
        for (&(ar, ac), sp) in &self.spills {
            max_row = max_row.max(ar + sp.rows);
            max_col = max_col.max(ac + sp.cols);
        }
        (max_row, max_col)
    }

    /// Is `(row, col)` covered by (but not the anchor of) a merged region?
    pub fn is_merged_continuation(&self, row: u32, col: u32) -> bool {
        self.merged
            .iter()
            .any(|m| m.contains(row, col) && (m.start.row != row || m.start.col != col))
    }

    /// Convert text cells in `range` that look like numbers (incl. thousands
    /// separators and `%`) into real [`Cell::Number`]s. Returns how many were
    /// converted. Handy for "numbers stored as text" exports that `SUM` ignores.
    pub fn coerce_text_to_numbers(&mut self, range: CellRange) -> usize {
        let mut converted = 0;
        for (r, c) in range.iter_cells() {
            let num = match self.get(r, c) {
                Some(Cell::Text(s)) => crate::core::formula::coerce::parse_number_text(s),
                _ => None,
            };
            if let Some(n) = num {
                self.set(r, c, Cell::Number(n));
                converted += 1;
            }
        }
        converted
    }

    /// Clear every cell (and its style) in a rectangular range.
    pub fn clear_range(&mut self, range: CellRange) {
        for (r, c) in range.iter_cells() {
            self.cells.remove(&(r, c));
            self.styles.remove(&(r, c));
        }
    }

    /// Insert `count` blank rows at `at`, shifting rows `>= at` downward.
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

    /// Delete `count` rows starting at `at`, shifting rows below upward.
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

    /// Insert `count` blank columns at `at`, shifting columns `>= at` rightward.
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

    /// Delete `count` columns starting at `at`, shifting columns to the right
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

/// Rebuild a single-axis metadata map (row/column info) through a transform.
fn remap_axis<V, F>(map: &mut BTreeMap<u32, V>, f: F)
where
    F: Fn(u32) -> Option<u32>,
{
    let old = std::mem::take(map);
    for (k, v) in old {
        if let Some(nk) = f(k) {
            map.insert(nk, v);
        }
    }
}

/// Adjust merged ranges for a row (`is_row`) or column deletion of `count`
/// units at `at`: ranges fully inside the deleted band are dropped, others are
/// clamped/shifted.
fn shrink_merged(merged: &mut Vec<CellRange>, at: u32, count: u32, is_row: bool) {
    let end = at.saturating_add(count);
    let adjust = |v: u32| -> u32 {
        if v >= end {
            v - count
        } else if v >= at {
            at.saturating_sub(1)
        } else {
            v
        }
    };
    merged.retain_mut(|m| {
        let (s, e) = if is_row {
            (&mut m.start.row, &mut m.end.row)
        } else {
            (&mut m.start.col, &mut m.end.col)
        };
        // Drop a merge entirely contained in the deleted band.
        if *s >= at && *e < end {
            return false;
        }
        *s = adjust(*s);
        *e = adjust(*e);
        *e >= *s
    });
}

/// Which part of a table a structured reference selects.
enum TableArea {
    All,
    Data,
    Headers,
    Totals,
}

/// Parse a `#`-area selector (`#All`, `#Data`, `#Headers`, `#Totals`).
fn parse_table_area(sel: &str) -> Option<TableArea> {
    match sel.trim() {
        s if s.eq_ignore_ascii_case("#All") => Some(TableArea::All),
        s if s.eq_ignore_ascii_case("#Data") => Some(TableArea::Data),
        s if s.eq_ignore_ascii_case("#Headers") => Some(TableArea::Headers),
        s if s.eq_ignore_ascii_case("#Totals") => Some(TableArea::Totals),
        _ => None,
    }
}

/// The header row range of a table (`None` if it has no header).
fn table_header_range(t: &Table) -> Option<CellRange> {
    if t.header_rows == 0 {
        return None;
    }
    Some(CellRange::new(
        CellAddress::new(t.range.start.row, t.range.start.col),
        CellAddress::new(t.range.start.row + t.header_rows - 1, t.range.end.col),
    ))
}

/// The totals row range of a table (`None` if it has no totals row).
fn table_totals_range(t: &Table) -> Option<CellRange> {
    if t.totals_rows == 0 {
        return None;
    }
    let top = t.range.end.row.checked_sub(t.totals_rows - 1)?;
    Some(CellRange::new(
        CellAddress::new(top, t.range.start.col),
        CellAddress::new(t.range.end.row, t.range.end.col),
    ))
}

/// Adjust table ranges for a row/column deletion, mirroring [`shrink_merged`].
/// A table fully inside the deleted band is dropped; otherwise its range is
/// clamped/shifted. (Column names are left as-is — a known limitation when
/// deleting columns through the middle of a table.)
fn shrink_tables(tables: &mut Vec<Table>, at: u32, count: u32, is_row: bool) {
    let end = at.saturating_add(count);
    let adjust = |v: u32| -> u32 {
        if v >= end {
            v - count
        } else if v >= at {
            at.saturating_sub(1)
        } else {
            v
        }
    };
    tables.retain_mut(|t| {
        let (s, e) = if is_row {
            (&mut t.range.start.row, &mut t.range.end.row)
        } else {
            (&mut t.range.start.col, &mut t.range.end.col)
        };
        if *s >= at && *e < end {
            return false;
        }
        *s = adjust(*s);
        *e = adjust(*e);
        *e >= *s
    });
}

/// A workbook-scoped or sheet-scoped defined name (named range / constant).
#[derive(Debug, Clone)]
pub struct DefinedName {
    pub name: String,
    /// The formula text the name refers to (e.g. `Sheet1!$A$1:$B$2`).
    pub refers_to: String,
    /// `None` for workbook scope, else the sheet index it is local to.
    pub scope: Option<usize>,
    pub hidden: bool,
}

/// Document metadata (Dublin Core-ish).
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub company: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub application: Option<String>,
}

/// The top-level spreadsheet: a collection of [`Sheet`]s plus shared state.
#[derive(Debug, Clone)]
pub struct Workbook {
    pub sheets: Vec<Sheet>,
    pub styles: StyleTable,
    pub defined_names: Vec<DefinedName>,
    pub date_system: DateSystem,
    pub metadata: Metadata,
    /// Workbook-level opaque parts (themes, pivot caches, VBA project, …).
    pub opaque: Vec<OpaquePart>,
    /// Active sheet index for the TUI.
    pub active_sheet: usize,
}

impl Default for Workbook {
    fn default() -> Self {
        Workbook::new()
    }
}

impl Workbook {
    /// An empty workbook with a single blank sheet named `Sheet1`.
    pub fn new() -> Self {
        Workbook {
            sheets: vec![Sheet::new("Sheet1")],
            styles: StyleTable::default(),
            defined_names: Vec::new(),
            date_system: DateSystem::Date1900,
            metadata: Metadata::default(),
            opaque: Vec::new(),
            active_sheet: 0,
        }
    }

    /// A completely empty workbook with no sheets.
    pub fn empty() -> Self {
        Workbook {
            sheets: Vec::new(),
            styles: StyleTable::default(),
            defined_names: Vec::new(),
            date_system: DateSystem::Date1900,
            metadata: Metadata::default(),
            opaque: Vec::new(),
            active_sheet: 0,
        }
    }

    pub fn add_sheet(&mut self, name: impl Into<String>) -> usize {
        self.sheets.push(Sheet::new(name));
        self.sheets.len() - 1
    }

    pub fn sheet_by_name(&self, name: &str) -> Option<&Sheet> {
        self.sheets
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    pub fn sheet_index(&self, name: &str) -> Option<usize> {
        self.sheets
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// Resolve a structured table reference to its `(sheet index, range)`.
    ///
    /// Supports the common forms: a bare `Table` (→ data body), `Table[Column]`,
    /// `Table[#All]`/`[#Data]`/`[#Headers]`/`[#Totals]`, and the combined
    /// `Table[[#Data],[Column]]`. The `[@...]` this-row form is not supported.
    pub fn resolve_structured(&self, raw: &str) -> Option<(usize, CellRange)> {
        let raw = raw.trim();
        let (name, spec) = match raw.find('[') {
            Some(open) => {
                let close = raw.rfind(']')?;
                (raw[..open].trim(), Some(raw[open + 1..close].trim()))
            }
            None => (raw, None),
        };
        let (sidx, table) = self.table_by_name(name)?;

        let Some(spec) = spec else {
            return table.data_range().map(|r| (sidx, r));
        };

        // Collect selectors (an `#`-area and/or a column name).
        let mut selectors: Vec<&str> = Vec::new();
        if spec.contains('[') {
            // Multiple bracketed selectors, e.g. `[#Data],[Amount]`.
            let mut rest = spec;
            while let Some(o) = rest.find('[') {
                let c = rest[o..].find(']')? + o;
                selectors.push(rest[o + 1..c].trim());
                rest = &rest[c + 1..];
            }
        } else {
            selectors.push(spec);
        }

        let mut area = TableArea::Data;
        let mut column: Option<&str> = None;
        for sel in selectors {
            if let Some(a) = parse_table_area(sel) {
                area = a;
            } else if !sel.is_empty() {
                column = Some(sel);
            }
        }

        let base = match area {
            TableArea::All => table.range,
            TableArea::Data => table.data_range()?,
            TableArea::Headers => table_header_range(table)?,
            TableArea::Totals => table_totals_range(table)?,
        };
        let range = match column {
            Some(col) => {
                let off = table.column_index(col)?;
                let c = table.range.start.col + off;
                CellRange::new(
                    CellAddress::new(base.start.row, c),
                    CellAddress::new(base.end.row, c),
                )
            }
            None => base,
        };
        Some((sidx, range))
    }

    /// Find a table by name (or display name), case-insensitive, across all
    /// sheets. Returns the owning sheet index and the table.
    pub fn table_by_name(&self, name: &str) -> Option<(usize, &Table)> {
        for (i, s) in self.sheets.iter().enumerate() {
            if let Some(t) = s.tables.iter().find(|t| {
                t.name.eq_ignore_ascii_case(name) || t.display_name.eq_ignore_ascii_case(name)
            }) {
                return Some((i, t));
            }
        }
        None
    }

    pub fn sheet_mut(&mut self, idx: usize) -> Option<&mut Sheet> {
        self.sheets.get_mut(idx)
    }

    /// Display text for a cell, applying its number format (date-aware).
    pub fn display_cell(&self, sheet_idx: usize, row: u32, col: u32) -> String {
        let Some(sheet) = self.sheets.get(sheet_idx) else {
            return String::new();
        };
        let value = sheet.value(row, col);
        match value {
            CellValue::Number(n) => {
                if let Some(style_idx) = sheet.style_at(row, col)
                    && let Some(style) = self.styles.get(style_idx)
                    && !style.number_format.is_empty()
                    && !style.number_format.eq_ignore_ascii_case("general")
                {
                    return numfmt::format_value(n, &style.number_format, self.date_system);
                }
                super::value::format_number_general(n)
            }
            other => other.to_display_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::styles::CellStyle;
    use super::*;

    #[test]
    fn basic_workbook() {
        let mut wb = Workbook::new();
        assert_eq!(wb.sheets.len(), 1);
        let s = wb.sheet_mut(0).unwrap();
        s.set_a1("A1", Cell::Number(42.0));
        s.set_a1("B2", Cell::Text("hi".into()));
        assert_eq!(s.value(0, 0), CellValue::Number(42.0));
        assert_eq!(s.dimensions(), (2, 2));
    }

    #[test]
    fn empty_cells_pruned() {
        let mut s = Sheet::new("x");
        s.set(0, 0, Cell::Number(1.0));
        s.set(0, 0, Cell::Empty);
        assert!(s.get(0, 0).is_none());
    }

    #[test]
    fn insert_and_delete_rows_shift_cells() {
        let mut s = Sheet::new("x");
        s.set_a1("A1", Cell::Number(1.0));
        s.set_a1("A2", Cell::Number(2.0));
        s.set_a1("A3", Cell::Number(3.0));
        // Insert one row at row index 1 (A2): A2→A3, A3→A4.
        s.insert_rows(1, 1);
        assert_eq!(s.value(0, 0), CellValue::Number(1.0)); // A1 unchanged
        assert_eq!(s.value(1, 0), CellValue::Empty); // new blank row
        assert_eq!(s.value(2, 0), CellValue::Number(2.0));
        assert_eq!(s.value(3, 0), CellValue::Number(3.0));
        // Delete the blank row again → back to original layout.
        s.delete_rows(1, 1);
        assert_eq!(s.value(1, 0), CellValue::Number(2.0));
        assert_eq!(s.value(2, 0), CellValue::Number(3.0));
        // Deleting a populated row removes it and pulls the rest up.
        s.delete_rows(0, 1);
        assert_eq!(s.value(0, 0), CellValue::Number(2.0));
        assert_eq!(s.dimensions(), (2, 1));
    }

    #[test]
    fn insert_and_delete_cols_shift_cells() {
        let mut s = Sheet::new("x");
        s.set_a1("A1", Cell::Number(1.0));
        s.set_a1("B1", Cell::Number(2.0));
        s.set_a1("C1", Cell::Number(3.0));
        s.insert_cols(1, 2); // shift B,C right by 2
        assert_eq!(s.value(0, 0), CellValue::Number(1.0));
        assert_eq!(s.value(0, 3), CellValue::Number(2.0));
        assert_eq!(s.value(0, 4), CellValue::Number(3.0));
        s.delete_cols(1, 2); // undo
        assert_eq!(s.value(0, 1), CellValue::Number(2.0));
        assert_eq!(s.value(0, 2), CellValue::Number(3.0));
    }

    #[test]
    fn delete_rows_drops_contained_merge() {
        let mut s = Sheet::new("x");
        s.merged.push(CellRange::parse_a1("A2:B3").unwrap()); // rows 1..=2
        s.merged.push(CellRange::parse_a1("A6:B6").unwrap()); // row 5
        s.delete_rows(1, 2); // removes rows 1,2 → first merge gone, second shifts up
        assert_eq!(s.merged.len(), 1);
        assert_eq!(s.merged[0].to_a1(), "A4:B4");
    }

    #[test]
    fn coerce_text_to_numbers_handles_separators() {
        let mut s = Sheet::new("x");
        s.set_a1("A1", Cell::Text("6,000.00".into()));
        s.set_a1("A2", Cell::Text("1,51,302.63".into())); // Indian grouping
        s.set_a1("A3", Cell::Text("hello".into())); // not numeric
        s.set_a1("A4", Cell::Number(5.0)); // already a number
        let n = s.coerce_text_to_numbers(CellRange::parse_a1("A1:A4").unwrap());
        assert_eq!(n, 2);
        assert_eq!(s.value(0, 0), CellValue::Number(6000.0));
        assert_eq!(s.value(1, 0), CellValue::Number(151302.63));
        assert_eq!(s.value(2, 0), CellValue::Text("hello".into())); // unchanged
    }

    #[test]
    fn clear_range_removes_cells_and_styles() {
        let mut s = Sheet::new("x");
        s.set_a1("A1", Cell::Number(1.0));
        s.set_a1("B2", Cell::Number(2.0));
        s.set_style(0, 0, 7);
        s.clear_range(CellRange::parse_a1("A1:B2").unwrap());
        assert_eq!(s.dimensions(), (0, 0));
        assert!(s.style_at(0, 0).is_none());
    }

    #[test]
    fn resolve_structured_refs() {
        let mut wb = Workbook::new();
        {
            let s = wb.sheet_mut(0).unwrap();
            s.set_a1("A1", Cell::Text("Name".into()));
            s.set_a1("B1", Cell::Text("Amount".into()));
            s.tables.push(Table {
                name: "Sales".into(),
                display_name: "Sales".into(),
                range: CellRange::parse_a1("A1:B4").unwrap(),
                columns: vec!["Name".into(), "Amount".into()],
                header_rows: 1,
                totals_rows: 0,
                id: 1,
                raw_xml: Vec::new(),
            });
        }
        let r = |raw: &str| wb.resolve_structured(raw).map(|(i, rng)| (i, rng.to_a1()));
        assert_eq!(r("Sales"), Some((0, "A2:B4".into()))); // data body
        assert_eq!(r("Sales[Amount]"), Some((0, "B2:B4".into())));
        assert_eq!(r("Sales[#All]"), Some((0, "A1:B4".into())));
        assert_eq!(r("Sales[#Headers]"), Some((0, "A1:B1".into())));
        assert_eq!(r("Sales[[#Data],[Amount]]"), Some((0, "B2:B4".into())));
        assert_eq!(r("Sales[#Headers]"), Some((0, "A1:B1".into())));
        assert_eq!(r("Nope[X]"), None);
        assert_eq!(r("Sales[Missing]"), None);
    }

    #[test]
    fn display_with_format() {
        let mut wb = Workbook::new();
        let style_idx = {
            let st = CellStyle {
                number_format: "0.00".into(),
                ..Default::default()
            };
            wb.styles.intern(st)
        };
        let s = wb.sheet_mut(0).unwrap();
        s.set(0, 0, Cell::Number(3.5));
        s.set_style(0, 0, style_idx);
        assert_eq!(wb.display_cell(0, 0, 0), "3.50");
    }
}
