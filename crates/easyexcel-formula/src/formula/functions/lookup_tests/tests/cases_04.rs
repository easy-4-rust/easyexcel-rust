    // --- 更多查找函数测试（覆盖 register_to_areas_fn.rs 未测分支） ---

    // vlookup: 文本查找（在第一列查找数字，返回第二列文本）
    #[test]
    fn vlookup_text_lookup() {
        let mut c = make_table();
        // 在第一列查找 3，返回第二列 → "cherry"
        let r = vlookup(
            &mut c,
            &[
                Value::Number(3.0),
                rng(0, 0, 4, 1),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Text("cherry".into()));
    }

    // hlookup: 基本测试
    #[test]
    fn hlookup_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (0, 1, Value::Number(2.0)),
            (0, 2, Value::Number(3.0)),
            (1, 0, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (1, 2, Value::Text("c".into())),
        ]);
        let r = hlookup(
            &mut c,
            &[
                Value::Number(2.0),
                rng(0, 0, 1, 2),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Text("b".into()));
    }

    // lookup: 基本测试
    #[test]
    fn lookup_basic() {
        let mut c = make_table();
        let r = lookup(
            &mut c,
            &[Value::Number(3.0), rng(0, 0, 4, 0), rng(0, 1, 4, 1)],
        );
        assert_eq!(r, Value::Text("cherry".into()));
    }

    // index_fn: 返回 Ref 类型
    #[test]
    fn index_basic_v2() {
        let mut c = make_table();
        let r = index_fn(
            &mut c,
            &[rng(0, 0, 4, 1), Value::Number(1.0), Value::Number(1.0)],
        );
        match r {
            Value::Ref(_) => {} // INDEX 返回 Ref 是正常的
            other => panic!("Expected Ref, got {other:?}"),
        }
    }

    // match_fn: 基本测试
    #[test]
    fn match_basic() {
        let mut c = make_table();
        let r = match_fn(
            &mut c,
            &[Value::Number(3.0), rng(0, 0, 4, 0), Value::Number(0.0)],
        );
        assert_eq!(r, Value::Number(3.0));
    }

    // offset: 返回 Ref 类型
    #[test]
    fn offset_basic_v2() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(5.0)),
        ]);
        let r = offset(
            &mut c,
            &[
                rng(0, 0, 0, 0),
                Value::Number(1.0),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(1.0),
            ],
        );
        match r {
            Value::Ref(_) => {} // OFFSET 返回 Ref 是正常的
            other => panic!("Expected Ref, got {other:?}"),
        }
    }

    // row_fn: 基本测试
    #[test]
    fn row_basic() {
        let mut c = TestCtx::with_cells(&[(5, 0, Value::Number(1.0))]);
        c.set_current(5, 0);
        let r = row_fn(&mut c, &[]);
        assert_eq!(r, Value::Number(6.0)); // 1-indexed
    }

    // rows_fn: 基本测试
    #[test]
    fn rows_basic() {
        let mut c = TestCtx::new();
        let r = rows_fn(&mut c, &[rng(0, 0, 4, 0)]);
        assert_eq!(r, Value::Number(5.0));
    }

    // column_fn: 基本测试
    #[test]
    fn column_basic() {
        let mut c = TestCtx::with_cells(&[(0, 3, Value::Number(1.0))]);
        c.set_current(0, 3);
        let r = column_fn(&mut c, &[]);
        assert_eq!(r, Value::Number(4.0)); // 1-indexed
    }

    // columns_fn: 基本测试
    #[test]
    fn columns_basic() {
        let mut c = TestCtx::new();
        let r = columns_fn(&mut c, &[rng(0, 0, 0, 4)]);
        assert_eq!(r, Value::Number(5.0));
    }

    // areas_fn: 基本测试
    #[test]
    fn areas_basic() {
        let mut c = TestCtx::new();
        let r = areas_fn(&mut c, &[rng(0, 0, 2, 2)]);
        assert_eq!(r, Value::Number(1.0));
    }

    // vlookup: 未找到 → #N/A (v2)
    #[test]
    fn vlookup_not_found_v2() {
        let mut c = make_table();
        let r = vlookup(
            &mut c,
            &[
                Value::Number(99.0),
                rng(0, 0, 4, 1),
                Value::Number(1.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // match: 未找到 → #N/A (v2)
    #[test]
    fn match_not_found_v2() {
        let mut c = make_table();
        let r = match_fn(
            &mut c,
            &[Value::Number(99.0), rng(0, 0, 4, 0), Value::Number(0.0)],
        );
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // index: 超出范围 → #REF!
    #[test]
    fn index_out_of_range() {
        let mut c = make_table();
        let r = index_fn(
            &mut c,
            &[rng(0, 0, 4, 0), Value::Number(10.0), Value::Number(1.0)],
        );
        assert_eq!(r, Value::Error(CellError::Ref));
    }
