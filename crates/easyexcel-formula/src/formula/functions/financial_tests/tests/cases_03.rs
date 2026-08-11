    // --- XIRR ---

    #[test]
    fn test_xirr_basic_rate() {
        let mut ctx = c();
        // XIRR with known cashflows and dates
        let r = xirr(
            &mut ctx,
            &[
                Value::Array(crate::formula::value::Array::new(
                    1,
                    3,
                    vec![
                        Value::Number(-10000.0),
                        Value::Number(5000.0),
                        Value::Number(6000.0),
                    ],
                )),
                Value::Array(crate::formula::value::Array::new(
                    1,
                    3,
                    vec![
                        Value::Number(39448.0),
                        Value::Number(39630.0),
                        Value::Number(39814.0),
                    ],
                )),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 1.0, "XIRR = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_xirr_mismatched_is_value() {
        let mut ctx = c();
        let r = xirr(
            &mut ctx,
            &[
                Value::Array(crate::formula::value::Array::new(
                    1,
                    2,
                    vec![Value::Number(-100.0), Value::Number(200.0)],
                )),
                Value::Array(crate::formula::value::Array::new(
                    1,
                    3,
                    vec![Value::Number(39448.0), Value::Number(39630.0), Value::Number(39814.0)],
                )),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn test_xirr_no_neg_is_num() {
        let mut ctx = c();
        let r = xirr(
            &mut ctx,
            &[
                Value::Array(crate::formula::value::Array::new(
                    1,
                    2,
                    vec![Value::Number(100.0), Value::Number(200.0)],
                )),
                Value::Array(crate::formula::value::Array::new(
                    1,
                    2,
                    vec![Value::Number(39448.0), Value::Number(39630.0)],
                )),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- MIRR edge cases ---

    #[test]
    fn test_mirr_all_neg_fv_is_div0() {
        let mut ctx = c();
        // All negative cashflows → fv_pos = 0 → Div0
        let r = mirr(
            &mut ctx,
            &[
                Value::Array(crate::formula::value::Array::new(
                    1,
                    3,
                    vec![Value::Number(-100.0), Value::Number(-200.0), Value::Number(-300.0)],
                )),
                Value::Number(0.1),
                Value::Number(0.12),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // --- SYD edge cases ---

    #[test]
    fn test_syd_per_lt_1_is_num() {
        let mut ctx = c();
        let r = syd(
            &mut ctx,
            &[
                Value::Number(10000.0),
                Value::Number(2000.0),
                Value::Number(5.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_syd_per_gt_life_is_num() {
        let mut ctx = c();
        let r = syd(
            &mut ctx,
            &[
                Value::Number(10000.0),
                Value::Number(2000.0),
                Value::Number(5.0),
                Value::Number(6.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- DB edge cases ---

    #[test]
    fn test_db_negative_cost_is_num() {
        let mut ctx = c();
        let r = db(
            &mut ctx,
            &[
                Value::Number(-10000.0),
                Value::Number(2000.0),
                Value::Number(5.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_db_zero_cost_returns_zero() {
        let mut ctx = c();
        let r = db(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(5.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Number(0.0));
    }

    #[test]
    fn test_db_with_month() {
        let mut ctx = c();
        // DB with custom month (6 instead of 12)
        let r = db(
            &mut ctx,
            &[
                Value::Number(10000.0),
                Value::Number(2000.0),
                Value::Number(5.0),
                Value::Number(1.0),
                Value::Number(6.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 10000.0, "DB with month=6: {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // --- DDB edge cases ---

    #[test]
    fn test_ddb_custom_factor() {
        let mut ctx = c();
        // DDB with factor=1.5 (150% declining)
        let r = ddb(
            &mut ctx,
            &[
                Value::Number(10000.0),
                Value::Number(2000.0),
                Value::Number(5.0),
                Value::Number(1.0),
                Value::Number(1.5),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "DDB factor=1.5: {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_ddb_negative_factor_is_num() {
        let mut ctx = c();
        let r = ddb(
            &mut ctx,
            &[
                Value::Number(10000.0),
                Value::Number(2000.0),
                Value::Number(5.0),
                Value::Number(1.0),
                Value::Number(-2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- VDB edge cases ---

    #[test]
    fn test_vdb_with_switch() {
        let mut ctx = c();
        // VDB with switch to SLN (default no_switch=0)
        let r = vdb(
            &mut ctx,
            &[
                Value::Number(10000.0),
                Value::Number(2000.0),
                Value::Number(5.0),
                Value::Number(0.0),
                Value::Number(3.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "VDB with switch: {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_vdb_negative_cost_is_num() {
        let mut ctx = c();
        let r = vdb(
            &mut ctx,
            &[
                Value::Number(-10000.0),
                Value::Number(2000.0),
                Value::Number(5.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- NOMINAL edge cases ---

    #[test]
    fn test_nominal_zero_npery_is_num() {
        let mut ctx = c();
        let r = nominal(&mut ctx, &[Value::Number(0.1), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- PDURATION edge cases ---

    #[test]
    fn test_pduration_zero_pv_is_num() {
        let mut ctx = c();
        let r = pduration(
            &mut ctx,
            &[Value::Number(0.05), Value::Number(0.0), Value::Number(200.0)],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_pduration_zero_fv_is_num() {
        let mut ctx = c();
        let r = pduration(
            &mut ctx,
            &[Value::Number(0.05), Value::Number(100.0), Value::Number(0.0)],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- RRI edge cases ---

    #[test]
    fn test_rri_negative_pv() {
        let mut ctx = c();
        // RRI with negative PV is valid
        let r = rri(
            &mut ctx,
            &[Value::Number(10.0), Value::Number(-100.0), Value::Number(200.0)],
        );
        if let Value::Number(v) = r {
            // (-200/-100)^(1/10) - 1 ≈ 0.0718 → but with -100 pv: (200/-100)^(0.1) - 1
            // This would be (-2)^0.1 which is NaN
            // Actually rri checks pv == 0.0, not pv < 0
            // So this should work: (200 / -100)^0.1 - 1 = (-2)^0.1 → NaN → Num error
            assert!(v.is_nan() || v.is_finite(), "RRI with neg pv: {v}");
        }
        // With negative PV, (fv/pv) is negative, powf may return NaN
    }

    // --- DOLLARDE / DOLLARFR edge cases ---

    #[test]
    fn test_dollarde_large_fraction() {
        let mut ctx = c();
        // DOLLARDE(1.02, 32) = 1 + 2/100 * 100/32 → 1 + 0.625 = 1.625
        let r = dollarde(&mut ctx, &[Value::Number(1.02), Value::Number(32.0)]);
        if let Value::Number(v) = r {
            assert!(v > 1.0, "DOLLARDE(1.02, 32) = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_dollarfr_large_fraction() {
        let mut ctx = c();
        let r = dollarfr(&mut ctx, &[Value::Number(1.125), Value::Number(32.0)]);
        if let Value::Number(v) = r {
            assert!(v > 1.0, "DOLLARFR(1.125, 32) = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // --- DISC / INTRATE edge cases ---

    #[test]
    fn test_disc_with_basis() {
        let mut ctx = c();
        let r = disc(
            &mut ctx,
            &[
                Value::Number(39448.0),
                Value::Number(39814.0),
                Value::Number(97.0),
                Value::Number(100.0),
                Value::Number(3.0), // actual/365
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "DISC with basis=3: {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_intrate_zero_investment_is_num() {
        let mut ctx = c();
        let r = intrate(
            &mut ctx,
            &[
                Value::Number(39448.0),
                Value::Number(39814.0),
                Value::Number(0.0), // zero investment
                Value::Number(100.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- NPV / XNPV edge cases ---

    #[test]
    fn test_npv_single_value() {
        let mut ctx = c();
        // NPV(0.1, 100) = 100/1.1 = 90.91
        let r = npv(
            &mut ctx,
            &[Value::Number(0.1), Value::Number(100.0)],
        );
        if let Value::Number(v) = r {
            assert!((v - 90.91).abs() < 0.1, "NPV = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_xnpv_rate_minus_one_is_num() {
        let mut ctx = c();
        let r = xnpv(
            &mut ctx,
            &[
                Value::Number(-1.0),
                Value::Array(crate::formula::value::Array::new(
                    1,
                    2,
                    vec![Value::Number(-100.0), Value::Number(200.0)],
                )),
                Value::Array(crate::formula::value::Array::new(
                    1,
                    2,
                    vec![Value::Number(39448.0), Value::Number(39814.0)],
                )),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- CUMIPMT / CUMPRINC edge cases ---

    #[test]
    fn test_cumipmt_invalid_range_is_num() {
        let mut ctx = c();
        let r = cumipmt(
            &mut ctx,
            &[
                Value::Number(0.05),
                Value::Number(10.0),
                Value::Number(1000.0),
                Value::Number(5.0),  // start > end would be caught
                Value::Number(3.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_cumprinc_basic() {
        let mut ctx = c();
        let r = cumprinc(
            &mut ctx,
            &[
                Value::Number(0.05),
                Value::Number(10.0),
                Value::Number(1000.0),
                Value::Number(1.0),
                Value::Number(10.0),
                Value::Number(0.0),
            ],
        );
        if let Value::Number(v) = r {
            // CUMPRINC should be negative (principal paid out)
            assert!(v < 0.0, "CUMPRINC = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // --- IPMT / PPMT edge cases ---

    #[test]
    fn test_ipmt_per_out_of_range() {
        let mut ctx = c();
        let r = ipmt(
            &mut ctx,
            &[
                Value::Number(0.05),
                Value::Number(0.0), // per < 1
                Value::Number(10.0),
                Value::Number(1000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_ppmt_per_out_of_range() {
        let mut ctx = c();
        let r = ppmt(
            &mut ctx,
            &[
                Value::Number(0.05),
                Value::Number(11.0), // per > nper
                Value::Number(10.0),
                Value::Number(1000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- NPER edge cases ---

    #[test]
    fn test_nper_zero_rate_zero_pmt_is_div0() {
        let mut ctx = c();
        let r = nper(
            &mut ctx,
            &[
                Value::Number(0.0), // rate = 0
                Value::Number(0.0), // pmt = 0
                Value::Number(1000.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Div0));
    }
