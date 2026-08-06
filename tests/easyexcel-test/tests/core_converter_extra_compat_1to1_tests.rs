//! Method-level 1:1 parity for Java core tests outside simple/fill/annotation batches:
//! `CompatibilityTest`, `BomDataTest`, `CharsetDataTest`, `CacheDataTest`, `CellDataDataTest`,
//! `DateFormatTest`, `EncryptDataTest`, `ExceptionDataTest`, `ExtraDataTest`,
//! `ConverterDataTest`, `ConverterTest`, `LargeDataTest`.
//!
//! Naming: `mod <java_class_snake>` + `fn <java_method_snake>` → `ClassName#methodName`.
//! No soft-skip; only-add. May reuse `bom_data_tests` / `cross_validation` / `java_full_parity`
//! assertion logic while keeping dedicated 1:1 function names.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use easyexcel::{
    AnalysisContext, CellExtra, CellExtraType, CellValue, CsvCharset, DynamicRow, DynamicValue,
    EasyExcel, ErrorAction, ExcelError, ExcelLocale, ExcelRow, FillWrapper, FormulaData, ImageData,
    PageReadListener, ReadCacheMode, ReadDefaultReturn, ReadListener, Result, StringImageConverter,
    TemplateData, WriteCellData,
};

include!("core_converter_extra_compat_1to1_tests_cases/temp_path_to_cell_data_data_test.rs");
include!("core_converter_extra_compat_1to1_tests_cases/date_format_test_to_extra_data_test.rs");
include!("core_converter_extra_compat_1to1_tests_cases/converter_data_test_to_large_data_test.rs");
