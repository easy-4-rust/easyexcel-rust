//! Comprehensive test suite mirroring Java `com.alibaba.excel.test.core.*`.
//!
//! Java 33 test classes: `SimpleDataTest`, `AnnotationDataTest`, `ConverterDataTest`,
//! `CellDataDataTest`, `ExceptionDataTest`, `ExtraDataTest`, `FillDataTest`,
//! `NoModelDataTest`, `ExcludeOrIncludeDataTest`, `LargeDataTest`, `TemplateDataTest`,
//! `StyleDataTest`, `BomDataTest`, `CharsetDataTest`, `EncryptDataTest`, etc.

use ::bigdecimal::BigDecimal;
use ::url::Url;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime};

use super::*;
use crate::constant::{
    CELL_FORMULA_TAG, CELL_TAG, CELL_VALUE_TAG, EXCEL_MATH_CONTEXT_PRECISION, ROW_TAG,
    get_builtin_format,
};
use crate::support::ExcelTypeEnum;

// ============================================================================
// 1. CsvCharset tests (Java: CsvCharsetTest)
// ============================================================================

include!("tests_cases/cases_01.rs");
include!("tests_cases/cases_02.rs");
include!("tests_cases/cases_03.rs");
include!("tests_cases/cases_04.rs");
include!("tests_cases/cases_05.rs");
