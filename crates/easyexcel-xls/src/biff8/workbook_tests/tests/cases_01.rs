    #[test]
    // 语义敏感：POI 序列号必须是精确的 f64 常量（1.0/59.0/61.0/0.0），
    // 严格比较即测试意图，不能改用误差容忍。
    #[allow(clippy::float_cmp)]
    fn excel_serials_match_poi_across_1900_leap_bug_and_1904_epoch() {
        assert_eq!(
            date_to_excel_serial_with_windowing(
                NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
                false
            ),
            1.0
        );
        assert_eq!(
            date_to_excel_serial_with_windowing(
                NaiveDate::from_ymd_opt(1900, 2, 28).unwrap(),
                false
            ),
            59.0
        );
        assert_eq!(
            date_to_excel_serial_with_windowing(
                NaiveDate::from_ymd_opt(1900, 3, 1).unwrap(),
                false
            ),
            61.0
        );
        assert_eq!(
            date_to_excel_serial_with_windowing(NaiveDate::from_ymd_opt(1904, 1, 1).unwrap(), true),
            0.0
        );
    }
