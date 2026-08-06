    #[test]
    fn numbervalue_same_sep_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            numbervalue(
                &mut c,
                &[
                    Value::Text("1.234".into()),
                    Value::Text(".".into()),
                    Value::Text(".".into()),
                ]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- fixed --------------------------------------------------------------

    #[test]
    fn fixed_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            fixed(
                &mut c,
                &[
                    Value::Number(1234.5),
                    Value::Number(2.0),
                    Value::Bool(false)
                ]
            ),
            Value::Text("1,234.50".into())
        );
        assert_eq!(
            fixed(
                &mut c,
                &[Value::Number(1234.5), Value::Number(2.0), Value::Bool(true)]
            ),
            Value::Text("1234.50".into())
        );
        assert_eq!(
            fixed(&mut c, &[Value::Number(-1234.5), Value::Number(1.0)]),
            Value::Text("-1,234.5".into())
        );
    }

    // --- dollar -------------------------------------------------------------

    #[test]
    fn dollar_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            dollar(&mut c, &[Value::Number(1234.567)]),
            Value::Text("$1,234.57".into())
        );
        assert_eq!(
            dollar(&mut c, &[Value::Number(-1234.567)]),
            Value::Text("-$1,234.57".into())
        );
        assert_eq!(
            dollar(&mut c, &[Value::Number(1234.0), Value::Number(0.0)]),
            Value::Text("$1,234".into())
        );
    }

    // --- exact --------------------------------------------------------------

    #[test]
    fn exact_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            exact(
                &mut c,
                &[Value::Text("hello".into()), Value::Text("hello".into())]
            ),
            Value::Bool(true)
        );
        assert_eq!(
            exact(
                &mut c,
                &[Value::Text("Hello".into()), Value::Text("hello".into())]
            ),
            Value::Bool(false)
        );
        assert_eq!(
            exact(
                &mut c,
                &[Value::Text("abc".into()), Value::Text("abc".into())]
            ),
            Value::Bool(true)
        );
    }

    // --- char_fn / code_fn --------------------------------------------------

    #[test]
    fn char_code_roundtrip() {
        let mut c = TestCtx::new();
        assert_eq!(
            char_fn(&mut c, &[Value::Number(65.0)]),
            Value::Text("A".into())
        );
        assert_eq!(
            code_fn(&mut c, &[Value::Text("A".into())]),
            Value::Number(65.0)
        );
        assert_eq!(
            char_fn(&mut c, &[Value::Number(0.0)]),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn char_fn_range() {
        let mut c = TestCtx::new();
        assert_eq!(
            char_fn(&mut c, &[Value::Number(32.0)]),
            Value::Text(" ".into())
        );
        assert_eq!(
            char_fn(&mut c, &[Value::Number(256.0)]),
            Value::Error(CellError::Value)
        );
    }

    // --- unichar / unicode --------------------------------------------------

    #[test]
    fn unichar_unicode_roundtrip() {
        let mut c = TestCtx::new();
        assert_eq!(
            unichar(&mut c, &[Value::Number(f64::from(0x1F600))]),
            Value::Text("\u{1F600}".into())
        );
        assert_eq!(
            unicode(&mut c, &[Value::Text("\u{1F600}".into())]),
            Value::Number(f64::from(0x1F600))
        );
    }

    #[test]
    fn unicode_empty_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            unicode(&mut c, &[Value::Text(String::new())]),
            Value::Error(CellError::Value)
        );
    }

    // --- clean --------------------------------------------------------------

    #[test]
    fn clean_basic() {
        let mut c = TestCtx::new();
        let s = "hel\x01lo\x1Fworld";
        assert_eq!(
            clean(&mut c, &[Value::Text(s.into())]),
            Value::Text("helloworld".into())
        );
        assert_eq!(
            clean(&mut c, &[Value::Text("normal".into())]),
            Value::Text("normal".into())
        );
        assert_eq!(
            clean(&mut c, &[Value::Text(String::new())]),
            Value::Text(String::new())
        );
    }

    // --- t_fn ---------------------------------------------------------------

    #[test]
    fn t_fn_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            t_fn(&mut c, &[Value::Text("hello".into())]),
            Value::Text("hello".into())
        );
        assert_eq!(
            t_fn(&mut c, &[Value::Number(42.0)]),
            Value::Text(String::new())
        );
        assert_eq!(
            t_fn(&mut c, &[Value::Bool(true)]),
            Value::Text(String::new())
        );
    }

    // --- textbefore ---------------------------------------------------------

    #[test]
    fn textbefore_first_occurrence() {
        let mut c = TestCtx::new();
        assert_eq!(
            textbefore(
                &mut c,
                &[Value::Text("a-b-c".into()), Value::Text("-".into())]
            ),
            Value::Text("a".into())
        );
    }

    #[test]
    fn textbefore_second_occurrence() {
        let mut c = TestCtx::new();
        assert_eq!(
            textbefore(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Number(2.0)
                ]
            ),
            Value::Text("a-b".into())
        );
    }

    #[test]
    fn textbefore_negative_instance() {
        let mut c = TestCtx::new();
        // instance -1 = last delimiter → before the last "-" → "a-b"
        assert_eq!(
            textbefore(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Number(-1.0)
                ]
            ),
            Value::Text("a-b".into())
        );
    }

    #[test]
    fn textbefore_not_found_returns_na() {
        let mut c = TestCtx::new();
        assert_eq!(
            textbefore(
                &mut c,
                &[Value::Text("abc".into()), Value::Text("x".into())]
            ),
            Value::Error(CellError::NA)
        );
    }

    #[test]
    fn textbefore_not_found_if_not_found_arg() {
        let mut c = TestCtx::new();
        // 6 args: text, delim, instance, match_mode, match_end, if_not_found
        assert_eq!(
            textbefore(
                &mut c,
                &[
                    Value::Text("abc".into()),
                    Value::Text("x".into()),
                    Value::Number(1.0),
                    Value::Number(0.0),
                    Value::Empty,
                    Value::Text("N/A".into()),
                ]
            ),
            Value::Text("N/A".into())
        );
    }

    #[test]
    fn textbefore_case_insensitive() {
        let mut c = TestCtx::new();
        assert_eq!(
            textbefore(
                &mut c,
                &[
                    Value::Text("helloXworld".into()),
                    Value::Text("x".into()),
                    Value::Number(1.0),
                    Value::Number(1.0), // case-insensitive
                ]
            ),
            Value::Text("hello".into())
        );
    }

    // --- textafter ----------------------------------------------------------

    #[test]
    fn textafter_first_occurrence() {
        let mut c = TestCtx::new();
        assert_eq!(
            textafter(
                &mut c,
                &[Value::Text("a-b-c".into()), Value::Text("-".into())]
            ),
            Value::Text("b-c".into())
        );
    }

    #[test]
    fn textafter_second_occurrence() {
        let mut c = TestCtx::new();
        assert_eq!(
            textafter(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Number(2.0)
                ]
            ),
            Value::Text("c".into())
        );
    }

    #[test]
    fn textafter_negative_instance() {
        let mut c = TestCtx::new();
        // instance -2 = second-to-last delimiter → after first "-" → "b-c"
        assert_eq!(
            textafter(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Number(-2.0)
                ]
            ),
            Value::Text("b-c".into())
        );
    }

    #[test]
    fn textafter_not_found_returns_na() {
        let mut c = TestCtx::new();
        assert_eq!(
            textafter(
                &mut c,
                &[Value::Text("abc".into()), Value::Text("x".into())]
            ),
            Value::Error(CellError::NA)
        );
    }

    // --- valuetotext --------------------------------------------------------

    #[test]
    fn valuetotext_number() {
        let mut c = TestCtx::new();
        assert_eq!(
            valuetotext(&mut c, &[Value::Number(42.0)]),
            Value::Text("42".into())
        );
    }

    #[test]
    fn valuetotext_text_concise() {
        let mut c = TestCtx::new();
        assert_eq!(
            valuetotext(&mut c, &[Value::Text("hello".into()), Value::Number(0.0)]),
            Value::Text("hello".into())
        );
    }

    #[test]
    fn valuetotext_text_strict() {
        let mut c = TestCtx::new();
        assert_eq!(
            valuetotext(&mut c, &[Value::Text("hello".into()), Value::Number(1.0)]),
            Value::Text("\"hello\"".into())
        );
    }

    #[test]
    fn valuetotext_bool() {
        let mut c = TestCtx::new();
        assert_eq!(
            valuetotext(&mut c, &[Value::Bool(true)]),
            Value::Text("TRUE".into())
        );
        assert_eq!(
            valuetotext(&mut c, &[Value::Bool(false)]),
            Value::Text("FALSE".into())
        );
    }

    // --- arraytotext --------------------------------------------------------

    #[test]
    fn arraytotext_concise() {
        use crate::formula::value::Array;
        let mut c = TestCtx::new();
        let arr = Value::Array(Array::from_rows(vec![
            vec![Value::Number(1.0), Value::Number(2.0)],
            vec![Value::Number(3.0), Value::Number(4.0)],
        ]));
        assert_eq!(
            arraytotext(&mut c, &[arr, Value::Number(0.0)]),
            Value::Text("1, 2, 3, 4".into())
        );
    }

    #[test]
    fn arraytotext_strict() {
        use crate::formula::value::Array;
        let mut c = TestCtx::new();
        let arr = Value::Array(Array::from_rows(vec![
            vec![Value::Text("a".into()), Value::Number(1.0)],
            vec![Value::Text("b".into()), Value::Number(2.0)],
        ]));
        assert_eq!(
            arraytotext(&mut c, &[arr, Value::Number(1.0)]),
            Value::Text("{\"a\",1;\"b\",2}".into())
        );
    }

    #[test]
    fn arraytotext_scalar_fallback() {
        let mut c = TestCtx::new();
        assert_eq!(
            arraytotext(&mut c, &[Value::Number(99.0)]),
            Value::Text("99".into())
        );
    }
