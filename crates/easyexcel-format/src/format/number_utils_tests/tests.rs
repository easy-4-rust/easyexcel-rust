#![allow(clippy::too_many_lines)]
    use super::*;

    fn decimal(value: &str) -> BigDecimal {
        value.parse().unwrap()
    }

    include!("tests/cases_01.rs");
