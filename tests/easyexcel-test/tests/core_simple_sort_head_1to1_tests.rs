//! Method-level 1:1 parity for Java core tests:
//! `SimpleDataTest`, `SortDataTest`, `SkipDataTest`, `NoModelDataTest`, `ParameterDataTest`,
//! `RepetitionDataTest`, `MultipleSheetsDataTest`, `ComplexHeadDataTest`, `ListHeadDataTest`,
//! `NoHeadDataTest`, `UnCamelDataTest`, `TemplateDataTest`.
//!
//! Naming: `mod <java_class_snake>` + `fn <java_method_snake>` so each Rust test
//! uniquely maps to `ClassName#methodName`.
//!
//! Format strategy:
//! - `.xlsx` / `.csv`: write → read round-trip
//! - `.xls`: real BIFF8 write → read; `.xls` template write is explicit `Unsupported`

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use easyexcel::{
    DynamicRow, DynamicValue, EasyExcel, ExcelRow, PageReadListener, ReadDefaultReturn,
};

include!("core_simple_sort_head_1to1_tests_cases/temp_path_to_simple_data_test.rs");
include!("core_simple_sort_head_1to1_tests_cases/sort_data_test_to_template_data_test.rs");
include!("core_simple_sort_head_1to1_tests_cases/complex_head_data_test.rs");
include!("core_simple_sort_head_1to1_tests_cases/list_head_data_test.rs");
include!("core_simple_sort_head_1to1_tests_cases/no_head_data_test.rs");
include!("core_simple_sort_head_1to1_tests_cases/multiple_sheets_data_test.rs");
include!("core_simple_sort_head_1to1_tests_cases/repetition_data_test.rs");
include!("core_simple_sort_head_1to1_tests_cases/un_camel_data_test.rs");
