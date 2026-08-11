    // --- sign error ---

    #[test]
    fn sign_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            sign(&mut c, &[Value::Error(CellError::NA)]),
            Value::Error(CellError::NA)
        );
    }

    // --- even with error ---

    #[test]
    fn even_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            even(&mut c, &[Value::Error(CellError::Num)]),
            Value::Error(CellError::Num)
        );
    }

    // --- odd with error ---

    #[test]
    fn odd_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            odd(&mut c, &[Value::Error(CellError::Value)]),
            Value::Error(CellError::Value)
        );
    }

    // --- trunc with negative digits ---

    #[test]
    fn trunc_negative_digits() {
        let mut c = TestCtx::new();
        assert_eq!(
            trunc(&mut c, &[Value::Number(1234.0), Value::Number(-2.0)]),
            Value::Number(1200.0)
        );
    }

    // --- mround with zero divisor ---

    #[test]
    fn mround_zero_divisor() {
        let mut c = TestCtx::new();
        assert_eq!(
            mround(&mut c, &[Value::Number(10.0), Value::Number(0.0)]),
            Value::Number(0.0)
        );
    }

    // --- sumproduct with text ---

    #[test]
    fn sumproduct_skips_text() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Text("x".into())),
            (0, 1, Value::Number(2.0)),
            (1, 1, Value::Number(3.0)),
        ]);
        // Only numeric pairs counted
        let r = sumproduct(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]);
        assert!(matches!(r, Value::Number(_)));
    }

    // --- sumifs with multiple criteria ---

    #[test]
    fn sumifs_multiple_criteria() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Number(20.0)),
            (2, 0, Value::Number(30.0)),
            (0, 1, Value::Text("A".into())),
            (1, 1, Value::Text("B".into())),
            (2, 1, Value::Text("A".into())),
            (0, 2, Value::Text("X".into())),
            (1, 2, Value::Text("X".into())),
            (2, 2, Value::Text("Y".into())),
        ]);
        assert_eq!(
            sumifs(
                &mut c,
                &[
                    rng(0, 0, 2, 0),
                    rng(0, 1, 2, 1),
                    Value::Text("A".into()),
                    rng(0, 2, 2, 2),
                    Value::Text("Y".into()),
                ]
            ),
            Value::Number(30.0) // only row 2 matches both criteria
        );
    }

    // --- percentof with error ---

    #[test]
    fn percentof_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            percentof(&mut c, &[Value::Error(CellError::NA), Value::Number(100.0)]),
            Value::Error(CellError::NA)
        );
    }

    // --- sumsq with single value ---

    #[test]
    fn sumsq_single() {
        let mut c = TestCtx::new();
        assert_eq!(
            sumsq(&mut c, &[Value::Number(5.0)]),
            Value::Number(25.0)
        );
    }

    // --- ceiling_precise with negative significance value ---

    #[test]
    fn ceiling_precise_neg_sig_value() {
        let mut c = TestCtx::new();
        assert_eq!(
            ceiling_precise(&mut c, &[Value::Number(5.3), Value::Number(-1.0)]),
            Value::Number(6.0)
        );
    }

    // --- floor_precise basic ---

    #[test]
    fn floor_precise_positive() {
        let mut c = TestCtx::new();
        assert_eq!(
            floor_precise(&mut c, &[Value::Number(5.7)]),
            Value::Number(5.0)
        );
    }

    // --- roundup with error ---

    #[test]
    fn roundup_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            roundup(
                &mut c,
                &[Value::Error(CellError::NA), Value::Number(0.0)]
            ),
            Value::Error(CellError::NA)
        );
    }

    // --- rounddown with error ---

    #[test]
    fn rounddown_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            rounddown(
                &mut c,
                &[Value::Error(CellError::Value), Value::Number(0.0)]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- ceiling with zero ---

    #[test]
    fn ceiling_zero_value() {
        let mut c = TestCtx::new();
        assert_eq!(
            ceiling(&mut c, &[Value::Number(0.0), Value::Number(1.0)]),
            Value::Number(0.0)
        );
    }

    // --- floor with zero ---

    #[test]
    fn floor_zero_value() {
        let mut c = TestCtx::new();
        assert_eq!(
            floor(&mut c, &[Value::Number(0.0), Value::Number(1.0)]),
            Value::Number(0.0)
        );
    }

    // --- product with negative ---

    #[test]
    fn product_with_negatives() {
        let mut c = TestCtx::new();
        assert_eq!(
            product(&mut c, &[Value::Number(-2.0), Value::Number(-3.0)]),
            Value::Number(6.0)
        );
    }

    // --- sumif with no match ---

    #[test]
    fn sumif_no_match() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(10.0)), (1, 0, Value::Number(20.0))]);
        assert_eq!(
            sumif(&mut c, &[rng(0, 0, 1, 0), Value::Text(">100".into())]),
            Value::Number(0.0)
        );
    }

    // --- int_fn with positive ---

    #[test]
    fn int_fn_positive() {
        let mut c = TestCtx::new();
        assert_eq!(int_fn(&mut c, &[Value::Number(2.9)]), Value::Number(2.0));
    }
