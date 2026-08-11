    #[test]
    fn date_basic() {
        let mut c = ctx();
        let r = date_fn(
            &mut c,
            &[
                Value::Number(2023.0),
                Value::Number(1.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Number(44927.0));
    }

    #[test]
    fn date_two_digit_year() {
        let mut c = ctx();
        let r = date_fn(
            &mut c,
            &[Value::Number(25.0), Value::Number(6.0), Value::Number(15.0)],
        );
        // Year 2025
        let expected =
            DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
        assert_eq!(r, Value::Number(expected as f64));
    }

    #[test]
    fn time_basic() {
        let mut c = ctx();
        let r = time_fn(
            &mut c,
            &[Value::Number(12.0), Value::Number(0.0), Value::Number(0.0)],
        );
        // 12:00:00 = 0.5
        assert_eq!(r, Value::Number(0.5));
    }

    #[test]
    fn year_month_day() {
        let mut c = ctx();
        // 2008-12-31 = serial 39813
        let s = Value::Number(39813.0);
        assert_eq!(
            year_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(2008.0)
        );
        assert_eq!(
            month_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(12.0)
        );
        assert_eq!(
            day_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(31.0)
        );
    }

    #[test]
    fn hour_minute_second() {
        let mut c = ctx();
        // 0.5 = 12:00:00
        let s = Value::Number(0.5);
        assert_eq!(
            hour_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(12.0)
        );
        assert_eq!(
            minute_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(0.0)
        );
        assert_eq!(
            second_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(0.0)
        );
    }

    #[test]
    fn weekday_sunday_first() {
        let mut c = ctx();
        // 2023-01-01 is a Sunday. Serial 44927.
        let s = Value::Number(44927.0);
        // type 1: Sun=1
        assert_eq!(
            weekday_fn(&mut c, &[s.clone(), Value::Number(1.0)]),
            Value::Number(1.0)
        );
        // type 2: Sun=7
        assert_eq!(
            weekday_fn(&mut c, &[s.clone(), Value::Number(2.0)]),
            Value::Number(7.0)
        );
    }

    #[test]
    fn edate_basic() {
        let mut c = ctx();
        // 2023-01-31 + 1 month = 2023-02-28
        let serial = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 31).unwrap())
            as f64;
        let r = edate_fn(&mut c, &[Value::Number(serial), Value::Number(1.0)]);
        let expected = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 2, 28).unwrap())
            as f64;
        assert_eq!(r, Value::Number(expected));
    }

    #[test]
    fn eomonth_basic() {
        let mut c = ctx();
        // End of month for 2023-01-15 + 0 months = 2023-01-31
        let serial = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
            as f64;
        let r = eomonth_fn(&mut c, &[Value::Number(serial), Value::Number(0.0)]);
        let expected = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 31).unwrap())
            as f64;
        assert_eq!(r, Value::Number(expected));
    }

    #[test]
    fn datedif_y() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("Y".into()),
            ],
        );
        assert_eq!(r, Value::Number(3.0));
    }

    #[test]
    fn datedif_d() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 31).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("D".into()),
            ],
        );
        assert_eq!(r, Value::Number(30.0));
    }

    #[test]
    fn networkdays_basic() {
        let mut c = ctx();
        // 2023-01-02 (Mon) to 2023-01-06 (Fri) = 5 workdays
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap())
            as f64;
        let s2 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 6).unwrap())
            as f64;
        let r = networkdays_fn(&mut c, &[Value::Number(s1), Value::Number(s2)]);
        assert_eq!(r, Value::Number(5.0));
    }

    #[test]
    fn workday_basic() {
        let mut c = ctx();
        // 2023-01-02 (Mon) + 5 workdays = 2023-01-09 (Mon)
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap())
            as f64;
        let r = workday_fn(&mut c, &[Value::Number(s1), Value::Number(5.0)]);
        let expected = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 9).unwrap())
            as f64;
        assert_eq!(r, Value::Number(expected));
    }

    #[test]
    fn days_fn_basic() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 31).unwrap())
            as f64;
        let r = days_fn(&mut c, &[Value::Number(s2), Value::Number(s1)]);
        assert_eq!(r, Value::Number(30.0));
    }

    #[test]
    fn isoweeknum_basic() {
        let mut c = ctx();
        // 2023-01-02 is ISO week 1
        let s = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap())
            as f64;
        let r = isoweeknum_fn(&mut c, &[Value::Number(s)]);
        assert_eq!(r, Value::Number(1.0));
    }

    // ── Agent 68 panic 回归：无效日期序列号不 panic ─────────────────────

    #[test]
    fn year_fn_invalid_serial_returns_error() {
        let mut c = ctx();
        // 负序列号 → #VALUE!
        let r = year_fn(&mut c, &[Value::Number(-1.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn month_fn_invalid_serial_returns_error() {
        let mut c = ctx();
        let r = month_fn(&mut c, &[Value::Number(-1.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn day_fn_invalid_serial_returns_error() {
        let mut c = ctx();
        let r = day_fn(&mut c, &[Value::Number(-1.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn edate_fn_invalid_serial_returns_error() {
        let mut c = ctx();
        let r = edate_fn(&mut c, &[Value::Number(-1.0), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn eomonth_fn_invalid_serial_returns_error() {
        let mut c = ctx();
        let r = eomonth_fn(&mut c, &[Value::Number(-1.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn weekday_fn_invalid_serial_returns_error() {
        let mut c = ctx();
        let r = weekday_fn(&mut c, &[Value::Number(-1.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn weeknum_fn_invalid_serial_returns_error() {
        let mut c = ctx();
        let r = weeknum_fn(&mut c, &[Value::Number(-1.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn isoweeknum_fn_invalid_serial_returns_error() {
        let mut c = ctx();
        let r = isoweeknum_fn(&mut c, &[Value::Number(-1.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // ── Agent 68 panic 回归：无效日期组合不 panic ──────────────────────

    #[test]
    fn date_fn_invalid_month_overflow() {
        let mut c = ctx();
        // month=13 → 日期溢出，应不 panic
        let r = date_fn(
            &mut c,
            &[Value::Number(2023.0), Value::Number(13.0), Value::Number(1.0)],
        );
        // 应为次年 1 月
        if let Value::Number(n) = r {
            assert!(n > 0.0);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn date_fn_invalid_day_overflow() {
        let mut c = ctx();
        // day=32 → 日期溢出
        let r = date_fn(
            &mut c,
            &[Value::Number(2023.0), Value::Number(1.0), Value::Number(32.0)],
        );
        if let Value::Number(n) = r {
            assert!(n > 0.0);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn date_fn_negative_year() {
        let mut c = ctx();
        let r = date_fn(
            &mut c,
            &[Value::Number(-1.0), Value::Number(1.0), Value::Number(1.0)],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // ── datedif 边界 ───────────────────────────────────────────────────

    #[test]
    fn datedif_start_after_end_returns_num_error() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("D".into()),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn datedif_invalid_unit() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("X".into()),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn datedif_unit_m() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("M".into()),
            ],
        );
        assert_eq!(r, Value::Number(5.0));
    }

    #[test]
    fn datedif_unit_md() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 3, 20).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("MD".into()),
            ],
        );
        assert_eq!(r, Value::Number(5.0)); // 20-15
    }

    #[test]
    fn datedif_unit_ym() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 3, 20).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("YM".into()),
            ],
        );
        assert_eq!(r, Value::Number(2.0));
    }

    #[test]
    fn datedif_unit_yd() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("YD".into()),
            ],
        );
        assert_eq!(r, Value::Number(165.0)); // days from Jan 1 to Jun 15
    }

    // ── time_fn 边界 ───────────────────────────────────────────────────

    #[test]
    fn time_fn_overflow() {
        let mut c = ctx();
        // 25 hours → wrap around
        let r = time_fn(
            &mut c,
            &[Value::Number(25.0), Value::Number(0.0), Value::Number(0.0)],
        );
        if let Value::Number(n) = r {
            assert!(n >= 0.0 && n < 1.0);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn time_fn_negative() {
        let mut c = ctx();
        let r = time_fn(
            &mut c,
            &[Value::Number(-1.0), Value::Number(0.0), Value::Number(0.0)],
        );
        if let Value::Number(n) = r {
            assert!(n >= 0.0 && n < 1.0);
        } else {
            panic!("expected number");
        }
    }

    // ── datevalue_fn / timevalue_fn ─────────────────────────────────────

    #[test]
    fn datevalue_fn_iso_date() {
        let mut c = ctx();
        let r = datevalue_fn(&mut c, &[Value::Text("2023-01-15".into())]);
        if let Value::Number(n) = r {
            assert!(n > 44000.0); // 2023 年的序列号
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn datevalue_fn_invalid() {
        let mut c = ctx();
        let r = datevalue_fn(&mut c, &[Value::Text("not-a-date".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn timevalue_fn_basic() {
        let mut c = ctx();
        let r = timevalue_fn(&mut c, &[Value::Text("12:00:00".into())]);
        assert_eq!(r, Value::Number(0.5));
    }

    #[test]
    fn timevalue_fn_invalid() {
        let mut c = ctx();
        let r = timevalue_fn(&mut c, &[Value::Text("not-a-time".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // ── days_fn 边界 ───────────────────────────────────────────────────

    #[test]
    fn days_fn_same_date() {
        let mut c = ctx();
        let s = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let r = days_fn(&mut c, &[Value::Number(s), Value::Number(s)]);
        assert_eq!(r, Value::Number(0.0));
    }

    // ── days360_fn ─────────────────────────────────────────────────────

    #[test]
    fn days360_fn_basic() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap())
            as f64;
        let r = days360_fn(&mut c, &[Value::Number(s1), Value::Number(s2)]);
        assert_eq!(r, Value::Number(360.0));
    }

    #[test]
    fn days360_fn_european() {
        let mut c = ctx();
        // European 30/360: 2023-01-01 到 2023-12-31
        // sd=min(1,30)=1, ed=min(31,30)=30
        // days = 0*360 + 11*30 + 30-1 = 359
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap())
            as f64;
        let r = days360_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Bool(true),
            ],
        );
        assert_eq!(r, Value::Number(359.0));
    }

    // ── yearfrac_fn ─────────────────────────────────────────────────────

    #[test]
    fn yearfrac_fn_basis_0() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 7, 1).unwrap())
            as f64;
        let r = yearfrac_fn(&mut c, &[Value::Number(s1), Value::Number(s2), Value::Number(0.0)]);
        if let Value::Number(n) = r {
            assert!((n - 0.5).abs() < 0.01);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn yearfrac_fn_invalid_basis() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 7, 1).unwrap())
            as f64;
        let r = yearfrac_fn(
            &mut c,
            &[Value::Number(s1), Value::Number(s2), Value::Number(99.0)],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // ── weekday_fn 多种 return_type ─────────────────────────────────────

    #[test]
    fn weekday_fn_type_3() {
        let mut c = ctx();
        // 2023-01-01 is Sunday → type 3: Mon=0..Sun=6 → Sun=6
        let s = Value::Number(44927.0);
        assert_eq!(
            weekday_fn(&mut c, &[s, Value::Number(3.0)]),
            Value::Number(6.0)
        );
    }

    #[test]
    fn weekday_fn_type_11() {
        let mut c = ctx();
        // 2023-01-01 is Sunday → type 11: Mon=1..Sun=7 → Sun=7
        let s = Value::Number(44927.0);
        assert_eq!(
            weekday_fn(&mut c, &[s, Value::Number(11.0)]),
            Value::Number(7.0)
        );
    }

    #[test]
    fn weekday_fn_invalid_type() {
        let mut c = ctx();
        let s = Value::Number(44927.0);
        assert_eq!(
            weekday_fn(&mut c, &[s, Value::Number(99.0)]),
            Value::Error(CellError::Num)
        );
    }

    // ── weeknum_fn 多种 return_type ─────────────────────────────────────

    #[test]
    fn weeknum_fn_type_2() {
        let mut c = ctx();
        let s = Value::Number(44927.0); // 2023-01-01
        let r = weeknum_fn(&mut c, &[s, Value::Number(2.0)]);
        if let Value::Number(n) = r {
            assert!(n >= 1.0);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn weeknum_fn_invalid_type() {
        let mut c = ctx();
        let s = Value::Number(44927.0);
        assert_eq!(
            weeknum_fn(&mut c, &[s, Value::Number(99.0)]),
            Value::Error(CellError::Num)
        );
    }

    // ── networkdays_fn 边界 ─────────────────────────────────────────────

    #[test]
    fn networkdays_fn_reversed_dates() {
        let mut c = ctx();
        // start > end → negative count
        let s1 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 6).unwrap())
            as f64;
        let s2 = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap())
            as f64;
        let r = networkdays_fn(&mut c, &[Value::Number(s1), Value::Number(s2)]);
        assert_eq!(r, Value::Number(-5.0));
    }

    // ── workday_fn 边界 ─────────────────────────────────────────────────

    #[test]
    fn workday_fn_zero_days() {
        let mut c = ctx();
        let s = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap())
            as f64;
        let r = workday_fn(&mut c, &[Value::Number(s), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(s));
    }

    // ── get_serial 文本日期解析 ──────────────────────────────────────────

    #[test]
    fn get_serial_text_date() {
        let mut c = ctx();
        // ISO 格式文本日期
        let r = get_serial(&mut c, &Value::Text("2023-01-15".into()));
        assert!(r.is_ok());
        assert!(r.unwrap() > 44000.0);
    }

    #[test]
    fn get_serial_invalid_text() {
        let mut c = ctx();
        let r = get_serial(&mut c, &Value::Text("not-a-date".into()));
        assert!(r.is_err());
    }

    #[test]
    fn get_serial_bool() {
        let mut c = ctx();
        assert_eq!(get_serial(&mut c, &Value::Bool(true)), Ok(1.0));
        assert_eq!(get_serial(&mut c, &Value::Bool(false)), Ok(0.0));
    }

    #[test]
    fn get_serial_empty() {
        let mut c = ctx();
        assert_eq!(get_serial(&mut c, &Value::Empty), Ok(0.0));
    }

    #[test]
    fn get_serial_error() {
        let mut c = ctx();
        assert_eq!(
            get_serial(&mut c, &Value::Error(CellError::NA)),
            Err(CellError::NA)
        );
    }

    // ── is_leap_year / days_in_month ────────────────────────────────────

    #[test]
    fn is_leap_year_basic() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn days_in_month_basic() {
        assert_eq!(days_in_month(2023, 1), 31);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29); // leap year
        assert_eq!(days_in_month(2023, 4), 30);
        assert_eq!(days_in_month(2023, 12), 31);
    }

    // ── weekend_code ────────────────────────────────────────────────────

    #[test]
    fn weekend_code_all_types() {
        assert!(weekend_code(1).is_ok()); // Sat+Sun
        assert!(weekend_code(2).is_ok()); // Sun+Mon
        assert!(weekend_code(7).is_ok()); // Fri+Sat
        assert!(weekend_code(11).is_ok()); // Sun only
        assert!(weekend_code(17).is_ok()); // Sat only
        assert!(weekend_code(99).is_err()); // invalid
    }

    #[test]
    fn parse_weekend_string() {
        // "0000011" → Fri+Sat are weekends
        let r = parse_weekend(&Value::Text("0000011".into()));
        assert!(r.is_ok());
        let arr = r.unwrap();
        assert!(arr[5]); // Fri (index 5)
        assert!(arr[6]); // Sat (index 6)
        assert!(!arr[0]); // Mon
    }

    #[test]
    fn parse_weekend_invalid_string() {
        let r = parse_weekend(&Value::Text("abc".into()));
        assert!(r.is_err());
    }

    #[test]
    fn parse_weekend_invalid_type() {
        let r = parse_weekend(&Value::Bool(true));
        assert!(r.is_err());
    }
