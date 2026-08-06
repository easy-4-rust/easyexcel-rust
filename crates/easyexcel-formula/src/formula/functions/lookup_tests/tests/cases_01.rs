    #[test]
    fn vlookup_exact() {
        let mut ctx = make_table();
        let result = vlookup(
            &mut ctx,
            &[
                Value::Number(3.0),
                rng(0, 0, 4, 1),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(result, Value::Text("cherry".into()));
    }

    #[test]
    fn vlookup_not_found() {
        let mut ctx = make_table();
        let result = vlookup(
            &mut ctx,
            &[
                Value::Number(99.0),
                rng(0, 0, 4, 1),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(result, Value::Error(CellError::NA));
    }

    #[test]
    fn vlookup_approx() {
        let mut ctx = make_table();
        // approx match: lookup 2.5 → row with 2 (largest ≤ 2.5) → "banana"
        let result = vlookup(
            &mut ctx,
            &[
                Value::Number(2.5),
                rng(0, 0, 4, 1),
                Value::Number(2.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(result, Value::Text("banana".into()));
    }

    #[test]
    fn match_exact() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Text("apple".into())),
            (1, 0, Value::Text("banana".into())),
            (2, 0, Value::Text("cherry".into())),
        ]);
        let result = match_fn(
            &mut ctx,
            &[
                Value::Text("banana".into()),
                rng(0, 0, 2, 0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn match_not_found() {
        let mut ctx =
            TestCtx::with_cells(&[(0, 0, Value::Number(1.0)), (1, 0, Value::Number(2.0))]);
        let result = match_fn(
            &mut ctx,
            &[Value::Number(5.0), rng(0, 0, 1, 0), Value::Number(0.0)],
        );
        assert_eq!(result, Value::Error(CellError::NA));
    }

    #[test]
    fn match_approx() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(5.0)),
        ]);
        let result = match_fn(
            &mut ctx,
            &[Value::Number(4.0), rng(0, 0, 2, 0), Value::Number(1.0)],
        );
        assert_eq!(result, Value::Number(2.0)); // 3 is largest ≤ 4
    }

    #[test]
    fn index_single() {
        let mut ctx = make_table();
        // INDEX(A1:B5, 2, 2) → "banana"
        let result = index_fn(
            &mut ctx,
            &[rng(0, 0, 4, 1), Value::Number(2.0), Value::Number(2.0)],
        );
        // Returns a Ref; deref it
        let v = match result {
            Value::Ref(r) => ctx.cell(r.sheet, r.start_row, r.start_col),
            other => other,
        };
        assert_eq!(v, Value::Text("banana".into()));
    }

    #[test]
    fn index_out_of_bounds() {
        let mut ctx = make_table();
        let result = index_fn(
            &mut ctx,
            &[rng(0, 0, 4, 1), Value::Number(10.0), Value::Number(1.0)],
        );
        assert_eq!(result, Value::Error(CellError::Ref));
    }

    #[test]
    fn offset_basic() {
        let mut ctx = make_table();
        // OFFSET(A1, 2, 0) → single cell 3 rows down from A1 = row 2, col 0 = 3.0
        let result = offset(
            &mut ctx,
            &[rng(0, 0, 0, 0), Value::Number(2.0), Value::Number(0.0)],
        );
        match result {
            Value::Ref(r) => {
                assert_eq!(r.start_row, 2);
                assert_eq!(r.start_col, 0);
                assert_eq!(r.end_row, 2);
                assert_eq!(r.end_col, 0);
            }
            other => panic!("expected Ref, got {other:?}"),
        }
    }

    #[test]
    fn offset_with_size() {
        let mut ctx = make_table();
        // OFFSET(A1, 0, 0, 3, 2) → range A1:B3
        let result = offset(
            &mut ctx,
            &[
                rng(0, 0, 0, 0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(3.0),
                Value::Number(2.0),
            ],
        );
        match result {
            Value::Ref(r) => {
                assert_eq!(r.rows(), 3);
                assert_eq!(r.cols(), 2);
            }
            other => panic!("expected Ref, got {other:?}"),
        }
    }

    #[test]
    fn row_column_no_arg() {
        let mut ctx = TestCtx::new();
        ctx.set_current(4, 2);
        assert_eq!(row_fn(&mut ctx, &[]), Value::Number(5.0));
        assert_eq!(column_fn(&mut ctx, &[]), Value::Number(3.0));
    }

    #[test]
    fn row_column_with_ref() {
        let mut ctx = TestCtx::new();
        assert_eq!(row_fn(&mut ctx, &[rng(2, 1, 2, 1)]), Value::Number(3.0));
        assert_eq!(column_fn(&mut ctx, &[rng(2, 1, 2, 1)]), Value::Number(2.0));
    }

    #[test]
    fn rows_columns_range() {
        let mut ctx = TestCtx::new();
        assert_eq!(rows_fn(&mut ctx, &[rng(0, 0, 4, 2)]), Value::Number(5.0));
        assert_eq!(columns_fn(&mut ctx, &[rng(0, 0, 4, 2)]), Value::Number(3.0));
    }

    #[test]
    fn transpose_basic() {
        let mut ctx = TestCtx::new();
        let arr = Value::Array(Array::from_rows(vec![
            vec![Value::Number(1.0), Value::Number(2.0)],
            vec![Value::Number(3.0), Value::Number(4.0)],
        ]));
        let result = transpose_fn(&mut ctx, &[arr]);
        if let Value::Array(a) = result {
            assert_eq!(a.rows, 2);
            assert_eq!(a.cols, 2);
            assert_eq!(a.get(0, 1), Some(&Value::Number(3.0)));
        } else {
            panic!("expected Array");
        }
    }

    #[test]
    fn address_abs() {
        let mut ctx = TestCtx::new();
        let result = address_fn(&mut ctx, &[Value::Number(3.0), Value::Number(2.0)]);
        assert_eq!(result, Value::Text("$B$3".into()));
    }

    #[test]
    fn address_relative() {
        let mut ctx = TestCtx::new();
        let result = address_fn(
            &mut ctx,
            &[Value::Number(3.0), Value::Number(2.0), Value::Number(4.0)],
        );
        assert_eq!(result, Value::Text("B3".into()));
    }

    #[test]
    fn hlookup_exact() {
        // Row 0: 1,2,3; Row 1: "a","b","c"
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (0, 1, Value::Number(2.0)),
            (0, 2, Value::Number(3.0)),
            (1, 0, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (1, 2, Value::Text("c".into())),
        ]);
        let result = hlookup(
            &mut ctx,
            &[
                Value::Number(2.0),
                rng(0, 0, 1, 2),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(result, Value::Text("b".into()));
    }

    #[test]
    fn xlookup_basic() {
        let mut ctx = make_table();
        let result = xlookup(
            &mut ctx,
            &[Value::Number(3.0), rng(0, 0, 4, 0), rng(0, 1, 4, 1)],
        );
        assert_eq!(result, Value::Text("cherry".into()));
    }

    #[test]
    fn xlookup_not_found_fallback() {
        let mut ctx = make_table();
        let result = xlookup(
            &mut ctx,
            &[
                Value::Number(99.0),
                rng(0, 0, 4, 0),
                rng(0, 1, 4, 1),
                Value::Text("nope".into()),
            ],
        );
        assert_eq!(result, Value::Text("nope".into()));
    }
