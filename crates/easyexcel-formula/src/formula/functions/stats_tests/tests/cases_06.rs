    // --- 更多统计函数测试（覆盖 trimmean_to_beta_pdf.rs 和 beta_dist_to_binom_dist_range.rs 未测分支） ---

    // trimmean: 非数字参数
    #[test]
    fn trimmean_err_text() {
        let mut c = ctx();
        let r = trimmean(
            &mut c,
            &[Value::Text("abc".into()), Value::Number(0.2)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // skew: 非数字参数
    #[test]
    fn skew_err_text() {
        let mut c = ctx();
        let r = skew(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // kurt: 非数字参数
    #[test]
    fn kurt_err_text() {
        let mut c = ctx();
        let r = kurt(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // standardize: 非数字参数
    #[test]
    fn standardize_err_text() {
        let mut c = ctx();
        let r = standardize(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(50.0),
                Value::Number(10.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // standardize: 零标准差 → #NUM!
    #[test]
    fn standardize_zero_stdev_is_num() {
        let mut c = ctx();
        let r = standardize(
            &mut c,
            &[Value::Number(50.0), Value::Number(50.0), Value::Number(0.0)],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // fisher: 非数字参数
    #[test]
    fn fisher_err_text() {
        let mut c = ctx();
        let r = fisher(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // fisher: x >= 1 → #NUM!
    #[test]
    fn fisher_x_gte_1_is_num() {
        let mut c = ctx();
        let r = fisher(&mut c, &[Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // fisher: x <= -1 → #NUM!
    #[test]
    fn fisher_x_lte_neg1_is_num() {
        let mut c = ctx();
        let r = fisher(&mut c, &[Value::Number(-1.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // fisherinv: 非数字参数
    #[test]
    fn fisherinv_err_text() {
        let mut c = ctx();
        let r = fisherinv(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // norm_dist: 基本 CDF
    #[test]
    fn norm_dist_cdf_basic() {
        let mut c = ctx();
        let r = norm_dist(
            &mut c,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Bool(true),
            ],
        );
        approx(r, 0.5, 1e-6, "NORM.DIST CDF(0)");
    }

    // norm_dist: PDF
    #[test]
    fn norm_dist_pdf_basic() {
        let mut c = ctx();
        let r = norm_dist(
            &mut c,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Bool(false),
            ],
        );
        approx(r, 0.3989, 1e-3, "NORM.DIST PDF(0)");
    }

    // norm_dist: 非数字参数
    #[test]
    fn norm_dist_err_text() {
        let mut c = ctx();
        let r = norm_dist(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // norm_dist: 零标准差 → #NUM!
    #[test]
    fn norm_dist_zero_stdev_is_num() {
        let mut c = ctx();
        let r = norm_dist(
            &mut c,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // norm_s_dist: CDF
    #[test]
    fn norm_s_dist_cdf() {
        let mut c = ctx();
        let r = norm_s_dist(&mut c, &[Value::Number(0.0), Value::Bool(true)]);
        approx(r, 0.5, 1e-6, "NORM.S.DIST CDF(0)");
    }

    // norm_s_dist: PDF
    #[test]
    fn norm_s_dist_pdf() {
        let mut c = ctx();
        let r = norm_s_dist(&mut c, &[Value::Number(0.0), Value::Bool(false)]);
        approx(r, 0.3989, 1e-3, "NORM.S.DIST PDF(0)");
    }

    // norm_s_dist: 非数字参数
    #[test]
    fn norm_s_dist_err_text() {
        let mut c = ctx();
        let r = norm_s_dist(
            &mut c,
            &[Value::Text("abc".into()), Value::Bool(true)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // norm_inv: 基本测试
    #[test]
    fn norm_inv_basic() {
        let mut c = ctx();
        let r = norm_inv(
            &mut c,
            &[
                Value::Number(0.5),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        );
        approx(r, 0.0, 1e-6, "NORM.INV(0.5)");
    }

    // norm_inv: 非数字参数
    #[test]
    fn norm_inv_err_text() {
        let mut c = ctx();
        let r = norm_inv(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // norm_inv: 零标准差 → #NUM!
    #[test]
    fn norm_inv_zero_stdev_is_num() {
        let mut c = ctx();
        let r = norm_inv(
            &mut c,
            &[Value::Number(0.5), Value::Number(0.0), Value::Number(0.0)],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // norm_s_inv: 基本测试
    #[test]
    fn norm_s_inv_basic() {
        let mut c = ctx();
        let r = norm_s_inv(&mut c, &[Value::Number(0.5)]);
        approx(r, 0.0, 1e-6, "NORM.S.INV(0.5)");
    }

    // norm_s_inv: 非数字参数
    #[test]
    fn norm_s_inv_err_text() {
        let mut c = ctx();
        let r = norm_s_inv(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // binom_dist: 非数字参数
    #[test]
    fn binom_dist_err_text() {
        let mut c = ctx();
        let r = binom_dist(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(10.0),
                Value::Number(0.5),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // poisson_dist: 非数字参数
    #[test]
    fn poisson_dist_err_text() {
        let mut c = ctx();
        let r = poisson_dist(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(5.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // expon_dist: 非数字参数
    #[test]
    fn expon_dist_err_text() {
        let mut c = ctx();
        let r = expon_dist(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(0.5),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // confidence_norm: 非数字参数
    #[test]
    fn confidence_norm_err_text() {
        let mut c = ctx();
        let r = confidence_norm(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(10.0),
                Value::Number(50.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // gauss: 非数字参数
    #[test]
    fn gauss_err_text() {
        let mut c = ctx();
        let r = gauss(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // phi: 非数字参数
    #[test]
    fn phi_err_text() {
        let mut c = ctx();
        let r = phi(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // gamma_dist: 非数字参数
    #[test]
    fn gamma_dist_err_text() {
        let mut c = ctx();
        let r = gamma_dist(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(2.0),
                Value::Number(1.5),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // gamma_inv: 非数字参数
    #[test]
    fn gamma_inv_err_text() {
        let mut c = ctx();
        let r = gamma_inv(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(2.0),
                Value::Number(1.5),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // beta_dist: 非数字参数
    #[test]
    fn beta_dist_err_text() {
        let mut c = ctx();
        let r = beta_dist(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // beta_inv: 非数字参数
    #[test]
    fn beta_inv_err_text() {
        let mut c = ctx();
        let r = beta_inv(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(2.0),
                Value::Number(3.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // binom_dist_range: 非数字参数
    #[test]
    fn binom_dist_range_err_text() {
        let mut c = ctx();
        let r = binom_dist_range(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(0.5),
                Value::Number(0.0),
                Value::Number(5.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // norm_dist: mean 分支
    #[test]
    fn norm_dist_nonzero_mean() {
        let mut c = ctx();
        let r = norm_dist(
            &mut c,
            &[
                Value::Number(100.0),
                Value::Number(100.0),
                Value::Number(15.0),
                Value::Bool(true),
            ],
        );
        approx(r, 0.5, 1e-6, "NORM.DIST(mean)");
    }

    // gamma_dist: 基本 CDF
    #[test]
    fn gamma_dist_cdf_basic() {
        let mut c = ctx();
        let r = gamma_dist(
            &mut c,
            &[
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(1.0),
                Value::Bool(true),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 1.0, "GAMMA.DIST CDF = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // gamma_dist: PDF
    #[test]
    fn gamma_dist_pdf_basic() {
        let mut c = ctx();
        let r = gamma_dist(
            &mut c,
            &[
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(1.0),
                Value::Bool(false),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v >= 0.0, "GAMMA.DIST PDF = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // gamma_dist: x < 0 → #NUM!
    #[test]
    fn gamma_dist_negative_x() {
        let mut c = ctx();
        let r = gamma_dist(
            &mut c,
            &[
                Value::Number(-1.0),
                Value::Number(3.0),
                Value::Number(1.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // beta_dist: 基本测试
    #[test]
    fn beta_dist_basic_cdf() {
        let mut c = ctx();
        let r = beta_dist(
            &mut c,
            &[
                Value::Number(0.5),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Bool(true),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 1.0, "BETA.DIST CDF = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // beta_dist: PDF
    #[test]
    fn beta_dist_pdf_basic() {
        let mut c = ctx();
        let r = beta_dist(
            &mut c,
            &[
                Value::Number(0.5),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Bool(false),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v >= 0.0, "BETA.DIST PDF = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // beta_dist: x < 0 → #NUM!
    #[test]
    fn beta_dist_x_lt_0() {
        let mut c = ctx();
        let r = beta_dist(
            &mut c,
            &[
                Value::Number(-0.1),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // beta_dist: x > 1 → #NUM!
    #[test]
    fn beta_dist_x_gt_1() {
        let mut c = ctx();
        let r = beta_dist(
            &mut c,
            &[
                Value::Number(1.1),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // binom_dist_range: 基本测试
    #[test]
    fn binom_dist_range_basic() {
        let mut c = ctx();
        let r = binom_dist_range(
            &mut c,
            &[
                Value::Number(10.0),
                Value::Number(0.5),
                Value::Number(3.0),
                Value::Number(7.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 1.0, "BINOM.DIST.RANGE = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // binom_inv: 基本测试
    #[test]
    fn binom_inv_basic() {
        let mut c = ctx();
        let r = binom_inv(
            &mut c,
            &[
                Value::Number(10.0),
                Value::Number(0.5),
                Value::Number(0.5),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v >= 0.0 && v <= 10.0, "BINOM.INV = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // binom_inv: 非数字参数
    #[test]
    fn binom_inv_err_text() {
        let mut c = ctx();
        let r = binom_inv(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(0.5),
                Value::Number(0.5),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }
