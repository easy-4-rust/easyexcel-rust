    // --- LOOKUP (vector + array form) ---

    #[test]
    fn lookup_vector_form() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(5.0)),
            (0, 1, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (2, 1, Value::Text("c".into())),
        ]);
        // LOOKUP(3, A1:A3, B1:B3) → "b"
        let r = lookup(
            &mut ctx,
            &[Value::Number(3.0), rng(0, 0, 2, 0), rng(0, 1, 2, 1)],
        );
        assert_eq!(r, Value::Text("b".into()));
    }

    #[test]
    fn lookup_array_form() {
        // Array form: search first col → return last col when rows >= cols
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (0, 1, Value::Text("x".into())),
            (1, 1, Value::Text("y".into())),
            (2, 1, Value::Text("z".into())),
        ]);
        let r = lookup(&mut ctx, &[Value::Number(2.0), rng(0, 0, 2, 1)]);
        assert_eq!(r, Value::Text("y".into()));
    }

    #[test]
    fn lookup_not_found_is_na() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(10.0)),
            (0, 1, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
        ]);
        let r = lookup(
            &mut ctx,
            &[Value::Number(1.0), rng(0, 0, 1, 0), rng(0, 1, 1, 1)],
        );
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- VLOOKUP / HLOOKUP edge cases ---

    #[test]
    fn vlookup_col_index_zero_is_value() {
        let mut ctx = make_table();
        let r = vlookup(
            &mut ctx,
            &[
                Value::Number(1.0),
                rng(0, 0, 4, 1),
                Value::Number(0.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn vlookup_wildcard_text() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Text("apple".into())),
            (1, 0, Value::Text("apricot".into())),
            (2, 0, Value::Text("banana".into())),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        // VLOOKUP("app*", range, 2, FALSE) → 1.0 (wildcard match)
        let r = vlookup(
            &mut ctx,
            &[
                Value::Text("app*".into()),
                rng(0, 0, 2, 1),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Number(1.0));
    }

    #[test]
    fn hlookup_not_found_is_na() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (0, 1, Value::Number(2.0)),
            (1, 0, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
        ]);
        let r = hlookup(
            &mut ctx,
            &[
                Value::Number(99.0),
                rng(0, 0, 1, 1),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- MATCH descending ---

    #[test]
    fn match_descending() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(1.0)),
        ]);
        // MATCH(4, {5,3,1}, -1) → 1 (5 is smallest >= 4)
        let r = match_fn(
            &mut ctx,
            &[Value::Number(4.0), rng(0, 0, 2, 0), Value::Number(-1.0)],
        );
        assert_eq!(r, Value::Number(1.0));
    }

    #[test]
    fn match_invalid_type_is_value() {
        let mut ctx = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        let r = match_fn(
            &mut ctx,
            &[Value::Number(1.0), rng(0, 0, 0, 0), Value::Number(5.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // --- INDEX edge cases ---

    #[test]
    fn index_row_zero_returns_column() {
        let mut ctx = make_table();
        // INDEX(A1:B5, 0, 1) → entire column A
        let r = index_fn(
            &mut ctx,
            &[rng(0, 0, 4, 1), Value::Number(0.0), Value::Number(1.0)],
        );
        // Should return a Ref for the whole column
        assert!(matches!(r, Value::Ref(_)));
    }

    #[test]
    fn index_col_zero_returns_row() {
        let mut ctx = make_table();
        // INDEX(A1:B5, 1, 0) → entire row 1
        let r = index_fn(
            &mut ctx,
            &[rng(0, 0, 4, 1), Value::Number(1.0), Value::Number(0.0)],
        );
        assert!(matches!(r, Value::Ref(_)));
    }

    #[test]
    fn index_col_out_of_bounds_is_ref() {
        let mut ctx = make_table();
        let r = index_fn(
            &mut ctx,
            &[rng(0, 0, 4, 1), Value::Number(1.0), Value::Number(10.0)],
        );
        assert_eq!(r, Value::Error(CellError::Ref));
    }

    // --- OFFSET edge cases ---

    #[test]
    fn offset_negative_result_is_ref() {
        let mut ctx = make_table();
        // OFFSET(A3, -5, 0) → negative row → #REF!
        let r = offset(
            &mut ctx,
            &[rng(2, 0, 2, 0), Value::Number(-5.0), Value::Number(0.0)],
        );
        assert_eq!(r, Value::Error(CellError::Ref));
    }

    #[test]
    fn offset_non_ref_base_is_value() {
        let mut ctx = TestCtx::new();
        let r = offset(
            &mut ctx,
            &[
                Value::Number(42.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // --- ROW / COLUMN with range ref ---

    #[test]
    fn row_range_returns_array() {
        let mut ctx = TestCtx::new();
        let r = row_fn(&mut ctx, &[rng(1, 0, 3, 0)]);
        if let Value::Array(a) = r {
            assert_eq!(a.rows, 3);
            assert_eq!(a.cols, 1);
        } else {
            panic!("expected Array, got {r:?}");
        }
    }

    #[test]
    fn column_range_returns_array() {
        let mut ctx = TestCtx::new();
        let r = column_fn(&mut ctx, &[rng(0, 1, 0, 3)]);
        if let Value::Array(a) = r {
            assert_eq!(a.rows, 1);
            assert_eq!(a.cols, 3);
        } else {
            panic!("expected Array, got {r:?}");
        }
    }

    // --- AREAS ---

    #[test]
    fn areas_always_one() {
        let mut ctx = TestCtx::new();
        assert_eq!(areas_fn(&mut ctx, &[rng(0, 0, 2, 2)]), Value::Number(1.0));
    }

    // --- INDIRECT ---

    #[test]
    fn indirect_a1_style() {
        let mut ctx = TestCtx::with_cells(&[(2, 1, Value::Number(42.0))]);
        let r = indirect(&mut ctx, &[Value::Text("B3".into())]);
        if let Value::Ref(rr) = r {
            assert_eq!(rr.start_row, 2);
            assert_eq!(rr.start_col, 1);
        } else {
            panic!("expected Ref, got {r:?}");
        }
    }

    #[test]
    fn indirect_invalid_ref_is_ref_error() {
        let mut ctx = TestCtx::new();
        let r = indirect(&mut ctx, &[Value::Text("NOTASHEET!A1".into())]);
        assert_eq!(r, Value::Error(CellError::Ref));
    }

    // --- XLOOKUP / XMATCH ---

    #[test]
    fn xmatch_exact() {
        let mut ctx = make_table();
        let r = xmatch(
            &mut ctx,
            &[
                Value::Text("cherry".into()),
                rng(0, 1, 4, 1),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Number(3.0));
    }

    #[test]
    fn xmatch_not_found_no_fallback_is_na() {
        let mut ctx = make_table();
        let r = xmatch(
            &mut ctx,
            &[
                Value::Text("zzz".into()),
                rng(0, 1, 4, 1),
            ],
        );
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- FORMULATEXT / HYPERLINK ---

    #[test]
    fn formulatext_returns_na_for_non_ref() {
        let mut ctx = TestCtx::new();
        let r = formulatext_fn(&mut ctx, &[Value::Number(42.0)]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    #[test]
    fn hyperlink_basic() {
        let mut ctx = TestCtx::new();
        let r = hyperlink_fn(
            &mut ctx,
            &[
                Value::Text("https://example.com".into()),
                Value::Text("Click here".into()),
            ],
        );
        assert_eq!(r, Value::Text("Click here".into()));
    }

    #[test]
    fn hyperlink_url_only() {
        let mut ctx = TestCtx::new();
        let r = hyperlink_fn(
            &mut ctx,
            &[Value::Text("https://example.com".into())],
        );
        assert_eq!(r, Value::Text("https://example.com".into()));
    }
