//! Cross-validation tests: Java easyexcel ↔ easyexcel-rust
//!
//! These tests read Java-generated .xlsx/.csv fixtures with the Rust
//! library and compare the parsed results against the expected Java
//! output documented in `docs/compatibility.md`.

use std::path::PathBuf;
use std::str::FromStr;

use easyexcel::{DynamicRow, DynamicValue, EasyExcel, ExcelRow, ReadDefaultReturn};

// ============================================================================
// Fixtures path helper - use local fixtures copied from Java
// ============================================================================

include!("cross_validation_cases/fixtures_root_to_cross_validation_shared_strings.rs");
include!(
    "cross_validation_cases/cross_validation_csv_encoding_to_cross_validation_read_cell_data_mode.rs"
);
