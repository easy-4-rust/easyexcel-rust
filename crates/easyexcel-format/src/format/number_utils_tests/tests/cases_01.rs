    #[test]
    fn decimal_format_matches_java_golden_patterns() {
        for (pattern, value, expected) in [
            ("#.##%", "1.235", "123.5%"),
            ("#", "1.235", "1"),
            ("0.00", "1.235", "1.24"),
            ("#,##0.00", "1234.5", "1,234.50"),
            ("0.00;[neg]0.00", "-1.235", "[neg]1.24"),
            ("0.00E00", "1235", "1.24E03"),
        ] {
            let value = decimal(value);
            assert_eq!(
                format_decimal(&value, value < 0, Some(pattern), NumberRoundingMode::HalfUp,)
                    .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn decimal_parse_matches_java_parse_position_behavior() {
        assert_eq!(
            parse_decimal("12.34%", Some("#.##%")).unwrap(),
            decimal("0.1234")
        );
        assert!(parse_decimal("12.34", Some("#.##%")).is_err());
        assert_eq!(
            parse_decimal("1,234.50", Some("#,##0.00")).unwrap(),
            decimal("1234.50")
        );
        assert_eq!(
            parse_decimal("1.00abc", Some("0.00")).unwrap(),
            decimal("1.00")
        );
        assert!(parse_decimal(" 1.00", Some("0.00")).is_err());
        assert!(parse_decimal("abc1.00", Some("0.00")).is_err());
    }

    #[test]
    fn no_format_is_full_input_big_decimal_and_unnecessary_rejects_rounding() {
        assert_eq!(parse_integer("1.00").unwrap(), 1);
        assert_eq!(parse_byte("255.9").unwrap(), -1);
        assert!(parse_big_decimal(" 1.00").is_err());
        assert!(parse_big_decimal("1.00 ").is_err());
        assert!(
            format_decimal(
                &decimal("1.001"),
                false,
                Some("0.00"),
                NumberRoundingMode::Unnecessary,
            )
            .is_err()
        );
    }

    #[test]
    fn all_java_rounding_modes_match_direction_and_tie_rules() {
        for (mode, positive, negative) in [
            (NumberRoundingMode::Up, "1.3", "-1.3"),
            (NumberRoundingMode::Down, "1.2", "-1.2"),
            (NumberRoundingMode::Ceiling, "1.3", "-1.2"),
            (NumberRoundingMode::Floor, "1.2", "-1.3"),
        ] {
            assert_eq!(
                format_decimal(&decimal("1.21"), false, Some("0.0"), mode).unwrap(),
                positive
            );
            assert_eq!(
                format_decimal(&decimal("-1.21"), true, Some("0.0"), mode).unwrap(),
                negative
            );
        }
        for (mode, expected) in [
            (NumberRoundingMode::HalfUp, "1.3"),
            (NumberRoundingMode::HalfDown, "1.2"),
            (NumberRoundingMode::HalfEven, "1.2"),
        ] {
            assert_eq!(
                format_decimal(&decimal("1.25"), false, Some("0.0"), mode).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn quoted_affixes_per_mille_and_scientific_parse_are_supported() {
        assert_eq!(
            format_decimal(
                &decimal("12.5"),
                false,
                Some("'USD '0.00"),
                NumberRoundingMode::HalfUp,
            )
            .unwrap(),
            "USD 12.50"
        );
        assert_eq!(
            format_decimal(
                &decimal("0.01234"),
                false,
                Some("#.##‰"),
                NumberRoundingMode::HalfUp,
            )
            .unwrap(),
            "12.34‰"
        );
        assert_eq!(
            parse_decimal("1.24E03", Some("0.00E00")).unwrap(),
            decimal("1240")
        );
    }
