//! Public facade for typed, event-driven Excel reading and writing.
//!
//! Java 对应：`com.alibaba.excel.EasyExcel` / `EasyExcelFactory`。
//! 本文件仅做 `mod` 声明与 `pub use` 重导出 + 少量 `use` 把内部 helper 引入
//! 当前作用域供测试（`super::*`）使用，不再定义任何类型。所有 facade 类型
//! 拆分到独立文件（每个类型一个 `.rs`，命名 1:1 对应 Java 类）。

pub mod analysis;
pub mod cache;
pub mod constant;
pub mod context;
pub mod converters;
pub mod core;
pub mod csv;
pub mod enums;
pub mod event;
pub mod exception;
pub mod formula;
pub mod format;
pub mod io;
pub mod metadata;
pub mod model;
pub mod read;
pub mod support;
pub mod tabular;
pub mod template;
pub mod util;
pub mod write;
pub mod xls;
pub mod xlsx;

mod collect_listener;
mod easy_excel;
mod easy_excel_factory;
mod excel_builder;
mod excel_output_stream_builder;
mod excel_owned_output_stream_builder;
mod excel_reader_builder;
mod excel_sync_reader_builder;
mod into_sheet_selector;
mod write_type_helpers;

pub use crate::cache::{
    Ehcache, EternalReadCacheSelector, MapCache, MokaCache, ReadCache, ReadCacheSelector,
    SimpleReadCacheSelector, XlsCache,
};
pub use crate::core::*;
pub use crate::metadata::GlobalConfiguration;
pub use crate::read::{
    CompatibleExcelReaderBuilder, CompatibleExcelReaderSheetBuilder, ExcelLocale, ExcelReader,
    apply_global_configuration_to_read_options, global_configuration_from_read_options,
};
pub use crate::template::{
    ExcelTemplateWriter, FillConfig, FillDirection, FillWrapper, IntoTemplateValue, TemplateData,
    TemplateSheet, fill_xlsx_template, fill_xlsx_template_list,
};
pub use crate::write::{
    CellStyle, CompatibleExcelWriterBuilder, CompatibleExcelWriterOutputStreamBuilder,
    CompatibleExcelWriterSheetBuilder, CsvEncodingWriter, ExcelBuilder, ExcelBuilderImpl,
    ExcelOutputStream, ExcelWriter, HorizontalAlignment, HorizontalCellStyleStrategy,
    LongestMatchColumnWidthStyleStrategy, MergeRange,
    MirroredLoopMergeStrategy as LoopMergeStrategy, SimpleColumnWidthStyleStrategy,
    SimpleRowHeightStyleStrategy, VerticalAlignment, VerticalCellStyleStrategy, WriteOptions,
    WriteSheet, write_csv_to_buffer, write_csv_to_writer, write_xls, write_xls_to_writer,
    write_xlsx_to_writer,
};
pub use easyexcel_derive::ExcelRow;
pub use excel_builder::{
    builder_from_writer, do_fill_template, do_fill_template_with_config, fill_builder_from_writer,
    wire_template_fill,
};

// crate 根作用域的内部 `use`：这些名称被 `reader` 等子模块通过
// `crate::ReadOptions` / `crate::SheetSelector` / `crate::read_csv` 等路径引用，
// 删除会导致整个 crate 内部的路径解析断裂。保持与原 `lib.rs` 一致。
use crate::read::{
    ReadOptions, ScientificFormatMode, SheetSelector, read_csv, read_xls, read_xlsx,
};

// 以下 `std` 导入保持原 `lib.rs` 的可见性：`tests.rs` 通过 `use super::*`
// 访问 `PathBuf`，删除会导致既有测试编译失败（`tests.rs` 自身已 `use std::path::Path`）。
#[cfg(test)]
use std::path::PathBuf;

// Facade 类型重导出（每个类型对应独立文件，公开 API 表面保持不变）。
pub use crate::write::builder::excel_writer_builder::ExcelWriterBuilder;
pub use easy_excel::EasyExcel;
pub use easy_excel_factory::EasyExcelFactory;
pub use excel_output_stream_builder::ExcelOutputStreamBuilder;
pub use excel_owned_output_stream_builder::ExcelOwnedOutputStreamBuilder;
pub use excel_reader_builder::ExcelReaderBuilder;
pub use excel_sync_reader_builder::ExcelSyncReaderBuilder;
pub use into_sheet_selector::IntoSheetSelector;

// 内部 helper / crate 内类型仅引入当前作用域，供 `tests.rs` 通过 `super::*` 访问
// （原 `lib.rs` 内联定义时这些 helper 即对测试可见；保持该可见性以避免改动测试）。
#[cfg(test)]
use collect_listener::CollectListener;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_extra;

// 补充缺失的顶层导出
pub use converters::read_converter_context::ReadConverterContext;
pub use converters::write_converter_context::WriteConverterContext;

pub use util::java_date::JavaDate;

pub use read::read_cache::ReadCacheMode;
pub use read::stored_read_cache_selector::StoredReadCacheSelector;

// 外部类型重导出（保持 crate::Url 等路径兼容）
pub use ::bigdecimal::BigDecimal;
pub use ::num_bigint::BigInt;
pub use ::url::Url;
