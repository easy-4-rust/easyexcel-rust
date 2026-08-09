//! 薄重导出层：保持 `crate::core::Type` 路径兼容性。
//!
//! 目录重排后，原 core/ 下的类型已拆分到 Java 对应的顶级模块。
//! 本文件通过 glob 重导出保持向后兼容（`use crate::core::*` 仍有效）。

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

// 从新顶级模块重导出（保持 crate::core::Type 路径兼容）
pub use crate::converters::{
    ConvertContext, Converter, FromExcelCell, ImageInputStream, InputStreamImageConverter,
    IntoExcelCell, StringImageConverter, UrlImageConverter, auto_converter, bigdecimal, biginteger,
    booleanconverter, bytearray, byteconverter, convert_context, converter, converter_key_build,
    converter_registry, custom_read_object, date, default_converter_loader, doubleconverter, file,
    floatconverter, from_excel_cell, from_into_impls, image_input_stream,
    input_stream_image_converter, inputstream, integer, into_excel_cell, localdate, localdatetime,
    longconverter, nullable_object_converter, read_converter_context, shortconverter, string, url,
    url_image_converter, write_converter_context,
};
pub use crate::enums::{
    BooleanEnum, ByteOrderMark, CellDataType, HeadKind, NumericCellType, RowType, WriteDirection,
    WriteLastRow, WriteLastRowTypeEnum, WriteTemplateAnalysisCellType, WriteType, boolean_enum,
    byte_order_mark_enum, cache_location_enum, cell_data_type_enum, cell_extra_type_enum,
    enum_boolean, enum_byte_order_mark, enum_cache_location, enum_cell_data_type,
    enum_cell_extra_type, enum_head_kind, enum_holder, enum_numeric_cell_type,
    enum_read_default_return, enum_row_type, enum_write_direction, enum_write_last_row,
    enum_write_template_analysis_cell_type, enum_write_type, head_kind_enum, holder_enum,
    numeric_cell_type_enum, poi, read_default_return_enum, row_type_enum, write_direction_enum,
    write_last_row_type_enum, write_template_analysis_cell_type_enum, write_type_enum,
};
pub use crate::event::{
    AbstractIgnoreExceptionListenerAdapter, AbstractIgnoreExceptionReadListener,
    AnalysisEventListener, AnalysisEventListenerAdapter, CompositeReadListener, ErrorAction,
    Handler, Listener, NotRepeatExecutor, Order, PageReadListener, ReadListenerList,
    SyncReadListener, abstract_ignore_exception_read_listener, analysis_event_listener, handler,
    listener, not_repeat_executor, order, page_read_listener, sync_read_listener,
};
pub use crate::metadata::{
    AbstractCell, AbstractHolder, AbstractParameterBuilder, AnalysisCell, AnchorType,
    BasicParameter, BasicParameterBuilder, Cell, CellData, CellRange, ClientAnchorData,
    ColumnWidthProperty, CommentData, ConfigurationHolder, CoordinateData, DataFormatData,
    DateTimeFormatProperty, DynamicRow, DynamicValue, ExcelBorderStyle, ExcelColor,
    ExcelContentProperty, ExcelDataFormat, ExcelFillPattern, ExcelFontScript, ExcelHeadProperty,
    ExcelHorizontalAlignment, ExcelReadHeadProperty, ExcelUnderline, ExcelVerticalAlignment,
    ExcelWriteHeadProperty, FieldCache, FieldWrapper, Font, FontProperty, FormulaData,
    GlobalConfiguration, Head, HyperlinkData, HyperlinkType, ImageData, ImageType, IntervalFont,
    MetadataHolder, NullObject, NumberFormatProperty, ReadCellData, RichTextStringData, RowData,
    RowHeightProperty, StyleProperty, abstract_cell, abstract_holder, abstract_parameter_builder,
    basic_parameter, cell, cell_extra, cell_range, configuration_holder, csv, data,
    excel_border_style, excel_cell_style, excel_color, excel_column, excel_data_format,
    excel_fill_pattern, excel_font_script, excel_font_style, excel_horizontal_alignment, excel_row,
    excel_underline, excel_vertical_alignment, excel_write_head_property, excel_write_metadata,
    field_cache, field_wrapper, fill, font, format, global_configuration, head, holder,
    null_object, property,
};
pub use crate::support::{
    ExcelDownloadErrorBody, ExcelTypeEnum, csv_charset, empty, excel_download_error_body,
    excel_error, excel_type_enum,
};

