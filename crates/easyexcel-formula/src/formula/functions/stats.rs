//! Statistical worksheet functions.

use super::{Criteria, Registry, VARIADIC, collect_numbers};
use crate::formula::coerce::to_number;
use crate::formula::context::Context;
use crate::formula::value::{Array, Value};
use easyexcel_model::error::CellError;

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

include!("stats/register_to_median.rs");
include!("stats/mode_mult_to_harmean.rs");
include!("stats/trimmean_to_beta_pdf.rs");
include!("stats/beta_dist_to_binom_dist_range.rs");
include!("stats/weibull_dist_to_tests.rs");
