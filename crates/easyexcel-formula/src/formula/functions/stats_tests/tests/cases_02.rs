    #[test]
    fn trimmean_basic() {
        let mut c = ctx();
        // TRIMMEAN({1,2,3,4,5,6,7,8,9,10}, 0.2) trims 1 from each end → mean(2..9)=5.5
        let _data: Vec<Value> = (1..=10).map(|i| Value::Number(f64::from(i))).collect();
        if let Value::Number(v) = trimmean(
            &mut c,
            &[
                // Use all as direct scalar args for simplicity
                Value::Number(1.0),
                Value::Number(0.2),
            ],
        ) {
            // Single element after trim p/2=0.1 → trim 0 elements → mean(1)=1
            let _ = v;
        }
        // More meaningful: use range
        let mut c2 = TestCtx::with_cells(
            &(1..=10u32)
                .map(|i| (i - 1, 0, Value::Number(f64::from(i))))
                .collect::<Vec<_>>(),
        );
        if let Value::Number(v) = trimmean(&mut c2, &[rng(0, 0, 9, 0), Value::Number(0.2)]) {
            assert!((v - 5.5).abs() < 1e-10, "trimmean={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- SKEW / KURT -------------------------------------------------------

    #[test]
    fn skew_symmetric() {
        let mut c = ctx();
        // Symmetric distribution: skew should be 0
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ];
        if let Value::Number(v) = skew(&mut c, &data) {
            assert!(v.abs() < 1e-10, "skew={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn skew_too_few() {
        let mut c = ctx();
        assert_eq!(
            skew(&mut c, &[Value::Number(1.0), Value::Number(2.0)]),
            Value::Error(CellError::Div0)
        );
    }

    // ---- STANDARDIZE -------------------------------------------------------

    #[test]
    fn standardize_basic() {
        let mut c = ctx();
        assert_eq!(
            standardize(
                &mut c,
                &[Value::Number(5.0), Value::Number(3.0), Value::Number(2.0)]
            ),
            Value::Number(1.0)
        );
    }

    #[test]
    fn standardize_invalid_std() {
        let mut c = ctx();
        assert_eq!(
            standardize(
                &mut c,
                &[Value::Number(5.0), Value::Number(3.0), Value::Number(0.0)]
            ),
            Value::Error(CellError::Num)
        );
    }

    // ---- FISHER / FISHERINV ------------------------------------------------

    #[test]
    fn fisher_fisherinv() {
        let mut c = ctx();
        let x = 0.5;
        if let Value::Number(f) = fisher(&mut c, &[Value::Number(x)]) {
            if let Value::Number(inv) = fisherinv(&mut c, &[Value::Number(f)]) {
                assert!((inv - x).abs() < 1e-10, "round-trip failed: {inv}");
            } else {
                panic!("fisherinv failed");
            }
        } else {
            panic!("fisher failed");
        }
    }

    #[test]
    fn fisher_out_of_range() {
        let mut c = ctx();
        assert_eq!(
            fisher(&mut c, &[Value::Number(1.0)]),
            Value::Error(CellError::Num)
        );
        assert_eq!(
            fisher(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    // ---- NORM.DIST ---------------------------------------------------------

    #[test]
    fn norm_dist_cumulative() {
        let mut c = ctx();
        // NORM.DIST(0, 0, 1, TRUE) = 0.5
        if let Value::Number(v) = norm_dist(
            &mut c,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(1.0),
            ],
        ) {
            assert!((v - 0.5).abs() < 1e-6, "norm_dist={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn norm_dist_pdf() {
        let mut c = ctx();
        // NORM.DIST(0, 0, 1, FALSE) = 1/sqrt(2π) ≈ 0.3989...
        if let Value::Number(v) = norm_dist(
            &mut c,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(0.0),
            ],
        ) {
            let expected = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
            assert!((v - expected).abs() < 1e-6, "norm_pdf={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn norm_dist_invalid_std() {
        let mut c = ctx();
        assert_eq!(
            norm_dist(
                &mut c,
                &[
                    Value::Number(0.0),
                    Value::Number(0.0),
                    Value::Number(-1.0),
                    Value::Number(1.0),
                ]
            ),
            Value::Error(CellError::Num)
        );
    }

    // ---- NORM.S.INV / NORM.INV ---------------------------------------------

    #[test]
    fn norm_s_inv_roundtrip() {
        let mut c = ctx();
        // norm_s_dist(norm_s_inv(0.75), cumulative) ≈ 0.75
        if let Value::Number(z) = norm_s_inv(&mut c, &[Value::Number(0.75)]) {
            let cdf = norm_cdf(z);
            assert!((cdf - 0.75).abs() < 1e-4, "roundtrip {cdf}");
        } else {
            panic!("expected number");
        }
    }

    // ---- BINOM.DIST --------------------------------------------------------

    #[test]
    fn binom_dist_pmf() {
        let mut c = ctx();
        // P(X=2 | n=5, p=0.5) = C(5,2)*0.5^5 = 10/32 = 0.3125
        if let Value::Number(v) = binom_dist(
            &mut c,
            &[
                Value::Number(2.0),
                Value::Number(5.0),
                Value::Number(0.5),
                Value::Number(0.0),
            ],
        ) {
            assert!((v - 0.3125).abs() < 1e-6, "binom_pmf={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn binom_dist_cdf() {
        let mut c = ctx();
        // P(X<=2 | n=5, p=0.5) = 0.5
        if let Value::Number(v) = binom_dist(
            &mut c,
            &[
                Value::Number(2.0),
                Value::Number(5.0),
                Value::Number(0.5),
                Value::Number(1.0),
            ],
        ) {
            assert!((v - 0.5).abs() < 1e-4, "binom_cdf={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- POISSON.DIST ------------------------------------------------------

    #[test]
    fn poisson_dist_pmf() {
        let mut c = ctx();
        // P(X=2 | lambda=3) = e^{-3} * 9/2 ≈ 0.2240...
        if let Value::Number(v) = poisson_dist(
            &mut c,
            &[Value::Number(2.0), Value::Number(3.0), Value::Number(0.0)],
        ) {
            let expected = (-3.0_f64).exp() * 9.0 / 2.0;
            assert!(
                (v - expected).abs() < 1e-8,
                "poisson_pmf={v} expected={expected}"
            );
        } else {
            panic!("expected number");
        }
    }

    // ---- EXPON.DIST --------------------------------------------------------

    #[test]
    fn expon_dist_cdf() {
        let mut c = ctx();
        // P(X<=1 | lambda=1) = 1 - e^{-1}
        if let Value::Number(v) = expon_dist(
            &mut c,
            &[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)],
        ) {
            let expected = 1.0 - (-1.0_f64).exp();
            assert!((v - expected).abs() < 1e-10, "expon_cdf={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- GAUSS / PHI -------------------------------------------------------

    #[test]
    fn gauss_zero() {
        let mut c = ctx();
        // GAUSS(0) = Phi(0) - 0.5 = 0
        if let Value::Number(v) = gauss(&mut c, &[Value::Number(0.0)]) {
            assert!(v.abs() < 1e-6, "gauss(0)={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn phi_zero() {
        let mut c = ctx();
        // PHI(0) = 1/sqrt(2π)
        if let Value::Number(v) = phi(&mut c, &[Value::Number(0.0)]) {
            let expected = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
            assert!((v - expected).abs() < 1e-10);
        } else {
            panic!("expected number");
        }
    }

    // ---- MAXIFS / MINIFS ---------------------------------------------------

    #[test]
    fn maxifs_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Number(20.0)),
            (2, 0, Value::Number(30.0)),
            (0, 1, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (2, 1, Value::Text("a".into())),
        ]);
        let r = maxifs(
            &mut c,
            &[rng(0, 0, 2, 0), rng(0, 1, 2, 1), Value::Text("a".into())],
        );
        assert_eq!(r, Value::Number(30.0));
    }

    #[test]
    fn minifs_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Number(20.0)),
            (2, 0, Value::Number(30.0)),
            (0, 1, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (2, 1, Value::Text("a".into())),
        ]);
        let r = minifs(
            &mut c,
            &[rng(0, 0, 2, 0), rng(0, 1, 2, 1), Value::Text("a".into())],
        );
        assert_eq!(r, Value::Number(10.0));
    }

    // ---- FORECAST.LINEAR ---------------------------------------------------

    #[test]
    fn forecast_linear_basic() {
        // y = 2x+1: forecast at x=4 → 9
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        if let Value::Number(v) = forecast_linear(
            &mut c,
            &[Value::Number(4.0), rng(0, 0, 2, 0), rng(0, 1, 2, 1)],
        ) {
            assert!((v - 9.0).abs() < 1e-10, "forecast={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- CONFIDENCE.NORM ---------------------------------------------------

    #[test]
    fn confidence_norm_basic() {
        let mut c = ctx();
        // For alpha=0.05, std=1, n=100: z≈1.96, result≈0.196
        if let Value::Number(v) = confidence_norm(
            &mut c,
            &[
                Value::Number(0.05),
                Value::Number(1.0),
                Value::Number(100.0),
            ],
        ) {
            assert!(v > 0.18 && v < 0.21, "confidence={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- PERCENTRANK.INC ---------------------------------------------------

    #[test]
    fn percentrank_inc_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
        ]);
        // rank of 2 in [1,2,3,4] with INC = 1/(4-1) = 0.333
        if let Value::Number(v) = percentrank_inc(&mut c, &[rng(0, 0, 3, 0), Value::Number(2.0)]) {
            assert!((v - 0.333).abs() < 0.001, "percentrank={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- helper for new distribution KATs ----------------------------------

    fn approx(v: Value, expected: f64, tol: f64, name: &str) {
        if let Value::Number(x) = v {
            assert!(
                (x - expected).abs() < tol,
                "{name}: got {x}, want {expected}"
            );
        } else {
            panic!("{name}: expected number, got {v:?}");
        }
    }

    // ---- GAMMA.DIST / GAMMA.INV --------------------------------------------

    #[test]
    fn gamma_dist_cdf_kat() {
        let mut c = ctx();
        // GAMMA.DIST(2,1,1,TRUE) = 1 - e^-2 ≈ 0.8646647
        approx(
            gamma_dist(
                &mut c,
                &[
                    Value::Number(2.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                ],
            ),
            1.0 - (-2.0_f64).exp(),
            1e-6,
            "GAMMA.DIST",
        );
    }

    #[test]
    fn gamma_inv_roundtrip() {
        let mut c = ctx();
        // GAMMA.INV(GAMMA.DIST(x)) ≈ x
        let p = 0.864_664_7;
        approx(
            gamma_inv(
                &mut c,
                &[Value::Number(p), Value::Number(1.0), Value::Number(1.0)],
            ),
            2.0,
            1e-4,
            "GAMMA.INV",
        );
    }

    // ---- BETA.DIST ---------------------------------------------------------

    #[test]
    fn beta_dist_cdf_kat() {
        let mut c = ctx();
        // BETA.DIST(0.5, 2, 3, TRUE) = I_0.5(2,3) = 0.6875
        approx(
            beta_dist(
                &mut c,
                &[
                    Value::Number(0.5),
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(1.0),
                ],
            ),
            0.6875,
            1e-5,
            "BETA.DIST",
        );
    }

    // ---- CHISQ -------------------------------------------------------------

    #[test]
    fn chisq_dist_kat() {
        let mut c = ctx();
        // CHISQ.DIST(3,4,TRUE) ≈ 0.4421746
        approx(
            chisq_dist(
                &mut c,
                &[Value::Number(3.0), Value::Number(4.0), Value::Number(1.0)],
            ),
            0.442_174_6,
            1e-5,
            "CHISQ.DIST",
        );
    }

    #[test]
    fn chisq_inv_roundtrip() {
        let mut c = ctx();
        approx(
            chisq_inv(&mut c, &[Value::Number(0.442_174_6), Value::Number(4.0)]),
            3.0,
            1e-3,
            "CHISQ.INV",
        );
    }

    // ---- F distribution ----------------------------------------------------

    #[test]
    fn f_dist_rt_kat() {
        let mut c = ctx();
        // F.DIST.RT(1,5,5) = 0.5 (median of F(5,5) is 1)
        approx(
            f_dist_rt(
                &mut c,
                &[Value::Number(1.0), Value::Number(5.0), Value::Number(5.0)],
            ),
            0.5,
            1e-4,
            "F.DIST.RT",
        );
    }

    #[test]
    fn f_inv_roundtrip() {
        let mut c = ctx();
        // F.INV(0.5,5,5) should be ~1
        approx(
            f_inv(
                &mut c,
                &[Value::Number(0.5), Value::Number(5.0), Value::Number(5.0)],
            ),
            1.0,
            1e-3,
            "F.INV",
        );
    }

    // ---- T distribution ----------------------------------------------------

    #[test]
    fn t_dist_cdf_kat() {
        let mut c = ctx();
        // T.DIST(2,10,TRUE) ≈ 0.9633
        approx(
            t_dist(
                &mut c,
                &[Value::Number(2.0), Value::Number(10.0), Value::Number(1.0)],
            ),
            0.9633,
            1e-4,
            "T.DIST",
        );
    }

    #[test]
    fn t_dist_2t_kat() {
        let mut c = ctx();
        // T.DIST.2T(2,10) = 2*(1-0.9633) ≈ 0.0734
        approx(
            t_dist_2t(&mut c, &[Value::Number(2.0), Value::Number(10.0)]),
            0.07339,
            1e-4,
            "T.DIST.2T",
        );
    }

    #[test]
    fn t_inv_2t_roundtrip() {
        let mut c = ctx();
        // T.INV.2T(0.05, 10) ≈ 2.2281
        approx(
            t_inv_2t(&mut c, &[Value::Number(0.05), Value::Number(10.0)]),
            2.2281,
            1e-3,
            "T.INV.2T",
        );
    }

    // ---- LOGNORM -----------------------------------------------------------

    #[test]
    fn lognorm_dist_kat() {
        let mut c = ctx();
        // LOGNORM.DIST(1,0,1,TRUE) = NORM.S.DIST(0) = 0.5
        approx(
            lognorm_dist(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(0.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                ],
            ),
            0.5,
            1e-6,
            "LOGNORM.DIST",
        );
    }

    // ---- NEGBINOM / HYPGEOM ------------------------------------------------

    #[test]
    fn negbinom_pmf_kat() {
        let mut c = ctx();
        // NEGBINOM.DIST(2,3,0.5,FALSE) = C(4,2) * 0.5^3 * 0.5^2 = 6/32 = 0.1875
        approx(
            negbinom_dist(
                &mut c,
                &[
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(0.5),
                    Value::Number(0.0),
                ],
            ),
            0.1875,
            1e-6,
            "NEGBINOM.DIST",
        );
    }

    #[test]
    fn hypgeom_pmf_kat() {
        let mut c = ctx();
        // HYPGEOM.DIST(1,4,8,20,FALSE): C(8,1)*C(12,3)/C(20,4)
        // = 8*220/4845 = 1760/4845 ≈ 0.36326
        approx(
            hypgeom_dist(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(4.0),
                    Value::Number(8.0),
                    Value::Number(20.0),
                    Value::Number(0.0),
                ],
            ),
            0.363_261,
            1e-5,
            "HYPGEOM.DIST",
        );
    }

    // ---- BINOM.INV / BINOM.DIST.RANGE --------------------------------------

    #[test]
    fn binom_inv_kat() {
        let mut c = ctx();
        // BINOM.INV(10, 0.5, 0.5) = 5
        approx(
            binom_inv(
                &mut c,
                &[Value::Number(10.0), Value::Number(0.5), Value::Number(0.5)],
            ),
            5.0,
            1e-9,
            "BINOM.INV",
        );
    }

