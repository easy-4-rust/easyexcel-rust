    #[test]
    fn test_accrintm() {
        let mut ctx = c();
        // ACCRINTM(issue=2008-4-1, settle=2008-6-15, 10%, par 1000, basis 3)
        let r = accrintm(
            &mut ctx,
            &[
                ds(2008, 4, 1),
                ds(2008, 6, 15),
                Value::Number(0.1),
                Value::Number(1000.0),
                Value::Number(3.0),
            ],
        );
        // 75 days / 365 * 0.1 * 1000 = 20.5479...
        assert!((n(&r) - 20.5479).abs() < 0.01, "ACCRINTM = {}", n(&r));
    }

    #[test]
    fn test_settlement_ge_maturity_is_num() {
        let mut ctx = c();
        let r = coupnum(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2019, 1, 1),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
        let p = price(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 1, 1),
                Value::Number(0.05),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(p, Value::Error(CellError::Num));
    }

    #[test]
    fn test_euroconvert() {
        let mut ctx = c();
        // 100 DEM -> EUR : 100 / 1.95583 = 51.13 (2 decimals)
        let r = euroconvert(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Text("DEM".into()),
                Value::Text("EUR".into()),
            ],
        );
        assert!((n(&r) - 51.13).abs() < 0.01, "DEM->EUR = {}", n(&r));
        // 1 EUR -> FRF = 6.55957, full precision
        let r2 = euroconvert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("EUR".into()),
                Value::Text("FRF".into()),
                Value::Bool(true),
            ],
        );
        assert!((n(&r2) - 6.55957).abs() < 1e-6, "EUR->FRF = {}", n(&r2));
        // Unknown currency -> #VALUE!
        let r3 = euroconvert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("USD".into()),
                Value::Text("EUR".into()),
            ],
        );
        assert_eq!(r3, Value::Error(CellError::Value));
    }

    #[test]
    fn test_amorlinc_first_period() {
        let mut ctx = c();
        // AMORLINC(2400, 2008-8-19, 2008-12-31, 300, 1, 15%, 1)
        let r = amorlinc(
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
        // Full-year depreciation = 2400 * 0.15 = 360 for period 1.
        assert!((n(&r) - 360.0).abs() < 1.0, "AMORLINC = {}", n(&r));
    }

    #[test]
    fn test_oddprice_stubbed_num() {
        let mut ctx = c();
        let r = oddfprice(
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
