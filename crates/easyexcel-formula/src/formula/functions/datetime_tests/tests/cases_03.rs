    // --- 更多日期时间函数测试（覆盖 register_to_datedif_fn.rs 未测分支） ---

    // date_fn: 非数字参数
    #[test]
    fn date_fn_err_text() {
        let mut c = ctx();
        let r = date_fn(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(1.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // time_fn: 基本测试
    #[test]
    fn time_fn_basic() {
        let mut c = ctx();
        let r = time_fn(
            &mut c,
            &[Value::Number(12.0), Value::Number(0.0), Value::Number(0.0)],
        );
        if let Value::Number(v) = r {
            assert!(v >= 0.0 && v < 1.0, "TIME = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // time_fn: 非数字参数
    #[test]
    fn time_fn_err_text() {
        let mut c = ctx();
        let r = time_fn(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(0.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // datevalue_fn: 基本测试
    #[test]
    fn datevalue_basic() {
        let mut c = ctx();
        let r = datevalue_fn(&mut c, &[Value::Text("2023-01-01".into())]);
        if let Value::Number(v) = r {
            assert!(v > 40000.0, "DATEVALUE = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // datevalue_fn: 非文本参数
    #[test]
    fn datevalue_err_number() {
        let mut c = ctx();
        let r = datevalue_fn(&mut c, &[Value::Number(44927.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // timevalue_fn: 基本测试
    #[test]
    fn timevalue_basic() {
        let mut c = ctx();
        let r = timevalue_fn(&mut c, &[Value::Text("12:00:00".into())]);
        if let Value::Number(v) = r {
            assert!((v - 0.5).abs() < 1e-6, "TIMEVALUE = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // timevalue_fn: 非文本参数
    #[test]
    fn timevalue_err_number() {
        let mut c = ctx();
        let r = timevalue_fn(&mut c, &[Value::Number(0.5)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // hour_fn: 基本测试
    #[test]
    fn hour_fn_basic() {
        let mut c = ctx();
        let r = hour_fn(&mut c, &[Value::Number(0.5)]);
        assert_eq!(r, Value::Number(12.0));
    }

    // hour_fn: 非数字参数
    #[test]
    fn hour_fn_err_text() {
        let mut c = ctx();
        let r = hour_fn(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // minute_fn: 基本测试
    #[test]
    fn minute_fn_basic() {
        let mut c = ctx();
        let r = minute_fn(&mut c, &[Value::Number(0.5)]);
        assert_eq!(r, Value::Number(0.0));
    }

    // minute_fn: 非数字参数
    #[test]
    fn minute_fn_err_text() {
        let mut c = ctx();
        let r = minute_fn(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // second_fn: 基本测试
    #[test]
    fn second_fn_basic() {
        let mut c = ctx();
        let r = second_fn(&mut c, &[Value::Number(0.5)]);
        assert_eq!(r, Value::Number(0.0));
    }

    // second_fn: 非数字参数
    #[test]
    fn second_fn_err_text() {
        let mut c = ctx();
        let r = second_fn(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // weekday_fn: 非数字参数
    #[test]
    fn weekday_fn_err_text() {
        let mut c = ctx();
        let r = weekday_fn(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // weeknum_fn: 非数字参数
    #[test]
    fn weeknum_fn_err_text() {
        let mut c = ctx();
        let r = weeknum_fn(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // edate_fn: 非数字参数
    #[test]
    fn edate_fn_err_text() {
        let mut c = ctx();
        let r = edate_fn(
            &mut c,
            &[Value::Text("abc".into()), Value::Number(1.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // eomonth_fn: 非数字参数
    #[test]
    fn eomonth_fn_err_text() {
        let mut c = ctx();
        let r = eomonth_fn(
            &mut c,
            &[Value::Text("abc".into()), Value::Number(1.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // datedif_fn: 非数字参数
    #[test]
    fn datedif_fn_err_text() {
        let mut c = ctx();
        let r = datedif_fn(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(44927.0),
                Value::Text("Y".into()),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // days_fn: 基本测试
    #[test]
    fn days_fn_basic_v2() {
        let mut c = ctx();
        let r = days_fn(
            &mut c,
            &[Value::Number(44927.0), Value::Number(44562.0)],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "DAYS = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // days_fn: 非数字参数
    #[test]
    fn days_fn_err_text() {
        let mut c = ctx();
        let r = days_fn(
            &mut c,
            &[Value::Text("abc".into()), Value::Number(44562.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // days360_fn: 基本测试 (v2)
    #[test]
    fn days360_fn_basic_v2() {
        let mut c = ctx();
        let r = days360_fn(
            &mut c,
            &[
                Value::Number(44927.0),
                Value::Number(44957.0),
                Value::Bool(false),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0, "DAYS360 = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // days360_fn: 非数字参数
    #[test]
    fn days360_fn_err_text() {
        let mut c = ctx();
        let r = days360_fn(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(44957.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // yearfrac_fn: 基本测试
    #[test]
    fn yearfrac_fn_basic() {
        let mut c = ctx();
        let r = yearfrac_fn(
            &mut c,
            &[
                Value::Number(44927.0),
                Value::Number(45292.0),
                Value::Number(0.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 2.0, "YEARFRAC = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // yearfrac_fn: 非数字参数
    #[test]
    fn yearfrac_fn_err_text() {
        let mut c = ctx();
        let r = yearfrac_fn(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(45292.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // networkdays_fn: 非数字参数
    #[test]
    fn networkdays_fn_err_text() {
        let mut c = ctx();
        let r = networkdays_fn(
            &mut c,
            &[Value::Text("abc".into()), Value::Number(44927.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // workday_fn: 非数字参数
    #[test]
    fn workday_fn_err_text() {
        let mut c = ctx();
        let r = workday_fn(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(10.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // networkdays_intl_fn: 非数字参数
    #[test]
    fn networkdays_intl_fn_err_text() {
        let mut c = ctx();
        let r = networkdays_intl_fn(
            &mut c,
            &[Value::Text("abc".into()), Value::Number(44927.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // workday_intl_fn: 非数字参数
    #[test]
    fn workday_intl_fn_err_text() {
        let mut c = ctx();
        let r = workday_intl_fn(
            &mut c,
            &[
                Value::Text("abc".into()),
                Value::Number(10.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // date_fn: 月份溢出
    #[test]
    fn date_fn_month_overflow() {
        let mut c = ctx();
        // 2023年13月 → 2024年1月
        let r = date_fn(
            &mut c,
            &[
                Value::Number(2023.0),
                Value::Number(13.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 40000.0, "DATE month overflow = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // date_fn: 日期溢出
    #[test]
    fn date_fn_day_overflow() {
        let mut c = ctx();
        // 2023年1月32日 → 2023年2月1日
        let r = date_fn(
            &mut c,
            &[
                Value::Number(2023.0),
                Value::Number(1.0),
                Value::Number(32.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 40000.0, "DATE day overflow = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }

    // datedif: 单位 YD (v2)
    #[test]
    fn datedif_unit_yd_v2() {
        let mut c = ctx();
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(44927.0),
                Value::Number(45292.0),
                Value::Text("YD".into()),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v >= 0.0 && v < 366.0, "DATEDIF YD = {v}");
        } else {
            panic!("Expected number, got {r:?}");
        }
    }
