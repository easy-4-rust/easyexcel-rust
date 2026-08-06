//! Java golden cross-check — load checked-in `tests/golden/*.expected.json`
//! exported by `scripts/export-java-golden.sh` (true Java `EasyExcel` read/write)
//! and compare Rust read / write+read results (`row_count` + display cells).
//!
//! Missing golden files **fail** (no soft-skip). Run:
//! `./scripts/export-java-golden.sh` (requires JDK + Maven).
//!
//! Coverage scenarios (≥100 expected.json；ofNoRows=0):
//! - compatibility t01–t07/t09 (xlsx + xls), BOM csv, demo (xlsx/csv/extra/cellData)
//! - dataformat (xlsx/xls/v2/date1/date2), template, multi-sheet
//! - simple write (xlsx/csv/xls), converter (fixture + write xlsx/xls/csv), fill, style
//! - exclude/include, no-head(+xls/csv), sort, encrypt (password)
//! - cache / celldata(+xls/csv full) / charset / exception / handler /
//!   large-sample(+xls/csv) / nomodel / noncamel / parameter (xlsx/csv/xls) /
//!   repetition / skip / complex-head(+xls) / annotation-index(+xls) /
//!   list-head(+xls/csv) / fill-horizontal(+xls) / fill-by-name

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use easyexcel::{CellValue, DynamicRow, DynamicValue, EasyExcel, ReadDefaultReturn};
use serde::Deserialize;

include!("java_golden_tests_cases/fixture_to_golden_compatibility_t06.rs");
include!("java_golden_tests_cases/golden_compatibility_t07_to_golden_parameter_data.rs");
include!("java_golden_tests_cases/golden_repetition_data_to_golden_p0_format_full_rows.rs");
