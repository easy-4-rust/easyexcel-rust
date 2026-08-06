//! Lookup & Reference functions.

use super::Registry;
use crate::formula::coerce::{compare, to_number, to_text};
use crate::formula::context::Context;
use crate::formula::value::{Array, RefRange, Value};
use easyexcel_model::addr::{CellAddress, col_index_to_letters};
use easyexcel_model::error::CellError;

include!("lookup/register_to_areas_fn.rs");
include!("lookup/address_fn_to_tests.rs");
