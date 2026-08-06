//! Date & Time functions.

use super::Registry;
use crate::formula::coerce::{to_number, to_text};
use crate::formula::context::Context;
use crate::formula::value::Value;
use chrono::{Datelike, Duration, NaiveDate};
use easyexcel_model::dates::{DateSystem, serial_time_parts};
use easyexcel_model::error::CellError;

include!("datetime/register_to_datedif_fn.rs");
include!("datetime/days_fn_to_tests.rs");
