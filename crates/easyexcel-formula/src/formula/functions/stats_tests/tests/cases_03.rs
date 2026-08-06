    #[test]
    fn binom_dist_range_kat() {
        let mut c = ctx();
        // P(2<=X<=3 | n=5,p=0.5) = (10+10)/32 = 0.625
        approx(
            binom_dist_range(
                &mut c,
                &[
                    Value::Number(5.0),
                    Value::Number(0.5),
                    Value::Number(2.0),
                    Value::Number(3.0),
                ],
            ),
            0.625,
            1e-5,
            "BINOM.DIST.RANGE",
        );
    }

    // ---- WEIBULL -----------------------------------------------------------

    #[test]
    fn weibull_cdf_kat() {
        let mut c = ctx();
        // WEIBULL.DIST(1,1,1,TRUE) = 1 - e^-1
        approx(
            weibull_dist(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                ],
            ),
            1.0 - (-1.0_f64).exp(),
            1e-9,
            "WEIBULL.DIST",
        );
    }

    // ---- CONFIDENCE.T ------------------------------------------------------

    #[test]
    fn confidence_t_kat() {
        let mut c = ctx();
        // CONFIDENCE.T(0.05, 1, 50): t_{0.975,49}≈2.0096, /sqrt(50)≈0.2842
        approx(
            confidence_t(
                &mut c,
                &[Value::Number(0.05), Value::Number(1.0), Value::Number(50.0)],
            ),
            0.28419,
            1e-3,
            "CONFIDENCE.T",
        );
    }

    // ---- PROB --------------------------------------------------------------

    #[test]
    fn prob_kat() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(0.0)),
            (1, 0, Value::Number(1.0)),
            (2, 0, Value::Number(2.0)),
            (3, 0, Value::Number(3.0)),
            (0, 1, Value::Number(0.2)),
            (1, 1, Value::Number(0.3)),
            (2, 1, Value::Number(0.1)),
            (3, 1, Value::Number(0.4)),
        ]);
        // P(1 <= x <= 2) = 0.3 + 0.1 = 0.4
        approx(
            prob(
                &mut c,
                &[
                    rng(0, 0, 3, 0),
                    rng(0, 1, 3, 1),
                    Value::Number(1.0),
                    Value::Number(2.0),
                ],
            ),
            0.4,
            1e-9,
            "PROB",
        );
    }

    #[test]
    fn prob_bad_sum() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(0.0)),
            (1, 0, Value::Number(1.0)),
            (0, 1, Value::Number(0.2)),
            (1, 1, Value::Number(0.3)),
        ]);
        // probabilities sum to 0.5 != 1 → #NUM!
        assert_eq!(
            prob(
                &mut c,
                &[rng(0, 0, 1, 0), rng(0, 1, 1, 1), Value::Number(0.0)],
            ),
            Value::Error(CellError::Num)
        );
    }

    // ---- Z.TEST ------------------------------------------------------------

    #[test]
    fn z_test_kat() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(6.0)),
            (2, 0, Value::Number(7.0)),
            (3, 0, Value::Number(8.0)),
            (4, 0, Value::Number(6.0)),
        ]);
        // mean=6, n=5. With x=4, sigma=2: z=(6-4)/(2/sqrt5)=2.236 → 1-Phi≈0.0127
        approx(
            z_test(
                &mut c,
                &[rng(0, 0, 4, 0), Value::Number(4.0), Value::Number(2.0)],
            ),
            0.012_674,
            1e-4,
            "Z.TEST",
        );
    }

    // ---- SKEW.P / STEYX ----------------------------------------------------

    #[test]
    fn skew_p_symmetric() {
        let mut c = ctx();
        approx(
            skew_p(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(4.0),
                    Value::Number(5.0),
                ],
            ),
            0.0,
            1e-9,
            "SKEW.P",
        );
    }

    #[test]
    fn steyx_perfect_fit() {
        // perfectly linear data → STEYX = 0
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (3, 0, Value::Number(9.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
            (3, 1, Value::Number(4.0)),
        ]);
        approx(
            steyx(&mut c, &[rng(0, 0, 3, 0), rng(0, 1, 3, 1)]),
            0.0,
            1e-9,
            "STEYX",
        );
    }

    // ---- FREQUENCY ---------------------------------------------------------

    #[test]
    fn frequency_kat() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
            (4, 0, Value::Number(5.0)),
            (0, 1, Value::Number(2.0)),
            (1, 1, Value::Number(4.0)),
        ]);
        // bins [2,4]: <=2 →{1,2}=2, (2,4]→{3,4}=2, >4 →{5}=1
        let r = frequency(&mut c, &[rng(0, 0, 4, 0), rng(0, 1, 1, 1)]);
        if let Value::Array(a) = r {
            assert_eq!(a.data.len(), 3);
            assert_eq!(a.data[0], Value::Number(2.0));
            assert_eq!(a.data[1], Value::Number(2.0));
            assert_eq!(a.data[2], Value::Number(1.0));
        } else {
            panic!("expected array, got {r:?}");
        }
    }

    // ---- TREND / LINEST ----------------------------------------------------

    #[test]
    fn trend_linear() {
        // y = 2x+1 at x=1,2,3 → predict at x=4 → 9
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
            (0, 2, Value::Number(4.0)),
        ]);
        let r = trend(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1), rng(0, 2, 0, 2)]);
        if let Value::Array(a) = r {
            approx(a.data[0].clone(), 9.0, 1e-9, "TREND");
        } else {
            panic!("expected array, got {r:?}");
        }
    }

    #[test]
    fn linest_slope_intercept() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        let r = linest(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        if let Value::Array(a) = r {
            approx(a.data[0].clone(), 2.0, 1e-9, "LINEST slope");
            approx(a.data[1].clone(), 1.0, 1e-9, "LINEST intercept");
        } else {
            panic!("expected array, got {r:?}");
        }
    }

    #[test]
    fn forecast_ets_is_na() {
        let mut c = ctx();
        assert_eq!(
            forecast_ets_na(
                &mut c,
                &[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]
            ),
            Value::Error(CellError::NA)
        );
    }
