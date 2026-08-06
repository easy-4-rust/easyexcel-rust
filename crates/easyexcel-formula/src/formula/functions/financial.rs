//! Financial functions (PMT, FV, PV, NPV, IRR, XNPV, XIRR, …).
//!
//! Standard annuity sign convention (matching Excel):
//!   • pv  = present value (positive = cash received)
//!   • pmt = periodic payment (negative = cash paid out)
//!   • fv  = future value (negative = cash paid out at end)
//!   • type = 0 (payments at period end, default) / 1 (beginning)
//!   • rate = periodic interest rate (not percent)
//!
//! Iterative solvers (RATE, IRR, XIRR) use Newton–Raphson with bisection
//! fallback, up to 100 iterations, tolerance 1e-7.
//! PARITY: precision tolerance 1e-7 may differ from Excel by a few ULPs.

use super::{Registry, VARIADIC};
use crate::formula::coerce::to_number;
use crate::formula::context::Context;
use crate::formula::value::Value;
use easyexcel_model::error::CellError;

include!("financial/register_to_xnpv.rs");
include!("financial/irr_to_intrate.rs");
include!("financial/received_to_pricemat.rs");
include!("financial/yieldmat_to_tests.rs");
