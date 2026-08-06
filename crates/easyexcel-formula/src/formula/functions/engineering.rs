//! Engineering functions:
//!  – Number-base conversions (BIN2DEC, DEC2BIN, HEX2DEC, …)
//!  – Bitwise (BITAND, BITOR, BITXOR, BITLSHIFT, BITRSHIFT)
//!  – Comparison (DELTA, GESTEP)
//!  – Unit conversion (CONVERT)
//!  – Error functions (ERF, ERFC, ERF.PRECISE, ERFC.PRECISE)
//!  – Complex number arithmetic (COMPLEX, IMABS, IMREAL, …)
//!
//! PARITY:
//!  - Base conversions: negative numbers use 10-bit two's complement, matching Excel.
//!  - CONVERT: subset of units (mass, distance, time, temperature) with metric prefixes;
//!    unknown units return #N/A.
//!  - ERF/ERFC: use the standard series/continued-fraction approximation; tolerance ~1e-15.
//!  - Complex numbers: represented as text "a+bi" or "a+bj" (user-selectable suffix).
//!  - BESSEL* functions are not implemented (skipped — complex series; marked as note).

use super::{Registry, VARIADIC};
use crate::formula::coerce::to_number;
use crate::formula::context::Context;
use crate::formula::value::Value;
use easyexcel_model::error::CellError;

include!("engineering/register_to_kelvin_to_temp.rs");
include!("engineering/convert_to_tests.rs");
