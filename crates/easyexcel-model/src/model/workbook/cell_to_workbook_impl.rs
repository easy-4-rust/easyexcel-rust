include!("cell_to_workbook_impl/cell.rs");



include!("cell_to_workbook_impl/col_info.rs");

include!("cell_to_workbook_impl/row_info.rs");

include!("cell_to_workbook_impl/frozen_panes.rs");

include!("cell_to_workbook_impl/visibility.rs");

include!("cell_to_workbook_impl/opaque_part.rs");

include!("cell_to_workbook_impl/spill.rs");

include!("cell_to_workbook_impl/table.rs");



include!("cell_to_workbook_impl/sheet.rs");



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

include!("cell_to_workbook_impl/defined_name.rs");

include!("cell_to_workbook_impl/metadata.rs");

include!("cell_to_workbook_impl/workbook.rs");



