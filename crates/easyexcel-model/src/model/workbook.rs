//! The in-memory spreadsheet data model: [`Workbook`], [`Sheet`], [`Cell`].

use std::collections::BTreeMap;

use super::addr::{CellAddress, CellRange};
use super::dates::DateSystem;
use super::error::CellError;
use super::numfmt;
use super::styles::StyleTable;
use super::value::CellValue;

include!("workbook/cell_to_workbook_impl.rs");
include!("workbook/workbook_impl_to_tests.rs");
