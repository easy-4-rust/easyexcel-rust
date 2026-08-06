//! Java parity tests — write → read → assert field values.
//!
//! Each test mirrors a specific Java `@Test` method from easyexcel-test.
//! The goal is to produce identical results: same row count, same column
//! values, same header names.
//!
//! Java 11 missing test classes → 54 @Test methods total.
//!
//! Format strategy:
//! - `.xlsx`: Full write→read round-trip (Rust supports both read and write)
//! - `.xls`:  Prefer real BIFF8 write→read; advanced features Unsupported or
//!   fixture-backed read (never rewrite as XLSX)
//! - `.csv`:  Full write→read round-trip with CSV-specific structure assertions

use std::collections::HashSet;

use chrono::NaiveDate;
use easyexcel::{DynamicRow, DynamicValue, EasyExcel, ExcelRow};
use tempfile::tempdir;

// ============================================================================
// Helpers
// ============================================================================

include!("java_parity_tests_cases/temp_path_to_t03_complex_head_read_and_write_csv.rs");
include!("java_parity_tests_cases/assert_complex_head_no_auto_merge_to_no_head_data.rs");
include!("java_parity_tests_cases/assert_no_head_xlsx_to_encrypt_round_trip_xlsx.rs");
