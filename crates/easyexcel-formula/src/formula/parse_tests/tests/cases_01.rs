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
