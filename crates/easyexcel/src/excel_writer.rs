//! 有状态 Excel 写入器。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter`
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/ExcelWriter.java

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use crate::core::{
    ConverterRegistry, CsvCharset, ExcelColumn, ExcelError, ExcelRow, Result, WriteHandler,
    WriteSheetContext, WriteWorkbookContext,
};
use crate::util::work_book_util::create_sheet;
use easyexcel_xlsx::xlsx::generation::{self, Workbook};

use crate::write::append_rows::append_rows_to_worksheet_with_gzip_and_context;
use crate::write::excel_output_stream::ExcelOutputStream;
use crate::write::excel_writer_core::{
    CapturedOutput, HandlerHolderScope, after_sheet, after_sheet_create, after_workbook,
    after_workbook_create, append_csv_rows, append_rows_to_biff8_sheet,
    apply_annotation_column_widths, apply_biff8_column_widths,
    apply_biff8_once_absolute_merge_property, apply_handler_column_widths,
    apply_once_absolute_merge_property, apply_template_holder_layout, apply_xlsx_mutations,
    automatic_dynamic_head_merge_ranges, before_sheet, before_workbook,
    collect_handler_once_absolute_merges, collect_once_absolute_merges,
    collect_template_append_rows, create_csv_record_writer, create_stateful_csv_writer,
    finish_csv_record_writer, format_error, handlers_request_auto_width, head_rows_for_schema,
    merge_range_to_biff8, relative_head_start_row, run_own_workbook_callbacks,
    run_template_handler_callbacks, save_template_package, save_workbook, save_workbook_to_writer,
    save_xls_book, set_xlsx_column_width_chars, sort_handlers, take_captured_output,
    template_append_cell_styles, template_append_row_heights, validate_excel_row_schema,
    validate_stateful_backend, validate_stateful_schema, write_sheet_to_workbook_with_gzip,
};
use crate::write::handler::default_write_handler_loader::DefaultWriteHandlerLoader;
use crate::write::handler_execution_scope::{
    HandlerExecutionScope, ensure_gzip_spill, load_annotation_handlers,
};
use crate::write::metadata::write_table::WriteTable as MirroredWriteTable;
use crate::write::shared_write_handler::{
    SharedWriteHandler, StatefulSheetState, boxed_handlers, share_handlers,
};
use crate::write::write_options::WriteOptions;
use crate::write::write_progress::WriteProgress;
use crate::write::write_sheet::WriteSheet;
use crate::write::xls_adapter::Biff8Book;
use crate::write_type_helpers::effective_write_type;
use easyexcel_csv::CsvRecordWriter;

/// 对应 Java：com.alibaba.excel.ExcelWriter。 Stateful XLSX or single-sheet CSV writer matching Java `ExcelWriter`'s lifecycle.
#[allow(clippy::struct_excessive_bools)]
pub struct ExcelWriter {
    path: PathBuf,
    excel_type: Option<crate::support::ExcelTypeEnum>,
    output_stream: Option<Box<dyn Write + Send>>,
    close_stream: Option<Box<dyn FnOnce() -> std::io::Result<()> + Send>>,
    pub(crate) workbook: Workbook,
    xls_book: Biff8Book,
    pub(crate) workbook_handlers: Vec<SharedWriteHandler>,
    pub(crate) sheet_annotation_handlers: HashMap<String, Vec<SharedWriteHandler>>,
    sheet_handlers: HashMap<String, Vec<SharedWriteHandler>>,
    table_annotation_handlers: HashMap<(String, i32), Vec<SharedWriteHandler>>,
    table_handlers: HashMap<(String, i32), Vec<SharedWriteHandler>>,
    table_schemas: HashMap<(String, i32), &'static [ExcelColumn]>,
    current_effective_handlers: Vec<SharedWriteHandler>,
    sheets: HashMap<String, StatefulSheetState>,
    sheet_indexes: HashMap<usize, String>,
    pub(crate) csv_writer: Option<CsvRecordWriter>,
    csv_capture: Option<CapturedOutput>,
    csv_charset: CsvCharset,
    csv_with_bom: bool,
    started: bool,
    finished: bool,
    auto_close_stream: bool,
    write_excel_on_exception: bool,
    password: Option<String>,
    converters: ConverterRegistry,
    /// Workbook-level spill preference from the builder. (Java SXSSF `setCompressTempFiles`)
    compress_temp_files: bool,
    /// Workbook-level constant-memory default from the builder.
    default_constant_memory: bool,
    template_file: Option<PathBuf>,
    template_bytes: Option<Vec<u8>>,
    /// First-write markers for sheets present in a `withTemplate` package.
    template_pending_rows: HashMap<String, u32>,
    /// ZIP/OOXML package used when preserving template styles and merges.
    template_package: Option<crate::write::template_write::TemplatePackage>,
    /// OLE/BIFF8 package used when `with_template` targets a `.xls` workbook.
    ///
    /// Java mapping: `HSSFWorkbook(template)` + append cells; unmodified BIFF
    /// records are copied verbatim by the `easyexcel-xls` template engine.
    xls_template: Option<crate::write::xls_adapter::Biff8TemplatePackage>,
    /// Explicit legacy value-replay for `with_template` (styles/merges discarded).
    use_legacy_template_seed: bool,
    /// Active gzip spill writers keyed by sheet name (when `compress_temp_files`).
    pub(crate) gzip_spills: HashMap<String, crate::write::gzip_spill::GzipSheetDataWriter>,
    /// Last finished gzip spill snapshot (for tests / observability).
    last_gzip_spill: Option<crate::write::gzip_spill::GzipSpillSnapshot>,
    mutation_plan: crate::context::write_mutation_plan::WriteMutationPlan,
}

include!("excel_writer/new_to_output_path.rs");
include!("excel_writer/write_raw_bytes_to_write_xls_batch_onto_template.rs");
include!("excel_writer/write_xlsx_batch_to_remember_sheet_index.rs");
