#![allow(clippy::too_many_lines)]
    use super::*;
    use crate::formula::functions::testutil::{TestCtx, rng};

    fn make_table() -> TestCtx {
        // A column (col 0): 1,2,3,4,5
        // B column (col 1): "apple","banana","cherry","date","elderberry"
        TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
            (4, 0, Value::Number(5.0)),
            (0, 1, Value::Text("apple".into())),
            (1, 1, Value::Text("banana".into())),
            (2, 1, Value::Text("cherry".into())),
            (3, 1, Value::Text("date".into())),
            (4, 1, Value::Text("elderberry".into())),
        ])
    }

    include!("tests/cases_01.rs");
    include!("tests/cases_02.rs");
