    #[test]
    fn sum_and_product() {
        let mut c = TestCtx::new();
        assert_eq!(
            sum(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(6.0)
        );
        assert_eq!(
            product(&mut c, &[Value::Number(2.0), Value::Number(3.0)]),
            Value::Number(6.0)
        );
    }

    #[test]
    fn sum_ignores_text_in_ranges() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Text("x".into())),
            (2, 0, Value::Number(5.0)),
        ]);
        assert_eq!(sum(&mut c, &[rng(0, 0, 2, 0)]), Value::Number(15.0));
    }

    #[test]
    fn sum_coerces_numeric_text_in_ranges() {
        // PARITY: Excel ignores text in ranges; we coerce numeric-looking text
        // ("numbers stored as text") so they still contribute. Non-numeric text
        // ("x") is still skipped.
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Text("6,000.00".into())),
            (1, 0, Value::Text("x".into())),
            (2, 0, Value::Text("500.00".into())),
            (3, 0, Value::Number(5.0)),
        ]);
        assert_eq!(sum(&mut c, &[rng(0, 0, 3, 0)]), Value::Number(6505.0));
    }

    #[test]
    fn rounding() {
        let mut c = TestCtx::new();
        assert_eq!(
            round(&mut c, &[Value::Number(2.5), Value::Number(0.0)]),
            Value::Number(3.0)
        );
        assert_eq!(
            round(&mut c, &[Value::Number(-2.5), Value::Number(0.0)]),
            Value::Number(-3.0)
        );
        assert_eq!(
            round(&mut c, &[Value::Number(9.87654), Value::Number(2.0)]),
            Value::Number(9.88)
        );
        assert_eq!(
            rounddown(&mut c, &[Value::Number(3.99), Value::Number(0.0)]),
            Value::Number(3.0)
        );
    }

    #[test]
    fn mod_and_int() {
        let mut c = TestCtx::new();
        assert_eq!(
            mod_fn(&mut c, &[Value::Number(-3.0), Value::Number(2.0)]),
            Value::Number(1.0)
        );
        assert_eq!(int_fn(&mut c, &[Value::Number(-2.5)]), Value::Number(-3.0));
        assert_eq!(
            mod_fn(&mut c, &[Value::Number(5.0), Value::Number(0.0)]),
            Value::Error(CellError::Div0)
        );
    }

    #[test]
    fn sumif_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(10.0)),
        ]);
        let r = sumif(&mut c, &[rng(0, 0, 2, 0), Value::Text(">3".into())]);
        assert_eq!(r, Value::Number(15.0));
    }

    #[test]
    fn fact_and_combin() {
        let mut c = TestCtx::new();
        assert_eq!(fact(&mut c, &[Value::Number(5.0)]), Value::Number(120.0));
        assert_eq!(
            combin(&mut c, &[Value::Number(5.0), Value::Number(2.0)]),
            Value::Number(10.0)
        );
        assert_eq!(
            fact(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn roman_arabic() {
        let mut c = TestCtx::new();
        assert_eq!(
            roman(&mut c, &[Value::Number(1994.0)]),
            Value::Text("MCMXCIV".into())
        );
        assert_eq!(
            arabic(&mut c, &[Value::Text("MCMXCIV".into())]),
            Value::Number(1994.0)
        );
    }

    #[test]
    fn rand_in_range() {
        let mut c = TestCtx::new();
        for _ in 0..100 {
            if let Value::Number(x) = Value::Number(next_rand()) {
                assert!((0.0..1.0).contains(&x));
            }
            let v = randbetween(&mut c, &[Value::Number(1.0), Value::Number(6.0)]);
            if let Value::Number(x) = v {
                assert!((1.0..=6.0).contains(&x));
            }
        }
    }

    // --- MULTINOMIAL ---

    #[test]
    fn multinomial_basic() {
        let mut c = TestCtx::new();
        // MULTINOMIAL(2,3,4) = 9! / (2! * 3! * 4!) = 362880 / (2*6*24) = 362880/288 = 1260
        assert_eq!(
            multinomial(
                &mut c,
                &[Value::Number(2.0), Value::Number(3.0), Value::Number(4.0)]
            ),
            Value::Number(1260.0)
        );
        // MULTINOMIAL(1) = 1
        assert_eq!(
            multinomial(&mut c, &[Value::Number(1.0)]),
            Value::Number(1.0)
        );
        // Negative arg → #NUM!
        assert_eq!(
            multinomial(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- SUMX2MY2 / SUMX2PY2 / SUMXMY2 ---

    #[test]
    fn paired_array_fns() {
        use crate::formula::value::Array;
        let mut c = TestCtx::new();

        let xs = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
        ));
        let ys = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(0.0), Value::Number(0.0), Value::Number(0.0)],
        ));

        // SUMX2MY2({1,2,3},{0,0,0}) = 1+4+9 = 14
        assert_eq!(
            sumx2my2(&mut c, &[xs.clone(), ys.clone()]),
            Value::Number(14.0)
        );

        // SUMX2PY2({1,2,3},{0,0,0}) = 1+4+9 = 14
        assert_eq!(
            sumx2py2(&mut c, &[xs.clone(), ys.clone()]),
            Value::Number(14.0)
        );

        // SUMXMY2({1,2},{0,0}) = 1+4 = 5
        let xs2 = Value::Array(Array::new(
            1,
            2,
            vec![Value::Number(1.0), Value::Number(2.0)],
        ));
        let ys2 = Value::Array(Array::new(
            1,
            2,
            vec![Value::Number(0.0), Value::Number(0.0)],
        ));
        assert_eq!(
            sumxmy2(&mut c, &[xs2.clone(), ys2.clone()]),
            Value::Number(5.0)
        );

        // Mismatched sizes → #N/A
        let ys3 = Value::Array(Array::new(1, 1, vec![Value::Number(0.0)]));
        assert_eq!(
            sumx2my2(&mut c, &[xs.clone(), ys3]),
            Value::Error(CellError::NA)
        );
    }

    // --- SERIESSUM ---

    #[test]
    fn seriessum_basic() {
        use crate::formula::value::Array;
        let mut c = TestCtx::new();

        // SERIESSUM(2, 0, 1, {1,1,1}) = 1*2^0 + 1*2^1 + 1*2^2 = 1+2+4 = 7
        let coeffs = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)],
        ));
        assert_eq!(
            seriessum(
                &mut c,
                &[
                    Value::Number(2.0),
                    Value::Number(0.0),
                    Value::Number(1.0),
                    coeffs
                ]
            ),
            Value::Number(7.0)
        );

        // SERIESSUM(1, 0, 1, {1,1,1}) = 1+1+1 = 3
        let coeffs2 = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)],
        ));
        assert_eq!(
            seriessum(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(0.0),
                    Value::Number(1.0),
                    coeffs2
                ]
            ),
            Value::Number(3.0)
        );

        // SERIESSUM(3, 1, 2, {1,2}) = 1*3^1 + 2*3^3 = 3 + 54 = 57
        let coeffs3 = Value::Array(Array::new(
            1,
            2,
            vec![Value::Number(1.0), Value::Number(2.0)],
        ));
        assert_eq!(
            seriessum(
                &mut c,
                &[
                    Value::Number(3.0),
                    Value::Number(1.0),
                    Value::Number(2.0),
                    coeffs3
                ]
            ),
            Value::Number(57.0)
        );
    }

    // --- SUBTOTAL ---

    #[test]
    fn subtotal_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
            (4, 0, Value::Number(5.0)),
        ]);
        let rng_arg = rng(0, 0, 4, 0);

        // fn_num=9 → SUM
        assert_eq!(
            subtotal(&mut c, &[Value::Number(9.0), rng_arg.clone()]),
            Value::Number(15.0)
        );

        // fn_num=1 → AVERAGE
        assert_eq!(
            subtotal(&mut c, &[Value::Number(1.0), rng_arg.clone()]),
            Value::Number(3.0)
        );

        // fn_num=4 → MAX
        assert_eq!(
            subtotal(&mut c, &[Value::Number(4.0), rng_arg.clone()]),
            Value::Number(5.0)
        );

        // fn_num=5 → MIN
        assert_eq!(
            subtotal(&mut c, &[Value::Number(5.0), rng_arg.clone()]),
            Value::Number(1.0)
        );

        // fn_num=109 → treated same as 9 (SUM)
        assert_eq!(
            subtotal(&mut c, &[Value::Number(109.0), rng_arg.clone()]),
            Value::Number(15.0)
        );
    }

    // --- AGGREGATE ---

    #[test]
    fn aggregate_basic() {
        use crate::formula::value::Array;
        let mut c = TestCtx::new();

        let data = Value::Array(Array::new(
            1,
            5,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
            ],
        ));

        // fn_num=9 (SUM), options=0
        assert_eq!(
            aggregate(
                &mut c,
                &[Value::Number(9.0), Value::Number(0.0), data.clone()]
            ),
            Value::Number(15.0)
        );

        // fn_num=12 (MEDIAN), options=0
        assert_eq!(
            aggregate(
                &mut c,
                &[Value::Number(12.0), Value::Number(0.0), data.clone()]
            ),
            Value::Number(3.0)
        );

        // fn_num=4 (MAX), options=6 (ignore errors)
        let data_with_err = Value::Array(Array::new(
            1,
            4,
            vec![
                Value::Number(1.0),
                Value::Error(CellError::Div0),
                Value::Number(5.0),
                Value::Number(3.0),
            ],
        ));
        assert_eq!(
            aggregate(
                &mut c,
                &[
                    Value::Number(4.0),
                    Value::Number(6.0),
                    data_with_err.clone()
                ]
            ),
            Value::Number(5.0)
        );

        // fn_num=14 (LARGE), options=0, k=2
        assert_eq!(
            aggregate(
                &mut c,
                &[
                    Value::Number(14.0),
                    Value::Number(0.0),
                    data.clone(),
                    Value::Number(2.0)
                ]
            ),
            Value::Number(4.0)
        );
    }

    // --- Matrix functions ---

    #[test]
    fn mdeterm_basic() {
        use crate::formula::value::Array;
        let mut c = TestCtx::new();

        // [[1,2],[3,4]] → det = 1*4 - 2*3 = -2
        let mat = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
            ],
        ));
        assert_eq!(mdeterm(&mut c, &[mat]), Value::Number(-2.0));

        // [[5]] → det = 5
        let mat2 = Value::Array(Array::new(1, 1, vec![Value::Number(5.0)]));
        assert_eq!(mdeterm(&mut c, &[mat2]), Value::Number(5.0));

        // Non-square → #VALUE!
        let mat3 = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
        ));
        assert_eq!(mdeterm(&mut c, &[mat3]), Value::Error(CellError::Value));
    }

    #[test]
    fn minverse_basic() {
        use crate::formula::value::Array;
        let mut c = TestCtx::new();

        // [[1,0],[0,1]] inverse = [[1,0],[0,1]]
        let identity = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(1.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        ));
        let result = minverse(&mut c, &[identity]);
        if let Value::Array(a) = result {
            let data: Vec<f64> = a
                .data
                .iter()
                .map(|v| {
                    if let Value::Number(n) = v {
                        *n
                    } else {
                        f64::NAN
                    }
                })
                .collect();
            assert!((data[0] - 1.0).abs() < 1e-10);
            assert!((data[1] - 0.0).abs() < 1e-10);
            assert!((data[2] - 0.0).abs() < 1e-10);
            assert!((data[3] - 1.0).abs() < 1e-10);
        } else {
            panic!("expected Array result");
        }
    }

    #[test]
    fn mmult_basic() {
        use crate::formula::value::Array;
        let mut c = TestCtx::new();

        // [[1,0],[0,1]] × [[3,4],[5,6]] = [[3,4],[5,6]]
        let identity = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(1.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        ));
        let other = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
                Value::Number(6.0),
            ],
        ));
        let result = mmult(&mut c, &[identity, other]);
        if let Value::Array(a) = result {
            let data: Vec<f64> = a
                .data
                .iter()
                .map(|v| {
                    if let Value::Number(n) = v {
                        *n
                    } else {
                        f64::NAN
                    }
                })
                .collect();
            assert_eq!(data, vec![3.0, 4.0, 5.0, 6.0]);
        } else {
            panic!("expected Array result");
        }

        // Mismatched inner dims → #VALUE!
        let a2 = Value::Array(Array::new(
            2,
            3,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
                Value::Number(6.0),
            ],
        ));
        let b2 = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(1.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        ));
        assert_eq!(mmult(&mut c, &[a2, b2]), Value::Error(CellError::Value));
    }

    #[test]
    fn munit_basic() {
        let mut c = TestCtx::new();

        let result = munit(&mut c, &[Value::Number(3.0)]);
        if let Value::Array(a) = result {
            assert_eq!(a.rows, 3);
            assert_eq!(a.cols, 3);
            // diagonal should be 1, off-diagonal 0
            for r in 0..3 {
                for c_idx in 0..3 {
                    let expected = if r == c_idx { 1.0 } else { 0.0 };
                    assert_eq!(a.get(r, c_idx), Some(&Value::Number(expected)));
                }
            }
        } else {
            panic!("expected Array result");
        }
    }

    // --- GAMMA ---

    #[test]
    fn gamma_basic() {
        let mut c = TestCtx::new();

        // Γ(1) = 1
        if let Value::Number(g) = gamma(&mut c, &[Value::Number(1.0)]) {
            assert!((g - 1.0).abs() < 1e-9, "Γ(1) ≈ 1, got {g}");
        } else {
            panic!("expected Number for Γ(1)");
        }

        // Γ(5) = 4! = 24
        if let Value::Number(g) = gamma(&mut c, &[Value::Number(5.0)]) {
            assert!((g - 24.0).abs() < 1e-6, "Γ(5) ≈ 24, got {g}");
        } else {
            panic!("expected Number");
        }

        // Γ(0.5) = √π ≈ 1.7724538509
        if let Value::Number(g) = gamma(&mut c, &[Value::Number(0.5)]) {
            assert!(
                (g - std::f64::consts::PI.sqrt()).abs() < 1e-6,
                "Γ(0.5) ≈ √π, got {g}"
            );
        } else {
            panic!("expected Number");
        }

        // Non-positive integer → #NUM!
        assert_eq!(
            gamma(&mut c, &[Value::Number(0.0)]),
            Value::Error(CellError::Num)
        );
        assert_eq!(
            gamma(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }
