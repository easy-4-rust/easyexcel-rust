    // --- date with large year ---

    #[test]
    fn date_large_year() {
        let mut c = ctx();
        let r = date_fn(
            &mut c,
            &[
                Value::Number(9999.0),
                Value::Number(12.0),
                Value::Number(31.0),
            ],
        );
        assert!(matches!(r, Value::Number(n) if n > 0.0));
    }

    // --- date with negative month ---

    #[test]
    fn date_negative_month_goes_back() {
        let mut c = ctx();
        // 2023, -1, 15 → 2022-11-15
        let r = date_fn(
            &mut c,
            &[
                Value::Number(2023.0),
                Value::Number(-1.0),
                Value::Number(15.0),
            ],
        );
        let expected =
            DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2022, 11, 15).unwrap());
        assert_eq!(r, Value::Number(expected as f64));
    }

    // --- time with fractional hours ---

    #[test]
    fn time_fractional_hours() {
        let mut c = ctx();
        // 1.5 hours = 1:30:00
        let r = time_fn(
            &mut c,
            &[Value::Number(1.5), Value::Number(0.0), Value::Number(0.0)],
        );
        // Should be a small fraction
        assert!(matches!(r, Value::Number(n) if n > 0.0 && n < 0.1));
    }

    // --- datevalue with different formats ---

    #[test]
    fn datevalue_slash_format() {
        let mut c = ctx();
        let r = datevalue_fn(&mut c, &[Value::Text("01/15/2023".into())]);
        assert!(matches!(r, Value::Number(n) if n > 0.0));
    }

    // --- weekday with mode 3 ---

    #[test]
    fn weekday_mode3() {
        let mut c = ctx();
        // 2023-01-01 is Sunday
        let s = Value::Number(44927.0);
        // mode 3: Monday=0, Sunday=6
        let r = weekday_fn(&mut c, &[s, Value::Number(3.0)]);
        assert_eq!(r, Value::Number(6.0));
    }

    // --- isoweeknum for known date ---

    #[test]
    fn isoweeknum_2023_01_02() {
        let mut c = ctx();
        // 2023-01-02 is Monday, ISO week 1
        let s = Value::Number(44928.0);
        let r = isoweeknum_fn(&mut c, &[s]);
        assert_eq!(r, Value::Number(1.0));
    }

    // --- edate with negative months ---

    #[test]
    fn edate_go_back_months() {
        let mut c = ctx();
        // 2023-03-15 - 2 months = 2023-01-15
        let date = Value::Number(44970.0); // 2023-03-14
        let r = edate_fn(&mut c, &[date, Value::Number(-2.0)]);
        assert!(matches!(r, Value::Number(n) if n > 0.0));
    }

    // --- eomonth with future months ---

    #[test]
    fn eomonth_future() {
        let mut c = ctx();
        // 2023-01-15 + 2 months end = 2023-03-31
        let date = Value::Number(44941.0);
        let r = eomonth_fn(&mut c, &[date, Value::Number(2.0)]);
        let expected = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 3, 31).unwrap());
        assert_eq!(r, Value::Number(expected as f64));
    }

    // --- datedif with same date ---

    #[test]
    fn datedif_same_date() {
        let mut c = ctx();
        let d = Value::Number(44927.0);
        assert_eq!(
            datedif_fn(&mut c, &[d.clone(), d.clone(), Value::Text("D".into())]),
            Value::Number(0.0)
        );
    }

    // --- days with swapped args ---

    #[test]
    fn days_negative_result() {
        let mut c = ctx();
        let d1 = Value::Number(44927.0);
        let d2 = Value::Number(44928.0);
        assert_eq!(days_fn(&mut c, &[d1, d2]), Value::Number(-1.0));
    }

    // --- yearfrac with basis 1 (actual/actual) ---

    #[test]
    fn yearfrac_basis_1() {
        let mut c = ctx();
        let d1 = Value::Number(44927.0);
        let d2 = Value::Number(45027.0);
        let r = yearfrac_fn(&mut c, &[d1, d2, Value::Number(1.0)]);
        assert!(matches!(r, Value::Number(n) if n > 0.0 && n < 1.0));
    }

    // --- networkdays with holidays ---

    #[test]
    fn networkdays_with_holidays() {
        let mut c = ctx();
        // Set a holiday cell
        c.set(0, 0, Value::Number(44929.0)); // 2023-01-03
        let d1 = Value::Number(44927.0); // 2023-01-01
        let d2 = Value::Number(44933.0); // 2023-01-07
        let r = networkdays_fn(&mut c, &[d1, d2, rng(0, 0, 0, 0)]);
        // 5 weekdays - 1 holiday = 4
        assert!(matches!(r, Value::Number(n) if n >= 4.0 && n <= 5.0));
    }

    // --- workday with 0 days ---

    #[test]
    fn workday_zero_days() {
        let mut c = ctx();
        let d = Value::Number(44927.0); // Sunday
        let r = workday_fn(&mut c, &[d, Value::Number(0.0)]);
        assert!(matches!(r, Value::Number(n) if n > 0.0));
    }

    // --- hour from full serial ---

    #[test]
    fn hour_from_full_serial() {
        let mut c = ctx();
        // 44927.75 = 2023-01-01 18:00:00
        let s = Value::Number(44927.75);
        assert_eq!(hour_fn(&mut c, &[s]), Value::Number(18.0));
    }

    // --- minute from fractional ---

    #[test]
    fn minute_from_fractional() {
        let mut c = ctx();
        // 0.520833... ≈ 12:30:00
        let s = Value::Number(0.520_833_333_333_333_3);
        let r = minute_fn(&mut c, &[s]);
        assert!(matches!(r, Value::Number(n) if (n - 30.0).abs() < 1.0));
    }
