#![allow(clippy::too_many_lines)]
    use super::*;

    fn enc(formula: &str) -> Vec<u8> {
        encode_formula_rpn(formula).unwrap_or_else(|e| panic!("编码失败 {formula}: {e}"))
    }

    fn hex(formula: &str) -> String {
        enc(formula)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    include!("tests/cases_01.rs");
