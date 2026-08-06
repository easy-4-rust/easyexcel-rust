//! Math & trigonometry functions.

use std::cell::Cell as StdCell;

use super::{Criteria, Registry, VARIADIC, collect_numbers};
use crate::formula::coerce::to_number;
use crate::formula::context::Context;
use crate::formula::value::Value;
use easyexcel_model::error::CellError;

include!("math/register_to_sign.rs");
include!("math/mod_fn_to_sample_stdev.rs");
include!("math/pop_stdev_to_tests.rs");
