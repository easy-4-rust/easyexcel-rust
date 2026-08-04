//! XLSX writer backed by `rust_xlsxwriter`.

/// BIFF8 (`.xls`) writer — Java `ExcelTypeEnum.XLS` / POI HSSF subset.
pub mod biff8;
pub mod builder;
pub mod cell_style;
pub mod csv_encoding_writer;
pub(crate) mod excel_builder;
pub mod excel_output_stream;
#[path = "../excel_writer.rs"]
pub mod excel_writer;
pub mod excel_writer_builder;
pub mod excel_writer_core;
pub mod executor;
pub mod global_configuration;
/// SXSSF `GZIPSheetDataWriter` equivalent — gzip row spill for `compress_temp_files`.
pub mod gzip_spill;
pub mod handler;
/// Holder 模块镜像 — 指向 `write/metadata/holder`。
pub use crate::write::metadata::holder;
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

/// ExcelWriter 内部实现拆分模块（追加行写入）。
pub(crate) mod append_rows;
/// ExcelWriter 内部实现拆分模块（对应 Java `WorkBookUtil` Creator 实现族）。
pub(crate) mod creators;
/// ExcelWriter 内部实现拆分模块（Handler 执行链作用域）。
pub(crate) mod handler_execution_scope;
/// ExcelWriter 内部实现拆分模块（图片像素布局）。
pub(crate) mod image_layout;
/// ExcelWriter 内部实现拆分模块（Handler 共享包装）。
pub(crate) mod shared_write_handler;
/// ExcelWriter 内部实现拆分模块（工作表样式上下文）。
pub(crate) mod sheet_style_context;

pub use excel_writer_core::*;
#[allow(unused_imports)]
pub use write_csv::*;
#[allow(unused_imports)]
pub use write_xls::*;
#[allow(unused_imports)]
pub use xlsx_write::*;

pub use crate::context::write_backend_handle;
pub use crate::context::write_backend_handle::*;
pub use crate::context::write_cell_context;
pub use crate::context::write_cell_context::*;
pub use crate::context::write_context;
pub use crate::context::write_context::*;
pub use crate::context::write_fill_executor;
pub use crate::context::write_fill_executor::*;
pub use crate::context::write_handler;
pub use crate::context::write_handler::*;
pub use crate::context::write_holder_context;
pub use crate::context::write_holder_context::*;
pub use crate::context::write_row_context;
pub use crate::context::write_row_context::*;
pub use crate::context::write_sheet_context;
pub use crate::context::write_sheet_context::*;
pub use crate::context::write_workbook_context;
pub use crate::context::write_workbook_context::*;
pub use crate::metadata::data::write_cell_data;
pub use crate::metadata::data::write_cell_data::*;
