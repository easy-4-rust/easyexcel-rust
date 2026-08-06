//! Demo package method-level 1:1 naming layer.
//!
//! Maps every Java `@Test` under `com.alibaba.easyexcel.test.demo.*` to a
//! searchable Rust `#[test]` name:
//! - `read.ReadTest#simpleRead` → `read_test_simple_read`
//! - `write.WriteTest#simpleWrite` → `write_test_simple_write`
//! - `fill.FillTest#simpleFill` → `fill_test_simple_fill`
//! - `rare.WriteTest#compressedTemporaryFile` → `rare_test_compressed_temporary_file`
//!
//! ## Inventory (Java `@Test` = 40)
//! - read.ReadTest: 12
//! - write.WriteTest: 20
//! - fill.FillTest: 6
//! - rare.WriteTest: 2
//!
//! ## web.WebTest
//! Spring `@Controller` only (`download` / `downloadFailedUsingJson` / `upload`);
//! **0 `@Test` methods** — documented here, no 1:1 test fn required.
//!
//! Existing logic lives in `demo_parity_tests.rs` / `demo_write_extra_tests.rs`;
//! this file is the searchable 1:1 naming layer (only-add). No soft-skip.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use easyexcel::{
    AnalysisContext, CellStyle, CellValue, ClientAnchorData, CommentData, CoordinateData,
    DynamicRow, DynamicValue, EasyExcel, ErrorAction, ExcelCellStyle, ExcelError, ExcelRow,
    FillConfig, FillWrapper, FormulaData, HorizontalCellStyleStrategy, HyperlinkData,
    HyperlinkType, ImageData, ImageType, LongestMatchColumnWidthStyleStrategy, LoopMergeStrategy,
    PageReadListener, ReadListener, Result, RichTextStringData, TemplateData, WriteCellContext,
    WriteCellData, WriteHandler, WriteWorkbookContext,
};
use tempfile::tempdir;

include!("demo_1to1_tests_cases/fixture_to_write_test_width_and_height_write.rs");
include!(
    "demo_1to1_tests_cases/write_test_annotation_style_write_to_rare_test_specified_cell_write.rs"
);
