#![allow(clippy::too_many_lines)]
    use super::*;
    use crate::formula::functions::testutil::{TestCtx, rng};

    fn ctx() -> TestCtx {
        TestCtx::new()
    }

    // ---- hypothesis tests --------------------------------------------------

    include!("tests/cases_01.rs");
    include!("tests/cases_02.rs");
    include!("tests/cases_03.rs");
