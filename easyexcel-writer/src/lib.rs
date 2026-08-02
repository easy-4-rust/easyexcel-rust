//! XLSX writer backed by `rust_xlsxwriter`.

/// BIFF8 (`.xls`) writer — Java `ExcelTypeEnum.XLS` / POI HSSF subset.
pub mod biff8;
pub mod builder;
pub mod cell_style;
pub mod csv_encoding_writer;
pub(crate) mod excel_builder;
pub mod excel_output_stream;
pub mod excel_writer;
pub mod excel_writer_core;
pub mod executor;
pub mod global_configuration;
/// SXSSF `GZIPSheetDataWriter` equivalent — gzip row spill for `compress_temp_files`.
pub mod gzip_spill;
pub mod handler;
pub mod holder;
pub mod horizontal_alignment;
pub mod merge;
pub mod merge_range;
pub mod metadata;
pub mod property;
pub mod style;
pub(crate) mod template_write;
pub mod vertical_alignment;
/// Java `com.alibaba.excel.write` package-compatible API paths.
pub mod write;
pub mod write_csv;
pub mod write_options;
pub mod write_progress;
pub mod write_sheet;
pub mod write_xls;
pub(crate) mod writer_helpers;
pub mod xlsx_write;

pub use excel_writer_core::*;
#[allow(unused_imports)]
pub use write_csv::*;
#[allow(unused_imports)]
pub use write_xls::*;
#[allow(unused_imports)]
pub use xlsx_write::*;
