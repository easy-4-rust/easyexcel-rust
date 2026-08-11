    // --- 更多财务函数测试（覆盖 register_to_xnpv.rs 和 irr_to_intrate.rs 未测分支） ---

    // PMT: 测试带 type=1 的情况
    #[test]
    fn test_pmt_type_begin() {
        let mut ctx = c();
        // PMT(0.08/12, 10, 10000, 0, 1)
        let r = pmt(
            &mut ctx,
            &[
                Value::Number(0.08 / 12.0),
                Value::Number(10.0),
                Value::Number(10000.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v < -1030.0 && v > -1040.0, "PMT type=1 = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // PMT: 非数字参数报错
    #[test]
    fn test_pmt_err_text() {
        let mut ctx = c();
        let r = pmt(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(10.0),
                Value::Number(10000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // FV: 非数字参数报错
    #[test]
    fn test_fv_err_text() {
        let mut ctx = c();
        let r = fv(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(10.0),
                Value::Number(-200.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // PV: 带 fv 和 type 的情况
    #[test]
    fn test_pv_fv_and_type() {
        let mut ctx = c();
        let r = pv(
            &mut ctx,
            &[
                Value::Number(0.08 / 12.0),
                Value::Number(20.0),
                Value::Number(500.0),
                Value::Number(10000.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v < 0.0, "PV with fv+type should be negative: {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // PV: 零利率
    #[test]
    fn test_pv_zero_rate() {
        let mut ctx = c();
        let r = pv(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(10.0),
                Value::Number(-100.0),
            ],
        );
        assert_eq!(r, Value::Number(1000.0));
    }

    // PV: 非数字参数报错
    #[test]
    fn test_pv_err_text() {
        let mut ctx = c();
        let r = pv(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(10.0),
                Value::Number(500.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // NPER: 非零利率，分母为零 → #DIV/0!
    #[test]
    fn test_nper_denom_zero() {
        let mut ctx = c();
        // pv=0, fv=0, pmt=0 → denom = 0
        let r = nper(
            &mut ctx,
            &[
                Value::Number(0.1),
                Value::Number(0.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // NPER: 非数字参数报错
    #[test]
    fn test_nper_err_text() {
        let mut ctx = c();
        let r = nper(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(100.0),
                Value::Number(-1000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // NPER: 带 fv 和 type
    #[test]
    fn test_nper_with_fv_type() {
        let mut ctx = c();
        let r = nper(
            &mut ctx,
            &[
                Value::Number(0.1),
                Value::Number(-100.0),
                Value::Number(0.0),
                Value::Number(10000.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "NPER should be positive: {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // RATE: 基本收敛测试
    #[test]
    fn test_rate_converges() {
        let mut ctx = c();
        let r = rate(
            &mut ctx,
            &[
                Value::Number(10.0),
                Value::Number(-100.0),
                Value::Number(1000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v.abs() < 0.1, "RATE should be near 0: {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // RATE: 非数字参数报错
    #[test]
    fn test_rate_err_text() {
        let mut ctx = c();
        let r = rate(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(-100.0),
                Value::Number(1000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // RATE: 自定义 guess
    #[test]
    fn test_rate_custom_guess() {
        let mut ctx = c();
        let r = rate(
            &mut ctx,
            &[
                Value::Number(10.0),
                Value::Number(-100.0),
                Value::Number(1000.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(0.05),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v.abs() < 0.1, "RATE custom guess = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // IPMT: period=1, type=1 → 利息为 0
    #[test]
    fn test_ipmt_type1_period1_zero() {
        let mut ctx = c();
        let r = ipmt(
            &mut ctx,
            &[
                Value::Number(0.1 / 12.0),
                Value::Number(1.0),
                Value::Number(24.0),
                Value::Number(10000.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Number(0.0));
    }

    // IPMT: period 超出范围 → #NUM!
    #[test]
    fn test_ipmt_period_zero_is_num() {
        let mut ctx = c();
        let r = ipmt(
            &mut ctx,
            &[
                Value::Number(0.1 / 12.0),
                Value::Number(0.0),
                Value::Number(24.0),
                Value::Number(10000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // IPMT: period > nper → #NUM!
    #[test]
    fn test_ipmt_period_too_large_is_num() {
        let mut ctx = c();
        let r = ipmt(
            &mut ctx,
            &[
                Value::Number(0.1 / 12.0),
                Value::Number(25.0),
                Value::Number(24.0),
                Value::Number(10000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // IPMT: 零利率情况
    #[test]
    fn test_ipmt_zero_rate() {
        let mut ctx = c();
        let r = ipmt(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(10.0),
                Value::Number(1000.0),
            ],
        );
        assert_eq!(r, Value::Number(0.0));
    }

    // IPMT: 非数字参数
    #[test]
    fn test_ipmt_err_text() {
        let mut ctx = c();
        let r = ipmt(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1.0),
                Value::Number(24.0),
                Value::Number(10000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // PPMT: period 超出范围 → #NUM!
    #[test]
    fn test_ppmt_period_out_range() {
        let mut ctx = c();
        let r = ppmt(
            &mut ctx,
            &[
                Value::Number(0.1 / 12.0),
                Value::Number(0.0),
                Value::Number(24.0),
                Value::Number(10000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // PPMT: 零利率
    #[test]
    fn test_ppmt_zero_rate_v2() {
        let mut ctx = c();
        let r = ppmt(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(10.0),
                Value::Number(1000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v < 0.0, "PPMT zero rate = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // PPMT: 非数字参数
    #[test]
    fn test_ppmt_err_text() {
        let mut ctx = c();
        let r = ppmt(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1.0),
                Value::Number(24.0),
                Value::Number(10000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // CUMIPMT: 非数字参数
    #[test]
    fn test_cumipmt_err_text() {
        let mut ctx = c();
        let r = cumipmt(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(360.0),
                Value::Number(125000.0),
                Value::Number(1.0),
                Value::Number(360.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // CUMPRINC: 非数字参数
    #[test]
    fn test_cumprinc_err_text() {
        let mut ctx = c();
        let r = cumprinc(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(360.0),
                Value::Number(125000.0),
                Value::Number(1.0),
                Value::Number(360.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // NPV: 非数字参数
    #[test]
    fn test_npv_err_text() {
        let mut ctx = c();
        let r = npv(
            &mut ctx,
            &[Value::Text("abc".into()), Value::Number(100.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // XNPV: values 和 dates 长度不同 → #VALUE!
    #[test]
    fn test_xnpv_mismatched_v2() {
        let mut ctx = c();
        let values = Value::Array(crate::formula::value::Array::new(
            1,
            2,
            vec![Value::Number(-100.0), Value::Number(200.0)],
        ));
        let dates = Value::Array(crate::formula::value::Array::new(
            1,
            3,
            vec![
                Value::Number(39448.0),
                Value::Number(39629.0),
                Value::Number(39813.0),
            ],
        ));
        let r = xnpv(&mut ctx, &[Value::Number(0.05), values, dates]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // XNPV: 空数组 → #VALUE!
    #[test]
    fn test_xnpv_empty_v2() {
        let mut ctx = c();
        let values = Value::Array(crate::formula::value::Array::new(1, 0, vec![]));
        let dates = Value::Array(crate::formula::value::Array::new(1, 0, vec![]));
        let r = xnpv(&mut ctx, &[Value::Number(0.05), values, dates]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // IRR: 自定义 guess
    #[test]
    fn test_irr_custom_guess() {
        let mut ctx = c();
        let cashflows = Value::Array(crate::formula::value::Array::new(
            1,
            5,
            vec![
                Value::Number(-10000.0),
                Value::Number(3000.0),
                Value::Number(4000.0),
                Value::Number(5000.0),
                Value::Number(1000.0),
            ],
        ));
        let r = irr(&mut ctx, &[cashflows, Value::Number(0.1)]);
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 1.0, "IRR = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // IRR: 空现金流 → #NUM!
    #[test]
    fn test_irr_empty_is_num() {
        let mut ctx = c();
        let cashflows = Value::Array(crate::formula::value::Array::new(1, 0, vec![]));
        let r = irr(&mut ctx, &[cashflows]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // IRR: 全正现金流 → #NUM!
    #[test]
    fn test_irr_all_positive_is_num() {
        let mut ctx = c();
        let cashflows = Value::Array(crate::formula::value::Array::new(
            1,
            3,
            vec![
                Value::Number(100.0),
                Value::Number(200.0),
                Value::Number(300.0),
            ],
        ));
        let r = irr(&mut ctx, &[cashflows]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // IRR: 全负现金流 → #NUM!
    #[test]
    fn test_irr_all_negative_is_num() {
        let mut ctx = c();
        let cashflows = Value::Array(crate::formula::value::Array::new(
            1,
            3,
            vec![
                Value::Number(-100.0),
                Value::Number(-200.0),
                Value::Number(-300.0),
            ],
        ));
        let r = irr(&mut ctx, &[cashflows]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // IRR: 非数字参数
    #[test]
    fn test_irr_err_text() {
        let mut ctx = c();
        let r = irr(&mut ctx, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // XIRR: 非数字参数
    #[test]
    fn test_xirr_err_text() {
        let mut ctx = c();
        let r = xirr(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Array(crate::formula::value::Array::new(
                    1,
                    2,
                    vec![Value::Number(39448.0), Value::Number(39813.0)],
                )),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // MIRR: 非数字参数
    #[test]
    fn test_mirr_err_text() {
        let mut ctx = c();
        let r = mirr(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(0.1),
                Value::Number(0.12),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // SLN: 非数字参数
    #[test]
    fn test_sln_err_text() {
        let mut ctx = c();
        let r = sln(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1000.0),
                Value::Number(10.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // SYD: 非数字参数
    #[test]
    fn test_syd_err_text() {
        let mut ctx = c();
        let r = syd(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1000.0),
                Value::Number(10.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // DB: period > life + 1 → #NUM!
    #[test]
    fn test_db_period_too_large_is_num() {
        let mut ctx = c();
        let r = db(
            &mut ctx,
            &[
                Value::Number(1000000.0),
                Value::Number(100000.0),
                Value::Number(6.0),
                Value::Number(8.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // DB: 零 cost → 0
    #[test]
    fn test_db_zero_cost_is_zero() {
        let mut ctx = c();
        let r = db(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(6.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Number(0.0));
    }

    // DB: 非数字参数
    #[test]
    fn test_db_err_text() {
        let mut ctx = c();
        let r = db(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1000.0),
                Value::Number(10.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // DDB: 非数字参数
    #[test]
    fn test_ddb_err_text() {
        let mut ctx = c();
        let r = ddb(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1000.0),
                Value::Number(10.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // VDB: 非数字参数
    #[test]
    fn test_vdb_err_text() {
        let mut ctx = c();
        let r = vdb(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1000.0),
                Value::Number(10.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // EFFECT: 非数字参数
    #[test]
    fn test_effect_err_text() {
        let mut ctx = c();
        let r = effect(
            &mut ctx,
            &[Value::Text("abc".into()), Value::Number(4.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // NOMINAL: 非数字参数
    #[test]
    fn test_nominal_err_text() {
        let mut ctx = c();
        let r = nominal(
            &mut ctx,
            &[Value::Text("abc".into()), Value::Number(4.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // FVSCHEDULE: 非数字参数
    #[test]
    fn test_fvschedule_err_text() {
        let mut ctx = c();
        let schedule = Value::Array(crate::formula::value::Array::new(
            1,
            3,
            vec![
                Value::Number(0.05),
                Value::Number(0.06),
                Value::Number(0.07),
            ],
        ));
        let r = fvschedule(&mut ctx, &[Value::Text("abc".into()), schedule]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // PDURATION: 非数字参数
    #[test]
    fn test_pduration_err_text() {
        let mut ctx = c();
        let r = pduration(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1000.0),
                Value::Number(2000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // RRI: 非数字参数
    #[test]
    fn test_rri_err_text() {
        let mut ctx = c();
        let r = rri(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1000.0),
                Value::Number(2000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // RRI: pv == 0 → #NUM!
    #[test]
    fn test_rri_zero_pv_is_num() {
        let mut ctx = c();
        let r = rri(
            &mut ctx,
            &[
                Value::Number(10.0),
                Value::Number(0.0),
                Value::Number(2000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // DOLLARDE: 非数字参数
    #[test]
    fn test_dollarde_err_text() {
        let mut ctx = c();
        let r = dollarde(
            &mut ctx,
            &[Value::Text("abc".into()), Value::Number(8.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // DOLLARFR: 非数字参数
    #[test]
    fn test_dollarfr_err_text() {
        let mut ctx = c();
        let r = dollarfr(
            &mut ctx,
            &[Value::Text("abc".into()), Value::Number(8.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // ISPMT: 非数字参数
    #[test]
    fn test_ispmt_err_text() {
        let mut ctx = c();
        let r = ispmt(
            &mut ctx,
            &[
                Value::Text("abc".into()),
                Value::Number(1.0),
                Value::Number(5.0),
                Value::Number(10000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // DISC: 非数字参数
    #[test]
    fn test_disc_err_text() {
        let mut ctx = c();
        let r = disc(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 6, 1),
                Value::Text("abc".into()),
                Value::Number(100.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // INTRATE: 基本测试
    #[test]
    fn test_intrate_basic_v2() {
        let mut ctx = c();
        let r = intrate(
            &mut ctx,
            &[
                ds(2008, 2, 15),
                ds(2008, 5, 15),
                Value::Number(1000000.0),
                Value::Number(1014420.0),
                Value::Number(2.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 0.2, "INTRATE = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // INTRATE: maturity <= settlement → #NUM!
    #[test]
    fn test_intrate_maturity_le_settlement() {
        let mut ctx = c();
        let r = intrate(
            &mut ctx,
            &[
                ds(2020, 6, 1),
                ds(2020, 1, 1),
                Value::Number(1000000.0),
                Value::Number(1014420.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // INTRATE: 非数字参数
    #[test]
    fn test_intrate_err_text() {
        let mut ctx = c();
        let r = intrate(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 6, 1),
                Value::Text("abc".into()),
                Value::Number(100.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // ACCRINT: 基本测试
    #[test]
    fn test_accrint_basic_v2() {
        let mut ctx = c();
        let r = accrint(
            &mut ctx,
            &[
                ds(2008, 3, 1),
                ds(2008, 8, 31),
                ds(2008, 5, 1),
                Value::Number(0.1),
                Value::Number(1000.0),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "ACCRINT = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // ACCRINTM: 基本测试
    #[test]
    fn test_accrintm_basic_v2() {
        let mut ctx = c();
        let r = accrintm(
            &mut ctx,
            &[
                ds(2008, 1, 1),
                ds(2008, 6, 30),
                Value::Number(0.1),
                Value::Number(1000.0),
                Value::Number(3.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "ACCRINTM = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // YIELDMAT: 基本测试
    #[test]
    fn test_yieldmat_basic_v2() {
        let mut ctx = c();
        let r = yieldmat(
            &mut ctx,
            &[
                ds(2008, 3, 15),
                ds(2008, 11, 3),
                ds(2007, 11, 8),
                Value::Number(0.0625),
                Value::Number(100.01),
                Value::Number(0.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 0.2, "YIELDMAT = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // YIELDDISC: 基本测试
    #[test]
    fn test_yielddisc_basic_v2() {
        let mut ctx = c();
        let r = yielddisc(
            &mut ctx,
            &[
                ds(2008, 2, 16),
                ds(2008, 3, 1),
                Value::Number(99.795),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 0.2, "YIELDDISC = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // collect_cashflows: 含布尔值
    #[test]
    fn test_npv_bool_values() {
        let mut ctx = c();
        let r = npv(
            &mut ctx,
            &[
                Value::Number(0.1),
                Value::Bool(true),
                Value::Bool(false),
                Value::Number(100.0),
            ],
        );
        if let Value::Number(v) = r {
            let expected = 1.0 / 1.1 + 0.0 / 1.21 + 100.0 / 1.331;
            assert!((v - expected).abs() < 0.01, "NPV with bools = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // collect_cashflows: 含 Empty
    #[test]
    fn test_npv_empty_values() {
        let mut ctx = c();
        let r = npv(
            &mut ctx,
            &[Value::Number(0.1), Value::Empty, Value::Number(100.0)],
        );
        if let Value::Number(v) = r {
            let expected = 0.0 / 1.1 + 100.0 / 1.21;
            assert!((v - expected).abs() < 0.01, "NPV with empty = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // collect_cashflows: 含数字文本
    #[test]
    fn test_npv_numeric_text() {
        let mut ctx = c();
        let r = npv(
            &mut ctx,
            &[Value::Number(0.1), Value::Text("50".into()), Value::Number(100.0)],
        );
        if let Value::Number(v) = r {
            let expected = 50.0 / 1.1 + 100.0 / 1.21;
            assert!((v - expected).abs() < 0.01, "NPV with text = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // collect_cashflows: 含不可解析文本 → #VALUE!
    #[test]
    fn test_npv_bad_text_is_value() {
        let mut ctx = c();
        let r = npv(
            &mut ctx,
            &[Value::Number(0.1), Value::Text("abc".into())],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // collect_cashflows: 含错误值
    #[test]
    fn test_npv_error_propagates() {
        let mut ctx = c();
        let r = npv(
            &mut ctx,
            &[Value::Number(0.1), Value::Error(CellError::NA)],
        );
        assert_eq!(r, Value::Error(CellError::NA));
    }

    // pv_factor: type=1 分支
    #[test]
    fn test_pmt_type1_nonzero_rate_v2() {
        let mut ctx = c();
        let r = pmt(
            &mut ctx,
            &[
                Value::Number(0.1),
                Value::Number(5.0),
                Value::Number(1000.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            let r0 = n(&pmt(
                &mut ctx,
                &[
                    Value::Number(0.1),
                    Value::Number(5.0),
                    Value::Number(1000.0),
                ],
            ));
            assert!(v.abs() < r0.abs(), "PMT type=1 ({v}) < type=0 ({r0})");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // solve_newton: 不收敛情况
    #[test]
    fn test_rate_may_not_converge() {
        let mut ctx = c();
        let r = rate(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(100.0),
            ],
        );
        match r {
            Value::Number(_) | Value::Error(CellError::Num) => {}
            other => panic!("Unexpected result: {other:?}"),
        }
    }
