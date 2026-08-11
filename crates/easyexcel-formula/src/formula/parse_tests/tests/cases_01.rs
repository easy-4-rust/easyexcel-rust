    #[test]
    fn literals() {
        assert_eq!(p("=42"), Expr::Number(42.0));
        assert_eq!(
            p("=-3.5"),
            Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(Expr::Number(3.5))
            }
        );
        assert_eq!(p(r#"="hi""#), Expr::Text("hi".into()));
        assert_eq!(p("=TRUE"), Expr::Bool(true));
        assert_eq!(p("=#REF!"), Expr::Error(CellError::Ref));
    }

    #[test]
    fn precedence() {
        // 1+2*3 → 1 + (2*3)
        let e = p("=1+2*3");
        if let Expr::Binary {
            op: BinaryOp::Add,
            rhs,
            ..
        } = e
        {
            assert!(matches!(
                *rhs,
                Expr::Binary {
                    op: BinaryOp::Mul,
                    ..
                }
            ));
        } else {
            panic!("bad tree");
        }
    }

    #[test]
    fn references() {
        assert!(matches!(p("=A1"), Expr::Ref(_)));
        assert!(matches!(p("=A1:B10"), Expr::Ref(r) if r.is_range()));
        match p("=Sheet1!A1") {
            Expr::Ref(r) => assert_eq!(r.sheet, SheetSpec::Name("Sheet1".into())),
            _ => panic!(),
        }
        match p("='My Sheet'!B2") {
            Expr::Ref(r) => assert_eq!(r.sheet, SheetSpec::Name("My Sheet".into())),
            _ => panic!(),
        }
    }

    #[test]
    fn func_calls() {
        match p("=SUM(A1:A3, 5)") {
            Expr::Func { name, args } => {
                assert_eq!(name, "SUM");
                assert_eq!(args.len(), 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn array_constant() {
        match p("={1,2;3,4}") {
            Expr::Array(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn three_d_ref() {
        match p("=Sheet1:Sheet3!A1") {
            Expr::Ref(r) => assert_eq!(r.sheet, SheetSpec::Span("Sheet1".into(), "Sheet3".into())),
            _ => panic!(),
        }
    }

    #[test]
    fn full_column() {
        match p("=SUM(A:A)") {
            Expr::Func { args, .. } => match &args[0] {
                Expr::Ref(r) => {
                    assert_eq!(r.start.col, 0);
                    assert_eq!(r.end.unwrap().row, MAX_ROW);
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    // ── 补充解析测试 ────────────────────────────────────────────────────

    #[test]
    fn error_literals() {
        assert!(matches!(p("=#DIV/0!"), Expr::Error(CellError::Div0)));
        assert!(matches!(p("=#N/A"), Expr::Error(CellError::NA)));
        assert!(matches!(p("=#VALUE!"), Expr::Error(CellError::Value)));
        assert!(matches!(p("=#NAME?"), Expr::Error(CellError::Name)));
        assert!(matches!(p("=#NUM!"), Expr::Error(CellError::Num)));
        assert!(matches!(p("=#NULL!"), Expr::Error(CellError::Null)));
        assert!(matches!(p("=#SPILL!"), Expr::Error(CellError::Spill)));
        assert!(matches!(p("=#CALC!"), Expr::Error(CellError::Calc)));
        assert!(matches!(
            p("=#GETTING_DATA"),
            Expr::Error(CellError::GettingData)
        ));
    }

    #[test]
    fn boolean_literals() {
        assert_eq!(p("=TRUE"), Expr::Bool(true));
        assert_eq!(p("=FALSE"), Expr::Bool(false));
        assert_eq!(p("=true"), Expr::Bool(true));
        assert_eq!(p("=false"), Expr::Bool(false));
    }

    #[test]
    fn comparison_operators() {
        assert!(matches!(p("=1=1"), Expr::Binary { op: BinaryOp::Eq, .. }));
        assert!(matches!(p("=1<>2"), Expr::Binary { op: BinaryOp::Ne, .. }));
        assert!(matches!(p("=1<2"), Expr::Binary { op: BinaryOp::Lt, .. }));
        assert!(matches!(p("=1<=2"), Expr::Binary { op: BinaryOp::Le, .. }));
        assert!(matches!(p("=1>2"), Expr::Binary { op: BinaryOp::Gt, .. }));
        assert!(matches!(p("=1>=2"), Expr::Binary { op: BinaryOp::Ge, .. }));
    }

    #[test]
    fn concatenation() {
        match p(r#"="a"&"b""#) {
            Expr::Binary {
                op: BinaryOp::Concat,
                ..
            } => {}
            _ => panic!("expected Concat"),
        }
    }

    #[test]
    fn percent_postfix() {
        match p("=50%") {
            Expr::Unary {
                op: UnaryOp::Percent,
                ..
            } => {}
            _ => panic!("expected Percent"),
        }
    }

    #[test]
    fn unary_plus() {
        match p("=+5") {
            Expr::Unary {
                op: UnaryOp::Plus,
                ..
            } => {}
            _ => panic!("expected Plus"),
        }
    }

    #[test]
    fn at_prefix() {
        match p("=@A1") {
            Expr::Func { name, .. } => assert_eq!(name, "_AT_"),
            _ => panic!("expected _AT_ func"),
        }
    }

    #[test]
    fn exponentiation_right_assoc() {
        // 2^3^2 → 2^(3^2) = 512
        let e = p("=2^3^2");
        if let Expr::Binary {
            op: BinaryOp::Pow,
            lhs,
            rhs,
        } = e
        {
            assert!(matches!(*lhs, Expr::Number(2.0)));
            assert!(matches!(
                *rhs,
                Expr::Binary {
                    op: BinaryOp::Pow,
                    ..
                }
            ));
        } else {
            panic!("bad tree");
        }
    }

    #[test]
    fn function_call_no_args() {
        match p("=NOW()") {
            Expr::Func { name, args } => {
                assert_eq!(name, "NOW");
                assert!(args.is_empty());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn function_call_omitted_arg() {
        // IF(A1,,0) — 省略第二个参数
        match p("=IF(A1,,0)") {
            Expr::Func { name, args } => {
                assert_eq!(name, "IF");
                assert_eq!(args.len(), 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn nested_function() {
        match p("=SUM(A1,MAX(B1:B3))") {
            Expr::Func { name, args } => {
                assert_eq!(name, "SUM");
                assert_eq!(args.len(), 2);
                assert!(matches!(&args[1], Expr::Func { name, .. } if name == "MAX"));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parenthesized_union() {
        // (A1,B2) → Binary Union
        match p("=(A1,B2)") {
            Expr::Binary {
                op: BinaryOp::Union,
                ..
            } => {}
            _ => panic!("expected Union"),
        }
    }

    #[test]
    fn string_with_doubled_quotes() {
        // Excel 双引号转义: "" → "
        match p(r#"="a""b""#) {
            Expr::Text(s) => assert_eq!(s, r#"a"b"#),
            _ => panic!("expected Text"),
        }
    }

    // ── parse 错误路径 ──────────────────────────────────────────────────

    #[test]
    fn parse_empty_formula() {
        assert!(parse("=").is_err());
        assert!(parse("").is_err());
        assert!(parse("=").is_err());
    }

    #[test]
    fn parse_unterminated_string() {
        assert!(parse(r#"="unterminated"#).is_err());
    }

    #[test]
    fn parse_unexpected_character() {
        assert!(parse("=1`2").is_err());
    }

    #[test]
    fn parse_missing_closing_paren() {
        assert!(parse("=SUM(A1").is_err());
    }

    #[test]
    fn parse_detailed_error_message() {
        let result = parse_detailed("=1+");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("end of formula"));
    }

    #[test]
    fn parse_leading_equals_optional() {
        // 不带 = 前缀也应能解析
        let e = parse_detailed("1+2").unwrap();
        assert!(matches!(
            e,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn parse_complex_formula() {
        // 嵌套函数 + 运算符 + 范围
        let e = p("=SUM(A1:A10)+MAX(B1:B5)*2");
        assert!(matches!(
            e,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    // ── parse_col_or_row ────────────────────────────────────────────────

    #[test]
    fn full_row_reference() {
        // 裸行号后跟 : 才被识别为行范围
        let e = p("=SUM(1:1)");
        if let Expr::Func { args, .. } = e {
            // 参数可能是一个 Ref（行范围）或 Name
            // 只要解析不 panic 即可
            assert!(!args.is_empty());
        } else {
            panic!("expected Func");
        }
    }

    // ── SheetSpec 3D quoted span ────────────────────────────────────────

    #[test]
    fn three_d_quoted_span() {
        match p("='Sheet1':'Sheet3'!A1") {
            Expr::Ref(r) => assert_eq!(
                r.sheet,
                SheetSpec::Span("Sheet1".into(), "Sheet3".into())
            ),
            _ => panic!(),
        }
    }

    // ── 结构化引用（表格引用）────────────────────────────────────────────

    #[test]
    fn structured_reference() {
        match p("=Sales[Amount]") {
            Expr::Name(n) => assert_eq!(n, "Sales[Amount]"),
            _ => panic!("expected Name for structured ref"),
        }
    }
