//! Text worksheet functions.

use super::{Registry, VARIADIC, wildcard_match};
use crate::formula::coerce::{to_number, to_text};
use crate::formula::context::Context;
use crate::formula::value::{Array, Value};
use easyexcel_model::error::CellError;

include!("text/register_to_replace.rs");
include!("text/rept_to_value_to_text_str.rs");
include!("text/arraytotext_to_tests.rs");
