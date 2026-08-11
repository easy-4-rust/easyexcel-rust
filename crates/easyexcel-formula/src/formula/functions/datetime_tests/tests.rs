#![allow(clippy::too_many_lines)]
    use super::*;
    use crate::formula::functions::testutil::{TestCtx, rng};

    fn ctx() -> TestCtx {
        TestCtx::new()
    }

    // DATE(2023,1,1) in 1900 system → serial 44927
    include!("tests/cases_01.rs");
    include!("tests/cases_02.rs");
