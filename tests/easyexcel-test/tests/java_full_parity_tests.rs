//! Full Java parity tests — covers ALL 33 Java core test classes.
//!
//! Each test mirrors a specific Java `@Test` method from easyexcel-test.
//! Test logic, fixtures, and assertions are kept identical to Java.
//!
//! Format strategy:
//! - `.xlsx`: Full write→read round-trip
//! - `.xls`: Real BIFF8 write→read (or explicit Unsupported for encrypt/image/fill/template)
//! - `.csv`:  Full write→read round-trip with CSV structure verification

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use chrono::NaiveDate;
use easyexcel::{
    AnalysisContext, CellExtraType, DynamicRow, DynamicValue, EasyExcel, ErrorAction,
    ExcelCellStyle, ExcelColor, ExcelError, ExcelFillPattern, ExcelRow,
    HorizontalCellStyleStrategy, LoopMergeStrategy, PageReadListener, ReadDefaultReturn,
    ReadListener, SimpleColumnWidthStyleStrategy, SimpleRowHeightStyleStrategy,
    VerticalCellStyleStrategy, WriteCellData,
};
use tempfile::tempdir;
use zip::ZipArchive;

// ============================================================================
// Helpers
// ============================================================================

include!("java_full_parity_tests_cases/temp_path_to_encrypt_t04_stream_xls.rs");
include!("java_full_parity_tests_cases/converterdata_to_style_data10.rs");
include!(
    "java_full_parity_tests_cases/style_t01_read_and_write_xlsx_to_handler_t22_table_write_xls.rs"
);
include!(
    "java_full_parity_tests_cases/handler_t23_table_write_csv_to_converter_float_number_converter.rs"
);
