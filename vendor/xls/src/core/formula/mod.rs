//! The formula engine: lexer/parser, evaluator, recalculation, and the function
//! library.
//!
//! Layering:
//! * [`ast`] — the expression tree (parser output / evaluator input).
//! * [`parse`] — text → [`ast::Expr`].
//! * [`value`] — the evaluator's value type ([`value::Value`]).
//! * [`coerce`] — Excel coercion + comparison rules.
//! * [`context`] — the [`context::Context`] trait functions use.
//! * [`eval`] — the tree-walking [`eval::Evaluator`].
//! * [`engine`] — dependency graph + recalculation ([`engine::Engine`]).
//! * [`functions`] — the worksheet-function library + dispatch [`functions::Registry`].

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
    use crate::core::model::{Cell, Workbook};
    use crate::core::value::CellValue;

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
            Value::Error(crate::core::error::CellError::Div0)
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
        use crate::core::addr::CellRange;
        use crate::core::model::Table;
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
            Value::Error(crate::core::error::CellError::Num)
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
            Value::Error(crate::core::error::CellError::Div0)
        );
    }
}
