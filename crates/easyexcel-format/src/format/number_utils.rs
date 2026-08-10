//! Java-compatible number conversion helpers.
//!
//! Mirrors `com.alibaba.excel.util.NumberUtils` and the `DecimalFormat`
//! subset used by `EasyExcel`'s built-in numeric string converters.

use std::str::FromStr;

use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::BigInt;

include!("number_utils/excel_math_context_precision_to_parse_long.rs");
include!("number_utils/parse_integer_to_tests_extra.rs");
