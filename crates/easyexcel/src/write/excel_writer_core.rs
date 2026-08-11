//! Excel 写入器核心实现。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter` 及其所有依赖的私有函数。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/ExcelWriter.java

use std::any::type_name;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

pub use crate::core::{
    AnchorType, CacheLocation, CellValue, Converter, ConverterRegistry, CsvCharset,
    ExcelBorderStyle, ExcelCellStyle, ExcelColor, ExcelColumn, ExcelDataFormat, ExcelError,
    ExcelFillPattern, ExcelFontScript, ExcelFontStyle, ExcelHorizontalAlignment, ExcelRow,
    ExcelUnderline, ExcelVerticalAlignment, ExcelWriteMetadata, Holder, HyperlinkType, ImageData,
    NullableObjectConverter, Result, RichTextStringData, WriteCellContext, WriteCellData,
    WriteContextHolderState, WriteFont, WriteHandler, WriteHolderContext, WriteRowContext,
    WriteSheetContext, WriteSheetHolderView, WriteTableHolderView, WriteWorkbookContext,
    WriteWorkbookHolderView,
};
pub use crate::event::NotRepeatExecutor;
pub use crate::metadata::csv::{CsvSheet, CsvWorkbook};
pub use crate::util::work_book_util::{
    CellCreator, RowCreator, SheetCreator, WorkBookCreator, create_cell, create_row, create_sheet,
    create_work_book,
};
use bigdecimal::BigDecimal;
use easyexcel_csv::CsvRecordWriter;
use easyexcel_xlsx::xlsx::generation::{
    self, Color, FontFormatSpec, Format, FormatAlign, FormatBorder, FormatPattern, FormatScript,
    FormatSpec, FormatUnderline, NumberFormatSpec, Workbook, Worksheet,
};

use crate::write::append_rows::append_rows_to_worksheet_with_gzip_and_context;
use crate::write::creators::{
    Biff8RowCreator, XlsxCell, XlsxRowCreator, XlsxSheetCreator, XlsxWorkBookCreator,
};
use crate::write::handler_execution_scope::HandlerExecutionScope;
use crate::write::image_layout::ImageLayout;
use crate::write::shared_write_handler::StatefulSheetState;
use crate::write::sheet_style_context::{CellFormatContext, SheetStyleContext};
use crate::write::xls_adapter::{
    Biff8Book, Biff8Cell, Biff8Color, Biff8Comment, Biff8FillPattern, Biff8HyperlinkKind,
    Biff8Merge, Biff8Sheet, Biff8StyleRequest, Biff8StyleTable, GeneratedBiff8CellValue,
    apply_excel_cell_style, apply_excel_font_style, apply_write_font,
    date_to_excel_serial_with_windowing, datetime_to_excel_serial_with_windowing,
    writer_horizontal_alignment, writer_vertical_alignment,
};

#[cfg(test)]
use crate::write::xls_adapter::Biff8Value;

pub use crate::write::append_rows::{append_rows_to_worksheet, append_rows_to_worksheet_with_gzip};
pub use crate::write::excel_writer::ExcelWriter;

