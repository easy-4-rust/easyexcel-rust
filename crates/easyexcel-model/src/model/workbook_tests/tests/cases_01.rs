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
        assert_eq!(s.value(1, 0), CellValue::Number(151_302.63));
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
