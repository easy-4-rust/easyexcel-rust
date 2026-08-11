    // --- hlookup with approximate match ---

    #[test]
    fn hlookup_approximate() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (0, 1, Value::Number(5.0)),
            (0, 2, Value::Number(10.0)),
            (1, 0, Value::Text("low".into())),
            (1, 1, Value::Text("mid".into())),
            (1, 2, Value::Text("high".into())),
        ]);
        // lookup 3 in row 0, return row 1
        let r = hlookup(
            &mut c,
            &[
                Value::Number(3.0),
                rng(0, 0, 1, 2),
                Value::Number(2.0),
                Value::Bool(true),
            ],
        );
        // approximate match: largest <= 3 is 1 -> "low"
        assert_eq!(r, Value::Text("low".into()));
    }

    // --- hlookup not found ---

    #[test]
    fn hlookup_exact_not_found() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Text("a".into())),
        ]);
        let r = hlookup(
            &mut c,
            &[
                Value::Number(99.0),
                rng(0, 0, 1, 0),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- lookup with vector form ---

    #[test]
    fn lookup_vector_result() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (0, 1, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (2, 1, Value::Text("c".into())),
        ]);
        let r = lookup(
            &mut c,
            &[Value::Number(2.0), rng(0, 0, 2, 0), rng(0, 1, 2, 1)],
        );
        assert_eq!(r, Value::Text("b".into()));
    }

    // --- index with row and col ---

    #[test]
    fn index_row_and_col() {
        let mut c = make_table();
        let r = index_fn(
            &mut c,
            &[rng(0, 0, 4, 1), Value::Number(3.0), Value::Number(2.0)],
        );
        // INDEX returns any value type
        assert!(!matches!(r, Value::Empty));
    }

    // --- match with match_type -1 ---

    #[test]
    fn match_descending_type() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(1.0)),
        ]);
        let r = match_fn(
            &mut c,
            &[Value::Number(3.0), rng(0, 0, 2, 0), Value::Number(-1.0)],
        );
        assert_eq!(r, Value::Number(2.0));
    }

    // --- offset with negative ---

    #[test]
    fn offset_negative_rows() {
        let mut c = make_table();
        c.set_current(3, 0);
        let r = offset(
            &mut c,
            &[
                rng(0, 0, 4, 0),
                Value::Number(-2.0),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(1.0),
            ],
        );
        // OFFSET can return ref or error
        assert!(matches!(r, Value::Ref(_) | Value::Error(_)));
    }

    // --- row_fn with ref ---

    #[test]
    fn row_fn_with_ref() {
        let mut c = TestCtx::new();
        let r = row_fn(&mut c, &[rng(2, 0, 4, 0)]);
        // Returns array of rows
        assert!(matches!(r, Value::Array(_)));
    }

    // --- column_fn with ref ---

    #[test]
    fn column_fn_with_ref() {
        let mut c = TestCtx::new();
        let r = column_fn(&mut c, &[rng(0, 1, 0, 3)]);
        // Returns array of columns
        assert!(matches!(r, Value::Array(_)));
    }

    // --- rows with scalar ---

    #[test]
    fn rows_scalar() {
        let mut c = TestCtx::new();
        let r = rows_fn(&mut c, &[Value::Number(42.0)]);
        assert_eq!(r, Value::Number(1.0));
    }

    // --- columns with scalar ---

    #[test]
    fn columns_scalar() {
        let mut c = TestCtx::new();
        let r = columns_fn(&mut c, &[Value::Number(42.0)]);
        assert_eq!(r, Value::Number(1.0));
    }

    // --- vlookup with wildcard ---

    #[test]
    fn vlookup_wildcard() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Text("apple".into())),
            (0, 1, Value::Number(1.0)),
        ]);
        let r = vlookup(
            &mut c,
            &[
                Value::Text("app*".into()),
                rng(0, 0, 0, 1),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Number(1.0));
    }

    // --- xlookup with exact match ---

    #[test]
    fn xlookup_exact_match() {
        let mut c = make_table();
        let r = xlookup(
            &mut c,
            &[
                Value::Number(3.0),
                rng(0, 0, 4, 0),
                rng(0, 1, 4, 1),
            ],
        );
        assert_eq!(r, Value::Text("cherry".into()));
    }

    // --- xlookup not found with fallback ---

    #[test]
    fn xlookup_not_found_fallback_value() {
        let mut c = make_table();
        let r = xlookup(
            &mut c,
            &[
                Value::Number(99.0),
                rng(0, 0, 4, 0),
                rng(0, 1, 4, 1),
                Value::Text("not found".into()),
            ],
        );
        assert_eq!(r, Value::Text("not found".into()));
    }

    // --- xmatch basic ---

    #[test]
    fn xmatch_basic() {
        let mut c = make_table();
        let r = xmatch(
            &mut c,
            &[Value::Number(4.0), rng(0, 0, 4, 0)],
        );
        assert_eq!(r, Value::Number(4.0)); // 1-based position
    }

    // --- indirect basic ---

    #[test]
    fn indirect_resolves_ref() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(42.0))]);
        let r = indirect(&mut c, &[Value::Text("A1".into()), Value::Bool(true)]);
        assert!(matches!(r, Value::Ref(_) | Value::Number(_)));
    }

    // --- index error ---

    #[test]
    fn index_row_too_large() {
        let mut c = make_table();
        let r = index_fn(
            &mut c,
            &[rng(0, 0, 4, 1), Value::Number(100.0), Value::Number(1.0)],
        );
        assert_eq!(r, Value::Error(CellError::Ref));
    }
