#![allow(clippy::too_many_lines)]
    use super::*;
    use crate::formula::functions::testutil::{TestCtx, rng};

    fn c() -> TestCtx {
        TestCtx::new()
    }

    // Helper: compare floats with tolerance
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }
    fn approx_fine(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    include!("tests/cases_01.rs");
    include!("tests/cases_02.rs");
    include!("tests/cases_03.rs");
    include!("tests/cases_04.rs");
