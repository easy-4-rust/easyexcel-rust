    #[test]
    fn format_without_pattern_returns_java_plain_string() {
        // 对应 Java：`NumberUtils.format` 在无格式时返回 `toPlainString()`
        assert_eq!(
            format_decimal(&decimal("1.50"), false, None, NumberRoundingMode::HalfUp).unwrap(),
            "1.50"
        );
        assert_eq!(
            format_decimal(
                &decimal("-1.50"),
                true,
                Some(""),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "-1.50"
        );
    }

    #[test]
    fn format_non_finite_negative_infinity_without_pattern() {
        // 对应 Java：`DecimalFormat` 对 `-Infinity` 使用负子模式
        assert_eq!(
            format_non_finite(NonFiniteNumber::NegativeInfinity, None).unwrap(),
            "-Infinity"
        );
        assert_eq!(
            format_non_finite(NonFiniteNumber::NegativeInfinity, Some("#")).unwrap(),
            "-∞"
        );
    }

    #[test]
    fn invalid_patterns_are_rejected_with_java_messages() {
        // 对应 Java：`DecimalFormat` 非法模式抛 `IllegalArgumentException`
        for pattern in ["abc", "0a0", "0.00E#", ",", "0;0;0", "'0.00", "0%‰"] {
            assert!(
                format_decimal(
                    &decimal("1.5"),
                    false,
                    Some(pattern),
                    NumberRoundingMode::HalfUp
                )
                .is_err(),
                "pattern {pattern:?} should be rejected"
            );
            assert!(parse_decimal("1.5", Some(pattern)).is_err());
        }
    }

    #[test]
    fn plain_format_pads_integer_and_strips_trailing_fraction_zeros() {
        // 对应 Java：`DecimalFormat` 整数位补零、小数位去除末尾零
        assert_eq!(
            format_decimal(
                &decimal("5"),
                false,
                Some("00.0"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "05.0"
        );
        assert_eq!(
            format_decimal(
                &decimal("2"),
                false,
                Some("#.##"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "2"
        );
        assert_eq!(
            format_decimal(
                &decimal("1.2"),
                false,
                Some("0.00"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "1.20"
        );
    }

    #[test]
    fn scientific_format_zero_and_exponent_carry() {
        // 对应 Java：`DecimalFormat` 科学计数法，零值指数为 0，舍入进位后修正指数
        assert_eq!(
            format_decimal(
                &decimal("0"),
                false,
                Some("0.00E00"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "0.00E00"
        );
        assert_eq!(
            format_decimal(
                &decimal("12.5"),
                false,
                Some("0.00E00"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "1.25E01"
        );
        assert_eq!(
            format_decimal(
                &decimal("9.95"),
                false,
                Some("0.0E0"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "1.0E1"
        );
    }

    #[test]
    fn parse_with_exponent_signs_and_quoted_apostrophes() {
        // 对应 Java：`DecimalFormat.parse` 支持指数正负号与转义单引号
        assert_eq!(
            parse_decimal("1.24E-03", Some("0.00E00")).unwrap(),
            decimal("0.00124")
        );
        assert_eq!(
            parse_decimal("1.24E+03", Some("0.00E00")).unwrap(),
            decimal("1240")
        );
        assert_eq!(
            format_decimal(
                &decimal("1.5"),
                false,
                Some("'it''s'0.00"),
                NumberRoundingMode::HalfUp,
            )
            .unwrap(),
            "it's1.50"
        );
    }

    #[test]
    fn unnecessary_rounding_succeeds_when_value_fits_scale() {
        // 对应 Java：`RoundingMode.UNNECESSARY` 在无需舍入时直接返回
        assert_eq!(
            format_decimal(
                &decimal("1.00"),
                false,
                Some("0.00"),
                NumberRoundingMode::Unnecessary,
            )
            .unwrap(),
            "1.00"
        );
    }

    #[test]
    fn parse_short_and_long_match_java_wrapping_and_errors() {
        // 对应 Java：`NumberUtils.parseShort` / `parseLong` 低位截断回绕
        assert_eq!(parse_short("127").unwrap(), 127);
        assert_eq!(parse_short("-128.9").unwrap(), -128);
        assert_eq!(parse_short("65535.9").unwrap(), -1);
        assert!(parse_short("abc").is_err());
        assert_eq!(parse_long("123").unwrap(), 123);
        assert_eq!(parse_long("18446744073709551615.9").unwrap(), -1);
        assert!(parse_long("abc").is_err());
        assert!(parse_integer("abc").is_err());
    }

    #[test]
    // 1.5 / 2.5 均可被 f32/f64 二进制精确表示，精确比较正是本测试的意图
    #[allow(clippy::float_cmp)]
    fn parse_float_double_and_big_int_match_java() {
        // 对应 Java：`NumberUtils.parseFloat` / `parseDouble` / Apache Commons `createBigInteger`
        assert_eq!(parse_float("1.5").unwrap(), 1.5);
        assert!(parse_float("1e100").unwrap().is_infinite());
        assert_eq!(parse_double("2.5").unwrap(), 2.5);
        assert!(parse_double("1e100000").unwrap().is_infinite());
        assert_eq!(parse_big_int("123").unwrap(), BigInt::from(123));
        assert_eq!(parse_big_int("-123").unwrap(), BigInt::from(-123));
        assert!(parse_big_int("abc").is_err());
    }

    #[test]
    fn parse_byte_negative_values_sign_extend_low_byte() {
        // 对应 Java：`NumberUtils.parseByte` 使用二进制补码低位字节（符号扩展）
        assert_eq!(parse_byte("-1.0").unwrap(), -1);
        assert_eq!(parse_byte("127.9").unwrap(), 127);
    }

    #[test]
    fn excel_date_format_code_translates_java_writer_patterns() {
        assert_eq!(excel_date_format_code(None, "yyyy-mm-dd"), "yyyy-mm-dd");
        assert_eq!(
            excel_date_format_code(Some("%Y/%m/%d %H:%M:%S"), "unused"),
            "yyyy/mm/dd hh:mm:ss"
        );
    }
