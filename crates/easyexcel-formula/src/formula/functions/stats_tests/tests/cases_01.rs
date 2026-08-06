    #[test]
    fn chisq_test_identical_is_one() {
        // actual == expected → χ² = 0 → p = 1
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Number(20.0)),
            (0, 1, Value::Number(10.0)),
            (1, 1, Value::Number(20.0)),
        ]);
        let r = chisq_test(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]);
        match r {
            Value::Number(p) => assert!((p - 1.0).abs() < 1e-9, "got {p}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn f_test_equal_variance() {
        // identical samples → F = 1 → two-tailed p = 1
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        let r = f_test(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        match r {
            Value::Number(p) => assert!((p - 1.0).abs() < 1e-6, "got {p}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn t_test_identical_samples() {
        // mean difference 0 (diffs [-2,2,0]) but nonzero variance → t = 0 → p = 1
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(7.0)),
            (2, 0, Value::Number(9.0)),
            (0, 1, Value::Number(7.0)),
            (1, 1, Value::Number(5.0)),
            (2, 1, Value::Number(9.0)),
        ]);
        let r = t_test(
            &mut c,
            &[
                rng(0, 0, 2, 0),
                rng(0, 1, 2, 1),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        );
        match r {
            Value::Number(p) => assert!((p - 1.0).abs() < 1e-9, "got {p}"),
            other => panic!("{other:?}"),
        }
        // wrong tails value → #NUM!
        assert_eq!(
            t_test(
                &mut c,
                &[
                    rng(0, 0, 2, 0),
                    rng(0, 1, 2, 1),
                    Value::Number(3.0),
                    Value::Number(1.0)
                ]
            ),
            Value::Error(CellError::Num)
        );
    }

    // ---- AVERAGE -----------------------------------------------------------

    #[test]
    fn average_basic() {
        let mut c = ctx();
        assert_eq!(
            average(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn average_empty_range() {
        let mut c = ctx();
        assert_eq!(
            average(&mut c, &[Value::Empty]),
            Value::Error(CellError::Div0)
        );
    }

    #[test]
    fn average_skips_text_in_range() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Text("hello".into())),
            (2, 0, Value::Number(20.0)),
        ]);
        assert_eq!(average(&mut c, &[rng(0, 0, 2, 0)]), Value::Number(15.0));
    }

    // ---- AVERAGEA ----------------------------------------------------------

    #[test]
    fn averagea_counts_text_as_zero() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Text("hello".into())),
            (2, 0, Value::Number(20.0)),
        ]);
        // 10 + 0 + 20 = 30, count = 3 → 10
        assert_eq!(averagea(&mut c, &[rng(0, 0, 2, 0)]), Value::Number(10.0));
    }

    // ---- AVERAGEIF ---------------------------------------------------------

    #[test]
    fn averageif_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(10.0)),
        ]);
        let r = averageif(&mut c, &[rng(0, 0, 2, 0), Value::Text(">3".into())]);
        assert_eq!(r, Value::Number(7.5));
    }

    #[test]
    fn averageif_no_match() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        let r = averageif(&mut c, &[rng(0, 0, 0, 0), Value::Text(">100".into())]);
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // ---- COUNT / COUNTA ----------------------------------------------------

    #[test]
    fn count_only_numbers() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Text("x".into())),
            (2, 0, Value::Bool(true)),
            (3, 0, Value::Empty),
        ]);
        assert_eq!(count(&mut c, &[rng(0, 0, 3, 0)]), Value::Number(1.0));
    }

    #[test]
    fn counta_non_empty() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Text("x".into())),
            (2, 0, Value::Bool(true)),
            (3, 0, Value::Empty),
        ]);
        assert_eq!(counta(&mut c, &[rng(0, 0, 3, 0)]), Value::Number(3.0));
    }

    #[test]
    fn countblank_fn() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Empty),
            (2, 0, Value::Text(String::new())),
        ]);
        assert_eq!(countblank(&mut c, &[rng(0, 0, 2, 0)]), Value::Number(2.0));
    }

    // ---- COUNTIF / COUNTIFS ------------------------------------------------

    #[test]
    fn countif_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(10.0)),
        ]);
        assert_eq!(
            countif(&mut c, &[rng(0, 0, 2, 0), Value::Text(">3".into())]),
            Value::Number(2.0)
        );
    }

    #[test]
    fn countifs_two_criteria() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(10.0)),
            (2, 0, Value::Number(15.0)),
        ]);
        let r = countifs(
            &mut c,
            &[
                rng(0, 0, 2, 0),
                Value::Text(">=5".into()),
                rng(0, 0, 2, 0),
                Value::Text("<=10".into()),
            ],
        );
        assert_eq!(r, Value::Number(2.0));
    }

    // ---- MAX / MIN ---------------------------------------------------------

    #[test]
    fn max_basic() {
        let mut c = ctx();
        assert_eq!(
            max(
                &mut c,
                &[Value::Number(3.0), Value::Number(1.0), Value::Number(2.0)]
            ),
            Value::Number(3.0)
        );
    }

    #[test]
    fn min_basic() {
        let mut c = ctx();
        assert_eq!(
            min(
                &mut c,
                &[Value::Number(3.0), Value::Number(1.0), Value::Number(2.0)]
            ),
            Value::Number(1.0)
        );
    }

    #[test]
    fn max_empty_returns_zero() {
        let mut c = ctx();
        assert_eq!(max(&mut c, &[Value::Empty]), Value::Number(0.0));
    }

    // ---- MEDIAN ------------------------------------------------------------

    #[test]
    fn median_odd() {
        let mut c = ctx();
        assert_eq!(
            median(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn median_even() {
        let mut c = ctx();
        // Excel: MEDIAN(1,2,3,4) = 2.5
        assert_eq!(
            median(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(4.0)
                ]
            ),
            Value::Number(2.5)
        );
    }

    #[test]
    fn median_empty_error() {
        let mut c = ctx();
        assert_eq!(
            median(&mut c, &[Value::Empty]),
            Value::Error(CellError::Num)
        );
    }

    // ---- MODE.SNGL ---------------------------------------------------------

    #[test]
    fn mode_basic() {
        let mut c = ctx();
        assert_eq!(
            mode_sngl(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(2.0),
                    Value::Number(3.0)
                ]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn mode_no_repeat() {
        let mut c = ctx();
        assert_eq!(
            mode_sngl(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Error(CellError::NA)
        );
    }

    // ---- LARGE / SMALL -----------------------------------------------------

    #[test]
    fn large_small() {
        let mut c = ctx();
        let data = [
            Value::Number(3.0),
            Value::Number(1.0),
            Value::Number(4.0),
            Value::Number(2.0),
        ];
        assert_eq!(
            large(&mut c, &[data[0].clone(), Value::Number(1.0)]),
            Value::Number(3.0)
        );
        // Use range for multi-value
        let mut c2 = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(1.0)),
            (2, 0, Value::Number(4.0)),
            (3, 0, Value::Number(2.0)),
        ]);
        assert_eq!(
            large(&mut c2, &[rng(0, 0, 3, 0), Value::Number(2.0)]),
            Value::Number(3.0)
        );
        assert_eq!(
            small(&mut c2, &[rng(0, 0, 3, 0), Value::Number(2.0)]),
            Value::Number(2.0)
        );
    }

    #[test]
    fn large_out_of_range() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        assert_eq!(
            large(&mut c, &[rng(0, 0, 0, 0), Value::Number(5.0)]),
            Value::Error(CellError::Num)
        );
    }

    // ---- RANK.EQ / RANK.AVG ------------------------------------------------

    #[test]
    fn rank_eq_desc() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(7.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(5.0)),
        ]);
        // 7 is rank 1 in descending
        assert_eq!(
            rank_eq(&mut c, &[Value::Number(7.0), rng(0, 0, 2, 0)]),
            Value::Number(1.0)
        );
    }

    #[test]
    fn rank_eq_asc() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(7.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(5.0)),
        ]);
        // 3 is rank 1 ascending
        assert_eq!(
            rank_eq(
                &mut c,
                &[Value::Number(3.0), rng(0, 0, 2, 0), Value::Number(1.0)]
            ),
            Value::Number(1.0)
        );
    }

    #[test]
    fn rank_avg_ties() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(3.0)),
        ]);
        // Both 5s tie for rank 1 and 2, avg = 1.5
        assert_eq!(
            rank_avg(&mut c, &[Value::Number(5.0), rng(0, 0, 2, 0)]),
            Value::Number(1.5)
        );
    }

    // ---- STDEV / VAR -------------------------------------------------------

    #[test]
    fn stdev_s_excel_example() {
        // Excel: STDEV.S(2,4,4,4,5,5,7,9) ≈ 2.138...
        let mut c = ctx();
        let data = vec![
            Value::Number(2.0),
            Value::Number(4.0),
            Value::Number(4.0),
            Value::Number(4.0),
            Value::Number(5.0),
            Value::Number(5.0),
            Value::Number(7.0),
            Value::Number(9.0),
        ];
        if let Value::Number(v) = stdev_s(&mut c, &data) {
            let diff = (v - 2.138_089_935_325_936).abs();
            assert!(diff < 1e-6, "stdev_s got {v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn stdev_p_basic() {
        let mut c = ctx();
        // Population std of [2,4] = 1.0
        assert_eq!(
            stdev_p(&mut c, &[Value::Number(2.0), Value::Number(4.0)]),
            Value::Number(1.0)
        );
    }

    #[test]
    fn var_s_single_error() {
        let mut c = ctx();
        assert_eq!(
            var_s(&mut c, &[Value::Number(5.0)]),
            Value::Error(CellError::Div0)
        );
    }

    // ---- PERCENTILE.INC ----------------------------------------------------

    #[test]
    fn percentile_inc_excel() {
        // Excel: PERCENTILE.INC({1,2,3,4}, 0.25) = 1.75
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
        ]);
        assert_eq!(
            percentile_inc(&mut c, &[rng(0, 0, 3, 0), Value::Number(0.25)]),
            Value::Number(1.75)
        );
    }

    #[test]
    fn percentile_inc_edges() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0)), (1, 0, Value::Number(2.0))]);
        assert_eq!(
            percentile_inc(&mut c, &[rng(0, 0, 1, 0), Value::Number(0.0)]),
            Value::Number(1.0)
        );
        let mut c2 = TestCtx::with_cells(&[(0, 0, Value::Number(1.0)), (1, 0, Value::Number(2.0))]);
        assert_eq!(
            percentile_inc(&mut c2, &[rng(0, 0, 1, 0), Value::Number(1.0)]),
            Value::Number(2.0)
        );
    }

    #[test]
    fn percentile_inc_invalid_p() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        assert_eq!(
            percentile_inc(&mut c, &[rng(0, 0, 0, 0), Value::Number(1.5)]),
            Value::Error(CellError::Num)
        );
    }

    // ---- QUARTILE.INC ------------------------------------------------------

    #[test]
    fn quartile_inc_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
        ]);
        // Q2 = median = 2.5
        assert_eq!(
            quartile_inc(&mut c, &[rng(0, 0, 3, 0), Value::Number(2.0)]),
            Value::Number(2.5)
        );
    }

    // ---- CORREL / COVARIANCE -----------------------------------------------

    #[test]
    fn correl_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        // Perfect positive correlation (allow floating-point near-1)
        if let Value::Number(v) = correl(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]) {
            assert!((v - 1.0).abs() < 1e-10, "correl={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn covariance_p_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (0, 1, Value::Number(3.0)),
            (1, 1, Value::Number(4.0)),
        ]);
        // COV_P([1,2],[3,4]) = 0.25
        assert_eq!(
            covariance_p(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]),
            Value::Number(0.25)
        );
    }

    // ---- SLOPE / INTERCEPT / RSQ -------------------------------------------

    #[test]
    fn slope_intercept() {
        // y = 2x + 1: slope=2, intercept=1
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        if let Value::Number(s) = slope(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]) {
            assert!((s - 2.0).abs() < 1e-10, "slope={s}");
        } else {
            panic!("expected number");
        }
        let mut c2 = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        if let Value::Number(ic) = intercept(&mut c2, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]) {
            assert!((ic - 1.0).abs() < 1e-10, "intercept={ic}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn rsq_perfect() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
        ]);
        if let Value::Number(v) = rsq(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]) {
            assert!((v - 1.0).abs() < 1e-10, "rsq={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- DEVSQ / AVEDEV ----------------------------------------------------

    #[test]
    fn devsq_basic() {
        let mut c = ctx();
        // {1,2,3}: mean=2, SS=(1+0+1)=2
        assert_eq!(
            devsq(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn avedev_basic() {
        let mut c = ctx();
        // {2,4}: mean=3, avg |deviation| = 1
        assert_eq!(
            avedev(&mut c, &[Value::Number(2.0), Value::Number(4.0)]),
            Value::Number(1.0)
        );
    }

    // ---- GEOMEAN / HARMEAN -------------------------------------------------

    #[test]
    fn geomean_basic() {
        let mut c = ctx();
        // GEOMEAN(4,9) = 6.0
        if let Value::Number(v) = geomean(&mut c, &[Value::Number(4.0), Value::Number(9.0)]) {
            assert!((v - 6.0).abs() < 1e-10);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn geomean_negative_error() {
        let mut c = ctx();
        assert_eq!(
            geomean(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn harmean_basic() {
        let mut c = ctx();
        // HARMEAN(1,2,4) = 3/(1+0.5+0.25) = 1.714...
        if let Value::Number(v) = harmean(
            &mut c,
            &[Value::Number(1.0), Value::Number(2.0), Value::Number(4.0)],
        ) {
            assert!((v - 12.0 / 7.0).abs() < 1e-10, "harmean={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- TRIMMEAN ----------------------------------------------------------

