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

#[cfg(test)]
mod helper_tests {
    use super::*;
    use crate::addr::{CellAddress, CellRange};

    // --- remap_axis 测试 -----------------------------------------------

    #[test]
    fn remap_axis_identity() {
        let mut map = BTreeMap::new();
        map.insert(0u32, "a");
        map.insert(1, "b");
        map.insert(2, "c");
        remap_axis(&mut map, |k| Some(k));
        assert_eq!(map.len(), 3);
        assert_eq!(map[&0], "a");
        assert_eq!(map[&2], "c");
    }

    #[test]
    fn remap_axis_shift() {
        let mut map = BTreeMap::new();
        map.insert(0u32, "a");
        map.insert(1, "b");
        map.insert(2, "c");
        // Shift all keys by +1
        remap_axis(&mut map, |k| Some(k + 1));
        assert_eq!(map.len(), 3);
        assert!(map.contains_key(&1));
        assert!(map.contains_key(&3));
        assert!(!map.contains_key(&0));
    }

    #[test]
    fn remap_axis_drop() {
        let mut map = BTreeMap::new();
        map.insert(0u32, "a");
        map.insert(1, "b");
        map.insert(2, "c");
        // Drop key 1
        remap_axis(&mut map, |k| if k == 1 { None } else { Some(k) });
        assert_eq!(map.len(), 2);
        assert!(!map.contains_key(&1));
    }

    #[test]
    fn remap_axis_empty() {
        let mut map: BTreeMap<u32, &str> = BTreeMap::new();
        remap_axis(&mut map, |k| Some(k));
        assert!(map.is_empty());
    }

    // --- shrink_merged 测试 --------------------------------------------

    #[test]
    fn shrink_merged_row_deletion() {
        let mut merged = vec![CellRange::new(
            CellAddress::new(0, 0),
            CellAddress::new(3, 1),
        )];
        // Delete row 1 (count=1)
        shrink_merged(&mut merged, 1, 1, true);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].start.row, 0);
        assert_eq!(merged[0].end.row, 2);
    }

    #[test]
    fn shrink_merged_row_deletion_fully_inside() {
        let mut merged = vec![CellRange::new(
            CellAddress::new(2, 0),
            CellAddress::new(3, 1),
        )];
        // Delete rows 2-3 (fully inside the merge)
        shrink_merged(&mut merged, 2, 2, true);
        assert!(merged.is_empty());
    }

    #[test]
    fn shrink_merged_col_deletion() {
        let mut merged = vec![CellRange::new(
            CellAddress::new(0, 0),
            CellAddress::new(1, 3),
        )];
        // Delete col 1 (count=1)
        shrink_merged(&mut merged, 1, 1, false);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].start.col, 0);
        assert_eq!(merged[0].end.col, 2);
    }

    #[test]
    fn shrink_merged_col_deletion_fully_inside() {
        let mut merged = vec![CellRange::new(
            CellAddress::new(0, 1),
            CellAddress::new(0, 2),
        )];
        // Delete cols 1-2
        shrink_merged(&mut merged, 1, 2, false);
        assert!(merged.is_empty());
    }

    #[test]
    fn shrink_merged_empty() {
        let mut merged: Vec<CellRange> = vec![];
        shrink_merged(&mut merged, 0, 1, true);
        assert!(merged.is_empty());
    }

    #[test]
    fn shrink_merged_no_overlap() {
        let mut merged = vec![CellRange::new(
            CellAddress::new(5, 0),
            CellAddress::new(6, 1),
        )];
        // Delete row 0 (far from the merge)
        shrink_merged(&mut merged, 0, 1, true);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].start.row, 4);
        assert_eq!(merged[0].end.row, 5);
    }

    #[test]
    fn shrink_merged_clamp_partial() {
        let mut merged = vec![CellRange::new(
            CellAddress::new(0, 0),
            CellAddress::new(5, 1),
        )];
        // Delete rows 3-4 (at=3, count=2, end=5)
        // start=0 < at → unchanged; end=5 >= end(5) → 5-2=3
        shrink_merged(&mut merged, 3, 2, true);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].start.row, 0);
        assert_eq!(merged[0].end.row, 3);
    }

    // --- parse_table_area 测试 -----------------------------------------

    #[test]
    fn parse_table_area_all() {
        assert!(matches!(parse_table_area("#All"), Some(TableArea::All)));
    }

    #[test]
    fn parse_table_area_data() {
        assert!(matches!(parse_table_area("#Data"), Some(TableArea::Data)));
    }

    #[test]
    fn parse_table_area_headers() {
        assert!(matches!(
            parse_table_area("#Headers"),
            Some(TableArea::Headers)
        ));
    }

    #[test]
    fn parse_table_area_totals() {
        assert!(matches!(
            parse_table_area("#Totals"),
            Some(TableArea::Totals)
        ));
    }

    #[test]
    fn parse_table_area_case_insensitive() {
        assert!(matches!(
            parse_table_area("#ALL"),
            Some(TableArea::All)
        ));
        assert!(matches!(
            parse_table_area("#data"),
            Some(TableArea::Data)
        ));
    }

    #[test]
    fn parse_table_area_with_whitespace() {
        assert!(matches!(
            parse_table_area("  #All  "),
            Some(TableArea::All)
        ));
    }

    #[test]
    fn parse_table_area_invalid() {
        assert!(parse_table_area("#Invalid").is_none());
        assert!(parse_table_area("All").is_none());
        assert!(parse_table_area("").is_none());
    }

    // --- table_header_range 测试 ---------------------------------------

    #[test]
    fn table_header_range_with_header() {
        let t = Table {
            name: "T".into(),
            display_name: "T".into(),
            range: CellRange::new(CellAddress::new(0, 0), CellAddress::new(9, 2)),
            columns: vec!["A".into(), "B".into(), "C".into()],
            header_rows: 1,
            totals_rows: 0,
            id: 1,
            raw_xml: vec![],
        };
        let hr = table_header_range(&t).unwrap();
        assert_eq!(hr.start.row, 0);
        assert_eq!(hr.end.row, 0);
        assert_eq!(hr.start.col, 0);
        assert_eq!(hr.end.col, 2);
    }

    #[test]
    fn table_header_range_no_header() {
        let t = Table {
            name: "T".into(),
            display_name: "T".into(),
            range: CellRange::new(CellAddress::new(0, 0), CellAddress::new(9, 2)),
            columns: vec!["A".into()],
            header_rows: 0,
            totals_rows: 0,
            id: 1,
            raw_xml: vec![],
        };
        assert!(table_header_range(&t).is_none());
    }

    // --- table_totals_range 测试 ---------------------------------------

    #[test]
    fn table_totals_range_with_totals() {
        let t = Table {
            name: "T".into(),
            display_name: "T".into(),
            range: CellRange::new(CellAddress::new(0, 0), CellAddress::new(9, 2)),
            columns: vec!["A".into()],
            header_rows: 1,
            totals_rows: 1,
            id: 1,
            raw_xml: vec![],
        };
        let tr = table_totals_range(&t).unwrap();
        assert_eq!(tr.start.row, 9);
        assert_eq!(tr.end.row, 9);
    }

    #[test]
    fn table_totals_range_no_totals() {
        let t = Table {
            name: "T".into(),
            display_name: "T".into(),
            range: CellRange::new(CellAddress::new(0, 0), CellAddress::new(9, 2)),
            columns: vec![],
            header_rows: 1,
            totals_rows: 0,
            id: 1,
            raw_xml: vec![],
        };
        assert!(table_totals_range(&t).is_none());
    }

    // --- shrink_tables 测试 --------------------------------------------

    #[test]
    fn shrink_tables_row_deletion() {
        let mut tables = vec![Table {
            name: "T".into(),
            display_name: "T".into(),
            range: CellRange::new(CellAddress::new(0, 0), CellAddress::new(5, 2)),
            columns: vec![],
            header_rows: 1,
            totals_rows: 0,
            id: 1,
            raw_xml: vec![],
        }];
        // Delete row 0
        shrink_tables(&mut tables, 0, 1, true);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].range.end.row, 4);
    }

    #[test]
    fn shrink_tables_fully_inside() {
        let mut tables = vec![Table {
            name: "T".into(),
            display_name: "T".into(),
            range: CellRange::new(CellAddress::new(2, 0), CellAddress::new(3, 2)),
            columns: vec![],
            header_rows: 0,
            totals_rows: 0,
            id: 1,
            raw_xml: vec![],
        }];
        // Delete rows 2-3 (fully inside)
        shrink_tables(&mut tables, 2, 2, true);
        assert!(tables.is_empty());
    }

    #[test]
    fn shrink_tables_col_deletion() {
        let mut tables = vec![Table {
            name: "T".into(),
            display_name: "T".into(),
            range: CellRange::new(CellAddress::new(0, 0), CellAddress::new(5, 3)),
            columns: vec![],
            header_rows: 1,
            totals_rows: 0,
            id: 1,
            raw_xml: vec![],
        }];
        // Delete col 1
        shrink_tables(&mut tables, 1, 1, false);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].range.start.col, 0);
        assert_eq!(tables[0].range.end.col, 2);
    }

    #[test]
    fn shrink_tables_empty() {
        let mut tables: Vec<Table> = vec![];
        shrink_tables(&mut tables, 0, 1, true);
        assert!(tables.is_empty());
    }
}



