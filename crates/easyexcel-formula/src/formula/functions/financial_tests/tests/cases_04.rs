    // --- RECEIVED ---
    // RECEIVED(settlement, maturity, investment, discount, [basis])

    #[test]
    fn test_received_basic() {
        let mut ctx = c();
        // RECEIVED(2008-2-15, 2008-9-30, 1000000, 0.0575, 2)
        // Excel: maturity > settlement, investment > 0, discount > 0
        let r = received(
            &mut ctx,
            &[
                ds(2008, 2, 15),
                ds(2008, 9, 30),
                Value::Number(1_000_000.0),
                Value::Number(0.0575),
                Value::Number(2.0),
            ],
        );
        if let Value::Number(v) = r {
            // actual/360: days = 228, dcf = 228/360 = 0.633333
            // denom = 1 - 0.0575 * 0.633333 = 0.963583
            // result = 1000000 / 0.963583 ≈ 1037780
            assert!(v > 1_030_000.0 && v < 1_050_000.0, "RECEIVED = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_received_maturity_le_settlement_is_num() {
        let mut ctx = c();
        let r = received(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2019, 1, 1),
                Value::Number(1000.0),
                Value::Number(0.05),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_received_zero_investment_is_num() {
        let mut ctx = c();
        let r = received(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 6, 1),
                Value::Number(0.0),
                Value::Number(0.05),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_received_denom_le_zero_is_num() {
        let mut ctx = c();
        // Very large discount and long period to make denom <= 0
        let r = received(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2021, 1, 1),
                Value::Number(1000.0),
                Value::Number(10.0), // 1000% discount, denom = 1 - 10 * dcf
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_received_with_default_basis() {
        let mut ctx = c();
        // RECEIVED without basis (defaults to 0, US 30/360)
        let r = received(
            &mut ctx,
            &[
                ds(2008, 2, 15),
                ds(2008, 9, 30),
                Value::Number(1_000_000.0),
                Value::Number(0.0575),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 1_000_000.0, "RECEIVED default basis = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- TBILLEQ ---
    // TBILLEQ(settlement, maturity, discount)

    #[test]
    fn test_tbilleq_basic() {
        let mut ctx = c();
        // TBILLEQ(2008-3-31, 2008-6-1, 0.0914)
        let r = tbilleq(
            &mut ctx,
            &[
                ds(2008, 3, 31),
                ds(2008, 6, 1),
                Value::Number(0.0914),
            ],
        );
        if let Value::Number(v) = r {
            // dsm = 62, discount = 0.0914
            // BEY = (365 * 0.0914) / (360 - 0.0914 * 62) ≈ 0.094167
            assert!((v - 0.0942).abs() < 0.001, "TBILLEQ = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_tbilleq_dsm_too_long_is_num() {
        let mut ctx = c();
        // dsm > 366 → #NUM!
        let r = tbilleq(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2021, 1, 3),
                Value::Number(0.05),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_tbilleq_negative_discount_is_num() {
        let mut ctx = c();
        let r = tbilleq(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 3, 1),
                Value::Number(-0.05),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_tbilleq_maturity_before_settlement_is_num() {
        let mut ctx = c();
        let r = tbilleq(
            &mut ctx,
            &[
                ds(2020, 3, 1),
                ds(2020, 1, 1),
                Value::Number(0.05),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- TBILLYIELD ---
    // TBILLYIELD(settlement, maturity, pr)

    #[test]
    fn test_tbillyield_basic() {
        let mut ctx = c();
        // TBILLYIELD(2008-3-31, 2008-6-1, 97.45)
        let r = tbillyield(
            &mut ctx,
            &[
                ds(2008, 3, 31),
                ds(2008, 6, 1),
                Value::Number(97.45),
            ],
        );
        if let Value::Number(v) = r {
            // dsm = 62, yield = (100-97.45)/97.45 * 360/62 ≈ 0.1519
            assert!((v - 0.152).abs() < 0.005, "TBILLYIELD = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_tbillyield_dsm_too_long_is_num() {
        let mut ctx = c();
        let r = tbillyield(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2021, 1, 3),
                Value::Number(98.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_tbillyield_zero_price_is_num() {
        let mut ctx = c();
        let r = tbillyield(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 3, 1),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- PRICEDISC ---
    // PRICEDISC(settlement, maturity, discount, redemption, [basis])

    #[test]
    fn test_pricedisc_basic() {
        let mut ctx = c();
        // PRICEDISC(2008-2-16, 2008-3-1, 0.0525, 100, 2)
        let r = pricedisc(
            &mut ctx,
            &[
                ds(2008, 2, 16),
                ds(2008, 3, 1),
                Value::Number(0.0525),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        if let Value::Number(v) = r {
            // actual days = 14, dcf = 14/360 = 0.038889
            // price = 100 * (1 - 0.0525 * 0.038889) ≈ 99.7958
            assert!((v - 99.80).abs() < 0.1, "PRICEDISC = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_pricedisc_maturity_le_settlement_is_num() {
        let mut ctx = c();
        let r = pricedisc(
            &mut ctx,
            &[
                ds(2020, 3, 1),
                ds(2020, 1, 1),
                Value::Number(0.05),
                Value::Number(100.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_pricedisc_negative_discount_is_num() {
        let mut ctx = c();
        let r = pricedisc(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 3, 1),
                Value::Number(-0.05),
                Value::Number(100.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_pricedisc_zero_redemption_is_num() {
        let mut ctx = c();
        let r = pricedisc(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 3, 1),
                Value::Number(0.05),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_pricedisc_with_default_basis() {
        let mut ctx = c();
        // PRICEDISC without basis (defaults to 0)
        let r = pricedisc(
            &mut ctx,
            &[
                ds(2008, 2, 16),
                ds(2008, 3, 1),
                Value::Number(0.0525),
                Value::Number(100.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 99.0 && v < 101.0, "PRICEDISC default basis = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- PRICEMAT ---
    // PRICEMAT(settlement, maturity, issue, rate, yld, [basis])

    #[test]
    fn test_pricemat_basic() {
        let mut ctx = c();
        // PRICEMAT(2008-2-15, 2008-4-13, 2007-11-11, 0.061, 0.061, 0)
        let r = pricemat(
            &mut ctx,
            &[
                ds(2008, 2, 15),
                ds(2008, 4, 13),
                ds(2007, 11, 11),
                Value::Number(0.061),
                Value::Number(0.061),
                Value::Number(0.0),
            ],
        );
        if let Value::Number(v) = r {
            // Par bond: rate == yield → price close to 100
            assert!(v > 99.0 && v < 102.0, "PRICEMAT = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_pricemat_settlement_ge_maturity_is_num() {
        let mut ctx = c();
        let r = pricemat(
            &mut ctx,
            &[
                ds(2020, 6, 1),
                ds(2020, 1, 1),
                ds(2019, 1, 1),
                Value::Number(0.05),
                Value::Number(0.05),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_pricemat_negative_rate_is_num() {
        let mut ctx = c();
        let r = pricemat(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 6, 1),
                ds(2019, 1, 1),
                Value::Number(-0.05),
                Value::Number(0.05),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_pricemat_negative_yield_is_num() {
        let mut ctx = c();
        let r = pricemat(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 6, 1),
                ds(2019, 1, 1),
                Value::Number(0.05),
                Value::Number(-0.05),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_pricemat_invalid_basis_is_num() {
        let mut ctx = c();
        let r = pricemat(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 6, 1),
                ds(2019, 1, 1),
                Value::Number(0.05),
                Value::Number(0.05),
                Value::Number(99.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_pricemat_with_basis_3() {
        let mut ctx = c();
        // basis 3 = actual/365
        let r = pricemat(
            &mut ctx,
            &[
                ds(2008, 2, 15),
                ds(2008, 4, 13),
                ds(2007, 11, 11),
                Value::Number(0.061),
                Value::Number(0.061),
                Value::Number(3.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 99.0 && v < 102.0, "PRICEMAT basis3 = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    // --- Additional edge-case tests for existing functions ---

    #[test]
    fn test_coupdays_basis1_actual() {
        let mut ctx = c();
        // COUPDAYS with basis 1 (actual/actual)
        let r = coupdays(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        );
        // actual/actual: actual days in coupon period
        let v = n(&r);
        assert!(v > 180.0 && v < 185.0, "COUPDAYS basis1 = {v}");
    }

    #[test]
    fn test_coupdays_basis3_actual365() {
        let mut ctx = c();
        // COUPDAYS with basis 3 (actual/365)
        let r = coupdays(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(3.0),
            ],
        );
        let v = n(&r);
        // 365/2 = 182.5
        assert!((v - 182.5).abs() < 1e-6, "COUPDAYS basis3 = {v}");
    }

    #[test]
    fn test_coupdays_basis4_eu() {
        let mut ctx = c();
        // COUPDAYS with basis 4 (European 30/360)
        let r = coupdays(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(4.0),
            ],
        );
        let v = n(&r);
        // 360/2 = 180
        assert!((v - 180.0).abs() < 1e-6, "COUPDAYS basis4 = {v}");
    }

    #[test]
    fn test_coupdaybs_basis1_actual() {
        let mut ctx = c();
        let r = coupdaybs(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        );
        let v = n(&r);
        // actual days since previous coupon
        assert!(v > 70.0 && v < 75.0, "COUPDAYBS basis1 = {v}");
    }

    #[test]
    fn test_coupdaysnc_basis1_actual() {
        let mut ctx = c();
        // COUPDAYSNC with basis 1 (actual/actual)
        let r = coupdaysnc(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        );
        let v = n(&r);
        // actual days settlement → next coupon
        assert!(v > 108.0 && v < 115.0, "COUPDAYSNC basis1 = {v}");
    }

    #[test]
    fn test_coupdaysnc_basis0_us() {
        let mut ctx = c();
        // COUPDAYSNC with basis 0 (US 30/360)
        let args = [
            ds(2007, 1, 25),
            ds(2008, 11, 15),
            Value::Number(2.0),
            Value::Number(0.0),
        ];
        let r = coupdaysnc(&mut ctx, &args);
        let v = n(&r);
        // COUPDAYSNC = COUPDAYS - COUPDAYBS
        let cd = n(&coupdays(&mut ctx, &args));
        let bs = n(&coupdaybs(&mut ctx, &args));
        assert!((v - (cd - bs)).abs() < 1e-6, "COUPDAYSNC basis0 = {v}");
    }

    #[test]
    fn test_coupnum_freq4() {
        let mut ctx = c();
        // Quarterly coupons
        let r = coupnum(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2021, 1, 1),
                Value::Number(4.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(n(&r), 4.0, "COUPNUM quarterly");
    }

    #[test]
    fn test_coupnum_freq1() {
        let mut ctx = c();
        // Annual coupons
        let r = coupnum(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2023, 1, 1),
                Value::Number(1.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(n(&r), 3.0, "COUPNUM annual");
    }

    #[test]
    fn test_price_zero_coupon() {
        let mut ctx = c();
        // Zero coupon bond: rate = 0 → price < 100
        let r = price(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2030, 1, 1),
                Value::Number(0.0),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        let v = n(&r);
        assert!(v < 100.0 && v > 50.0, "Zero coupon PRICE = {v}");
    }

    #[test]
    fn test_price_high_yield() {
        let mut ctx = c();
        // High yield bond: price much lower than par
        let r = price(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2030, 1, 1),
                Value::Number(0.05),
                Value::Number(0.15),
                Value::Number(100.0),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        let v = n(&r);
        assert!(v < 70.0, "High yield PRICE = {v}");
    }

    #[test]
    fn test_price_negative_rate_is_num() {
        let mut ctx = c();
        let r = price(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2030, 1, 1),
                Value::Number(-0.05),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_price_negative_yield_is_num() {
        let mut ctx = c();
        let r = price(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2030, 1, 1),
                Value::Number(0.05),
                Value::Number(-0.05),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_price_zero_redemption_is_num() {
        let mut ctx = c();
        let r = price(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2030, 1, 1),
                Value::Number(0.05),
                Value::Number(0.05),
                Value::Number(0.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_price_invalid_freq_is_num() {
        let mut ctx = c();
        let r = price(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2030, 1, 1),
                Value::Number(0.05),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(3.0), // invalid freq
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_yield_fn_negative_rate_is_num() {
        let mut ctx = c();
        let r = yield_fn(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2030, 1, 1),
                Value::Number(-0.05),
                Value::Number(95.0),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_yield_fn_zero_price_is_num() {
        let mut ctx = c();
        let r = yield_fn(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2030, 1, 1),
                Value::Number(0.05),
                Value::Number(0.0),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_duration_negative_coupon_is_num() {
        let mut ctx = c();
        let r = duration(
            &mut ctx,
            &[
                ds(2008, 1, 1),
                ds(2016, 1, 1),
                Value::Number(-0.05),
                Value::Number(0.09),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_duration_basis4_eu() {
        let mut ctx = c();
        // Duration with European 30/360
        let r = duration(
            &mut ctx,
            &[
                ds(2008, 1, 1),
                ds(2016, 1, 1),
                Value::Number(0.08),
                Value::Number(0.09),
                Value::Number(2.0),
                Value::Number(4.0),
            ],
        );
        let d = n(&r);
        assert!(d > 5.0 && d < 7.0, "DURATION basis4 = {d}");
    }

    #[test]
    fn test_amordegrc_basic() {
        let mut ctx = c();
        // AMORDEGRC(2400, 2008-8-19, 2008-12-31, 300, 1, 0.15, 1)
        let r = amordegrc(
            &mut ctx,
            &[
                Value::Number(2400.0),
                ds(2008, 8, 19),
                ds(2008, 12, 31),
                Value::Number(300.0),
                Value::Number(1.0),
                Value::Number(0.15),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            // life = 1/0.15 = 6.67, coeff = 2.0
            assert!(v > 0.0, "AMORDEGRC = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_amordegrc_period0() {
        let mut ctx = c();
        let r = amordegrc(
            &mut ctx,
            &[
                Value::Number(2400.0),
                ds(2008, 8, 19),
                ds(2008, 12, 31),
                Value::Number(300.0),
                Value::Number(0.0),
                Value::Number(0.15),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            // Period 0 = first-period prorated depreciation
            assert!(v > 0.0, "AMORDEGRC period0 = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_amordegrc_rate_too_high_is_num() {
        let mut ctx = c();
        let r = amordegrc(
            &mut ctx,
            &[
                Value::Number(2400.0),
                ds(2008, 8, 19),
                ds(2008, 12, 31),
                Value::Number(300.0),
                Value::Number(1.0),
                Value::Number(0.6), // rate >= 0.5
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_amordegrc_negative_period_is_num() {
        let mut ctx = c();
        let r = amordegrc(
            &mut ctx,
            &[
                Value::Number(2400.0),
                ds(2008, 8, 19),
                ds(2008, 12, 31),
                Value::Number(300.0),
                Value::Number(-1.0),
                Value::Number(0.15),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_amorlinc_zero_rate_is_num() {
        let mut ctx = c();
        let r = amorlinc(
            &mut ctx,
            &[
                Value::Number(2400.0),
                ds(2008, 8, 19),
                ds(2008, 12, 31),
                Value::Number(300.0),
                Value::Number(1.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_amorlinc_period0() {
        let mut ctx = c();
        // Period 0 = first-period prorated depreciation
        let r = amorlinc(
            &mut ctx,
            &[
                Value::Number(2400.0),
                ds(2008, 8, 19),
                ds(2008, 12, 31),
                Value::Number(300.0),
                Value::Number(0.0),
                Value::Number(0.15),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 500.0, "AMORLINC period0 = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_amorlinc_period3() {
        let mut ctx = c();
        let r = amorlinc(
            &mut ctx,
            &[
                Value::Number(2400.0),
                ds(2008, 8, 19),
                ds(2008, 12, 31),
                Value::Number(300.0),
                Value::Number(3.0),
                Value::Number(0.15),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "AMORLINC period3 = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_amorlinc_later_period_zero_dep() {
        let mut ctx = c();
        // Period far in the future where book value equals salvage
        let r = amorlinc(
            &mut ctx,
            &[
                Value::Number(1000.0),
                ds(2008, 1, 1),
                ds(2008, 1, 1),
                Value::Number(900.0), // high salvage
                Value::Number(100.0),
                Value::Number(0.1),
                Value::Number(0.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v >= 0.0, "AMORLINC later period = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_euroconvert_with_triangulation() {
        let mut ctx = c();
        // With triangulation precision
        let r = euroconvert(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Text("DEM".into()),
                Value::Text("FRF".into()),
                Value::Number(0.0), // not full precision
                Value::Number(3.0), // triangulation precision 3
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "EUROCONVERT triang = {v}");
        } else {
            panic!("expected Number, got {r:?}");
        }
    }

    #[test]
    fn test_euroconvert_target_unknown_is_value() {
        let mut ctx = c();
        let r = euroconvert(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Text("EUR".into()),
                Value::Text("USD".into()),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn test_odd_yield_stubbed_num() {
        let mut ctx = c();
        let r = oddlyield(
            &mut ctx,
            &[
                ds(2008, 11, 11),
                ds(2021, 3, 1),
                ds(2008, 10, 15),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_oddlprice_stubbed_num() {
        let mut ctx = c();
        let r = oddlprice(
            &mut ctx,
            &[
                ds(2008, 11, 11),
                ds(2021, 3, 1),
                ds(2008, 10, 15),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_oddfyield_stubbed_num() {
        let mut ctx = c();
        let r = oddfyield(
            &mut ctx,
            &[
                ds(2008, 11, 11),
                ds(2021, 3, 1),
                ds(2008, 10, 15),
                ds(2009, 3, 1),
                Value::Number(0.0785),
                Value::Number(0.0625),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_couppcd_returns_before_settlement() {
        let mut ctx = c();
        let r = couppcd(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        let settle = n(&ds(2007, 1, 25));
        assert!(n(&r) <= settle, "COUPPCD should be <= settlement");
    }

    #[test]
    fn test_coupncd_returns_after_settlement() {
        let mut ctx = c();
        let r = coupncd(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        let settle = n(&ds(2007, 1, 25));
        assert!(n(&r) > settle, "COUPNCD should be > settlement");
    }
