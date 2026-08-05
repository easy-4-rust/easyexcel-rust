//! `ExcelRow` 派生宏展开入口。

mod column;
mod derive;
mod field_expansion;
mod field_tokens;
mod metadata;
mod trait_impl;

pub(crate) use derive::expand_excel_row_tokens;
