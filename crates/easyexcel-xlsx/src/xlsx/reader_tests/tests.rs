#![allow(clippy::too_many_lines)]
use super::*;
use crate::xlsx::write;
use easyexcel_model::styles::{CellStyle, HAlign};
use std::io::{Cursor, Write};

fn roundtrip(wb: &Workbook) -> Workbook {
    let mut buf = Vec::new();
    write(wb, Cursor::new(&mut buf)).expect("write");
    read(Cursor::new(buf)).expect("read")
}

include!("tests/cases_01.rs");
