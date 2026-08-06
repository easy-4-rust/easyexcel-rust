#![allow(clippy::too_many_lines)]
#[cfg(test)]
    use super::*;

    fn decimal(value: &str) -> BigDecimal {
        value.parse().unwrap()
    }

    include!("tests_extra/cases_01.rs");
