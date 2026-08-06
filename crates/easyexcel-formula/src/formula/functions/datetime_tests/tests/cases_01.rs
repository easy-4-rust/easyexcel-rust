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