use crate::metadata::excel_cell_style::merge_excel_cell_style;
pub use crate::write::builder::abstract_excel_writer_parameter_builder::AbstractExcelWriterParameterBuilder;
pub use crate::write::builder::excel_writer_sheet_builder::ExcelWriterSheetBuilder as CompatibleExcelWriterSheetBuilder;
pub use crate::write::builder::excel_writer_table_builder::ExcelWriterTableBuilder;
pub use crate::write::cell_style::CellStyle;
pub use crate::write::csv_encoding_writer::{
    CsvEncoding, CsvEncodingWriter, csv_bom, csv_encoding,
};
pub use crate::write::excel_builder::{
    ExcelBuilder, ExcelBuilderImpl, FillConfig as BuilderFillConfig,
};
pub use crate::write::excel_output_stream::ExcelOutputStream;
pub use crate::write::excel_writer_builder::ExcelWriterBuilder as CompatibleExcelWriterBuilder;
pub use crate::write::excel_writer_builder::ExcelWriterOutputStreamBuilder as CompatibleExcelWriterOutputStreamBuilder;
pub use crate::write::executor::abstract_excel_write_executor::AbstractExcelWriteExecutor;
pub use crate::write::executor::excel_write_add_executor::ExcelWriteAddExecutor;
pub use crate::write::executor::excel_write_executor::ExcelWriteExecutor;
pub use crate::write::executor::excel_write_fill_executor::ExcelWriteFillExecutor;
pub use crate::write::global_configuration::{
    apply_global_configuration_to_write_options, global_configuration_from_write_options,
};
pub use crate::write::gzip_spill::{GZIP_MAGIC, GzipSpillSnapshot, file_has_gzip_magic};
#[allow(deprecated)]
pub use crate::write::handler::abstract_cell_write_handler::AbstractCellWriteHandler;
#[allow(deprecated)]
pub use crate::write::handler::abstract_row_write_handler::AbstractRowWriteHandler;
#[allow(deprecated)]
pub use crate::write::handler::abstract_sheet_write_handler::AbstractSheetWriteHandler;
#[allow(deprecated)]
pub use crate::write::handler::abstract_workbook_write_handler::AbstractWorkbookWriteHandler;
pub use crate::write::handler::cell_write_handler::CellWriteHandler;
pub use crate::write::handler::default_write_handler_loader::DefaultWriteHandlerLoader;
pub use crate::write::handler::r#impl::impl_default_row_write_handler::{
    DefaultRowWriteHandler, new_default_row_write_handler,
};
pub use crate::write::handler::r#impl::impl_dimension_workbook_write_handler::DimensionWorkbookWriteHandler;
pub use crate::write::handler::r#impl::impl_fill_style_cell_write_handler::FillStyleCellWriteHandler;
pub use crate::write::handler::row_write_handler::RowWriteHandler;
pub use crate::write::handler::sheet_write_handler::SheetWriteHandler;
pub use crate::write::handler::workbook_write_handler::WorkbookWriteHandler;
pub use crate::write::holder::abstract_write_holder::AbstractWriteHolder;
pub use crate::write::holder::write_holder::WriteHolder;
pub use crate::write::holder::write_sheet_holder::WriteSheetHolder as MirroredWriteSheetHolder;
pub use crate::write::holder::write_table_holder::WriteTableHolder as MirroredWriteTableHolder;
pub use crate::write::holder::write_workbook_holder::WriteWorkbookHolder as MirroredWriteWorkbookHolder;
pub use crate::write::horizontal_alignment::HorizontalAlignment;
pub use crate::write::merge::abstract_merge_strategy::AbstractMergeStrategy;
pub use crate::write::merge::loop_merge_strategy::LoopMergeStrategy as MirroredLoopMergeStrategy;
pub use crate::write::merge::once_absolute_merge_strategy::OnceAbsoluteMergeStrategy;
pub use crate::write::merge::once_absolute_merge_strategy::OnceAbsoluteMergeStrategy as MirroredOnceAbsoluteMerge;
pub use crate::write::merge_range::MergeRange;
pub use crate::write::metadata::collection_row_data::CollectionRowData;
pub use crate::write::metadata::map_row_data::MapRowData;
pub use crate::write::metadata::row_data::RowData as MirroredRowData;
use crate::write::metadata::style::write_cell_style::merge_write_cell_style;
use crate::write::metadata::style::write_font::merge_excel_font_style as merge_handler_font_style;
pub use crate::write::metadata::style::write_font::{
    excel_font_style_from_write_font, merge_excel_font_style, merge_write_font,
    write_font_from_excel_font_style,
};
pub use crate::write::metadata::write_basic_parameter::WriteBasicParameter as MirroredWriteBasicParameter;
pub use crate::write::metadata::write_sheet::WriteSheet as MirroredWriteSheet;
pub use crate::write::metadata::write_table::WriteTable as MirroredWriteTable;
pub use crate::write::metadata::write_workbook::WriteWorkbook as MirroredWriteWorkbook;
pub use crate::write::property::excel_write_head_property::ExcelWriteHeadProperty;
pub use crate::write::style::abstract_cell_style_strategy::AbstractCellStyleStrategy;
pub use crate::write::style::abstract_vertical_cell_style_strategy::AbstractVerticalCellStyleStrategy;
pub use crate::write::style::column::longest_match_column_width_style_strategy::LongestMatchColumnWidthStyleStrategy;
pub use crate::write::style::column::simple_column_width_style_strategy::SimpleColumnWidthStyleStrategy;
pub use crate::write::style::default_style::DefaultStyle;
pub use crate::write::style::horizontal_cell_style_strategy::HorizontalCellStyleStrategy;
pub use crate::write::style::row::simple_row_height_style_strategy::SimpleRowHeightStyleStrategy;
pub use crate::write::style::vertical_cell_style_strategy::VerticalCellStyleStrategy;
pub use crate::write::vertical_alignment::VerticalAlignment;
pub use crate::write::write_options::WriteOptions;
pub use crate::write::write_progress::WriteProgress;
pub use crate::write::write_sheet::WriteSheet;

include!("excel_writer_core/state_and_conversion.rs");
include!("excel_writer_core/csv_write.rs");
include!("excel_writer_core/xls_write.rs");
include!("excel_writer_core/handler_scope.rs");
include!("excel_writer_core/xlsx_sheet_write.rs");
include!("excel_writer_core/xlsx_template_write.rs");
include!("excel_writer_core/handler_lifecycle.rs");
include!("excel_writer_core/schema_and_head.rs");
include!("excel_writer_core/xlsx_row_emission.rs");
include!("excel_writer_core/xlsx_cell_emission.rs");
include!("excel_writer_core/xlsx_workbook_mutations.rs");

#[cfg(test)]
#[path = "missing_tests.rs"]
mod missing_tests;
pub use crate::write::write_csv::{
    write_csv_to_buffer, write_csv_to_writer, write_csv_with_handlers,
};
#[cfg(test)]
// Re-exports for tests
pub use crate::write::write_xls::*;
pub use crate::write::xlsx_write::{write_xlsx, write_xlsx_to_writer, write_xlsx_with_handlers};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "excel_writer_core_tests/tests_extra.rs"]
mod tests_extra;

#[cfg(test)]
#[path = "excel_writer_core_tests/tests_extra2.rs"]
mod tests_extra2;

#[cfg(test)]
#[path = "excel_writer_core_tests/tests_extra3.rs"]
mod tests_extra3;
