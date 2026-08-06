    #[test]
    fn regex_functions() {
        let mut c = TestCtx::new();
        // REGEXTEST
        assert_eq!(
            regextest(
                &mut c,
                &[Value::Text("abc123".into()), Value::Text(r"\d+".into())]
            ),
            Value::Bool(true)
        );
        assert_eq!(
            regextest(
                &mut c,
                &[
                    Value::Text("ABC".into()),
                    Value::Text("abc".into()),
                    Value::Number(1.0)
                ]
            ),
            Value::Bool(true)
        );
        // invalid pattern → #VALUE!
        assert_eq!(
            regextest(&mut c, &[Value::Text("x".into()), Value::Text("(".into())]),
            Value::Error(CellError::Value)
        );
        // REGEXEXTRACT first match
        assert_eq!(
            regexextract(
                &mut c,
                &[Value::Text("id=42;".into()), Value::Text(r"\d+".into())]
            ),
            Value::Text("42".into())
        );
        // no match → #N/A
        assert_eq!(
            regexextract(
                &mut c,
                &[Value::Text("abc".into()), Value::Text(r"\d+".into())]
            ),
            Value::Error(CellError::NA)
        );
        // REGEXREPLACE all, with backref
        assert_eq!(
            regexreplace(
                &mut c,
                &[
                    Value::Text("a1b2".into()),
                    Value::Text(r"(\d)".into()),
                    Value::Text("[$1]".into())
                ]
            ),
            Value::Text("a[1]b[2]".into())
        );
        // REGEXREPLACE only 2nd occurrence
        assert_eq!(
            regexreplace(
                &mut c,
                &[
                    Value::Text("x-x-x".into()),
                    Value::Text("x".into()),
                    Value::Text("Y".into()),
                    Value::Number(2.0)
                ]
            ),
            Value::Text("x-Y-x".into())
        );
    }

    // --- concat / concatenate -----------------------------------------------

    #[test]
    fn concat_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            concat(
                &mut c,
                &[Value::Text("Hello".into()), Value::Text(" World".into())]
            ),
            Value::Text("Hello World".into())
        );
    }

    #[test]
    fn concat_numbers() {
        let mut c = TestCtx::new();
        assert_eq!(
            concat(&mut c, &[Value::Number(1.0), Value::Text("x".into())]),
            Value::Text("1x".into())
        );
    }

    #[test]
    fn concat_error_propagates() {
        let mut c = TestCtx::new();
        assert_eq!(
            concat(&mut c, &[Value::Error(CellError::Value)]),
            Value::Error(CellError::Value)
        );
    }

    // --- textjoin -----------------------------------------------------------

    #[test]
    fn textjoin_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            textjoin(
                &mut c,
                &[
                    Value::Text(",".into()),
                    Value::Bool(true),
                    Value::Text("a".into()),
                    Value::Text("b".into()),
                    Value::Text("c".into()),
                ]
            ),
            Value::Text("a,b,c".into())
        );
    }

    #[test]
    fn textjoin_ignore_empty() {
        let mut c = TestCtx::new();
        assert_eq!(
            textjoin(
                &mut c,
                &[
                    Value::Text("-".into()),
                    Value::Bool(true),
                    Value::Text("a".into()),
                    Value::Empty,
                    Value::Text("c".into()),
                ]
            ),
            Value::Text("a-c".into())
        );
    }

    #[test]
    fn textjoin_keep_empty() {
        let mut c = TestCtx::new();
        assert_eq!(
            textjoin(
                &mut c,
                &[
                    Value::Text("-".into()),
                    Value::Bool(false),
                    Value::Text("a".into()),
                    Value::Empty,
                    Value::Text("c".into()),
                ]
            ),
            Value::Text("a--c".into())
        );
    }

    // --- len ----------------------------------------------------------------

    #[test]
    fn len_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            len(&mut c, &[Value::Text("hello".into())]),
            Value::Number(5.0)
        );
        assert_eq!(
            len(&mut c, &[Value::Text(String::new())]),
            Value::Number(0.0)
        );
        assert_eq!(
            len(&mut c, &[Value::Error(CellError::Value)]),
            Value::Error(CellError::Value)
        );
    }

    // --- left ---------------------------------------------------------------

    #[test]
    fn left_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            left(&mut c, &[Value::Text("hello".into()), Value::Number(2.0)]),
            Value::Text("he".into())
        );
        assert_eq!(
            left(&mut c, &[Value::Text("hello".into()), Value::Number(0.0)]),
            Value::Text(String::new())
        );
        assert_eq!(
            left(&mut c, &[Value::Text("hello".into())]),
            Value::Text("h".into())
        );
    }

    #[test]
    fn left_over_length() {
        let mut c = TestCtx::new();
        assert_eq!(
            left(&mut c, &[Value::Text("hi".into()), Value::Number(10.0)]),
            Value::Text("hi".into())
        );
    }

    #[test]
    fn left_negative_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            left(&mut c, &[Value::Text("hi".into()), Value::Number(-1.0)]),
            Value::Error(CellError::Value)
        );
    }

    // --- right --------------------------------------------------------------

    #[test]
    fn right_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            right(&mut c, &[Value::Text("hello".into()), Value::Number(3.0)]),
            Value::Text("llo".into())
        );
        assert_eq!(
            right(&mut c, &[Value::Text("hello".into())]),
            Value::Text("o".into())
        );
    }

    #[test]
    fn right_over_length() {
        let mut c = TestCtx::new();
        assert_eq!(
            right(&mut c, &[Value::Text("hi".into()), Value::Number(10.0)]),
            Value::Text("hi".into())
        );
    }

    // --- mid ----------------------------------------------------------------

    #[test]
    fn mid_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            mid(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Number(2.0),
                    Value::Number(3.0)
                ]
            ),
            Value::Text("ell".into())
        );
    }

    #[test]
    fn mid_past_end() {
        let mut c = TestCtx::new();
        assert_eq!(
            mid(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Number(4.0),
                    Value::Number(10.0)
                ]
            ),
            Value::Text("lo".into())
        );
    }

    #[test]
    fn mid_start_zero_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            mid(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Number(0.0),
                    Value::Number(3.0)
                ]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- trim ---------------------------------------------------------------

    #[test]
    fn trim_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            trim(&mut c, &[Value::Text("  hello   world  ".into())]),
            Value::Text("hello world".into())
        );
        assert_eq!(
            trim(&mut c, &[Value::Text("  spaces  ".into())]),
            Value::Text("spaces".into())
        );
        assert_eq!(
            trim(&mut c, &[Value::Text("no spaces".into())]),
            Value::Text("no spaces".into())
        );
    }

    // --- upper / lower / proper ---------------------------------------------

    #[test]
    fn case_fns() {
        let mut c = TestCtx::new();
        assert_eq!(
            upper(&mut c, &[Value::Text("hello".into())]),
            Value::Text("HELLO".into())
        );
        assert_eq!(
            lower(&mut c, &[Value::Text("HELLO".into())]),
            Value::Text("hello".into())
        );
        assert_eq!(
            proper(&mut c, &[Value::Text("hello world".into())]),
            Value::Text("Hello World".into())
        );
    }

    #[test]
    fn proper_mixed() {
        let mut c = TestCtx::new();
        assert_eq!(
            proper(&mut c, &[Value::Text("HELLO WORLD".into())]),
            Value::Text("Hello World".into())
        );
        assert_eq!(
            proper(&mut c, &[Value::Text("it's a test".into())]),
            Value::Text("It'S A Test".into())
        );
    }

    // --- find ---------------------------------------------------------------

    #[test]
    fn find_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            find(
                &mut c,
                &[Value::Text("e".into()), Value::Text("hello".into())]
            ),
            Value::Number(2.0)
        );
        assert_eq!(
            find(
                &mut c,
                &[Value::Text("l".into()), Value::Text("hello".into())]
            ),
            Value::Number(3.0)
        );
    }

    #[test]
    fn find_case_sensitive() {
        let mut c = TestCtx::new();
        // FIND is case-sensitive — 'E' not in 'hello'
        assert_eq!(
            find(
                &mut c,
                &[Value::Text("E".into()), Value::Text("hello".into())]
            ),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn find_not_found() {
        let mut c = TestCtx::new();
        assert_eq!(
            find(
                &mut c,
                &[Value::Text("z".into()), Value::Text("hello".into())]
            ),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn find_with_start() {
        let mut c = TestCtx::new();
        // Find second 'l' starting from position 4
        assert_eq!(
            find(
                &mut c,
                &[
                    Value::Text("l".into()),
                    Value::Text("hello".into()),
                    Value::Number(4.0)
                ]
            ),
            Value::Number(4.0)
        );
    }

    // --- search -------------------------------------------------------------

    #[test]
    fn search_case_insensitive() {
        let mut c = TestCtx::new();
        assert_eq!(
            search(
                &mut c,
                &[Value::Text("E".into()), Value::Text("hello".into())]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn search_wildcard() {
        let mut c = TestCtx::new();
        // "h*o" should match "hello" at position 1
        assert_eq!(
            search(
                &mut c,
                &[Value::Text("h*o".into()), Value::Text("hello".into())]
            ),
            Value::Number(1.0)
        );
    }

    #[test]
    fn search_not_found() {
        let mut c = TestCtx::new();
        assert_eq!(
            search(
                &mut c,
                &[Value::Text("z".into()), Value::Text("hello".into())]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- substitute ---------------------------------------------------------

    #[test]
    fn substitute_all() {
        let mut c = TestCtx::new();
        assert_eq!(
            substitute(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Text("+".into()),
                ]
            ),
            Value::Text("a+b+c".into())
        );
    }

    #[test]
    fn substitute_instance() {
        let mut c = TestCtx::new();
        assert_eq!(
            substitute(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Text("+".into()),
                    Value::Number(2.0),
                ]
            ),
            Value::Text("a-b+c".into())
        );
    }

    #[test]
    fn substitute_not_found() {
        let mut c = TestCtx::new();
        assert_eq!(
            substitute(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Text("z".into()),
                    Value::Text("x".into()),
                ]
            ),
            Value::Text("hello".into())
        );
    }

    // --- replace ------------------------------------------------------------

    #[test]
    fn replace_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            replace(
                &mut c,
                &[
                    Value::Text("hello world".into()),
                    Value::Number(7.0),
                    Value::Number(5.0),
                    Value::Text("there".into()),
                ]
            ),
            Value::Text("hello there".into())
        );
    }

    #[test]
    fn replace_insert() {
        let mut c = TestCtx::new();
        // REPLACE with num_chars=0 inserts
        assert_eq!(
            replace(
                &mut c,
                &[
                    Value::Text("abc".into()),
                    Value::Number(2.0),
                    Value::Number(0.0),
                    Value::Text("X".into()),
                ]
            ),
            Value::Text("aXbc".into())
        );
    }

    #[test]
    fn replace_start_1_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            replace(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Number(0.0),
                    Value::Number(1.0),
                    Value::Text("x".into()),
                ]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- rept ---------------------------------------------------------------

    #[test]
    fn rept_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            rept(&mut c, &[Value::Text("ab".into()), Value::Number(3.0)]),
            Value::Text("ababab".into())
        );
        assert_eq!(
            rept(&mut c, &[Value::Text("x".into()), Value::Number(0.0)]),
            Value::Text(String::new())
        );
        assert_eq!(
            rept(&mut c, &[Value::Text("x".into()), Value::Number(-1.0)]),
            Value::Error(CellError::Value)
        );
    }

    // --- text_fn ------------------------------------------------------------

    #[test]
    fn text_fn_basic() {
        let mut c = TestCtx::new();
        // TEXT(1234.5, "#,##0.00") → "1,234.50"
        assert_eq!(
            text_fn(
                &mut c,
                &[Value::Number(1234.5), Value::Text("#,##0.00".into())]
            ),
            Value::Text("1,234.50".into())
        );
    }

    #[test]
    fn text_fn_text_passthrough() {
        let mut c = TestCtx::new();
        assert_eq!(
            text_fn(
                &mut c,
                &[Value::Text("hello".into()), Value::Text("@".into())]
            ),
            Value::Text("hello".into())
        );
    }

    #[test]
    fn text_fn_zero() {
        let mut c = TestCtx::new();
        assert_eq!(
            text_fn(&mut c, &[Value::Number(0.0), Value::Text("0.00".into())]),
            Value::Text("0.00".into())
        );
    }

    // --- value_fn -----------------------------------------------------------

    #[test]
    fn value_fn_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            value_fn(&mut c, &[Value::Text("9.88".into())]),
            Value::Number(9.88)
        );
        assert_eq!(
            value_fn(&mut c, &[Value::Text("1,234".into())]),
            Value::Number(1234.0)
        );
        assert_eq!(
            value_fn(&mut c, &[Value::Text("abc".into())]),
            Value::Error(CellError::Value)
        );
    }

    // --- numbervalue --------------------------------------------------------

    #[test]
    fn numbervalue_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            numbervalue(
                &mut c,
                &[
                    Value::Text("1.234,56".into()),
                    Value::Text(",".into()),
                    Value::Text(".".into()),
                ]
            ),
            Value::Number(1234.56)
        );
    }

    #[test]
    fn numbervalue_percent() {
        let mut c = TestCtx::new();
        assert_eq!(
            numbervalue(
                &mut c,
                &[Value::Text("50%".into()), Value::Text(".".into()),]
            ),
            Value::Number(0.5)
        );
    }

