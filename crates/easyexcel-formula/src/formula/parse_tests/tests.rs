#![allow(clippy::too_many_lines)]
use super::*;

fn p(s: &str) -> Expr {
    parse_detailed(s).unwrap_or_else(|e| panic!("parse {s:?}: {e}"))
}

include!("tests/cases_01.rs");