// 模块路径重导出（保持 crate::core::<module>::Type 路径兼容）
pub use crate::context::analysis_context;

// metadata::data 子模块路径镜像（保持 crate::core::<data_module> 路径兼容）
pub use crate::metadata::data::{
    anchor_type, cell_value, client_anchor_data, comment_data, coordinate_data, dynamic_row,
    dynamic_value, formula_data, hyperlink_data, image_data, image_type, interval_font,
    read_cell_data, rich_text_string_data, row_data, write_font,
};

// read::listener::read_listener 模块路径镜像
pub use crate::read::listener::read_listener;

mod result;
pub use result::Result;

// write/ 相关类型
pub use crate::write::write_backend_handle::{WriteCellHandle, WriteRowHandle};
pub use crate::write::write_context::{
    WriteContext, WriteContextHolder, WriteContextHolderState, WriteContextImpl,
    WriteContextLifecycle, finish_write_context,
};
pub use crate::write::write_fill_executor::{
    WriteFillConfig, WriteFillExecutor, WriteFillSheet, csv_fill_unsupported_error,
    fill_requires_template_error,
};
pub use crate::write::write_holder_context::{
    WriteHolderContext, WriteSheetHolderView, WriteTableHolderView, WriteWorkbookHolderView,
};
pub use crate::write::{ChartMutation, ChartRange, ChartSeries, ChartType};

// metadata 子包重导出
pub use crate::metadata::property::{
    ExcelDataValidationMeta, LoopMergeProperty, NumberRoundingMode, OnceAbsoluteMergeProperty,
};

// 具名类型重导出
pub use crate::context::analysis_context::{AnalysisContext, AnalysisContextLifecycle};
pub use crate::converters::converter_registry::ConverterRegistry;
pub use crate::converters::nullable_object_converter::NullableObjectConverter;
pub use crate::enums::enum_cache_location::CacheLocation;
pub use crate::enums::enum_holder::Holder;
pub use crate::metadata::cell_extra::{CellExtra, CellExtraType};
pub use crate::metadata::data::cell_value::CellValue;
pub use crate::metadata::excel_cell_style::ExcelCellStyle;
pub use crate::metadata::excel_font_style::ExcelFontStyle;
pub use crate::metadata::excel_row::ExcelRow;
pub use crate::read::listener::read_listener::ReadListener;
pub use crate::support::Empty;
pub use crate::support::csv_charset::CsvCharset;
pub use crate::support::excel_error::ExcelError;
pub use crate::write::metadata::style::write_font::WriteFont;
pub use crate::write::write_cell_data::WriteCellData;

#[cfg(test)]
mod tests;

// 补充缺失的重导出
pub use crate::converters::custom_read_object::CustomReadObject;
pub use crate::enums::enum_read_default_return::ReadDefaultReturn;
pub use crate::write::write_handler::{WriteHandler, WriteHandlerCapability};
pub use crate::write::write_row_context::WriteRowContext;
pub use crate::write::write_sheet_context::WriteSheetContext;
pub use crate::write::write_workbook_context::WriteWorkbookContext;

// 补充 WriteCellContext 及相关类型
pub use crate::converters::read_converter_context::ReadConverterContext;
pub use crate::converters::write_converter_context::WriteConverterContext;
pub use crate::metadata::excel_column::ExcelColumn;
pub use crate::metadata::excel_write_metadata::ExcelWriteMetadata;
pub use crate::write::write_cell_context::WriteCellContext;

/// 任意精度十进制类型，对应 Java `BigDecimal`。
pub use ::bigdecimal::BigDecimal;
/// 任意精度整数类型，对应 Java `BigInteger`。
pub use ::num_bigint::BigInt;
