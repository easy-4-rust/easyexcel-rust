#![allow(clippy::too_many_lines)]
    use super::*;
    use crate::formula::functions::testutil::TestCtx;

    fn c() -> TestCtx {
        TestCtx::new()
    }
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    // --- Base conversions ---

    include!("tests/cases_01.rs");
    include!("tests/cases_02.rs");
