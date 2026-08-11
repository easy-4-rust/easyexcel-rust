//! The formula engine: lexer/parser, evaluator, recalculation, and the function
//! library.
//!
//! Layering:
//! * [`ast`] — the expression tree (parser output / evaluator input).
//! * [`parse()`] — text → [`ast::Expr`].
//! * [`value`] — the evaluator's value type ([`value::Value`]).
//! * [`coerce`] — Excel coercion + comparison rules.
//! * [`context`] — the [`context::Context`] trait functions use.
//! * [`eval`] — the tree-walking [`eval::Evaluator`].
//! * [`engine`] — dependency graph + recalculation ([`engine::Engine`]).
//! * [`functions`] — the worksheet-function library + dispatch [`functions::Registry`].

// Excel 公式参数统一处于 IEEE-754 double 数值域；与 Java EasyExcel/POI 一致，
// 整数型参数在各函数完成范围校验后采用截断转换，统计/财务算法再回到 double。
// 这些转换是兼容契约而非 Rust 领域模型的通用做法，因此豁免严格限定在公式模块。
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]
// 数学公式沿用论文和 Excel 规范中的短变量名；展开会降低公式可核对性。
#![allow(clippy::many_single_char_names, clippy::similar_names)]
// 解析器、依赖图和数值迭代保持单个算法块，便于与来源公式逐行核对。
#![allow(clippy::items_after_statements, clippy::too_many_lines)]
// 分支结构显式对应不同 Excel 错误/标记分支，即使当前返回值相同也不合并；
// 这能在与 Java 来源和函数规范核对时保留一一对应关系。
#![allow(
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::needless_continue,
    clippy::redundant_else
)]
// 工作表函数由统一函数指针签名注册，少数轻量参数必须随该 ABI 传递。
#![allow(clippy::needless_pass_by_value, clippy::trivially_copy_pass_by_ref)]
// 正负 instance 参数是 Excel 文本函数的三路契约，显式分支比排序匹配更清楚。
#![allow(clippy::comparison_chain)]

pub mod ast;
pub mod coerce;
pub mod context;
pub mod engine;
pub mod eval;
pub mod functions;
pub mod parse;
pub mod value;

