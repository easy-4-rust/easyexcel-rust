    #[test]
    fn test_bin2dec() {
        let mut ctx = c();
        assert_eq!(
            bin2dec(&mut ctx, &[Value::Text("1010".into())]),
            Value::Number(10.0)
        );
        assert_eq!(
            bin2dec(&mut ctx, &[Value::Text("0".into())]),
            Value::Number(0.0)
        );
        // Negative: 1111111111 in 10-bit two's complement = -1
        assert_eq!(
            bin2dec(&mut ctx, &[Value::Text("1111111111".into())]),
            Value::Number(-1.0)
        );
    }

    #[test]
    fn test_dec2bin() {
        let mut ctx = c();
        assert_eq!(
            dec2bin(&mut ctx, &[Value::Number(10.0)]),
            Value::Text("1010".into())
        );
        assert_eq!(
            dec2bin(&mut ctx, &[Value::Number(0.0)]),
            Value::Text("0".into())
        );
        // DEC2BIN(-1) = 1111111111 (10-bit two's complement)
        if let Value::Text(s) = dec2bin(&mut ctx, &[Value::Number(-1.0)]) {
            assert_eq!(s, "1111111111");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_dec2bin_overflow() {
        let mut ctx = c();
        // 512 > 511 (max for 10-bit signed)
        assert_eq!(
            dec2bin(&mut ctx, &[Value::Number(512.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn test_dec2hex() {
        let mut ctx = c();
        assert_eq!(
            dec2hex(&mut ctx, &[Value::Number(255.0)]),
            Value::Text("FF".into())
        );
        assert_eq!(
            dec2hex(&mut ctx, &[Value::Number(0.0)]),
            Value::Text("0".into())
        );
        // With places
        assert_eq!(
            dec2hex(&mut ctx, &[Value::Number(255.0), Value::Number(4.0)]),
            Value::Text("00FF".into())
        );
    }

    #[test]
    fn test_hex2dec() {
        let mut ctx = c();
        assert_eq!(
            hex2dec(&mut ctx, &[Value::Text("FF".into())]),
            Value::Number(255.0)
        );
        assert_eq!(
            hex2dec(&mut ctx, &[Value::Text("0".into())]),
            Value::Number(0.0)
        );
        // Negative: FFFFFFFFFF = -1 in 40-bit two's complement
        if let Value::Number(v) = hex2dec(&mut ctx, &[Value::Text("FFFFFFFFFF".into())]) {
            assert_eq!(v, -1.0);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_oct2dec() {
        let mut ctx = c();
        assert_eq!(
            oct2dec(&mut ctx, &[Value::Text("777".into())]),
            Value::Number(511.0)
        );
        assert_eq!(
            oct2dec(&mut ctx, &[Value::Text("0".into())]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn test_dec2oct() {
        let mut ctx = c();
        assert_eq!(
            dec2oct(&mut ctx, &[Value::Number(8.0)]),
            Value::Text("10".into())
        );
    }

    #[test]
    fn test_bin2hex() {
        let mut ctx = c();
        assert_eq!(
            bin2hex(&mut ctx, &[Value::Text("11111111".into())]),
            Value::Text("FF".into())
        );
    }

    #[test]
    fn test_hex2bin() {
        let mut ctx = c();
        assert_eq!(
            hex2bin(&mut ctx, &[Value::Text("F".into())]),
            Value::Text("1111".into())
        );
    }

    // --- Bitwise ---

    #[test]
    fn test_bitwise() {
        let mut ctx = c();
        assert_eq!(
            bitand(&mut ctx, &[Value::Number(13.0), Value::Number(25.0)]),
            Value::Number(9.0)
        );
        assert_eq!(
            bitor(&mut ctx, &[Value::Number(13.0), Value::Number(25.0)]),
            Value::Number(29.0)
        );
        assert_eq!(
            bitxor(&mut ctx, &[Value::Number(13.0), Value::Number(25.0)]),
            Value::Number(20.0)
        );
        assert_eq!(
            bitlshift(&mut ctx, &[Value::Number(4.0), Value::Number(2.0)]),
            Value::Number(16.0)
        );
        assert_eq!(
            bitrshift(&mut ctx, &[Value::Number(16.0), Value::Number(2.0)]),
            Value::Number(4.0)
        );
    }

    #[test]
    fn test_bitwise_negative_error() {
        let mut ctx = c();
        assert_eq!(
            bitand(&mut ctx, &[Value::Number(-1.0), Value::Number(1.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- DELTA / GESTEP ---

    #[test]
    fn test_delta_gestep() {
        let mut ctx = c();
        assert_eq!(
            delta(&mut ctx, &[Value::Number(5.0), Value::Number(5.0)]),
            Value::Number(1.0)
        );
        assert_eq!(
            delta(&mut ctx, &[Value::Number(5.0), Value::Number(4.0)]),
            Value::Number(0.0)
        );
        assert_eq!(
            gestep(&mut ctx, &[Value::Number(5.0), Value::Number(4.0)]),
            Value::Number(1.0)
        );
        assert_eq!(
            gestep(&mut ctx, &[Value::Number(3.0), Value::Number(4.0)]),
            Value::Number(0.0)
        );
        // DELTA with default second arg (0)
        assert_eq!(delta(&mut ctx, &[Value::Number(0.0)]), Value::Number(1.0));
    }

    // --- CONVERT ---

    #[test]
    fn test_convert_weight() {
        let mut ctx = c();
        // 1 lbm = 0.453592 kg
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("lbm".into()),
                Value::Text("kg".into()),
            ],
        ) {
            assert!(approx(v, 0.4536), "lbm→kg = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_convert_distance() {
        let mut ctx = c();
        // 1 mi = 1609.344 m
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("mi".into()),
                Value::Text("m".into()),
            ],
        ) {
            assert!(approx(v, 1609.344), "mi→m = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_convert_temperature() {
        let mut ctx = c();
        // 100 C = 212 F
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Text("C".into()),
                Value::Text("F".into()),
            ],
        ) {
            assert!(approx(v, 212.0), "C→F = {v}");
        } else {
            panic!("Expected number");
        }

        // 0 C = 273.15 K
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Text("C".into()),
                Value::Text("K".into()),
            ],
        ) {
            assert!(approx(v, 273.15), "C→K = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_convert_unknown_unit() {
        let mut ctx = c();
        assert_eq!(
            convert(
                &mut ctx,
                &[
                    Value::Number(1.0),
                    Value::Text("frobble".into()),
                    Value::Text("kg".into()),
                ]
            ),
            Value::Error(CellError::NA)
        );
    }

    #[test]
    fn test_convert_metric_prefix() {
        let mut ctx = c();
        // 1 km = 1000 m
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("km".into()),
                Value::Text("m".into()),
            ],
        ) {
            assert!(approx(v, 1000.0), "km→m = {v}");
        } else {
            panic!("Expected number");
        }
    }

    // --- ERF / ERFC ---

    #[test]
    fn test_erf() {
        let mut ctx = c();
        // erf(0) = 0
        assert_eq!(erf_fn(&mut ctx, &[Value::Number(0.0)]), Value::Number(0.0));
        // erf(1) ≈ 0.8427
        if let Value::Number(v) = erf_fn(&mut ctx, &[Value::Number(1.0)]) {
            assert!(approx(v, 0.8427), "erf(1) = {v}");
        } else {
            panic!("Expected number");
        }
        // erf(lower, upper) = erf(upper) - erf(lower)
        if let Value::Number(v) = erf_fn(&mut ctx, &[Value::Number(0.0), Value::Number(1.0)]) {
            assert!(approx(v, 0.8427), "erf(0,1) = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_erfc() {
        let mut ctx = c();
        // erfc(0) = 1
        assert_eq!(erfc_fn(&mut ctx, &[Value::Number(0.0)]), Value::Number(1.0));
        // erfc(1) ≈ 0.1573
        if let Value::Number(v) = erfc_fn(&mut ctx, &[Value::Number(1.0)]) {
            assert!(approx(v, 0.1573), "erfc(1) = {v}");
        } else {
            panic!("Expected number");
        }
    }

    // --- Complex numbers ---

    #[test]
    fn test_complex_create() {
        let mut ctx = c();
        assert_eq!(
            complex(&mut ctx, &[Value::Number(3.0), Value::Number(4.0)]),
            Value::Text("3+4i".into())
        );
        assert_eq!(
            complex(&mut ctx, &[Value::Number(3.0), Value::Number(-4.0)]),
            Value::Text("3-4i".into())
        );
        assert_eq!(
            complex(&mut ctx, &[Value::Number(0.0), Value::Number(1.0)]),
            Value::Text("i".into())
        );
    }

    #[test]
    fn test_imabs() {
        let mut ctx = c();
        // |3+4i| = 5
        if let Value::Number(v) = imabs(&mut ctx, &[Value::Text("3+4i".into())]) {
            assert!(approx(v, 5.0), "IMABS = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_imreal_imaginary() {
        let mut ctx = c();
        assert_eq!(
            imreal(&mut ctx, &[Value::Text("3+4i".into())]),
            Value::Number(3.0)
        );
        assert_eq!(
            imaginary(&mut ctx, &[Value::Text("3+4i".into())]),
            Value::Number(4.0)
        );
        assert_eq!(
            imreal(&mut ctx, &[Value::Text("5".into())]),
            Value::Number(5.0)
        );
        assert_eq!(
            imaginary(&mut ctx, &[Value::Text("5".into())]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn test_imconjugate() {
        let mut ctx = c();
        assert_eq!(
            imconjugate(&mut ctx, &[Value::Text("3+4i".into())]),
            Value::Text("3-4i".into())
        );
    }

    #[test]
    fn test_improduct() {
        let mut ctx = c();
        // (3+4i)(3-4i) = 9+16 = 25
        if let Value::Text(s) = improduct(
            &mut ctx,
            &[Value::Text("3+4i".into()), Value::Text("3-4i".into())],
        ) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 25.0) && approx(c.im, 0.0), "product = {s}");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_imdiv() {
        let mut ctx = c();
        // (3+4i)/(3+4i) = 1
        if let Value::Text(s) = imdiv(
            &mut ctx,
            &[Value::Text("3+4i".into()), Value::Text("3+4i".into())],
        ) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 1.0) && approx(c.im, 0.0), "div = {s}");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_imexp() {
        let mut ctx = c();
        // exp(i*pi) ≈ -1 (Euler's identity: exp(iπ) = cos(π)+i*sin(π) = -1)
        if let Value::Text(s) = imexp(&mut ctx, &[Value::Text("3.14159265358979i".into())]) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, -1.0) && approx(c.im, 0.0), "exp(iπ) = {s}");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_imsqrt() {
        let mut ctx = c();
        // sqrt(-1) = i
        if let Value::Text(s) = imsqrt(&mut ctx, &[Value::Text("-1".into())]) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 0.0) && approx(c.im, 1.0), "sqrt(-1) = {s}");
        } else if let Value::Number(v) = imsqrt(&mut ctx, &[Value::Text("-1".into())]) {
            panic!("Expected text but got Number({v})");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_imsum_imsub() {
        let mut ctx = c();
        if let Value::Text(s) = imsum(
            &mut ctx,
            &[Value::Text("3+4i".into()), Value::Text("1+2i".into())],
        ) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 4.0) && approx(c.im, 6.0), "sum = {s}");
        } else {
            panic!("Expected text");
        }

        if let Value::Text(s) = imsub(
            &mut ctx,
            &[Value::Text("3+4i".into()), Value::Text("1+2i".into())],
        ) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 2.0) && approx(c.im, 2.0), "sub = {s}");
        } else {
            panic!("Expected text");
        }
    }
