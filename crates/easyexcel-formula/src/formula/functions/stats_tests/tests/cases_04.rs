    // --- BETA.INV ---

    #[test]
    fn beta_inv_basic() {
        let c = &mut ctx();
        // BETA.INV(0.5, 2, 3) ≈ 0.3858
        let r = beta_inv(c, &[Value::Number(0.5), Value::Number(2.0), Value::Number(3.0)]);
        if let Value::Number(v) = r {
            assert!((v - 0.3858).abs() < 0.01, "BETA.INV = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn beta_inv_bad_alpha_is_num() {
        let c = &mut ctx();
        let r = beta_inv(c, &[Value::Number(0.5), Value::Number(-1.0), Value::Number(3.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn beta_inv_bad_p_is_num() {
        let c = &mut ctx();
        let r = beta_inv(c, &[Value::Number(1.5), Value::Number(2.0), Value::Number(3.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- CHISQ.DIST.RT ---

    #[test]
    fn chisq_dist_rt_basic() {
        let c = &mut ctx();
        // CHISQ.DIST.RT(3.84, 1) ≈ 0.05
        let r = chisq_dist_rt(c, &[Value::Number(3.84), Value::Number(1.0)]);
        if let Value::Number(v) = r {
            assert!((v - 0.05).abs() < 0.01, "CHISQ.DIST.RT = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn chisq_dist_rt_negative_x_is_num() {
        let c = &mut ctx();
        let r = chisq_dist_rt(c, &[Value::Number(-1.0), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- CHISQ.INV.RT ---

    #[test]
    fn chisq_inv_rt_basic() {
        let c = &mut ctx();
        // CHISQ.INV.RT(0.05, 1) ≈ 3.84
        let r = chisq_inv_rt(c, &[Value::Number(0.05), Value::Number(1.0)]);
        if let Value::Number(v) = r {
            assert!((v - 3.84).abs() < 0.1, "CHISQ.INV.RT = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn chisq_inv_rt_zero_p_is_num() {
        let c = &mut ctx();
        let r = chisq_inv_rt(c, &[Value::Number(0.0), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- F.DIST (cumulative) ---

    #[test]
    fn f_dist_cumulative_basic() {
        let c = &mut ctx();
        // F.DIST(5.0, 5, 10, TRUE) — some valid CDF value between 0 and 1
        let r = f_dist(c, &[Value::Number(5.0), Value::Number(5.0), Value::Number(10.0), Value::Number(1.0)]);
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 1.0, "F.DIST CDF = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn f_dist_pdf_basic() {
        let c = &mut ctx();
        // F.DIST(1.0, 5, 10, FALSE) — PDF at x=1
        let r = f_dist(c, &[Value::Number(1.0), Value::Number(5.0), Value::Number(10.0), Value::Number(0.0)]);
        if let Value::Number(v) = r {
            assert!(v > 0.0, "F.DIST PDF = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn f_dist_negative_x_is_num() {
        let c = &mut ctx();
        let r = f_dist(c, &[Value::Number(-1.0), Value::Number(5.0), Value::Number(10.0), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- F.INV.RT ---

    #[test]
    fn f_inv_rt_basic() {
        let c = &mut ctx();
        // F.INV.RT(0.05, 5, 10) should be a positive number
        let r = f_inv_rt(c, &[Value::Number(0.05), Value::Number(5.0), Value::Number(10.0)]);
        if let Value::Number(v) = r {
            assert!(v > 0.0, "F.INV.RT = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn f_inv_rt_zero_p_is_num() {
        let c = &mut ctx();
        let r = f_inv_rt(c, &[Value::Number(0.0), Value::Number(5.0), Value::Number(10.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- T.DIST.RT ---

    #[test]
    fn t_dist_rt_basic() {
        let c = &mut ctx();
        // T.DIST.RT(2.0, 10) ≈ 0.037
        let r = t_dist_rt(c, &[Value::Number(2.0), Value::Number(10.0)]);
        if let Value::Number(v) = r {
            assert!((v - 0.037).abs() < 0.01, "T.DIST.RT = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- T.INV ---

    #[test]
    fn t_inv_basic() {
        let c = &mut ctx();
        // T.INV(0.975, 10) ≈ 2.228
        let r = t_inv(c, &[Value::Number(0.975), Value::Number(10.0)]);
        if let Value::Number(v) = r {
            assert!((v - 2.228).abs() < 0.05, "T.INV = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn t_inv_zero_p_is_num() {
        let c = &mut ctx();
        let r = t_inv(c, &[Value::Number(0.0), Value::Number(10.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- LOGNORM.INV ---

    #[test]
    fn lognorm_inv_basic() {
        let c = &mut ctx();
        // LOGNORM.INV(0.5, 0, 1) = exp(0) = 1.0
        let r = lognorm_inv(c, &[Value::Number(0.5), Value::Number(0.0), Value::Number(1.0)]);
        if let Value::Number(v) = r {
            assert!((v - 1.0).abs() < 0.01, "LOGNORM.INV = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn lognorm_inv_bad_sd_is_num() {
        let c = &mut ctx();
        let r = lognorm_inv(c, &[Value::Number(0.5), Value::Number(0.0), Value::Number(-1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn lognorm_inv_p_one_is_num() {
        let c = &mut ctx();
        let r = lognorm_inv(c, &[Value::Number(1.0), Value::Number(0.0), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- BETA.DIST error branches ---

    #[test]
    fn beta_dist_bad_alpha_is_num() {
        let c = &mut ctx();
        let r = beta_dist(c, &[Value::Number(0.5), Value::Number(-1.0), Value::Number(3.0), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn beta_dist_x_out_of_range_is_num() {
        let c = &mut ctx();
        let r = beta_dist(c, &[Value::Number(1.5), Value::Number(2.0), Value::Number(3.0), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- CHISQ.DIST PDF mode ---

    #[test]
    fn chisq_dist_pdf_basic() {
        let c = &mut ctx();
        // CHISQ.DIST(3.84, 1, FALSE) — PDF
        let r = chisq_dist(c, &[Value::Number(3.84), Value::Number(1.0), Value::Number(0.0)]);
        if let Value::Number(v) = r {
            assert!(v > 0.0, "CHISQ PDF = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- F.INV ---

    #[test]
    fn f_inv_basic() {
        let c = &mut ctx();
        // F.INV(0.95, 5, 10) should be positive
        let r = f_inv(c, &[Value::Number(0.95), Value::Number(5.0), Value::Number(10.0)]);
        if let Value::Number(v) = r {
            assert!(v > 0.0, "F.INV = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- HYPGEOM.DIST cumulative ---

    #[test]
    fn hypgeom_dist_cumulative() {
        let c = &mut ctx();
        // HYPGEOM.DIST(1, 4, 8, 20, TRUE)
        let r = hypgeom_dist(c, &[
            Value::Number(1.0), Value::Number(4.0),
            Value::Number(8.0), Value::Number(20.0), Value::Number(1.0),
        ]);
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v <= 1.0, "HYPGEOM CDF = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- NEGBINOM.DIST cumulative ---

    #[test]
    fn negbinom_dist_cumulative() {
        let c = &mut ctx();
        // NEGBINOM.DIST(5, 2, 0.5, TRUE)
        let r = negbinom_dist(c, &[
            Value::Number(5.0), Value::Number(2.0),
            Value::Number(0.5), Value::Number(1.0),
        ]);
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v <= 1.0, "NEGBINOM CDF = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- BINOM.DIST.RANGE with default s2 ---

    #[test]
    fn binom_dist_range_default_s2() {
        let c = &mut ctx();
        // BINOM.DIST.RANGE(10, 0.5, 5) — s2 defaults to s1
        let r = binom_dist_range(c, &[
            Value::Number(10.0), Value::Number(0.5), Value::Number(5.0),
        ]);
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v <= 1.0, "BINOM.DIST.RANGE = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn binom_dist_range_bad_n_is_num() {
        let c = &mut ctx();
        let r = binom_dist_range(c, &[
            Value::Number(-1.0), Value::Number(0.5), Value::Number(0.0), Value::Number(1.0),
        ]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- CHISQ.INV error branch ---

    #[test]
    fn chisq_inv_bad_df_is_num() {
        let c = &mut ctx();
        let r = chisq_inv(c, &[Value::Number(0.5), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- F.DIST.RT error branch ---

    #[test]
    fn f_dist_rt_bad_df_is_num() {
        let c = &mut ctx();
        let r = f_dist_rt(c, &[Value::Number(1.0), Value::Number(0.0), Value::Number(10.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }
