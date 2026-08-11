    // --- ceiling_math / floor_math ---

    #[test]
    fn ceiling_math_positive() {
        let mut c = TestCtx::new();
        // CEILING.MATH(5.3) = 6
        assert_eq!(
            ceiling_math(&mut c, &[Value::Number(5.3)]),
            Value::Number(6.0)
        );
    }

    #[test]
    fn ceiling_math_negative_with_mode() {
        let mut c = TestCtx::new();
        // CEILING.MATH(-5.3, 1, 1) = -6 (floor away from zero with mode=1)
        assert_eq!(
            ceiling_math(&mut c, &[Value::Number(-5.3), Value::Number(1.0), Value::Number(1.0)]),
            Value::Number(-6.0)
        );
    }

    #[test]
    fn ceiling_math_negative_no_mode() {
        let mut c = TestCtx::new();
        // CEILING.MATH(-5.3) = -5 (ceiling toward zero)
        assert_eq!(
            ceiling_math(&mut c, &[Value::Number(-5.3)]),
            Value::Number(-5.0)
        );
    }

    #[test]
    fn ceiling_math_zero_significance() {
        let mut c = TestCtx::new();
        assert_eq!(
            ceiling_math(&mut c, &[Value::Number(5.3), Value::Number(0.0)]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn floor_math_positive() {
        let mut c = TestCtx::new();
        // FLOOR.MATH(5.7) = 5
        assert_eq!(
            floor_math(&mut c, &[Value::Number(5.7)]),
            Value::Number(5.0)
        );
    }

    #[test]
    fn floor_math_negative_with_mode() {
        let mut c = TestCtx::new();
        // FLOOR.MATH(-5.3, 1, 1) = -5 (ceil toward zero with mode=1)
        assert_eq!(
            floor_math(&mut c, &[Value::Number(-5.3), Value::Number(1.0), Value::Number(1.0)]),
            Value::Number(-5.0)
        );
    }

    #[test]
    fn floor_math_negative_no_mode() {
        let mut c = TestCtx::new();
        // FLOOR.MATH(-5.3) = -6
        assert_eq!(
            floor_math(&mut c, &[Value::Number(-5.3)]),
            Value::Number(-6.0)
        );
    }

    #[test]
    fn floor_math_zero_significance() {
        let mut c = TestCtx::new();
        assert_eq!(
            floor_math(&mut c, &[Value::Number(5.3), Value::Number(0.0)]),
            Value::Number(0.0)
        );
    }

    // --- ceiling / floor (classic) ---

    #[test]
    fn ceiling_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            ceiling(&mut c, &[Value::Number(5.3), Value::Number(0.5)]),
            Value::Number(5.5)
        );
    }

    #[test]
    fn ceiling_positive_negative_sig_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            ceiling(&mut c, &[Value::Number(5.3), Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn ceiling_zero_sig() {
        let mut c = TestCtx::new();
        assert_eq!(
            ceiling(&mut c, &[Value::Number(5.3), Value::Number(0.0)]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn floor_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            floor(&mut c, &[Value::Number(5.7), Value::Number(1.0)]),
            Value::Number(5.0)
        );
    }

    #[test]
    fn floor_positive_negative_sig_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            floor(&mut c, &[Value::Number(5.3), Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn floor_zero_sig_is_div0() {
        let mut c = TestCtx::new();
        assert_eq!(
            floor(&mut c, &[Value::Number(5.3), Value::Number(0.0)]),
            Value::Error(CellError::Div0)
        );
    }

    // --- ceiling_precise / floor_precise ---

    #[test]
    fn ceiling_precise_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            ceiling_precise(&mut c, &[Value::Number(5.3), Value::Number(0.5)]),
            Value::Number(5.5)
        );
    }

    #[test]
    fn ceiling_precise_negative_sig() {
        let mut c = TestCtx::new();
        // CEILING.PRECISE uses abs(significance)
        assert_eq!(
            ceiling_precise(&mut c, &[Value::Number(5.3), Value::Number(-0.5)]),
            Value::Number(5.5)
        );
    }

    #[test]
    fn ceiling_precise_zero_sig() {
        let mut c = TestCtx::new();
        assert_eq!(
            ceiling_precise(&mut c, &[Value::Number(5.3), Value::Number(0.0)]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn floor_precise_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            floor_precise(&mut c, &[Value::Number(5.7), Value::Number(0.5)]),
            Value::Number(5.5)
        );
    }

    #[test]
    fn floor_precise_zero_sig() {
        let mut c = TestCtx::new();
        assert_eq!(
            floor_precise(&mut c, &[Value::Number(5.7), Value::Number(0.0)]),
            Value::Number(0.0)
        );
    }

    // --- even / odd ---

    #[test]
    fn even_positive() {
        let mut c = TestCtx::new();
        assert_eq!(even(&mut c, &[Value::Number(3.0)]), Value::Number(4.0));
        assert_eq!(even(&mut c, &[Value::Number(2.0)]), Value::Number(2.0));
    }

    #[test]
    fn even_negative() {
        let mut c = TestCtx::new();
        assert_eq!(even(&mut c, &[Value::Number(-3.0)]), Value::Number(-4.0));
    }

    #[test]
    fn odd_positive() {
        let mut c = TestCtx::new();
        assert_eq!(odd(&mut c, &[Value::Number(2.0)]), Value::Number(3.0));
        assert_eq!(odd(&mut c, &[Value::Number(3.0)]), Value::Number(3.0));
    }

    #[test]
    fn odd_negative() {
        let mut c = TestCtx::new();
        assert_eq!(odd(&mut c, &[Value::Number(-2.0)]), Value::Number(-3.0));
    }

    #[test]
    fn odd_zero() {
        let mut c = TestCtx::new();
        assert_eq!(odd(&mut c, &[Value::Number(0.0)]), Value::Number(1.0));
    }

    // --- sign ---

    #[test]
    fn sign_positive() {
        let mut c = TestCtx::new();
        assert_eq!(sign(&mut c, &[Value::Number(5.0)]), Value::Number(1.0));
    }

    #[test]
    fn sign_negative() {
        let mut c = TestCtx::new();
        assert_eq!(sign(&mut c, &[Value::Number(-5.0)]), Value::Number(-1.0));
    }

    #[test]
    fn sign_zero() {
        let mut c = TestCtx::new();
        assert_eq!(sign(&mut c, &[Value::Number(0.0)]), Value::Number(0.0));
    }

    // --- quotient ---

    #[test]
    fn quotient_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            quotient(&mut c, &[Value::Number(7.0), Value::Number(2.0)]),
            Value::Number(3.0)
        );
    }

    #[test]
    fn quotient_div_zero() {
        let mut c = TestCtx::new();
        assert_eq!(
            quotient(&mut c, &[Value::Number(7.0), Value::Number(0.0)]),
            Value::Error(CellError::Div0)
        );
    }

    // --- power ---

    #[test]
    fn power_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            power(&mut c, &[Value::Number(2.0), Value::Number(3.0)]),
            Value::Number(8.0)
        );
    }

    #[test]
    fn power_negative_base_fractional_exp() {
        let mut c = TestCtx::new();
        // (-2)^0.5 = NaN → #NUM!
        let r = power(&mut c, &[Value::Number(-2.0), Value::Number(0.5)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // --- log ---

    #[test]
    fn log_default_base() {
        let mut c = TestCtx::new();
        // LOG(100) = log10(100) = 2
        let r = log(&mut c, &[Value::Number(100.0)]);
        if let Value::Number(v) = r {
            assert!((v - 2.0).abs() < 1e-10, "LOG(100) = {v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn log_custom_base() {
        let mut c = TestCtx::new();
        // LOG(8, 2) = 3
        let r = log(&mut c, &[Value::Number(8.0), Value::Number(2.0)]);
        if let Value::Number(v) = r {
            assert!((v - 3.0).abs() < 1e-10, "LOG(8,2) = {v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn log_negative_x_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            log(&mut c, &[Value::Number(-10.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn log_base_one_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            log(&mut c, &[Value::Number(10.0), Value::Number(1.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- atan2 ---

    #[test]
    fn atan2_basic() {
        let mut c = TestCtx::new();
        // ATAN2(1, 1) = π/4
        let r = atan2(&mut c, &[Value::Number(1.0), Value::Number(1.0)]);
        if let Value::Number(v) = r {
            assert!((v - std::f64::consts::FRAC_PI_4).abs() < 1e-10, "ATAN2(1,1) = {v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn atan2_both_zero_is_div0() {
        let mut c = TestCtx::new();
        assert_eq!(
            atan2(&mut c, &[Value::Number(0.0), Value::Number(0.0)]),
            Value::Error(CellError::Div0)
        );
    }

    // --- factdouble ---

    #[test]
    fn factdouble_basic() {
        let mut c = TestCtx::new();
        // FACTDOUBLE(6) = 6*4*2 = 48
        assert_eq!(
            factdouble(&mut c, &[Value::Number(6.0)]),
            Value::Number(48.0)
        );
        // FACTDOUBLE(5) = 5*3*1 = 15
        assert_eq!(
            factdouble(&mut c, &[Value::Number(5.0)]),
            Value::Number(15.0)
        );
    }

    #[test]
    fn factdouble_negative_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            factdouble(&mut c, &[Value::Number(-3.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- combina / permut / permutationa ---

    #[test]
    fn combina_basic() {
        let mut c = TestCtx::new();
        // COMBINA(3, 2) = C(3+2-1, 2) = C(4,2) = 6
        assert_eq!(
            combina(&mut c, &[Value::Number(3.0), Value::Number(2.0)]),
            Value::Number(6.0)
        );
    }

    #[test]
    fn combina_negative_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            combina(&mut c, &[Value::Number(-1.0), Value::Number(2.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn permut_basic() {
        let mut c = TestCtx::new();
        // PERMUT(5, 2) = 5*4 = 20
        assert_eq!(
            permut(&mut c, &[Value::Number(5.0), Value::Number(2.0)]),
            Value::Number(20.0)
        );
    }

    #[test]
    fn permut_k_gt_n_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            permut(&mut c, &[Value::Number(2.0), Value::Number(5.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn permutationa_basic() {
        let mut c = TestCtx::new();
        // PERMUTATIONA(3, 2) = 3^2 = 9
        assert_eq!(
            permutationa(&mut c, &[Value::Number(3.0), Value::Number(2.0)]),
            Value::Number(9.0)
        );
    }

    #[test]
    fn permutationa_negative_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            permutationa(&mut c, &[Value::Number(-1.0), Value::Number(2.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- gcd / lcm ---

    #[test]
    fn gcd_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            gcd(&mut c, &[Value::Number(12.0), Value::Number(8.0)]),
            Value::Number(4.0)
        );
    }

    #[test]
    fn gcd_negative_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            gcd(&mut c, &[Value::Number(-12.0), Value::Number(8.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn lcm_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            lcm(&mut c, &[Value::Number(4.0), Value::Number(6.0)]),
            Value::Number(12.0)
        );
    }

    #[test]
    fn lcm_with_zero() {
        let mut c = TestCtx::new();
        assert_eq!(
            lcm(&mut c, &[Value::Number(4.0), Value::Number(0.0)]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn lcm_negative_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            lcm(&mut c, &[Value::Number(-4.0), Value::Number(6.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- base / decimal ---

    #[test]
    fn base_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            base(&mut c, &[Value::Number(10.0), Value::Number(2.0)]),
            Value::Text("1010".into())
        );
    }

    #[test]
    fn base_with_min_length() {
        let mut c = TestCtx::new();
        assert_eq!(
            base(&mut c, &[Value::Number(10.0), Value::Number(2.0), Value::Number(8.0)]),
            Value::Text("00001010".into())
        );
    }

    #[test]
    fn base_negative_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            base(&mut c, &[Value::Number(-10.0), Value::Number(2.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn base_invalid_radix_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            base(&mut c, &[Value::Number(10.0), Value::Number(1.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn decimal_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            decimal(&mut c, &[Value::Text("1010".into()), Value::Number(2.0)]),
            Value::Number(10.0)
        );
    }

    #[test]
    fn decimal_invalid_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            decimal(&mut c, &[Value::Text("xyz".into()), Value::Number(10.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn decimal_invalid_radix_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            decimal(&mut c, &[Value::Text("10".into()), Value::Number(1.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- roundup / rounddown / mround / trunc ---

    #[test]
    fn roundup_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            roundup(&mut c, &[Value::Number(3.1), Value::Number(0.0)]),
            Value::Number(4.0)
        );
        assert_eq!(
            roundup(&mut c, &[Value::Number(-3.1), Value::Number(0.0)]),
            Value::Number(-4.0)
        );
    }

    #[test]
    fn rounddown_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            rounddown(&mut c, &[Value::Number(3.9), Value::Number(0.0)]),
            Value::Number(3.0)
        );
    }

    #[test]
    fn mround_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            mround(&mut c, &[Value::Number(10.0), Value::Number(3.0)]),
            Value::Number(9.0)
        );
    }

    #[test]
    fn mround_zero_multiple() {
        let mut c = TestCtx::new();
        assert_eq!(
            mround(&mut c, &[Value::Number(10.0), Value::Number(0.0)]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn mround_mismatched_signs_is_num() {
        let mut c = TestCtx::new();
        assert_eq!(
            mround(&mut c, &[Value::Number(10.0), Value::Number(-3.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn trunc_with_digits() {
        let mut c = TestCtx::new();
        assert_eq!(
            trunc(&mut c, &[Value::Number(3.14159), Value::Number(2.0)]),
            Value::Number(3.14)
        );
    }

    // --- sumifs / sumproduct / sumsq / percentof ---

    #[test]
    fn sumifs_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Number(20.0)),
            (2, 0, Value::Number(30.0)),
            (0, 1, Value::Text("A".into())),
            (1, 1, Value::Text("B".into())),
            (2, 1, Value::Text("A".into())),
        ]);
        // SUMIFS(A1:A3, B1:B3, "A") = 10 + 30 = 40
        let r = sumifs(
            &mut c,
            &[rng(0, 0, 2, 0), rng(0, 1, 2, 1), Value::Text("A".into())],
        );
        assert_eq!(r, Value::Number(40.0));
    }

    #[test]
    fn sumifs_odd_args_is_value() {
        let mut c = TestCtx::new();
        let r = sumifs(
            &mut c,
            &[Value::Number(1.0), Value::Number(2.0)],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn sumproduct_basic() {
        let mut c = TestCtx::new();
        // SUMPRODUCT({1,2,3}, {4,5,6}) = 1*4 + 2*5 + 3*6 = 32
        let r = sumproduct(
            &mut c,
            &[
                Value::Array(crate::formula::value::Array::new(
                    1,
                    3,
                    vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
                )),
                Value::Array(crate::formula::value::Array::new(
                    1,
                    3,
                    vec![Value::Number(4.0), Value::Number(5.0), Value::Number(6.0)],
                )),
            ],
        );
        assert_eq!(r, Value::Number(32.0));
    }

    #[test]
    fn sumproduct_mismatched_sizes_is_value() {
        let mut c = TestCtx::new();
        let r = sumproduct(
            &mut c,
            &[
                Value::Array(crate::formula::value::Array::new(
                    1,
                    2,
                    vec![Value::Number(1.0), Value::Number(2.0)],
                )),
                Value::Array(crate::formula::value::Array::new(
                    1,
                    3,
                    vec![Value::Number(4.0), Value::Number(5.0), Value::Number(6.0)],
                )),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Value));
    }

    #[test]
    fn sumsq_basic() {
        let mut c = TestCtx::new();
        // SUMSQ(1, 2, 3) = 1 + 4 + 9 = 14
        assert_eq!(
            sumsq(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(14.0)
        );
    }

    #[test]
    fn percentof_basic() {
        let mut c = TestCtx::new();
        // PERCENTOF(30, 100) = 30%
        assert_eq!(
            percentof(&mut c, &[Value::Number(30.0), Value::Number(100.0)]),
            Value::Number(0.3)
        );
    }

    #[test]
    fn percentof_zero_total_is_div0() {
        let mut c = TestCtx::new();
        assert_eq!(
            percentof(&mut c, &[Value::Number(30.0), Value::Number(0.0)]),
            Value::Error(CellError::Div0)
        );
    }

    #[test]
    fn product_empty_is_zero() {
        let mut c = TestCtx::new();
        assert_eq!(product(&mut c, &[]), Value::Number(0.0));
    }

    #[test]
    fn mod_positive_divisor() {
        let mut c = TestCtx::new();
        // MOD(7, 3) = 1
        assert_eq!(
            mod_fn(&mut c, &[Value::Number(7.0), Value::Number(3.0)]),
            Value::Number(1.0)
        );
    }

    // --- arabic edge cases ---

    #[test]
    fn arabic_negative() {
        let mut c = TestCtx::new();
        assert_eq!(
            arabic(&mut c, &[Value::Text("-IV".into())]),
            Value::Number(-4.0)
        );
    }

    #[test]
    fn arabic_invalid_is_value() {
        let mut c = TestCtx::new();
        assert_eq!(
            arabic(&mut c, &[Value::Text("ABC".into())]),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn roman_out_of_range_is_value() {
        let mut c = TestCtx::new();
        assert_eq!(
            roman(&mut c, &[Value::Number(5000.0)]),
            Value::Error(CellError::Value)
        );
    }
