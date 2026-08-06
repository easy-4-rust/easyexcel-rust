#![allow(clippy::too_many_lines)]
#[cfg(test)]
use super::*;
use crate::ReadOptions;
use crate::core::CellExtraType;

fn enabled_options() -> ReadOptions {
    let mut options = ReadOptions::default();
    options.extra_read.insert(CellExtraType::Hyperlink);
    options.extra_read.insert(CellExtraType::Merge);
    options.extra_read.insert(CellExtraType::Comment);
    options
}

include!("tests_extra/cases_01.rs");
