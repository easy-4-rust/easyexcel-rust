//! Public facade for typed, event-driven Excel reading and writing.
//!
//! Java 对应：`com.alibaba.excel.EasyExcel` / `EasyExcelFactory`。
//! 本文件仅做 `mod` 声明与 `pub use` 重导出 + 少量 `use` 把内部 helper 引入
//! 当前作用域供测试（`super::*`）使用，不再定义任何类型。所有 facade 类型
//! 拆分到独立文件（每个类型一个 `.rs`，命名 1:1 对应 Java 类）。

// Java API 镜像方法（getter/setter/builder）大量缺中文 doc 注释，
// 后续逐步补充；暂不因 missing_docs 产生 warning 噪音。
#![allow(missing_docs)]
//! 拆分到独立文件（每个类型一个 `.rs`，命名 1:1 对应 Java 类）。

pub mod analysis;
pub mod annotation;
pub mod cache;
pub mod constant;
pub mod context;
pub mod converters;
pub mod core;
pub mod csv;
pub mod enums;
pub mod event;
pub mod exception;
pub mod format;
pub mod formula;
pub mod io;
pub mod markdown;
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

pub use crate::annotation::format::{DateTimeFormat, NumberFormat};
pub use crate::annotation::write::style::{
    ColumnWidth, ContentFontStyle, ContentLoopMerge, ContentRowHeight, ContentStyle, HeadFontStyle,
    HeadRowHeight, HeadStyle, OnceAbsoluteMerge,
};
pub use crate::annotation::{ExcelIgnore, ExcelIgnoreUnannotated, ExcelProperty};
pub use crate::cache::{
    EternalReadCacheSelector, FileCache, MapCache, MokaCache, ReadCache, ReadCacheSelector,
    SimpleReadCacheSelector, XlsCache,
};
pub use crate::core::{
    AbstractCell, AbstractHolder, AbstractIgnoreExceptionListenerAdapter,
    AbstractIgnoreExceptionReadListener, AbstractParameterBuilder, AnalysisCell, AnalysisContext,
    AnalysisEventListener, AnalysisEventListenerAdapter, AnchorType, BasicParameter,
    BasicParameterBuilder, BooleanEnum, ByteOrderMark, CacheLocation, Cell, CellData, CellDataType,
    CellExtra, CellExtraType, CellRange, CellValue, ChartMutation, ChartRange, ChartSeries,
    ChartType, ClientAnchorData, ColumnWidthProperty, CommentData, CompositeReadListener,
    ConfigurationHolder, ConvertContext, Converter, ConverterRegistry, CoordinateData, CsvCharset,
    CustomReadObject, DataFormatData, DateTimeFormatProperty, DynamicRow, DynamicValue, Empty,
    ErrorAction, ExcelBorderStyle, ExcelCellStyle, ExcelColor, ExcelColumn, ExcelContentProperty,
    ExcelDataFormat, ExcelDataValidationMeta, ExcelDownloadErrorBody, ExcelError, ExcelFillPattern,
    ExcelFontScript, ExcelFontStyle, ExcelHeadProperty, ExcelHorizontalAlignment,
    ExcelReadHeadProperty, ExcelRow, ExcelTypeEnum, ExcelUnderline, ExcelVerticalAlignment,
    ExcelWriteHeadProperty, ExcelWriteMetadata, FieldCache, FieldWrapper, Font, FontProperty,
    FormulaData, FromExcelCell, Handler, Head, HeadKind, Holder, HyperlinkData, HyperlinkType,
    ImageData, ImageInputStream, ImageType, InputStreamImageConverter, IntervalFont, IntoExcelCell,
    Listener, LoopMergeProperty, MetadataHolder, NotRepeatExecutor, NullObject,
    NullableObjectConverter, NumberFormatProperty, NumberRoundingMode, NumericCellType,
    OnceAbsoluteMergeProperty, Order, PageReadListener, ReadCellData, ReadDefaultReturn,
    ReadListener, ReadListenerList, Result, RichTextStringData, RowData, RowHeightProperty,
    RowType, StringImageConverter, StyleProperty, SyncReadListener, UrlImageConverter,
    WriteCellContext, WriteCellData, WriteCellHandle, WriteContext, WriteContextHolder,
    WriteContextHolderState, WriteContextImpl, WriteContextLifecycle, WriteDirection,
    WriteFillConfig, WriteFillExecutor, WriteFillSheet, WriteFont, WriteHandler,
    WriteHandlerCapability, WriteHolderContext, WriteLastRow, WriteLastRowTypeEnum,
    WriteRowContext, WriteRowHandle, WriteSheetContext, WriteSheetHolderView, WriteTableHolderView,
    WriteTemplateAnalysisCellType, WriteType, WriteWorkbookContext, WriteWorkbookHolderView,
    abstract_cell, abstract_holder, abstract_ignore_exception_read_listener,
    abstract_parameter_builder, analysis_context, analysis_event_listener, anchor_type,
    auto_converter, basic_parameter, bigdecimal, biginteger, boolean_enum, booleanconverter,
    byte_order_mark_enum, bytearray, byteconverter, cache_location_enum, cell, cell_data_type_enum,
    cell_extra, cell_extra_type_enum, cell_range, cell_value, client_anchor_data, comment_data,
    configuration_holder, convert_context, converter, converter_key_build, converter_registry,
    coordinate_data, csv_charset, csv_fill_unsupported_error, custom_read_object, data, date,
    default_converter_loader, doubleconverter, dynamic_row, dynamic_value, empty, enum_boolean,
    enum_byte_order_mark, enum_cache_location, enum_cell_data_type, enum_cell_extra_type,
    enum_head_kind, enum_holder, enum_numeric_cell_type, enum_read_default_return, enum_row_type,
    enum_write_direction, enum_write_last_row, enum_write_template_analysis_cell_type,
    enum_write_type, excel_border_style, excel_cell_style, excel_color, excel_column,
    excel_data_format, excel_download_error_body, excel_error, excel_fill_pattern,
    excel_font_script, excel_font_style, excel_horizontal_alignment, excel_row, excel_type_enum,
    excel_underline, excel_vertical_alignment, excel_write_head_property, excel_write_metadata,
    field_cache, field_wrapper, file, fill, fill_requires_template_error, finish_write_context,
    floatconverter, font, formula_data, from_excel_cell, from_into_impls, global_configuration,
    handler, head, head_kind_enum, holder, holder_enum, hyperlink_data, image_data,
    image_input_stream, image_type, input_stream_image_converter, inputstream, integer,
    interval_font, into_excel_cell, listener, localdate, localdatetime, longconverter,
    not_repeat_executor, null_object, nullable_object_converter, numeric_cell_type_enum, order,
    page_read_listener, poi, property, read_cell_data, read_converter_context,
    read_default_return_enum, read_listener, rich_text_string_data, row_data, row_type_enum,
    shortconverter, string, sync_read_listener, url, url_image_converter, write_converter_context,
    write_direction_enum, write_font, write_last_row_type_enum,
    write_template_analysis_cell_type_enum, write_type_enum,
};
pub use crate::metadata::GlobalConfiguration;
pub use crate::read::listener::{
    IgnoreExceptionListenerAdapter, IgnoreExceptionReadListener, ModelBuildEventListener,
};
pub use crate::read::{
    CompatibleExcelReaderBuilder, CompatibleExcelReaderSheetBuilder, ExcelLocale, ExcelReader,
    ParallelMapReadListener, apply_global_configuration_to_read_options,
    global_configuration_from_read_options,
};
pub use crate::template::{
    ExcelTemplateWriter, FillConfig, FillConfigBuilder, FillDirection, FillWrapper,
    IntoTemplateValue, TemplateData, TemplateSheet, fill_xlsx_template, fill_xlsx_template_list,
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
pub use easyexcel_io::EasyExcelTempFileCreationStrategy;
pub use easyexcel_xls::biff8::Biff8MacroPolicy;
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
pub use crate::write::WriteBackendSelection;
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

// 枚举 / 类型重导出（保持 crate::TypeName 路径兼容）
pub use crate::context::xlsx::xlsx_read_context::XlsxReadContext;
pub use crate::enums::holder_enum::HolderEnum;
pub use crate::enums::row_type_enum::RowTypeEnum;
pub use crate::enums::write_direction_enum::WriteDirectionEnum;
pub use crate::read::metadata::{ReadSheet, ReadWorkbook};
pub use crate::write::metadata::style::write_cell_style::WriteCellStyle;
pub use crate::write::metadata::{WriteTable, WriteWorkbook};

pub use util::java_date::JavaDate;

pub use read::read_cache::ReadCacheMode;
pub use read::stored_read_cache_selector::StoredReadCacheSelector;

// 外部类型重导出（保持 crate::Url 等路径兼容）
pub use ::bigdecimal::BigDecimal;
pub use ::num_bigint::BigInt;
pub use ::url::Url;
