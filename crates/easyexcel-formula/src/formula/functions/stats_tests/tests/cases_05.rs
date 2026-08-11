    // --- MODE.MULT ---

    #[test]
    fn mode_mult_basic() {
        let mut c = ctx();
        // MODE.MULT(1,2,3,2,4,2) → [2]
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(2.0));
        c.set(4, 0, Value::Number(4.0));
        c.set(5, 0, Value::Number(2.0));
        let r = mode_mult(&mut c, &[rng(0, 0, 5, 0)]);
        if let Value::Array(a) = r {
            assert_eq!(a.rows, 1);
            assert_eq!(a.cols, 1);
            assert_eq!(a.data[0], Value::Number(2.0));
        } else {
            panic!("expected Array, got {r:?}");
        }
    }

    #[test]
    fn mode_mult_multiple_modes() {
        let mut c = ctx();
        // MODE.MULT(1,2,2,3,3) → [2,3] (both appear twice)
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(2.0));
        c.set(3, 0, Value::Number(3.0));
        c.set(4, 0, Value::Number(3.0));
        let r = mode_mult(&mut c, &[rng(0, 0, 4, 0)]);
        if let Value::Array(a) = r {
            assert_eq!(a.rows, 2);
            assert_eq!(a.cols, 1);
            assert_eq!(a.data[0], Value::Number(2.0));
            assert_eq!(a.data[1], Value::Number(3.0));
        } else {
            panic!("expected Array, got {r:?}");
        }
    }

    #[test]
    fn mode_mult_no_mode_is_na() {
        let mut c = ctx();
        // MODE.MULT(1,2,3) → #N/A (each appears once)
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = mode_mult(&mut c, &[rng(0, 0, 2, 0)]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    #[test]
    fn mode_mult_empty_is_na() {
        let mut c = ctx();
        let r = mode_mult(&mut c, &[]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- MODE.SNGL ---

    #[test]
    fn mode_sngl_basic() {
        let mut c = ctx();
        // MODE.SNGL(1,2,3,2,4) → 2
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(2.0));
        c.set(4, 0, Value::Number(4.0));
        let r = mode_sngl(&mut c, &[rng(0, 0, 4, 0)]);
        assert_eq!(r, Value::Number(2.0));
    }

    #[test]
    fn mode_sngl_no_mode_is_na() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        let r = mode_sngl(&mut c, &[rng(0, 0, 1, 0)]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- LARGE / SMALL ---

    #[test]
    fn large_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(5.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        c.set(4, 0, Value::Number(2.0));
        // LARGE(..., 2) → 4
        let r = large(&mut c, &[rng(0, 0, 4, 0), Value::Number(2.0)]);
        assert_eq!(r, Value::Number(4.0));
    }

    #[test]
    fn large_k_zero_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        let r = large(&mut c, &[rng(0, 0, 0, 0), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn large_k_too_large_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        let r = large(&mut c, &[rng(0, 0, 0, 0), Value::Number(5.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn small_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(5.0));
        c.set(1, 0, Value::Number(1.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        c.set(4, 0, Value::Number(2.0));
        // SMALL(..., 2) → 2
        let r = small(&mut c, &[rng(0, 0, 4, 0), Value::Number(2.0)]);
        assert_eq!(r, Value::Number(2.0));
    }

    #[test]
    fn small_k_zero_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        let r = small(&mut c, &[rng(0, 0, 0, 0), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- RANK.EQ ---

    #[test]
    fn rank_eq_descending() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(7.0));
        c.set(1, 0, Value::Number(3.0));
        c.set(2, 0, Value::Number(5.0));
        c.set(3, 0, Value::Number(1.0));
        // RANK.EQ(5, ...) descending → rank 2
        let r = rank_eq(&mut c, &[Value::Number(5.0), rng(0, 0, 3, 0)]);
        assert_eq!(r, Value::Number(2.0));
    }

    #[test]
    fn rank_eq_ascending() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(7.0));
        c.set(1, 0, Value::Number(3.0));
        c.set(2, 0, Value::Number(5.0));
        c.set(3, 0, Value::Number(1.0));
        // RANK.EQ(5, ..., 1) ascending → rank 3
        let r = rank_eq(
            &mut c,
            &[Value::Number(5.0), rng(0, 0, 3, 0), Value::Number(1.0)],
        );
        assert_eq!(r, Value::Number(3.0));
    }

    #[test]
    fn rank_eq_not_in_list_is_na() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(7.0));
        c.set(1, 0, Value::Number(3.0));
        let r = rank_eq(&mut c, &[Value::Number(99.0), rng(0, 0, 1, 0)]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- RANK.AVG ---

    #[test]
    fn rank_avg_no_ties() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(7.0));
        c.set(1, 0, Value::Number(3.0));
        c.set(2, 0, Value::Number(5.0));
        // RANK.AVG(5, ...) → 2 (no ties)
        let r = rank_avg(&mut c, &[Value::Number(5.0), rng(0, 0, 2, 0)]);
        assert_eq!(r, Value::Number(2.0));
    }

    #[test]
    fn rank_avg_with_ties() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(5.0));
        c.set(1, 0, Value::Number(3.0));
        c.set(2, 0, Value::Number(5.0));
        c.set(3, 0, Value::Number(1.0));
        // RANK.AVG(5, ...) → (1+2)/2 = 1.5
        let r = rank_avg(&mut c, &[Value::Number(5.0), rng(0, 0, 3, 0)]);
        assert_eq!(r, Value::Number(1.5));
    }

    #[test]
    fn rank_avg_not_in_list_is_na() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(7.0));
        let r = rank_avg(&mut c, &[Value::Number(99.0), rng(0, 0, 0, 0)]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- VAR.S / VAR.P ---

    #[test]
    fn var_s_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        c.set(4, 0, Value::Number(5.0));
        let r = var_s(&mut c, &[rng(0, 0, 4, 0)]);
        if let Value::Number(v) = r {
            // VAR.S = 2.5
            assert!((v - 2.5).abs() < 1e-10, "VAR.S = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn var_p_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        c.set(4, 0, Value::Number(5.0));
        let r = var_p(&mut c, &[rng(0, 0, 4, 0)]);
        if let Value::Number(v) = r {
            // VAR.P = 2.0
            assert!((v - 2.0).abs() < 1e-10, "VAR.P = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn var_p_single_value_is_zero() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(5.0));
        let r = var_p(&mut c, &[rng(0, 0, 0, 0)]);
        // Single value: population variance is 0
        assert_eq!(r, Value::Number(0.0));
    }

    // --- STDEV.S / STDEV.P ---

    #[test]
    fn stdev_s_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        c.set(4, 0, Value::Number(5.0));
        let r = stdev_s(&mut c, &[rng(0, 0, 4, 0)]);
        if let Value::Number(v) = r {
            // sqrt(2.5) ≈ 1.5811
            assert!((v - 2.5_f64.sqrt()).abs() < 1e-10, "STDEV.S = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn stdev_p_basic_2() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        c.set(4, 0, Value::Number(5.0));
        let r = stdev_p(&mut c, &[rng(0, 0, 4, 0)]);
        if let Value::Number(v) = r {
            assert!((v - 2.0_f64.sqrt()).abs() < 1e-10, "STDEV.P = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- PERCENTILE.INC ---

    #[test]
    fn percentile_inc_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        // PERCENTILE.INC(0.3) → rank = 0.3*3 = 0.9 → interp(1,2) = 1.9
        let r = percentile_inc(&mut c, &[rng(0, 0, 3, 0), Value::Number(0.3)]);
        if let Value::Number(v) = r {
            assert!((v - 1.9).abs() < 0.01, "PERCENTILE.INC = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn percentile_inc_0_is_min() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(5.0));
        c.set(1, 0, Value::Number(10.0));
        let r = percentile_inc(&mut c, &[rng(0, 0, 1, 0), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(5.0));
    }

    #[test]
    fn percentile_inc_1_is_max() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(5.0));
        c.set(1, 0, Value::Number(10.0));
        let r = percentile_inc(&mut c, &[rng(0, 0, 1, 0), Value::Number(1.0)]);
        assert_eq!(r, Value::Number(10.0));
    }

    #[test]
    fn percentile_inc_out_of_range_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        let r = percentile_inc(&mut c, &[rng(0, 0, 0, 0), Value::Number(1.5)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn percentile_inc_single_value() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(42.0));
        let r = percentile_inc(&mut c, &[rng(0, 0, 0, 0), Value::Number(0.5)]);
        assert_eq!(r, Value::Number(42.0));
    }

    // --- PERCENTILE.EXC ---

    #[test]
    fn percentile_exc_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        // PERCENTILE.EXC(0.25) → rank = 0.25*5 - 1 = 0.25 → interp(1,2) = 1.25
        let r = percentile_exc(&mut c, &[rng(0, 0, 3, 0), Value::Number(0.25)]);
        if let Value::Number(v) = r {
            assert!((v - 1.25).abs() < 0.01, "PERCENTILE.EXC = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn percentile_exc_0_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        let r = percentile_exc(&mut c, &[rng(0, 0, 0, 0), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- QUARTILE.INC ---

    #[test]
    fn quartile_inc_q2_is_median() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        c.set(4, 0, Value::Number(5.0));
        let r = quartile_inc(&mut c, &[rng(0, 0, 4, 0), Value::Number(2.0)]);
        assert_eq!(r, Value::Number(3.0));
    }

    #[test]
    fn quartile_inc_q0_is_min() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = quartile_inc(&mut c, &[rng(0, 0, 2, 0), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(1.0));
    }

    #[test]
    fn quartile_inc_q4_is_max() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = quartile_inc(&mut c, &[rng(0, 0, 2, 0), Value::Number(4.0)]);
        assert_eq!(r, Value::Number(3.0));
    }

    #[test]
    fn quartile_inc_invalid_q_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        let r = quartile_inc(&mut c, &[rng(0, 0, 0, 0), Value::Number(5.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- QUARTILE.EXC ---

    #[test]
    fn quartile_exc_q2_is_median() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        let r = quartile_exc(&mut c, &[rng(0, 0, 3, 0), Value::Number(2.0)]);
        if let Value::Number(v) = r {
            assert!((v - 2.5).abs() < 0.01, "QUARTILE.EXC Q2 = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn quartile_exc_invalid_q_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        let r = quartile_exc(&mut c, &[rng(0, 0, 0, 0), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- PERCENTRANK.INC ---

    #[test]
    fn percentrank_inc_basic_2() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        // PERCENTRANK.INC(3) → rank position 2/3 = 0.666
        let r = percentrank_inc(&mut c, &[rng(0, 0, 3, 0), Value::Number(3.0)]);
        if let Value::Number(v) = r {
            assert!((v - 0.666).abs() < 0.01, "PERCENTRANK.INC = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn percentrank_inc_out_of_range_is_na() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(5.0));
        let r = percentrank_inc(&mut c, &[rng(0, 0, 1, 0), Value::Number(10.0)]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    #[test]
    fn percentrank_inc_with_significance() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        let r = percentrank_inc(
            &mut c,
            &[rng(0, 0, 3, 0), Value::Number(3.0), Value::Number(2.0)],
        );
        if let Value::Number(v) = r {
            assert!((v - 0.66).abs() < 0.01, "PERCENTRANK.INC sig=2 = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- PERCENTRANK.EXC ---

    #[test]
    fn percentrank_exc_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(3, 0, Value::Number(4.0));
        let r = percentrank_exc(&mut c, &[rng(0, 0, 3, 0), Value::Number(2.5)]);
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 1.0, "PERCENTRANK.EXC = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn percentrank_exc_at_min_is_na() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(5.0));
        let r = percentrank_exc(&mut c, &[rng(0, 0, 1, 0), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- CORREL ---

    #[test]
    fn correl_perfect_positive() {
        let mut c = ctx();
        // x = [1,2,3], y = [2,4,6] → r = 1.0
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(0, 1, Value::Number(2.0));
        c.set(1, 1, Value::Number(4.0));
        c.set(2, 1, Value::Number(6.0));
        let r = correl(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        if let Value::Number(v) = r {
            assert!((v - 1.0).abs() < 1e-10, "CORREL = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn correl_mismatched_lengths_is_na() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(0, 1, Value::Number(1.0));
        let r = correl(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 0, 1)]);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    #[test]
    fn correl_constant_is_div0() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(5.0));
        c.set(1, 0, Value::Number(5.0));
        c.set(0, 1, Value::Number(1.0));
        c.set(1, 1, Value::Number(2.0));
        let r = correl(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]);
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // --- COVARIANCE.P / COVARIANCE.S ---

    #[test]
    fn covariance_p_basic_2() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(0, 1, Value::Number(4.0));
        c.set(1, 1, Value::Number(5.0));
        c.set(2, 1, Value::Number(6.0));
        let r = covariance_p(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        if let Value::Number(v) = r {
            // COV.P = 2/3
            assert!((v - 2.0 / 3.0).abs() < 1e-10, "COVARIANCE.P = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn covariance_s_basic_2() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(0, 1, Value::Number(4.0));
        c.set(1, 1, Value::Number(5.0));
        c.set(2, 1, Value::Number(6.0));
        let r = covariance_s(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        if let Value::Number(v) = r {
            // COV.S = 2/2 = 1.0
            assert!((v - 1.0).abs() < 1e-10, "COVARIANCE.S = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn covariance_s_single_is_div0() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(0, 1, Value::Number(4.0));
        let r = covariance_s(&mut c, &[rng(0, 0, 0, 0), rng(0, 1, 0, 1)]);
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // --- SLOPE / INTERCEPT ---

    #[test]
    fn slope_basic() {
        let mut c = ctx();
        // y = 2x + 1: slope should be 2
        c.set(0, 0, Value::Number(3.0));
        c.set(1, 0, Value::Number(5.0));
        c.set(2, 0, Value::Number(7.0));
        c.set(0, 1, Value::Number(1.0));
        c.set(1, 1, Value::Number(2.0));
        c.set(2, 1, Value::Number(3.0));
        let r = slope(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        if let Value::Number(v) = r {
            assert!((v - 2.0).abs() < 1e-10, "SLOPE = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn slope_constant_x_is_div0() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(0, 1, Value::Number(5.0));
        c.set(1, 1, Value::Number(5.0));
        let r = slope(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]);
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    #[test]
    fn intercept_basic() {
        let mut c = ctx();
        // y = 2x + 1: intercept should be 1
        c.set(0, 0, Value::Number(3.0));
        c.set(1, 0, Value::Number(5.0));
        c.set(2, 0, Value::Number(7.0));
        c.set(0, 1, Value::Number(1.0));
        c.set(1, 1, Value::Number(2.0));
        c.set(2, 1, Value::Number(3.0));
        let r = intercept(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        if let Value::Number(v) = r {
            assert!((v - 1.0).abs() < 1e-10, "INTERCEPT = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- RSQ ---

    #[test]
    fn rsq_basic() {
        let mut c = ctx();
        // Perfect correlation → r² = 1.0
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        c.set(0, 1, Value::Number(2.0));
        c.set(1, 1, Value::Number(4.0));
        c.set(2, 1, Value::Number(6.0));
        let r = rsq(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        if let Value::Number(v) = r {
            assert!((v - 1.0).abs() < 1e-10, "RSQ = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- FORECAST.LINEAR ---

    #[test]
    fn forecast_linear_basic_2() {
        let mut c = ctx();
        // y = 2x + 1: forecast x=4 → y=9
        c.set(0, 0, Value::Number(3.0));
        c.set(1, 0, Value::Number(5.0));
        c.set(2, 0, Value::Number(7.0));
        c.set(0, 1, Value::Number(1.0));
        c.set(1, 1, Value::Number(2.0));
        c.set(2, 1, Value::Number(3.0));
        let r = forecast_linear(
            &mut c,
            &[Value::Number(4.0), rng(0, 0, 2, 0), rng(0, 1, 2, 1)],
        );
        if let Value::Number(v) = r {
            assert!((v - 9.0).abs() < 1e-10, "FORECAST.LINEAR = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn forecast_linear_mismatched_is_na() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(0, 1, Value::Number(1.0));
        c.set(1, 1, Value::Number(2.0));
        let r = forecast_linear(
            &mut c,
            &[Value::Number(3.0), rng(0, 0, 0, 0), rng(0, 1, 1, 1)],
        );
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // --- DEVSQ ---

    #[test]
    fn devsq_basic_2() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = devsq(&mut c, &[rng(0, 0, 2, 0)]);
        if let Value::Number(v) = r {
            // mean=2, SS = (1-2)²+(2-2)²+(3-2)² = 2
            assert!((v - 2.0).abs() < 1e-10, "DEVSQ = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn devsq_empty_is_zero() {
        let mut c = ctx();
        let r = devsq(&mut c, &[]);
        assert_eq!(r, Value::Number(0.0));
    }

    // --- AVEDEV ---

    #[test]
    fn avedev_basic_2() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = avedev(&mut c, &[rng(0, 0, 2, 0)]);
        if let Value::Number(v) = r {
            // mean=2, AVEDEV = (1+0+1)/3 = 0.6667
            assert!((v - 2.0 / 3.0).abs() < 1e-10, "AVEDEV = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn avedev_empty_is_div0() {
        let mut c = ctx();
        let r = avedev(&mut c, &[]);
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // --- GEOMEAN ---

    #[test]
    fn geomean_basic_2() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(4.0));
        let r = geomean(&mut c, &[rng(0, 0, 2, 0)]);
        if let Value::Number(v) = r {
            // GEOMEAN(1,2,4) = (1*2*4)^(1/3) = 8^(1/3) ≈ 2.0
            assert!((v - 2.0).abs() < 0.01, "GEOMEAN = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn geomean_negative_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(-1.0));
        c.set(1, 0, Value::Number(2.0));
        let r = geomean(&mut c, &[rng(0, 0, 1, 0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn geomean_empty_is_num() {
        let mut c = ctx();
        let r = geomean(&mut c, &[]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- HARMEAN ---

    #[test]
    fn harmean_basic_2() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(4.0));
        let r = harmean(&mut c, &[rng(0, 0, 2, 0)]);
        if let Value::Number(v) = r {
            // HARMEAN = 3/(1+0.5+0.25) = 3/1.75 ≈ 1.7143
            assert!((v - 12.0 / 7.0).abs() < 0.01, "HARMEAN = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn harmean_negative_is_num() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(-1.0));
        let r = harmean(&mut c, &[rng(0, 0, 0, 0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn harmean_empty_is_num() {
        let mut c = ctx();
        let r = harmean(&mut c, &[]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- VARA / VARPA / STDEVA / STDEVPA ---

    #[test]
    fn vara_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = vara(&mut c, &[rng(0, 0, 2, 0)]);
        if let Value::Number(v) = r {
            assert!((v - 1.0).abs() < 1e-10, "VARA = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn varpa_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = varpa(&mut c, &[rng(0, 0, 2, 0)]);
        if let Value::Number(v) = r {
            assert!((v - 2.0 / 3.0).abs() < 1e-10, "VARPA = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn stdeva_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = stdeva(&mut c, &[rng(0, 0, 2, 0)]);
        if let Value::Number(v) = r {
            assert!((v - 1.0).abs() < 1e-10, "STDEVA = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn stdevpa_basic() {
        let mut c = ctx();
        c.set(0, 0, Value::Number(1.0));
        c.set(1, 0, Value::Number(2.0));
        c.set(2, 0, Value::Number(3.0));
        let r = stdevpa(&mut c, &[rng(0, 0, 2, 0)]);
        if let Value::Number(v) = r {
            assert!((v - (2.0 / 3.0_f64).sqrt()).abs() < 1e-10, "STDEVPA = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }
