    #[test]
    fn binary_add_encodes_refs_with_relative_flags() {
        // A1+B1 → [tRef A1][tRef B1][tAdd]
        // tRef: 0x24 rw(00 00) col(00 C0: col=0 | rowRel 0x8000 | colRel 0x4000)
        assert_eq!(hex("A1+B1"), "24 00 00 00 c0 24 00 00 01 c0 03");
    }

    #[test]
    fn absolute_and_mixed_refs() {
        // $A$1 → col 字段无相对标志；A$1 → colRel(0x4000)
        assert_eq!(hex("$A$1"), "24 00 00 00 00");
        assert_eq!(hex("A$1"), "24 00 00 00 40");
        assert_eq!(hex("$A1"), "24 00 00 00 80");
    }

    #[test]
    fn integer_and_float_literals() {
        assert_eq!(hex("1+2"), "1e 01 00 1e 02 00 03");
        assert_eq!(hex("1.5"), "1f 00 00 00 00 00 00 f8 3f");
        assert_eq!(hex("32768"), "1f 00 00 00 00 00 00 e0 40");
    }

    #[test]
    fn string_and_bool_and_error() {
        assert_eq!(hex("\"Y\""), "17 01 00 59");
        assert_eq!(hex("\"中文\""), "17 02 01 2d 4e 87 65");
        assert_eq!(hex("TRUE"), "1d 01");
        assert_eq!(hex("#N/A"), "1c 2a");
        assert_eq!(hex("#DIV/0!"), "1c 07");
    }

    #[test]
    fn area_range_encodes_corners() {
        // SUM(A1:B2) → [tArea 0..1 × 0..1][tFuncVar(SUM, 1 参)]
        // tArea: 25 rwFirst(00 80: row0+rel) rwLast(01 80) colFirst(00 c0) colLast(01 c0)
        // SUM 索引 4, V 类 → base 0x42, cparams=1
        assert_eq!(hex("SUM(A1:B2)"), "25 00 00 01 00 00 c0 01 c0 42 01 04 00");
    }

    #[test]
    fn if_with_string_args_and_missing_arg() {
        // IF(A1>0,"Y","N")：A1 tRef, 0 tInt, tGT, "Y", "N", FuncVar(IF,3)
        assert_eq!(
            hex("IF(A1>0,\"Y\",\"N\")"),
            "24 00 00 00 c0 1e 00 00 0d 17 01 00 59 17 01 00 4e 22 03 01 00"
        );
        // IF(A1,,2) → 空参数 tMissArg
        assert_eq!(hex("IF(A1,,2)"), "24 00 00 00 c0 16 1e 02 00 22 03 01 00");
    }

    #[test]
    fn unary_and_percent_and_power() {
        // -2^2 = 4（Excel 语义：一元负号先于幂运算）
        assert_eq!(hex("-2^2"), "1e 02 00 13 1e 02 00 07");
        // 50% → tPercent
        assert_eq!(hex("50%"), "1e 32 00 14");
        // 2^3^2 右结合
        assert_eq!(hex("2^3^2"), "1e 02 00 1e 03 00 1e 02 00 07 07");
    }

    #[test]
    fn parenthesis_emits_tparen() {
        assert_eq!(
            hex("(A1+B1)*2"),
            "24 00 00 00 c0 24 00 00 01 c0 03 15 1e 02 00 05"
        );
    }

    #[test]
    fn concat_and_comparison() {
        assert_eq!(hex("A1&\"x\""), "24 00 00 00 c0 17 01 00 78 08");
        assert_eq!(hex("A1>=2"), "24 00 00 00 c0 1e 02 00 0c");
        assert_eq!(hex("A1<>B1"), "24 00 00 00 c0 24 00 00 01 c0 0e");
    }

    #[test]
    fn fixed_arg_function_uses_tfunc() {
        // ROUND(A1,2) 固定 2 参数 → tFunc(0x41+0x20? ROUND 是 V 类 → 0x41)
        // ROUND 索引 27
        assert_eq!(hex("ROUND(A1,2)"), "24 00 00 00 c0 1e 02 00 41 1b 00");
        // PI() 0 参数固定 → tFunc 0x41, 索引 19
        assert_eq!(hex("PI()"), "41 13 00");
    }

    #[test]
    fn variable_arg_function_uses_tfuncvar() {
        // SUM(A1,B1,2) → 3 参数 → FuncVar 0x42, cparams=3, ifunc=4
        assert_eq!(
            hex("SUM(A1,B1,2)"),
            "24 00 00 00 c0 24 00 00 01 c0 1e 02 00 42 03 04 00"
        );
    }

    #[test]
    fn leading_equals_is_stripped() {
        assert_eq!(enc("=A1+B1"), enc("A1+B1"));
    }

    #[test]
    fn nested_function_and_parens() {
        // IF(SUM(A1:B2)>10,SUM(C1:C2),0)
        let rpn = enc("IF(SUM(A1:B2)>10,SUM(C1:C2),0)");
        // 以 tFuncVar(IF) 结尾：0x42 03 01 00
        assert!(rpn.ends_with(&[0x22, 0x03, 0x01, 0x00]));
        // SUM 出现两次
        assert_eq!(
            rpn.windows(4)
                .filter(|w| *w == [0x42, 0x01, 0x04, 0x00])
                .count(),
            2
        );
    }

    #[test]
    fn errors_are_typed() {
        assert!(encode_formula_rpn("UNKNOWNFN(A1)").is_err());
        assert!(encode_formula_rpn("Sheet2!A1").is_err());
        assert!(encode_formula_rpn("A1:").is_err());
        assert!(encode_formula_rpn("(A1+B1").is_err());
        assert!(encode_formula_rpn("\"unterminated").is_err());
    }

    #[test]
    fn three_dimensional_refs_use_link_table_ixti() {
        let formulas = [
            "Sheet2!A1",
            "'销售 数据'!$B$2:$C$3",
            "Sheet1:Sheet2!A1",
        ];
        let links = Biff8LinkTable::from_formulas(
            &["Sheet1".to_owned(), "Sheet2".to_owned(), "销售 数据".to_owned()],
            &formulas,
        );
        assert_eq!(
            encode_formula_rpn_with_link_table(formulas[0], &links).unwrap(),
            vec![0x3a, 0, 0, 0, 0, 0, 0xc0]
        );
        assert_eq!(
            encode_formula_rpn_with_link_table(formulas[1], &links).unwrap(),
            vec![0x3b, 1, 0, 1, 0, 2, 0, 1, 0, 2, 0]
        );
        assert_eq!(links.supbook_payload(), [3, 0, 1, 4]);
        assert_eq!(
            links.externsheet_payload(),
            vec![3, 0, 0, 0, 1, 0, 1, 0, 0, 0, 2, 0, 2, 0, 0, 0, 0, 0, 1, 0]
        );
    }

    #[test]
    fn countif_cetab_index_is_encoded() {
        // COUNTIF=346 (0x015a)，固定 2 参数 → tFunc(0x21+0x20=0x41) + UShort
        let rpn = enc("COUNTIF(A1:A10,\">5\")");
        assert!(rpn.ends_with(&[0x41, 0x5a, 0x01]));
    }
