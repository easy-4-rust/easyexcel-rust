    // --- 更多数学函数测试（覆盖 register_to_sign.rs 未测分支） ---

    // sum: 非数字参数
    #[test]
    fn sum_err_text_v2() {
        let mut c = TestCtx::new();
        let r = sum(&mut c, &[Value::Text("abc".into()), Value::Number(1.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // percentof: 分母为零 → #DIV/0!
    #[test]
    fn percentof_div_zero() {
        let mut c = TestCtx::new();
        let r = percentof(&mut c, &[Value::Number(25.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // percentof: 非数字参数
    #[test]
    fn percentof_err_text() {
        let mut c = TestCtx::new();
        let r = percentof(&mut c, &[Value::Text("abc".into()), Value::Number(100.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // product: 空输入
    #[test]
    fn product_empty() {
        let mut c = TestCtx::new();
        let r = product(&mut c, &[]);
        assert_eq!(r, Value::Number(0.0));
    }

    // product: 非数字参数
    #[test]
    fn product_err_text() {
        let mut c = TestCtx::new();
        let r = product(&mut c, &[Value::Text("abc".into()), Value::Number(2.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // sumsq: 非数字参数
    #[test]
    fn sumsq_err_text() {
        let mut c = TestCtx::new();
        let r = sumsq(&mut c, &[Value::Text("abc".into()), Value::Number(2.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // sumifs: 参数数量不足
    #[test]
    fn sumifs_err_few_args() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        let r = sumifs(&mut c, &[rng(0, 0, 0, 0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // sumproduct: 含布尔值
    #[test]
    fn sumproduct_with_bool() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Bool(true)),
            (1, 0, Value::Number(2.0)),
        ]);
        let r = sumproduct(&mut c, &[rng(0, 0, 1, 0), rng(0, 0, 1, 0)]);
        // 1*1 + 2*2 = 5
        assert_eq!(r, Value::Number(5.0));
    }

    // sumproduct: 长度不匹配
    #[test]
    fn sumproduct_mismatched() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        let r = sumproduct(&mut c, &[rng(0, 0, 0, 0), rng(0, 0, 1, 0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // round: 负数位数
    #[test]
    fn round_neg_digits() {
        let mut c = TestCtx::new();
        let r = round(&mut c, &[Value::Number(1234.5), Value::Number(-2.0)]);
        assert_eq!(r, Value::Number(1200.0));
    }

    // roundup: 正数
    #[test]
    fn roundup_positive_v2() {
        let mut c = TestCtx::new();
        let r = roundup(&mut c, &[Value::Number(3.14), Value::Number(1.0)]);
        assert_eq!(r, Value::Number(3.2));
    }

    // roundup: 负数
    #[test]
    fn roundup_negative() {
        let mut c = TestCtx::new();
        let r = roundup(&mut c, &[Value::Number(-3.14), Value::Number(1.0)]);
        assert_eq!(r, Value::Number(-3.2));
    }

    // rounddown: 正数
    #[test]
    fn rounddown_positive_v2() {
        let mut c = TestCtx::new();
        let r = rounddown(&mut c, &[Value::Number(3.14), Value::Number(1.0)]);
        assert_eq!(r, Value::Number(3.1));
    }

    // rounddown: 负数
    #[test]
    fn rounddown_negative() {
        let mut c = TestCtx::new();
        let r = rounddown(&mut c, &[Value::Number(-3.14), Value::Number(1.0)]);
        assert_eq!(r, Value::Number(-3.1));
    }

    // mround: 零除数
    #[test]
    fn mround_zero_div() {
        let mut c = TestCtx::new();
        let r = mround(&mut c, &[Value::Number(10.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(0.0));
    }

    // mround: 符号不同 → #NUM!
    #[test]
    fn mround_mismatched_signs() {
        let mut c = TestCtx::new();
        let r = mround(&mut c, &[Value::Number(10.0), Value::Number(-3.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // mround: 非数字参数
    #[test]
    fn mround_err_text() {
        let mut c = TestCtx::new();
        let r = mround(&mut c, &[Value::Text("abc".into()), Value::Number(3.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // trunc: 带参数 (v2)
    #[test]
    fn trunc_with_digits_v2() {
        let mut c = TestCtx::new();
        let r = trunc(&mut c, &[Value::Number(3.14159), Value::Number(2.0)]);
        assert_eq!(r, Value::Number(3.14));
    }

    // trunc: 非数字参数
    #[test]
    fn trunc_err_text() {
        let mut c = TestCtx::new();
        let r = trunc(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // ceiling_math: 负数，mode=1 → floor 模式
    #[test]
    fn ceiling_math_neg_mode1() {
        let mut c = TestCtx::new();
        let r = ceiling_math(&mut c, &[Value::Number(-5.5), Value::Number(2.0), Value::Number(1.0)]);
        // mode!=0: 负数时 floor → (-5.5/2).floor()*2 = -3*2 = -6
        assert_eq!(r, Value::Number(-6.0));
    }

    // ceiling_math: 非数字参数
    #[test]
    fn ceiling_math_err_text() {
        let mut c = TestCtx::new();
        let r = ceiling_math(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // floor_math: 负数，mode=1 → ceil 模式
    #[test]
    fn floor_math_neg_mode1() {
        let mut c = TestCtx::new();
        let r = floor_math(&mut c, &[Value::Number(-5.5), Value::Number(2.0), Value::Number(1.0)]);
        // mode!=0: 负数时 ceil → (-5.5/2).ceil()*2 = -2*2 = -4
        assert_eq!(r, Value::Number(-4.0));
    }

    // floor_math: 非数字参数
    #[test]
    fn floor_math_err_text() {
        let mut c = TestCtx::new();
        let r = floor_math(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // ceiling: 零 significance → 0
    #[test]
    fn ceiling_zero_sig_v2() {
        let mut c = TestCtx::new();
        let r = ceiling(&mut c, &[Value::Number(5.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(0.0));
    }

    // ceiling: 正数、负 significance → #NUM!
    #[test]
    fn ceiling_pos_neg_sig() {
        let mut c = TestCtx::new();
        let r = ceiling(&mut c, &[Value::Number(5.0), Value::Number(-2.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // ceiling: 非数字参数
    #[test]
    fn ceiling_err_text() {
        let mut c = TestCtx::new();
        let r = ceiling(&mut c, &[Value::Text("abc".into()), Value::Number(2.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // floor: 零 significance → #DIV/0!
    #[test]
    fn floor_zero_sig_div0_v2() {
        let mut c = TestCtx::new();
        let r = floor(&mut c, &[Value::Number(5.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // floor: 正数、负 significance → #NUM!
    #[test]
    fn floor_pos_neg_sig() {
        let mut c = TestCtx::new();
        let r = floor(&mut c, &[Value::Number(5.0), Value::Number(-2.0)]);
        assert_eq!(r, Value::Error(CellError::Num));
    }

    // floor: 非数字参数
    #[test]
    fn floor_err_text() {
        let mut c = TestCtx::new();
        let r = floor(&mut c, &[Value::Text("abc".into()), Value::Number(2.0)]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // ceiling_precise: 零 significance → 0
    #[test]
    fn ceiling_precise_zero_sig_v2() {
        let mut c = TestCtx::new();
        let r = ceiling_precise(&mut c, &[Value::Number(5.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(0.0));
    }

    // ceiling_precise: 非数字参数
    #[test]
    fn ceiling_precise_err_text() {
        let mut c = TestCtx::new();
        let r = ceiling_precise(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // floor_precise: 零 significance → 0
    #[test]
    fn floor_precise_zero_sig_v2() {
        let mut c = TestCtx::new();
        let r = floor_precise(&mut c, &[Value::Number(5.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(0.0));
    }

    // floor_precise: 非数字参数
    #[test]
    fn floor_precise_err_text() {
        let mut c = TestCtx::new();
        let r = floor_precise(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // even: 非数字参数
    #[test]
    fn even_err_text() {
        let mut c = TestCtx::new();
        let r = even(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // odd: 非数字参数
    #[test]
    fn odd_err_text() {
        let mut c = TestCtx::new();
        let r = odd(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // sign: 非数字参数
    #[test]
    fn sign_err_text() {
        let mut c = TestCtx::new();
        let r = sign(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // int_fn: 非数字参数
    #[test]
    fn int_err_text() {
        let mut c = TestCtx::new();
        let r = int_fn(&mut c, &[Value::Text("abc".into())]);
        assert_eq!(r, Value::Error(CellError::Value));
    }

    // roundup: 零位数
    #[test]
    fn roundup_zero_digits() {
        let mut c = TestCtx::new();
        let r = roundup(&mut c, &[Value::Number(3.14), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(4.0));
    }

    // rounddown: 零位数
    #[test]
    fn rounddown_zero_digits() {
        let mut c = TestCtx::new();
        let r = rounddown(&mut c, &[Value::Number(3.14), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(3.0));
    }

    // ceiling_math: 零 significance
    #[test]
    fn ceiling_math_zero_sig() {
        let mut c = TestCtx::new();
        let r = ceiling_math(&mut c, &[Value::Number(5.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(0.0));
    }

    // floor_math: 零 significance
    #[test]
    fn floor_math_zero_sig() {
        let mut c = TestCtx::new();
        let r = floor_math(&mut c, &[Value::Number(5.0), Value::Number(0.0)]);
        assert_eq!(r, Value::Number(0.0));
    }
