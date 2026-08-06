//! XLSX writer backed by `rust_xlsxwriter`.

pub mod builder;
pub mod cell_style;
pub mod csv_encoding_writer;
pub(crate) mod excel_builder;
pub(crate) mod excel_builder_impl;
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
pub mod write_csv;
pub mod write_options;
pub mod write_progress;
pub mod write_sheet;
pub mod write_xls;
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
/// EasyExcel metadata/CellValue 到 `easyexcel-xls` 的门面适配。
pub(crate) mod xls_adapter;

#[cfg(test)]
pub(crate) use excel_writer_core::save_workbook;
#[allow(deprecated)]
pub use excel_writer_core::{
    AbstractCellStyleStrategy, AbstractCellWriteHandler, AbstractExcelWriteExecutor,
    AbstractExcelWriterParameterBuilder, AbstractMergeStrategy, AbstractRowWriteHandler,
    AbstractSheetWriteHandler, AbstractVerticalCellStyleStrategy, AbstractWorkbookWriteHandler,
    AbstractWriteHolder, AnchorType, BuilderFillConfig, CacheLocation, CellCreator, CellStyle,
    CellValue, CellWriteHandler, CollectionRowData, CompatibleExcelWriterBuilder,
    CompatibleExcelWriterOutputStreamBuilder, CompatibleExcelWriterSheetBuilder, Converter,
    ConverterRegistry, CsvCharset, CsvEncoding, CsvEncodingWriter, CsvSheet, CsvWorkbook,
    DefaultRowWriteHandler, DefaultStyle, DefaultWriteHandlerLoader, DimensionWorkbookWriteHandler,
    ExcelBorderStyle, ExcelBuilder, ExcelBuilderImpl, ExcelCellStyle, ExcelColor, ExcelColumn,
    ExcelDataFormat, ExcelError, ExcelFillPattern, ExcelFontScript, ExcelFontStyle,
    ExcelHorizontalAlignment, ExcelOutputStream, ExcelRow, ExcelUnderline, ExcelVerticalAlignment,
    ExcelWriteAddExecutor, ExcelWriteExecutor, ExcelWriteFillExecutor, ExcelWriteHeadProperty,
    ExcelWriteMetadata, ExcelWriter, ExcelWriterTableBuilder, FillStyleCellWriteHandler,
    GZIP_MAGIC, GzipSpillSnapshot, Holder, HorizontalAlignment, HorizontalCellStyleStrategy,
    ImageData, LongestMatchColumnWidthStyleStrategy, MapRowData, MergeRange,
    MirroredLoopMergeStrategy, MirroredOnceAbsoluteMerge, MirroredRowData,
    MirroredWriteBasicParameter, MirroredWriteSheet, MirroredWriteSheetHolder, MirroredWriteTable,
    MirroredWriteTableHolder, MirroredWriteWorkbook, MirroredWriteWorkbookHolder,
    NotRepeatExecutor, NullableObjectConverter, OnceAbsoluteMergeStrategy, Result,
    RichTextStringData, RowCreator, RowWriteHandler, SheetCreator, SheetWriteHandler,
    SimpleColumnWidthStyleStrategy, SimpleRowHeightStyleStrategy, VerticalAlignment,
    VerticalCellStyleStrategy, WorkBookCreator, WorkbookWriteHandler, WriteFont, WriteHolder,
    WriteOptions, WriteProgress, WriteSheet, append_rows_to_worksheet,
    append_rows_to_worksheet_with_gzip, apply_global_configuration_to_write_options, create_cell,
    create_row, create_sheet, create_work_book, csv_bom, csv_encoding,
    excel_font_style_from_write_font, file_has_gzip_magic, global_configuration_from_write_options,
    merge_excel_font_style, merge_write_font, new_default_row_write_handler,
};
pub(crate) use excel_writer_core::{
    decimal_integer_requires_text, finite_decimal_f64, resolved_write_context_holder_state,
};
#[allow(unused_imports)]
pub use write_csv::{write_csv_to_buffer, write_csv_to_writer, write_csv_with_handlers};
#[allow(unused_imports)]
pub use write_xls::{write_xls, write_xls_to_writer, write_xls_with_handlers};
#[allow(unused_imports)]
pub use xlsx_write::{write_xlsx, write_xlsx_to_writer, write_xlsx_with_handlers};

pub use crate::context::write_backend_handle;
pub use crate::context::write_backend_handle::{WriteCellHandle, WriteRowHandle};
pub use crate::context::write_cell_context;
pub use crate::context::write_cell_context::WriteCellContext;
pub use crate::context::write_context;
pub use crate::context::write_context::{
    WriteContext, WriteContextHolder, WriteContextHolderState, WriteContextImpl,
    WriteContextLifecycle, finish_write_context,
};
pub use crate::context::write_fill_executor;
pub use crate::context::write_fill_executor::{
    WriteFillConfig, WriteFillExecutor, WriteFillSheet, csv_fill_unsupported_error,
    fill_requires_template_error,
};
pub use crate::context::write_handler;
pub use crate::context::write_handler::WriteHandler;
pub use crate::context::write_holder_context;
pub use crate::context::write_holder_context::{
    WriteHolderContext, WriteSheetHolderView, WriteTableHolderView, WriteWorkbookHolderView,
};
pub use crate::context::write_row_context;
pub use crate::context::write_row_context::WriteRowContext;
pub use crate::context::write_sheet_context;
pub use crate::context::write_sheet_context::WriteSheetContext;
pub use crate::context::write_workbook_context;
pub use crate::context::write_workbook_context::WriteWorkbookContext;
pub use crate::metadata::data::write_cell_data;
pub use crate::metadata::data::write_cell_data::WriteCellData;
