    #[test]
    fn test_pmt_basic() {
        let mut ctx = c();
        // PMT(0.08/12, 10, 10000) ≈ -1037.03
        let r = pmt(
            &mut ctx,
            &[
                Value::Number(0.08 / 12.0),
                Value::Number(10.0),
                Value::Number(10000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, -1037.03), "PMT = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_pmt_zero_rate() {
        let mut ctx = c();
        // PMT(0, 10, 1000) = -100
        let r = pmt(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(10.0),
                Value::Number(1000.0),
            ],
        );
        assert_eq!(r, Value::Number(-100.0));
    }

    #[test]
    fn test_pmt_with_fv() {
        let mut ctx = c();
        // PMT(0.1/12, 24, 0, 50000) - saving to reach future value
        let r = pmt(
            &mut ctx,
            &[
                Value::Number(0.1 / 12.0),
                Value::Number(24.0),
                Value::Number(0.0),
                Value::Number(50000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v < 0.0, "saving PMT should be negative: {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_fv_basic() {
        let mut ctx = c();
        // FV(0.06/12, 10, -200, -500, 1) ≈ 2581.40
        let r = fv(
            &mut ctx,
            &[
                Value::Number(0.06 / 12.0),
                Value::Number(10.0),
                Value::Number(-200.0),
                Value::Number(-500.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 2581.40), "FV = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_fv_zero_rate() {
        let mut ctx = c();
        // FV(0, 3, -100, -1000) = 1000 + 300 = 1300
        let r = fv(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(3.0),
                Value::Number(-100.0),
                Value::Number(-1000.0),
            ],
        );
        assert_eq!(r, Value::Number(1300.0));
    }

    #[test]
    fn test_pv_basic() {
        let mut ctx = c();
        // PV(0.08/12, 20*12, 500) — PV of annuity
        let r = pv(
            &mut ctx,
            &[
                Value::Number(0.08 / 12.0),
                Value::Number(240.0),
                Value::Number(500.0),
            ],
        );
        if let Value::Number(v) = r {
            // Should be a large negative number (present cost of receiving 500/mo for 20 yrs)
            assert!(v < -50000.0, "PV = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_nper_basic() {
        let mut ctx = c();
        // NPER(0.12/12, -100, -1000, 10000) ≈ 60.08
        // pv=-1000, fv=10000, rate=0.01, pmt=-100
        let r = nper(
            &mut ctx,
            &[
                Value::Number(0.12 / 12.0),
                Value::Number(-100.0),
                Value::Number(-1000.0),
                Value::Number(10000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 60.08), "NPER = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_rate_basic() {
        let mut ctx = c();
        // RATE(48, -200, 8000) — monthly rate for a loan of 8000, 48 payments of 200
        // Excel: ~0.77% per month
        let r = rate(
            &mut ctx,
            &[
                Value::Number(48.0),
                Value::Number(-200.0),
                Value::Number(8000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v * 100.0, 0.77), "RATE = {}", v * 100.0);
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_npv_basic() {
        let mut ctx = c();
        // NPV(0.1, -10000, 3000, 4200, 6800) ≈ 1188.44
        let r = npv(
            &mut ctx,
            &[
                Value::Number(0.1),
                Value::Number(-10000.0),
                Value::Number(3000.0),
                Value::Number(4200.0),
                Value::Number(6800.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 1188.44), "NPV = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_npv_range() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(3000.0)),
            (1, 0, Value::Number(4200.0)),
            (2, 0, Value::Number(6800.0)),
        ]);
        let r = npv(
            &mut ctx,
            &[Value::Number(0.1), Value::Number(-10000.0), rng(0, 0, 2, 0)],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 1188.44), "NPV range = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_irr_basic() {
        let mut ctx = c();
        // IRR([-70000, 12000, 15000, 18000, 21000, 26000]) ≈ 8.66%
        let r = irr(
            &mut ctx,
            &[Value::Array(crate::formula::value::Array::from_rows(vec![
                vec![
                    Value::Number(-70000.0),
                    Value::Number(12000.0),
                    Value::Number(15000.0),
                    Value::Number(18000.0),
                    Value::Number(21000.0),
                    Value::Number(26000.0),
                ],
            ]))],
        );
        if let Value::Number(v) = r {
            assert!(approx(v * 100.0, 8.66), "IRR = {}", v * 100.0);
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_irr_error_no_sign_change() {
        let mut ctx = c();
        // All positive — no IRR
        let r = irr(
            &mut ctx,
            &[Value::Array(crate::formula::value::Array::from_rows(vec![
                vec![Value::Number(100.0), Value::Number(200.0)],
            ]))],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_mirr_basic() {
        let mut ctx = c();
        // MIRR([-120000, 39000, 30000, 21000, 37000, 46000], 10%, 12%)
        let cfs = vec![
            Value::Number(-120_000.0),
            Value::Number(39000.0),
            Value::Number(30000.0),
            Value::Number(21000.0),
            Value::Number(37000.0),
            Value::Number(46000.0),
        ];
        let r = mirr(
            &mut ctx,
            &[
                Value::Array(crate::formula::value::Array::from_rows(vec![cfs])),
                Value::Number(0.10),
                Value::Number(0.12),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v * 100.0, 12.61), "MIRR = {}", v * 100.0);
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_sln() {
        let mut ctx = c();
        // SLN(30000, 7500, 10) = 2250
        let r = sln(
            &mut ctx,
            &[
                Value::Number(30000.0),
                Value::Number(7500.0),
                Value::Number(10.0),
            ],
        );
        assert_eq!(r, Value::Number(2250.0));
    }

    #[test]
    fn test_sln_zero_life() {
        let mut ctx = c();
        let r = sln(
            &mut ctx,
            &[
                Value::Number(1000.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    #[test]
    fn test_syd_basic() {
        let mut ctx = c();
        // SYD(30000, 7500, 10, 1) ≈ 4090.91
        let r = syd(
            &mut ctx,
            &[
                Value::Number(30000.0),
                Value::Number(7500.0),
                Value::Number(10.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 4090.91), "SYD = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_ddb_basic() {
        let mut ctx = c();
        // DDB(2400, 300, 10, 1) — year 1 depreciation
        let r = ddb(
            &mut ctx,
            &[
                Value::Number(2400.0),
                Value::Number(300.0),
                Value::Number(10.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Number(480.0));
    }

    #[test]
    fn test_ddb_period_exceeds_life() {
        let mut ctx = c();
        let r = ddb(
            &mut ctx,
            &[
                Value::Number(2400.0),
                Value::Number(300.0),
                Value::Number(10.0),
                Value::Number(11.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_effect_nominal() {
        let mut ctx = c();
        // EFFECT(5.25%, 4) ≈ 5.354%
        let e = effect(&mut ctx, &[Value::Number(0.0525), Value::Number(4.0)]);
        if let Value::Number(v) = e {
            assert!(approx(v * 100.0, 5.354), "EFFECT = {}", v * 100.0);
        } else {
            panic!("Expected number");
        }

        // NOMINAL(EFFECT(5.25%, 4), 4) should return ~5.25%
        let n = nominal(&mut ctx, &[e.clone(), Value::Number(4.0)]);
        // e is the result of effect, use manually
        let effect_val = if let Value::Number(v) = e { v } else { 0.0 };
        let n2 = nominal(&mut ctx, &[Value::Number(effect_val), Value::Number(4.0)]);
        if let Value::Number(v) = n2 {
            assert!(
                approx(v * 100.0, 5.25),
                "NOMINAL round-trip = {}",
                v * 100.0
            );
        }
        let _ = n;
    }

    #[test]
    fn test_effect_invalid() {
        let mut ctx = c();
        assert_eq!(
            effect(&mut ctx, &[Value::Number(-0.1), Value::Number(4.0)]),
            Value::Error(CellError::Num)
        );
        assert_eq!(
            effect(&mut ctx, &[Value::Number(0.05), Value::Number(0.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn test_pduration() {
        let mut ctx = c();
        // PDURATION(0.025, 2000, 2200) ≈ 3.86
        let r = pduration(
            &mut ctx,
            &[
                Value::Number(0.025),
                Value::Number(2000.0),
                Value::Number(2200.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 3.86), "PDURATION = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_rri() {
        let mut ctx = c();
        // RRI(96, 10000, 11000) — quarterly equivalent rate
        let r = rri(
            &mut ctx,
            &[
                Value::Number(96.0),
                Value::Number(10000.0),
                Value::Number(11000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 0.01, "RRI = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_dollarde_dollarfr() {
        let mut ctx = c();
        // DOLLARDE(1.02, 16) = 1.125 (1 + 2/16 = 1 + 0.125)
        let r = dollarde(&mut ctx, &[Value::Number(1.02), Value::Number(16.0)]);
        if let Value::Number(v) = r {
            assert!(approx_fine(v, 1.125), "DOLLARDE = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    #[test]
    fn test_ispmt() {
        let mut ctx = c();
        // ISPMT(0.1/12, 1, 36, 8000000) ≈ -66667
        let r = ispmt(
            &mut ctx,
            &[
                Value::Number(0.1 / 12.0),
                Value::Number(1.0),
                Value::Number(36.0),
                Value::Number(8_000_000.0),
            ],
        );
        if let Value::Number(v) = r {
            // ISPMT(0.1/12, 1, 36, 8000000) = 8000000*(0.1/12)*(1/36-1) ≈ -64814.81
            assert!(approx(v, -64814.81), "ISPMT = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_tbillprice() {
        let mut ctx = c();
        // TBILLPRICE with 91-day bill at 9% discount
        // settlement=0, maturity=91, discount=0.09
        let r = tbillprice(
            &mut ctx,
            &[Value::Number(0.0), Value::Number(91.0), Value::Number(0.09)],
        );
        if let Value::Number(v) = r {
            // 100 * (1 - 0.09 * 91/360) = 100 * (1 - 0.02275) = 97.725
            assert!(approx(v, 97.725), "TBILLPRICE = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_fvschedule() {
        let mut ctx = c();
        // FVSCHEDULE(100, [0.09, 0.11, 0.1]) = 100*1.09*1.11*1.1 ≈ 133.09
        let r = fvschedule(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Array(crate::formula::value::Array::from_rows(vec![vec![
                    Value::Number(0.09),
                    Value::Number(0.11),
                    Value::Number(0.1),
                ]])),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 133.09), "FVSCHEDULE = {v}");
        } else {
            panic!("Expected number");
        }
    }

    // ---- Coupon / bond function tests ----

    // Build a serial-number Value for a y/m/d in the 1900 date system.
    fn ds(y: i32, m: u32, d: u32) -> Value {
        let serial = easyexcel_model::dates::ymd_to_serial(DateSystem::Date1900, y, m, d).unwrap();
        Value::Number(serial)
    }

    fn n(v: &Value) -> f64 {
        match v {
            Value::Number(x) => *x,
            other => panic!("expected number, got {other:?}"),
        }
    }

    #[test]
    fn test_coupnum_excel() {
        let mut ctx = c();
        // COUPNUM(DATE(2007,1,25), DATE(2008,11,15), 2, 0) = 4
        let r = coupnum(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(n(&r), 4.0);
    }

    #[test]
    fn test_coupncd_couppcd_bracket() {
        let mut ctx = c();
        // Next coupon must be after settlement, previous on/before.
        let nc = coupncd(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        let pc = couppcd(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        let settle = n(&ds(2007, 1, 25));
        assert!(n(&nc) > settle);
        assert!(n(&pc) <= settle);
        // Coupons are 6 months apart -> ~182/183 days
        assert!((n(&nc) - n(&pc)).abs() > 180.0 && (n(&nc) - n(&pc)).abs() < 185.0);
    }

    #[test]
    fn test_coupdays_consistency() {
        let mut ctx = c();
        let args = [
            ds(2007, 1, 25),
            ds(2008, 11, 15),
            Value::Number(2.0),
            Value::Number(0.0),
        ];
        let cd = n(&coupdays(&mut ctx, &args));
        let bs = n(&coupdaybs(&mut ctx, &args));
        let nc = n(&coupdaysnc(&mut ctx, &args));
        // COUPDAYBS + COUPDAYSNC == COUPDAYS
        assert!((bs + nc - cd).abs() < 1e-6, "bs={bs} nc={nc} cd={cd}");
        // basis 0 -> 360/2 = 180
        assert!((cd - 180.0).abs() < 1e-6);
    }

    #[test]
    fn test_price_par_bond() {
        let mut ctx = c();
        // Par bond: coupon == yield -> price ~ 100 (settlement on a coupon date).
        // Settlement = 2010-01-01, maturity 2020-01-01, 5% coupon, 5% yield, freq 2.
        let r = price(
            &mut ctx,
            &[
                ds(2010, 1, 1),
                ds(2020, 1, 1),
                Value::Number(0.05),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        assert!((n(&r) - 100.0).abs() < 0.1, "PRICE = {}", n(&r));
    }

    #[test]
    fn test_price_yield_roundtrip() {
        let mut ctx = c();
        let base = [
            ds(2008, 2, 15),
            ds(2017, 11, 15),
            Value::Number(0.0575),
            Value::Number(0.065),
            Value::Number(100.0),
            Value::Number(2.0),
            Value::Number(0.0),
        ];
        let p = n(&price(&mut ctx, &base));
        // Excel PRICE for these args ~= 94.63
        assert!((p - 94.63).abs() < 0.2, "PRICE = {p}");
        // Feed price into YIELD, expect ~0.065 back.
        let yargs = [
            base[0].clone(),
            base[1].clone(),
            base[2].clone(),
            Value::Number(p),
            base[4].clone(),
            base[5].clone(),
            base[6].clone(),
        ];
        let y = n(&yield_fn(&mut ctx, &yargs));
        assert!((y - 0.065).abs() < 1e-4, "YIELD = {y}");
    }

    #[test]
    fn test_duration_reasonable() {
        let mut ctx = c();
        // DURATION(2008-1-1, 2016-1-1, 8% coupon, 9% yield, 2) ~ 5.99 yrs (Excel ~5.99)
        let r = duration(
            &mut ctx,
            &[
                ds(2008, 1, 1),
                ds(2016, 1, 1),
                Value::Number(0.08),
                Value::Number(0.09),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        );
        let d = n(&r);
        assert!(d > 5.5 && d < 6.5, "DURATION = {d}");
        // MDURATION = DURATION / (1 + y/freq)
        let md = n(&mduration(
            &mut ctx,
            &[
                ds(2008, 1, 1),
                ds(2016, 1, 1),
                Value::Number(0.08),
                Value::Number(0.09),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        ));
        assert!(
            (md - d / (1.0 + 0.09 / 2.0)).abs() < 1e-6,
            "MDURATION = {md}"
        );
    }

