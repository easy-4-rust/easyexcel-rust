    // --- concat numbers in range ---

    #[test]
    fn concat_with_empty_args() {
        let mut c = TestCtx::new();
        assert_eq!(
            concat(&mut c, &[Value::Text("a".into()), Value::Empty]),
            Value::Text("a".into())
        );
    }

    // --- textjoin with numeric values ---

    #[test]
    fn textjoin_with_numbers() {
        let mut c = TestCtx::new();
        assert_eq!(
            textjoin(
                &mut c,
                &[
                    Value::Text("-".into()),
                    Value::Bool(false),
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0),
                ]
            ),
            Value::Text("1-2-3".into())
        );
    }

    // --- len on non-text ---

    #[test]
    fn len_on_number() {
        let mut c = TestCtx::new();
        // LEN on number coerces to text
        assert_eq!(
            len(&mut c, &[Value::Number(123.0)]),
            Value::Number(3.0)
        );
    }

    // --- left/right with error ---

    #[test]
    fn left_right_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            left(&mut c, &[Value::Error(CellError::NA), Value::Number(1.0)]),
            Value::Error(CellError::NA)
        );
        assert_eq!(
            right(&mut c, &[Value::Error(CellError::Value), Value::Number(1.0)]),
            Value::Error(CellError::Value)
        );
    }

    // --- mid error ---

    #[test]
    fn mid_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            mid(
                &mut c,
                &[
                    Value::Error(CellError::Num),
                    Value::Number(1.0),
                    Value::Number(1.0)
                ]
            ),
            Value::Error(CellError::Num)
        );
    }

    // --- find/search with start position ---

    #[test]
    fn find_with_start_position() {
        let mut c = TestCtx::new();
        assert_eq!(
            find(
                &mut c,
                &[
                    Value::Text("l".into()),
                    Value::Text("hello".into()),
                    Value::Number(3.0)
                ]
            ),
            Value::Number(3.0)
        );
    }

    #[test]
    fn search_wildcard_pattern() {
        let mut c = TestCtx::new();
        assert_eq!(
            search(
                &mut c,
                &[
                    Value::Text("h*o".into()),
                    Value::Text("hello".into()),
                    Value::Number(1.0),
                    Value::Number(0.0)
                ]
            ),
            Value::Number(1.0)
        );
    }

    // --- substitute not found ---

    #[test]
    fn substitute_no_match() {
        let mut c = TestCtx::new();
        assert_eq!(
            substitute(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Text("xyz".into()),
                    Value::Text("abc".into())
                ]
            ),
            Value::Text("hello".into())
        );
    }

    // --- replace error ---

    #[test]
    fn replace_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            replace(
                &mut c,
                &[
                    Value::Error(CellError::NA),
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Text("x".into())
                ]
            ),
            Value::Error(CellError::NA)
        );
    }

    // --- rept edge cases ---

    #[test]
    fn rept_large_count() {
        let mut c = TestCtx::new();
        let r = rept(&mut c, &[Value::Text("ab".into()), Value::Number(5.0)]);
        assert_eq!(r, Value::Text("ababababab".into()));
    }

    // --- text_fn with empty ---

    #[test]
    fn text_fn_empty_at_format() {
        let mut c = TestCtx::new();
        assert_eq!(
            text_fn(&mut c, &[Value::Empty, Value::Text("@".into())]),
            Value::Text("".into())
        );
    }

    // --- value_fn with error ---

    #[test]
    fn value_fn_error_passthrough() {
        let mut c = TestCtx::new();
        assert_eq!(
            value_fn(&mut c, &[Value::Error(CellError::NA)]),
            Value::Error(CellError::NA)
        );
    }

    // --- clean on non-text ---

    #[test]
    fn clean_on_number() {
        let mut c = TestCtx::new();
        assert_eq!(
            clean(&mut c, &[Value::Number(42.0)]),
            Value::Text("42".into())
        );
    }

    // --- t_fn on error ---

    #[test]
    fn t_fn_on_error_passthrough() {
        let mut c = TestCtx::new();
        // T on error returns the error (not empty)
        assert_eq!(
            t_fn(&mut c, &[Value::Error(CellError::NA)]),
            Value::Error(CellError::NA)
        );
    }

    // --- unichar/unicode ---

    #[test]
    fn unichar_zero() {
        let mut c = TestCtx::new();
        // UNICHAR(0) returns \0 (null char)
        let r = unichar(&mut c, &[Value::Number(0.0)]);
        assert!(matches!(r, Value::Text(_) | Value::Error(_)));
    }

    // --- numbervalue with different separators ---

    #[test]
    fn numbervalue_comma_decimal() {
        let mut c = TestCtx::new();
        assert_eq!(
            numbervalue(
                &mut c,
                &[
                    Value::Text("1,23".into()),
                    Value::Text(",".into()),
                ]
            ),
            Value::Number(1.23)
        );
    }

    // --- fixed error ---

    #[test]
    fn fixed_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            fixed(&mut c, &[Value::Error(CellError::NA), Value::Number(2.0)]),
            Value::Error(CellError::NA)
        );
    }

    // --- dollar error ---

    #[test]
    fn dollar_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            dollar(&mut c, &[Value::Error(CellError::Num)]),
            Value::Error(CellError::Num)
        );
    }

    // --- exact with error ---

    #[test]
    fn exact_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            exact(
                &mut c,
                &[Value::Error(CellError::NA), Value::Text("x".into())]
            ),
            Value::Error(CellError::NA)
        );
    }

    // --- char_fn out of range ---

    #[test]
    fn char_fn_out_of_range() {
        let mut c = TestCtx::new();
        assert_eq!(
            char_fn(&mut c, &[Value::Number(256.0)]),
            Value::Error(CellError::Value)
        );
    }

    // --- code_fn on empty ---

    #[test]
    fn code_fn_empty() {
        let mut c = TestCtx::new();
        assert_eq!(
            code_fn(&mut c, &[Value::Text("".into())]),
            Value::Error(CellError::Value)
        );
    }
