//! Method-level 1:1 parity for Java core tests:
//! `AnnotationDataTest`, `AnnotationIndexAndNameDataTest`, `StyleDataTest`,
//! `WriteHandlerTest`, `ExcludeOrIncludeDataTest`.
//!
//! Naming: `mod <java_class_snake>` + `fn <java_method_snake>` so each Rust test
//! uniquely maps to `ClassName#methodName`.
//!
//! Format strategy:
//! - `.xlsx` / `.csv`: write → read round-trip with real assertions
//! - `.xls`: real BIFF8 write → read; XLSX-only style/dimension XML checks skipped

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

use easyexcel::{
    DynamicValue, EasyExcel, ExcelCellStyle, ExcelColor, ExcelFillPattern, ExcelRow,
    HorizontalCellStyleStrategy, LoopMergeStrategy, SimpleColumnWidthStyleStrategy,
    SimpleRowHeightStyleStrategy, VerticalCellStyleStrategy, WriteCellContext, WriteHandler,
    WriteRowContext, WriteSheetContext, WriteWorkbookContext,
};
use zip::ZipArchive;

include!(
    "core_annotation_style_handler_1to1_tests_cases/temp_path_to_assert_include_field_name_order.rs"
);
include!(
    "core_annotation_style_handler_1to1_tests_cases/assert_include_field_name_order_index_to_exclude_or_include_data_test.rs"
);