pub use ast::Expr;
pub use context::{CellRef, Context};
pub use engine::{Engine, RecalcReport};
pub use eval::Evaluator;
pub use parse::{parse, parse_detailed};
pub use value::{Array, RefRange, Value};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use easyexcel_model::model::{Cell, Workbook};
    use easyexcel_model::value::CellValue;

    fn eval_str(formula: &str) -> Value {
        let wb = Workbook::new();
        let mut eng = Engine::new();
        eng.eval_formula(
            &wb,
            CellRef {
                sheet: 0,
                row: 0,
                col: 0,
            },
            formula,
        )
    }

    #[test]
    fn arithmetic() {
        assert_eq!(eval_str("=1+2*3"), Value::Number(7.0));
        assert_eq!(eval_str("=(1+2)*3"), Value::Number(9.0));
        assert_eq!(eval_str("=2^3^2"), Value::Number(512.0)); // right-assoc
        assert_eq!(eval_str("=-2^2"), Value::Number(4.0)); // unary binds tighter
        assert_eq!(
            eval_str("=10/0"),
            Value::Error(easyexcel_model::error::CellError::Div0)
        );
    }

    #[test]
    fn comparisons_and_text() {
        assert_eq!(eval_str("=1<2"), Value::Bool(true));
        assert_eq!(eval_str(r#"="a"&"b""#), Value::Text("ab".into()));
        assert_eq!(eval_str("=5%"), Value::Number(0.05));
    }

    #[test]
    fn functions_basic() {
        assert_eq!(eval_str("=SUM(1,2,3)"), Value::Number(6.0));
        assert_eq!(eval_str("=IF(1>2,10,20)"), Value::Number(20.0));
        assert_eq!(eval_str("=IFERROR(1/0,99)"), Value::Number(99.0));
        assert_eq!(eval_str("=PRODUCT(2,3,4)"), Value::Number(24.0));
    }

    #[test]
    fn structured_table_references() {
        use easyexcel_model::addr::CellRange;
        use easyexcel_model::model::Table;
        let mut wb = Workbook::new();
        {
            let s = wb.sheet_mut(0).unwrap();
            s.set_a1("A1", Cell::Text("Name".into()));
            s.set_a1("B1", Cell::Text("Amount".into()));
            s.set_a1("A2", Cell::Text("foo".into()));
            s.set_a1("B2", Cell::Number(10.0));
            s.set_a1("A3", Cell::Text("bar".into()));
            s.set_a1("B3", Cell::Number(20.0));
            s.set_a1("A4", Cell::Text("baz".into()));
            s.set_a1("B4", Cell::Number(30.0));
            s.tables.push(Table {
                name: "Sales".into(),
                display_name: "Sales".into(),
                range: CellRange::parse_a1("A1:B4").unwrap(),
                columns: vec!["Name".into(), "Amount".into()],
                header_rows: 1,
                totals_rows: 0,
                id: 1,
                raw_xml: Vec::new(),
            });
        }
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 0,
            col: 0,
        };
        assert_eq!(
            eng.eval_formula(&wb, at, "=SUM(Sales[Amount])"),
            Value::Number(60.0)
        );
        assert_eq!(
            eng.eval_formula(&wb, at, "=COUNTA(Sales[Name])"),
            Value::Number(3.0)
        );
        // Bare table name → data body (4 data cells across 2 cols, 3 rows).
        assert_eq!(
            eng.eval_formula(&wb, at, "=SUM(Sales)"),
            Value::Number(60.0)
        );
    }

    #[test]
    fn refs_and_recalc() {
        let mut wb = Workbook::new();
        let s = wb.sheet_mut(0).unwrap();
        s.set_a1("A1", Cell::Number(2.0));
        s.set_a1("A2", Cell::Number(3.0));
        s.set_a1(
            "A3",
            Cell::Formula {
                expr: "=SUM(A1:A2)*2".into(),
                cached: CellValue::Empty,
            },
        );
        let mut eng = Engine::new();
        eng.recalc(&mut wb);
        assert_eq!(
            wb.sheet_mut(0).unwrap().value(2, 0),
            CellValue::Number(10.0)
        );
    }

    fn approx(formula: &str, expected: f64) {
        match eval_str(formula) {
            Value::Number(v) => assert!(
                (v - expected).abs() < 1e-9,
                "{formula} = {v}, expected {expected}"
            ),
            other => panic!("{formula} => {other:?}, expected number {expected}"),
        }
    }

    #[test]
    fn legacy_compat_aliases_resolve() {
        // Pure synonyms delegate to their modern replacement.
        assert_eq!(eval_str("=MODE(1,2,2,3)"), eval_str("=MODE.SNGL(1,2,2,3)"));
        assert_eq!(
            eval_str("=POISSON(2,3,TRUE)"),
            eval_str("=POISSON.DIST(2,3,TRUE)")
        );
        assert_eq!(
            eval_str("=CRITBINOM(10,0.5,0.9)"),
            eval_str("=BINOM.INV(10,0.5,0.9)")
        );
        assert_eq!(eval_str("=NORMSINV(0.5)"), eval_str("=NORM.S.INV(0.5)"));

        // Wrappers add the implicit argument the modern signature expects.
        assert_eq!(eval_str("=NORMSDIST(0)"), eval_str("=NORM.S.DIST(0,TRUE)"));
        assert_eq!(
            eval_str("=LOGNORMDIST(4,3.5,1.2)"),
            eval_str("=LOGNORM.DIST(4,3.5,1.2,TRUE)")
        );
        assert_eq!(
            eval_str("=BETADIST(0.5,2,3)"),
            eval_str("=BETA.DIST(0.5,2,3,TRUE)")
        );
        assert_eq!(
            eval_str("=NEGBINOMDIST(10,5,0.25)"),
            eval_str("=NEGBINOM.DIST(10,5,0.25,FALSE)")
        );
        assert_eq!(
            eval_str("=HYPGEOMDIST(1,4,8,20)"),
            eval_str("=HYPGEOM.DIST(1,4,8,20,FALSE)")
        );
        // TDIST tails selector.
        assert_eq!(eval_str("=TDIST(1.5,10,1)"), eval_str("=T.DIST.RT(1.5,10)"));
        assert_eq!(eval_str("=TDIST(1.5,10,2)"), eval_str("=T.DIST.2T(1.5,10)"));
        assert_eq!(
            eval_str("=TDIST(-1,10,1)"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    /// Materialize an array/scalar formula result into row-major scalars.
    fn eval_grid(formula: &str) -> (usize, usize, Vec<Value>) {
        match eval_str(formula) {
            Value::Array(a) => (a.rows, a.cols, a.data),
            other => (1, 1, vec![other]),
        }
    }

    fn nums(formula: &str) -> Vec<f64> {
        eval_grid(formula)
            .2
            .into_iter()
            .map(|v| match v {
                Value::Number(n) => n,
                other => panic!("expected number, got {other:?}"),
            })
            .collect()
    }

    #[test]
    fn array_broadcast_operators() {
        // A range compared to a scalar yields a boolean array.
        assert_eq!(
            eval_grid("={1;2;3}>2").2,
            vec![Value::Bool(false), Value::Bool(false), Value::Bool(true)]
        );
        // Element-wise arithmetic, scalar broadcast.
        assert_eq!(nums("={1,2,3}*2"), vec![2.0, 4.0, 6.0]);
        // Unary negation over an array.
        assert_eq!(nums("=-{1,2,3}"), vec![-1.0, -2.0, -3.0]);
    }

    #[test]
    fn sort_and_sortby() {
        assert_eq!(nums("=SORT({3;1;2})"), vec![1.0, 2.0, 3.0]);
        assert_eq!(nums("=SORT({3;1;2},1,-1)"), vec![3.0, 2.0, 1.0]);
        // SORTBY orders the first array by the key array.
        assert_eq!(
            eval_grid(r#"=SORTBY({"c";"a";"b"},{3;1;2})"#).2,
            vec![
                Value::Text("a".into()),
                Value::Text("b".into()),
                Value::Text("c".into())
            ]
        );
    }

    #[test]
    fn unique_and_filter() {
        assert_eq!(nums("=UNIQUE({1;2;2;3;1})"), vec![1.0, 2.0, 3.0]);
        // FILTER with a boolean-array condition derived from a comparison.
        assert_eq!(nums("=FILTER({1;2;3;4},{1;2;3;4}>2)"), vec![3.0, 4.0]);
        // No matches → if_empty fallback.
        assert_eq!(
            eval_str(r#"=FILTER({1;2},{0;0},"none")"#),
            Value::Text("none".into())
        );
    }

    #[test]
    fn sequence_take_drop_stack() {
        assert_eq!(nums("=SEQUENCE(1,5)"), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(nums("=SEQUENCE(1,3,10,5)"), vec![10.0, 15.0, 20.0]);
        assert_eq!(nums("=TAKE({1;2;3;4},2)"), vec![1.0, 2.0]);
        assert_eq!(nums("=TAKE({1;2;3;4},-2)"), vec![3.0, 4.0]);
        assert_eq!(nums("=DROP({1;2;3;4},1)"), vec![2.0, 3.0, 4.0]);
        assert_eq!(nums("=VSTACK({1;2},{3;4})"), vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(nums("=CHOOSECOLS({1,2,3},3,1)"), vec![3.0, 1.0]);
    }

    #[test]
    fn textsplit_and_mode_mult() {
        // Column split → single row.
        assert_eq!(
            eval_grid(r#"=TEXTSPLIT("a,b,c",",")"#).2,
            vec![
                Value::Text("a".into()),
                Value::Text("b".into()),
                Value::Text("c".into())
            ]
        );
        // Row + column delimiters → 2×2 grid.
        let (r, c, _) = eval_grid(r#"=TEXTSPLIT("a,b;c,d",",",";")"#);
        assert_eq!((r, c), (2, 2));
        // MODE.MULT returns all modes (2 and 3 both appear twice).
        assert_eq!(nums("=MODE.MULT({1;2;2;3;3})"), vec![2.0, 3.0]);
    }

    #[test]
    fn let_bindings() {
        assert_eq!(eval_str("=LET(x,5,x+1)"), Value::Number(6.0));
        // Later bindings can reference earlier ones.
        assert_eq!(eval_str("=LET(x,2,y,x*3,x+y)"), Value::Number(8.0));
    }

    #[test]
    fn lambda_and_higher_order() {
        // LAMBDA bound by LET, then called by name.
        assert_eq!(
            eval_str("=LET(inc,LAMBDA(n,n+1),inc(9))"),
            Value::Number(10.0)
        );
        // MAP element-wise.
        assert_eq!(nums("=MAP({1,2,3},LAMBDA(x,x*x))"), vec![1.0, 4.0, 9.0]);
        // REDUCE folds to a scalar.
        assert_eq!(
            eval_str("=REDUCE(0,{1,2,3,4},LAMBDA(a,b,a+b))"),
            Value::Number(10.0)
        );
        // SCAN is the running fold.
        assert_eq!(
            nums("=SCAN(0,{1,2,3},LAMBDA(a,b,a+b))"),
            vec![1.0, 3.0, 6.0]
        );
        // BYROW reduces each row.
        assert_eq!(nums("=BYROW({1,2;3,4},LAMBDA(r,SUM(r)))"), vec![3.0, 7.0]);
        // MAKEARRAY builds from (row, col).
        assert_eq!(
            nums("=MAKEARRAY(2,2,LAMBDA(r,c,r*c))"),
            vec![1.0, 2.0, 2.0, 4.0]
        );
    }

    #[test]
    fn isomitted_default_args() {
        let f = "LAMBDA(a,b,IF(ISOMITTED(b),a,a+b))";
        assert_eq!(eval_str(&format!("=LET(f,{f},f(5))")), Value::Number(5.0));
        assert_eq!(eval_str(&format!("=LET(f,{f},f(5,2))")), Value::Number(7.0));
    }

    #[test]
    fn percentof_basic() {
        approx("=PERCENTOF(25,100)", 0.25);
        // subset / total of literal sums
        approx("=PERCENTOF(10,40)", 0.25);
        assert_eq!(
            eval_str("=PERCENTOF(1,0)"),
            Value::Error(easyexcel_model::error::CellError::Div0)
        );
    }

    // ── Evaluator 特殊形式深度测试 ──────────────────────────────────────

    #[test]
    fn if_true_branch() {
        assert_eq!(eval_str("=IF(TRUE,1,2)"), Value::Number(1.0));
    }

    #[test]
    fn if_false_branch() {
        assert_eq!(eval_str("=IF(FALSE,1,2)"), Value::Number(2.0));
    }

    #[test]
    fn if_no_false_branch() {
        // IF(cond, val) — 无 false 分支时返回 FALSE
        assert_eq!(eval_str("=IF(FALSE,1)"), Value::Bool(false));
    }

    #[test]
    fn if_error_in_condition() {
        assert_eq!(
            eval_str("=IF(#N/A,1,2)"),
            Value::Error(easyexcel_model::error::CellError::NA)
        );
    }

    #[test]
    fn if_wrong_arg_count() {
        assert_eq!(
            eval_str("=IF()"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn iferror_catches_error() {
        assert_eq!(eval_str("=IFERROR(1/0,99)"), Value::Number(99.0));
    }

    #[test]
    fn iferror_passes_non_error() {
        assert_eq!(eval_str("=IFERROR(42,99)"), Value::Number(42.0));
    }

    #[test]
    fn ifna_catches_na_only() {
        assert_eq!(eval_str("=IFNA(#N/A,99)"), Value::Number(99.0));
        // 其他错误不被 IFNA 捕获
        assert_eq!(
            eval_str("=IFNA(#VALUE!,99)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn iferror_wrong_arg_count() {
        assert_eq!(
            eval_str("=IFERROR(1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn choose_basic() {
        assert_eq!(eval_str("=CHOOSE(2,10,20,30)"), Value::Number(20.0));
    }

    #[test]
    fn choose_out_of_bounds() {
        assert_eq!(
            eval_str("=CHOOSE(5,10,20)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn choose_zero_index() {
        assert_eq!(
            eval_str("=CHOOSE(0,10,20)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn ifs_basic() {
        assert_eq!(eval_str("=IFS(FALSE,1,TRUE,2,FALSE,3)"), Value::Number(2.0));
    }

    #[test]
    fn ifs_no_match() {
        assert_eq!(
            eval_str("=IFS(FALSE,1,FALSE,2)"),
            Value::Error(easyexcel_model::error::CellError::NA)
        );
    }

    #[test]
    fn ifs_odd_args() {
        // IFS(TRUE,1) 只有 1 对，不是奇数参数问题；它会匹配并返回 1
        assert_eq!(eval_str("=IFS(TRUE,1)"), Value::Number(1.0));
        // 真正的奇数参数：IFS(TRUE) 只有 1 个参数
        assert_eq!(
            eval_str("=IFS(TRUE)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn switch_basic() {
        assert_eq!(
            eval_str("=SWITCH(2,1,\"a\",2,\"b\",3,\"c\")"),
            Value::Text("b".into())
        );
    }

    #[test]
    fn switch_default() {
        assert_eq!(
            eval_str("=SWITCH(99,1,\"a\",2,\"b\",\"default\")"),
            Value::Text("default".into())
        );
    }

    #[test]
    fn switch_no_match_no_default() {
        assert_eq!(
            eval_str("=SWITCH(99,1,\"a\",2,\"b\")"),
            Value::Error(easyexcel_model::error::CellError::NA)
        );
    }

    #[test]
    fn switch_too_few_args() {
        assert_eq!(
            eval_str("=SWITCH(1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn and_basic() {
        assert_eq!(eval_str("=AND(TRUE,TRUE)"), Value::Bool(true));
        assert_eq!(eval_str("=AND(TRUE,FALSE)"), Value::Bool(false));
    }

    #[test]
    fn and_empty_args() {
        assert_eq!(
            eval_str("=AND()"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn or_basic() {
        assert_eq!(eval_str("=OR(FALSE,TRUE)"), Value::Bool(true));
        assert_eq!(eval_str("=OR(FALSE,FALSE)"), Value::Bool(false));
    }

    #[test]
    fn or_error_propagation() {
        assert_eq!(
            eval_str("=OR(TRUE,#N/A)"),
            // OR 短路：TRUE 后不再求值
            Value::Bool(true)
        );
    }

    #[test]
    fn not_fn() {
        assert_eq!(eval_str("=NOT(FALSE)"), Value::Bool(true));
        assert_eq!(eval_str("=NOT(TRUE)"), Value::Bool(false));
    }

    // ── 引用运算符 ─────────────────────────────────────────────────────

    #[test]
    fn range_operator() {
        // A1:B2 构造范围
        let v = eval_str("=ROWS(A1:B3)");
        assert_eq!(v, Value::Number(3.0));
    }

    #[test]
    fn intersect_operator() {
        // A1:C1 与 B1:B3 的交集是 B1
        let wb = Workbook::new();
        let mut eng = Engine::new();
        eng.eval_formula(
            &wb,
            CellRef {
                sheet: 0,
                row: 0,
                col: 0,
            },
            "=A1:C1 B1:B3",
        );
        // 这里主要是验证不 panic
    }

    // ── Agent 68 panic 回归：无效日期路径 ───────────────────────────────

    #[test]
    fn year_fn_invalid_serial_no_panic() {
        // 无效序列号（负数）应返回 #VALUE! 而非 panic
        assert_eq!(
            eval_str("=YEAR(-1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn month_fn_invalid_serial_no_panic() {
        assert_eq!(
            eval_str("=MONTH(-1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn day_fn_invalid_serial_no_panic() {
        assert_eq!(
            eval_str("=DAY(-1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn edate_fn_invalid_serial_no_panic() {
        assert_eq!(
            eval_str("=EDATE(-1,1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn eomonth_fn_invalid_serial_no_panic() {
        assert_eq!(
            eval_str("=EOMONTH(-1,0)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn weekday_fn_invalid_serial_no_panic() {
        assert_eq!(
            eval_str("=WEEKDAY(-1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn datedif_start_after_end_no_panic() {
        // start > end 应返回 #NUM! 而非 panic
        assert_eq!(
            eval_str("=DATEDIF(45000,44000,\"D\")"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    #[test]
    fn isoweeknum_invalid_no_panic() {
        assert_eq!(
            eval_str("=ISOWEEKNUM(-1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    // ── Agent 68 panic 回归：NaN 运算路径 ──────────────────────────────

    #[test]
    fn sum_with_nan_in_range_no_panic() {
        let mut wb = Workbook::new();
        wb.sheet_mut(0)
            .unwrap()
            .set_a1("A1", Cell::Number(f64::NAN));
        wb.sheet_mut(0).unwrap().set_a1("A2", Cell::Number(5.0));
        let mut eng = Engine::new();
        let at = CellRef {
            sheet: 0,
            row: 0,
            col: 0,
        };
        // SUM 应跳过 NaN 或产生有限结果，不 panic
        let _v = eng.eval_formula(&wb, at, "=SUM(A1:A2)");
    }

    #[test]
    fn comparison_with_nan_no_panic() {
        // NaN 比较不应 panic — 通过让 NaN 出现在计算中来触发
        // 0/0 产生 NaN → 与标量比较不 panic
        let _ = eval_str("=0/0>1");
        let _ = eval_str("=0/0<1");
    }

    // ── 动态数组函数测试 ────────────────────────────────────────────────

    #[test]
    fn sequence_2d() {
        let (r, c, data) = eval_grid("=SEQUENCE(2,3,1,1)");
        assert_eq!((r, c), (2, 3));
        assert_eq!(data.len(), 6);
    }

    #[test]
    fn sequence_negative_dims() {
        assert_eq!(
            eval_str("=SEQUENCE(-1,1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn randarray_basic() {
        let (r, c, data) = eval_grid("=RANDARRAY(2,3)");
        assert_eq!((r, c), (2, 3));
        assert_eq!(data.len(), 6);
        for v in &data {
            if let Value::Number(n) = v {
                assert!(*n >= 0.0 && *n < 1.0);
            }
        }
    }

    #[test]
    fn randarray_invalid_dims() {
        assert_eq!(
            eval_str("=RANDARRAY(-1,1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn vstack_basic() {
        assert_eq!(nums("=VSTACK({1,2},{3,4})"), vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn hstack_basic() {
        assert_eq!(nums("=HSTACK({1;2},{3;4})"), vec![1.0, 3.0, 2.0, 4.0]);
    }

    #[test]
    fn torow_basic() {
        assert_eq!(nums("=TOROW({1,2;3,4})"), vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn tocol_basic() {
        // TOCOL 按行扫描默认: {1,2;3,4} → [1,2,3,4] 作为 4×1 列
        let (r, c, data) = eval_grid("=TOCOL({1,2;3,4})");
        assert_eq!((r, c), (4, 1));
        assert_eq!(data[0], Value::Number(1.0));
        assert_eq!(data[1], Value::Number(2.0));
        assert_eq!(data[2], Value::Number(3.0));
        assert_eq!(data[3], Value::Number(4.0));
    }

    #[test]
    fn wraprows_basic() {
        let (r, c, _) = eval_grid("=WRAPROWS({1;2;3;4},2)");
        assert_eq!((r, c), (2, 2));
    }

    #[test]
    fn wrapcols_basic() {
        let (r, c, _) = eval_grid("=WRAPCOLS({1;2;3;4},2)");
        assert_eq!((r, c), (2, 2));
    }

    #[test]
    fn expand_basic() {
        let (r, c, _) = eval_grid("=EXPAND({1,2;3,4},3,3,0)");
        assert_eq!((r, c), (3, 3));
    }

    #[test]
    fn expand_too_small() {
        assert_eq!(
            eval_str("=EXPAND({1,2;3,4},1,1)"),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn chooserows_basic() {
        assert_eq!(nums("=CHOOSEROWS({1;2;3;4},3,1)"), vec![3.0, 1.0]);
    }

    #[test]
    fn chooserows_negative_index() {
        assert_eq!(nums("=CHOOSEROWS({1;2;3;4},-1)"), vec![4.0]);
    }

    #[test]
    fn choosecols_basic() {
        assert_eq!(nums("=CHOOSECOLS({1,2,3;4,5,6},2)"), vec![2.0, 5.0]);
    }

    #[test]
    fn trimrange_basic() {
        let (r, c, _) = eval_grid("=TRIMRANGE({0,0;1,2;0,0})");
        // 0 在数组常量中是数字不是空值，TRIMRANGE 不会裁剪它
        assert_eq!(r, 3);
        assert_eq!(c, 2);
    }

    // ── 数组常量 ────────────────────────────────────────────────────────

    #[test]
    fn array_constant_1d_row() {
        assert_eq!(nums("={1,2,3}"), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn array_constant_1d_col() {
        assert_eq!(nums("={1;2;3}"), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn array_constant_2d() {
        let (r, c, _) = eval_grid("={1,2;3,4}");
        assert_eq!((r, c), (2, 2));
    }

    // ── 信息函数 ────────────────────────────────────────────────────────

    #[test]
    fn isblank_integration() {
        assert_eq!(eval_str("=ISBLANK(1)"), Value::Bool(false));
    }

    #[test]
    fn iserror_integration() {
        assert_eq!(eval_str("=ISERROR(1/0)"), Value::Bool(true));
        assert_eq!(eval_str("=ISERROR(42)"), Value::Bool(false));
    }

    #[test]
    fn iseven_isodd_integration() {
        assert_eq!(eval_str("=ISEVEN(4)"), Value::Bool(true));
        assert_eq!(eval_str("=ISODD(3)"), Value::Bool(true));
    }

    #[test]
    fn type_integration() {
        assert_eq!(eval_str("=TYPE(42)"), Value::Number(1.0));
        assert_eq!(eval_str(r#"=TYPE("hi")"#), Value::Number(2.0));
        assert_eq!(eval_str("=TYPE(TRUE)"), Value::Number(4.0));
    }

    #[test]
    fn error_type_integration() {
        assert_eq!(eval_str("=ERROR.TYPE(#N/A)"), Value::Number(7.0));
        assert_eq!(
            eval_str("=ERROR.TYPE(42)"),
            Value::Error(easyexcel_model::error::CellError::NA)
        );
    }

    #[test]
    fn na_integration() {
        assert_eq!(
            eval_str("=NA()"),
            Value::Error(easyexcel_model::error::CellError::NA)
        );
    }

    // ── 数学函数边界值 ──────────────────────────────────────────────────

    #[test]
    fn abs_integration() {
        assert_eq!(eval_str("=ABS(-5)"), Value::Number(5.0));
        assert_eq!(eval_str("=ABS(5)"), Value::Number(5.0));
    }

    #[test]
    fn sqrt_negative() {
        assert_eq!(
            eval_str("=SQRT(-1)"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    #[test]
    fn ln_negative() {
        assert_eq!(
            eval_str("=LN(-1)"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    #[test]
    fn log10_negative() {
        assert_eq!(
            eval_str("=LOG10(-1)"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    #[test]
    fn power_negative_fractional() {
        // (-2)^0.5 → #NUM!
        assert_eq!(
            eval_str("=POWER(-2,0.5)"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    #[test]
    fn divide_by_zero() {
        assert_eq!(
            eval_str("=1/0"),
            Value::Error(easyexcel_model::error::CellError::Div0)
        );
    }

    // ── 文本函数 ────────────────────────────────────────────────────────

    #[test]
    fn len_integration() {
        assert_eq!(eval_str(r#"=LEN("hello")"#), Value::Number(5.0));
    }

    #[test]
    fn upper_lower_integration() {
        assert_eq!(eval_str(r#"=UPPER("hello")"#), Value::Text("HELLO".into()));
        assert_eq!(eval_str(r#"=LOWER("HELLO")"#), Value::Text("hello".into()));
    }

    #[test]
    fn left_right_mid_integration() {
        assert_eq!(eval_str(r#"=LEFT("hello",2)"#), Value::Text("he".into()));
        assert_eq!(eval_str(r#"=RIGHT("hello",2)"#), Value::Text("lo".into()));
        assert_eq!(eval_str(r#"=MID("hello",2,3)"#), Value::Text("ell".into()));
    }

    #[test]
    fn trim_integration() {
        assert_eq!(
            eval_str(r#"=TRIM("  hello  ")"#),
            Value::Text("hello".into())
        );
    }

    #[test]
    fn substitute_integration() {
        assert_eq!(
            eval_str(r#"=SUBSTITUTE("hello","l","r")"#),
            Value::Text("herro".into())
        );
    }

    #[test]
    fn concatenate_integration() {
        assert_eq!(
            eval_str(r#"=CONCATENATE("a","b","c")"#),
            Value::Text("abc".into())
        );
    }

    // ── 逻辑函数边界 ───────────────────────────────────────────────────

    #[test]
    fn if_nested() {
        assert_eq!(eval_str("=IF(TRUE,IF(FALSE,1,2),3)"), Value::Number(2.0));
    }

    #[test]
    fn if_text_condition() {
        // 非空文本无法转为布尔 → #VALUE!
        assert_eq!(
            eval_str(r#"=IF("x",1,2)"#),
            Value::Error(easyexcel_model::error::CellError::Value)
        );
    }

    #[test]
    fn if_number_condition() {
        assert_eq!(eval_str("=IF(0,1,2)"), Value::Number(2.0));
        assert_eq!(eval_str("=IF(1,1,2)"), Value::Number(1.0));
    }

    // ── 日期函数 ────────────────────────────────────────────────────────

    #[test]
    fn date_basic_integration() {
        // DATE(2023,1,1) → serial 44927
        assert_eq!(eval_str("=DATE(2023,1,1)"), Value::Number(44927.0));
    }

    #[test]
    fn time_basic_integration() {
        // TIME(12,0,0) → 0.5
        assert_eq!(eval_str("=TIME(12,0,0)"), Value::Number(0.5));
    }

    #[test]
    fn date_two_digit_year() {
        // Year 25 → 2025
        let v = eval_str("=DATE(25,6,15)");
        if let Value::Number(n) = v {
            assert!(n > 45000.0); // 2025 年的序列号
        }
    }

    #[test]
    fn date_negative_year() {
        assert_eq!(
            eval_str("=DATE(-1,1,1)"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    // ── 查找函数 ────────────────────────────────────────────────────────

    #[test]
    fn row_column_integration() {
        assert_eq!(eval_str("=ROW(A5)"), Value::Number(5.0));
        assert_eq!(eval_str("=COLUMN(C1)"), Value::Number(3.0));
    }

    #[test]
    fn rows_columns_integration() {
        assert_eq!(eval_str("=ROWS(A1:C5)"), Value::Number(5.0));
        assert_eq!(eval_str("=COLUMNS(A1:C5)"), Value::Number(3.0));
    }

    // ── 比较运算符全面覆盖 ─────────────────────────────────────────────

    #[test]
    fn all_comparison_ops() {
        assert_eq!(eval_str("=1=1"), Value::Bool(true));
        assert_eq!(eval_str("=1<>2"), Value::Bool(true));
        assert_eq!(eval_str("=1<2"), Value::Bool(true));
        assert_eq!(eval_str("=1<=1"), Value::Bool(true));
        assert_eq!(eval_str("=2>1"), Value::Bool(true));
        assert_eq!(eval_str("=1>=1"), Value::Bool(true));
    }

    // ── 算术运算边界 ───────────────────────────────────────────────────

    #[test]
    fn arithmetic_overflow() {
        // 极大数运算应返回 #NUM!
        assert_eq!(
            eval_str("=1E308*10"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    #[test]
    fn pow_nan_result() {
        // (-1)^0.5 → NaN → #NUM!
        assert_eq!(
            eval_str("=(-1)^0.5"),
            Value::Error(easyexcel_model::error::CellError::Num)
        );
    }

    // ── 统计函数 ────────────────────────────────────────────────────────

    #[test]
    fn average_integration() {
        assert_eq!(eval_str("=AVERAGE(1,2,3,4,5)"), Value::Number(3.0));
    }

    #[test]
    fn count_integration() {
        assert_eq!(eval_str("=COUNT(1,2,3)"), Value::Number(3.0));
    }

    #[test]
    fn counta_integration() {
        assert_eq!(eval_str(r#"=COUNTA(1,"hi",TRUE)"#), Value::Number(3.0));
    }

    #[test]
    fn max_min_integration() {
        assert_eq!(eval_str("=MAX(1,5,3)"), Value::Number(5.0));
        assert_eq!(eval_str("=MIN(1,5,3)"), Value::Number(1.0));
    }

    // ── 跨表引用 ────────────────────────────────────────────────────────

    #[test]
    fn unknown_sheet_ref_returns_ref_error() {
        assert_eq!(
            eval_str("=UnknownSheet!A1"),
            Value::Error(easyexcel_model::error::CellError::Ref)
        );
    }

    // ── 定义名称 ────────────────────────────────────────────────────────

    #[test]
    fn unknown_name_returns_name_error() {
        assert_eq!(
            eval_str("=NonExistentName"),
            Value::Error(easyexcel_model::error::CellError::Name)
        );
    }

    // ── 错误传播 ────────────────────────────────────────────────────────

    #[test]
    fn error_propagation_in_binary() {
        assert_eq!(
            eval_str("=#N/A+1"),
            Value::Error(easyexcel_model::error::CellError::NA)
        );
        assert_eq!(
            eval_str("=1+#REF!"),
            Value::Error(easyexcel_model::error::CellError::Ref)
        );
    }

    #[test]
    fn error_propagation_in_unary() {
        assert_eq!(
            eval_str("=-#N/A"),
            Value::Error(easyexcel_model::error::CellError::NA)
        );
    }

    // ── 布尔函数集成 ────────────────────────────────────────────────────

    #[test]
    fn true_false_literals() {
        assert_eq!(eval_str("=TRUE"), Value::Bool(true));
        assert_eq!(eval_str("=FALSE"), Value::Bool(false));
    }

    #[test]
    fn xor_integration() {
        assert_eq!(eval_str("=XOR(TRUE,FALSE)"), Value::Bool(true));
        assert_eq!(eval_str("=XOR(TRUE,TRUE)"), Value::Bool(false));
    }
}
