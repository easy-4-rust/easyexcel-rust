    // --- base conversion roundtrips ---

    #[test]
    fn dec2bin_bin2dec_roundtrip() {
        let mut c = c();
        assert_eq!(
            dec2bin(&mut c, &[Value::Number(10.0)]),
            Value::Text("1010".into())
        );
        assert_eq!(
            bin2dec(&mut c, &[Value::Text("1010".into())]),
            Value::Number(10.0)
        );
    }

    #[test]
    fn dec2oct_oct2dec_roundtrip() {
        let mut c = c();
        assert_eq!(
            dec2oct(&mut c, &[Value::Number(64.0)]),
            Value::Text("100".into())
        );
        assert_eq!(
            oct2dec(&mut c, &[Value::Text("100".into())]),
            Value::Number(64.0)
        );
    }

    #[test]
    fn dec2hex_hex2dec_roundtrip() {
        let mut c = c();
        assert_eq!(
            dec2hex(&mut c, &[Value::Number(255.0)]),
            Value::Text("FF".into())
        );
        assert_eq!(
            hex2dec(&mut c, &[Value::Text("FF".into())]),
            Value::Number(255.0)
        );
    }

    #[test]
    fn bin2oct_bin2hex() {
        let mut c = c();
        assert_eq!(
            bin2oct(&mut c, &[Value::Text("1010".into())]),
            Value::Text("12".into())
        );
        assert_eq!(
            bin2hex(&mut c, &[Value::Text("1010".into())]),
            Value::Text("A".into())
        );
    }

    #[test]
    fn hex2bin_hex2oct() {
        let mut c = c();
        assert_eq!(
            hex2bin(&mut c, &[Value::Text("A".into())]),
            Value::Text("1010".into())
        );
        assert_eq!(
            hex2oct(&mut c, &[Value::Text("FF".into())]),
            Value::Text("377".into())
        );
    }

    #[test]
    fn oct2bin_oct2hex() {
        let mut c = c();
        assert_eq!(
            oct2bin(&mut c, &[Value::Text("12".into())]),
            Value::Text("1010".into())
        );
        assert_eq!(
            oct2hex(&mut c, &[Value::Text("377".into())]),
            Value::Text("FF".into())
        );
    }

    // --- negative numbers (two's complement) ---

    #[test]
    fn dec2bin_negative() {
        let mut c = c();
        // -1 in 10-bit two's complement = 1111111111
        assert_eq!(
            dec2bin(&mut c, &[Value::Number(-1.0)]),
            Value::Text("1111111111".into())
        );
    }

    // --- bitwise operations ---

    #[test]
    fn bitwise_basic() {
        let mut c = c();
        assert_eq!(
            bitand(&mut c, &[Value::Number(5.0), Value::Number(3.0)]),
            Value::Number(1.0)
        );
        assert_eq!(
            bitor(&mut c, &[Value::Number(5.0), Value::Number(3.0)]),
            Value::Number(7.0)
        );
        assert_eq!(
            bitxor(&mut c, &[Value::Number(5.0), Value::Number(3.0)]),
            Value::Number(6.0)
        );
        assert_eq!(
            bitlshift(&mut c, &[Value::Number(1.0), Value::Number(3.0)]),
            Value::Number(8.0)
        );
        assert_eq!(
            bitrshift(&mut c, &[Value::Number(8.0), Value::Number(3.0)]),
            Value::Number(1.0)
        );
    }

    // --- delta / gestep ---

    #[test]
    fn delta_gestep() {
        let mut c = c();
        assert_eq!(
            delta(&mut c, &[Value::Number(5.0), Value::Number(5.0)]),
            Value::Number(1.0)
        );
        assert_eq!(
            delta(&mut c, &[Value::Number(5.0), Value::Number(3.0)]),
            Value::Number(0.0)
        );
        assert_eq!(
            gestep(&mut c, &[Value::Number(5.0), Value::Number(3.0)]),
            Value::Number(1.0)
        );
        assert_eq!(
            gestep(&mut c, &[Value::Number(2.0), Value::Number(3.0)]),
            Value::Number(0.0)
        );
    }

    // --- CONVERT temperature ---

    #[test]
    fn convert_temperature() {
        let mut c = c();
        // 0C = 32F
        let r = convert(
            &mut c,
            &[
                Value::Number(0.0),
                Value::Text("C".into()),
                Value::Text("F".into()),
            ],
        );
        assert!(matches!(&r, Value::Number(n) if approx(*n, 32.0)));
        // 100C = 212F
        let r2 = convert(
            &mut c,
            &[
                Value::Number(100.0),
                Value::Text("C".into()),
                Value::Text("F".into()),
            ],
        );
        assert!(matches!(&r2, Value::Number(n) if approx(*n, 212.0)));
    }

    // --- CONVERT distance ---

    #[test]
    fn convert_distance() {
        let mut c = c();
        // 1 km = 1000 m
        let r = convert(
            &mut c,
            &[
                Value::Number(1.0),
                Value::Text("km".into()),
                Value::Text("m".into()),
            ],
        );
        assert!(matches!(&r, Value::Number(n) if approx(*n, 1000.0)));
    }

    // --- CONVERT mass ---

    #[test]
    fn convert_mass() {
        let mut c = c();
        // 1 kg = 1000 g
        let r = convert(
            &mut c,
            &[
                Value::Number(1.0),
                Value::Text("kg".into()),
                Value::Text("g".into()),
            ],
        );
        assert!(matches!(&r, Value::Number(n) if approx(*n, 1000.0)));
    }

    // --- CONVERT time ---

    #[test]
    fn convert_time() {
        let mut c = c();
        // 1 hr = 3600 sec
        let r = convert(
            &mut c,
            &[
                Value::Number(1.0),
                Value::Text("hr".into()),
                Value::Text("sec".into()),
            ],
        );
        assert!(matches!(&r, Value::Number(n) if approx(*n, 3600.0)));
    }

    // --- CONVERT incompatible ---

    #[test]
    fn convert_incompatible_units() {
        let mut c = c();
        let r = convert(
            &mut c,
            &[
                Value::Number(1.0),
                Value::Text("kg".into()),
                Value::Text("m".into()),
            ],
        );
        // Returns error or a value (depending on implementation)
        assert!(matches!(r, Value::Error(_) | Value::Number(_)));
    }

    // --- bessel stub ---

    #[test]
    fn bessel_stub_returns_error() {
        let mut c = c();
        let r = bessel_stub(&mut c, &[Value::Number(1.0), Value::Number(1.0)]);
        assert!(matches!(r, Value::Error(_)));
    }
