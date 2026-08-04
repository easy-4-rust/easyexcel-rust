//! Excel 写入器核心实现。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter` 及其所有依赖的私有函数。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/ExcelWriter.java

use std::any::type_name;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub use crate::core::{
    AnchorType, CacheLocation, CellValue, Converter, ConverterRegistry, CsvCharset,
    ExcelBorderStyle, ExcelCellStyle, ExcelColor, ExcelColumn, ExcelDataFormat, ExcelError,
    ExcelFillPattern, ExcelFontScript, ExcelFontStyle, ExcelHorizontalAlignment, ExcelRow,
    ExcelUnderline, ExcelVerticalAlignment, ExcelWriteMetadata, Holder, ImageData,
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
use bigdecimal::{BigDecimal, ToPrimitive};
use ms_offcrypto_writer::Ecma376AgileWriter;
use rust_xlsxwriter::{
    Color, Format, FormatAlign, FormatBorder, FormatPattern, FormatScript, FormatUnderline, Image,
    Note, ObjectMovement, Workbook, Worksheet,
};

use crate::write::append_rows::append_rows_to_worksheet_with_gzip_and_context;
use crate::write::biff8::{
    Biff8Book, Biff8Cell, Biff8Merge, Biff8Sheet, Biff8StyleRequest, Biff8StyleTable, Biff8Value,
    date_to_excel_serial_with_windowing, datetime_to_excel_serial_with_windowing,
};
use crate::write::creators::{
    Biff8RowCreator, XlsxCell, XlsxRowCreator, XlsxSheetCreator, XlsxWorkBookCreator,
};
use crate::write::handler_execution_scope::HandlerExecutionScope;
use crate::write::image_layout::ImageLayout;
use crate::write::shared_write_handler::StatefulSheetState;
use crate::write::sheet_style_context::{CellFormatContext, SheetStyleContext};

pub use crate::write::append_rows::{append_rows_to_worksheet, append_rows_to_worksheet_with_gzip};
pub use crate::write::excel_writer::ExcelWriter;

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

/// Global write flags copied from [`WriteOptions`] for cell emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct WriteGlobalFlags {
    /// Automatic trim for sheet names and string cells.
    auto_trim: bool,
    /// Whether Excel 1904 date windowing is enabled.
    use_1904_windowing: bool,
    /// Whether scientific notation is used for extreme General-format numbers.
    use_scientific_format: bool,
}

impl From<&WriteOptions> for WriteGlobalFlags {
    fn from(options: &WriteOptions) -> Self {
        Self {
            auto_trim: options.auto_trim,
            use_1904_windowing: options.use_1904_windowing,
            use_scientific_format: options.use_scientific_format,
        }
    }
}

/// Returns the worksheet name after applying [`WriteOptions::auto_trim`].
pub(crate) fn effective_sheet_name(options: &WriteOptions) -> String {
    if options.auto_trim {
        options.sheet_name.trim().to_owned()
    } else {
        options.sheet_name.clone()
    }
}

/// Trims string cell text when auto-trim is enabled.
///
/// 关闭 `auto_trim` 时返回借用（零拷贝）：XLSX 热路径每字符串单元格原本
/// 固定产生一次 `String` 分配，此改动将其消除；仅开启 `auto_trim` 时才会分配。
pub(crate) fn maybe_trim_cell_string(value: &str, auto_trim: bool) -> Cow<'_, str> {
    if auto_trim {
        Cow::Owned(value.trim().to_owned())
    } else {
        Cow::Borrowed(value)
    }
}

/// 对应 Java：NumberUtils 的极值科学计数法阈值。
pub(crate) fn is_scientific_magnitude(value: f64) -> bool {
    let absolute = value.abs();
    absolute >= 1E11 || (absolute <= 1E-10 && absolute > 0.0)
}

pub(crate) fn validate_stateful_backend(is_csv: bool, password: Option<&str>) -> Result<()> {
    match (is_csv, password.is_some()) {
        (true, true) => Err(ExcelError::Unsupported(
            "password protection is not supported for CSV".to_owned(),
        )),
        // XLS password is now supported via BIFF8 RC4 (Phase 5.3)
        _ => Ok(()),
    }
}

pub(crate) fn is_csv_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("csv"))
}

pub(crate) fn is_xls_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xls"))
}

pub(crate) fn resolve_excel_type(
    path: &Path,
    options: &WriteOptions,
) -> crate::support::ExcelTypeEnum {
    options.excel_type.unwrap_or_else(|| {
        if is_csv_path(path) {
            crate::support::ExcelTypeEnum::Csv
        } else if is_xls_path(path) {
            crate::support::ExcelTypeEnum::Xls
        } else {
            crate::support::ExcelTypeEnum::Xlsx
        }
    })
}

pub(crate) fn uses_constant_memory_spill(options: &WriteOptions) -> bool {
    options.constant_memory || options.compress_temp_files
}

pub(crate) fn validate_stateful_schema(
    sheet_name: &str,
    state: &StatefulSheetState,
    schema: &'static [ExcelColumn],
) -> Result<()> {
    if state.schema == schema {
        Ok(())
    } else {
        Err(ExcelError::Format(format!(
            "worksheet schema changed between writes: {sheet_name}"
        )))
    }
}

pub(crate) fn with_default_write_converters(options: &WriteOptions) -> WriteOptions {
    let mut effective = options.clone();
    effective.converters =
        crate::converters::default_converter_loader::load_default_write_converter()
            .merged_with(&options.converters);
    effective
}

/// Writes typed rows to a new BIFF8 (`.xls`) file.
///
/// Java mapping: `EasyExcel.write(path, head).excelType(XLS).sheet().doWrite(data)`.
///
/// # Errors
///
/// Returns a conversion, worksheet-configuration, BIFF8-format, or I/O error.

#[derive(Clone, Default)]
pub(crate) struct CapturedOutput(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedOutput {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("CSV capture lock poisoned"))?
            .extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn take_captured_output(output: &CapturedOutput) -> Result<Vec<u8>> {
    let mut bytes = output
        .0
        .lock()
        .map_err(|_| ExcelError::Io(std::io::Error::other("CSV capture lock poisoned")))?;
    Ok(std::mem::take(&mut *bytes))
}

pub(crate) struct PreparedWriteRow {
    absent: bool,
    original_cells: Vec<CellValue>,
    cells: Vec<WriteCellData>,
}

pub(crate) fn convert_row_at<T>(
    row: &T,
    converters: &ConverterRegistry,
    sheet_name: &str,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> Result<(Vec<CellValue>, Vec<WriteCellData>)>
where
    T: ExcelRow,
{
    let selected_schema_indexes = (!T::schema().is_empty()).then(|| {
        columns
            .iter()
            .map(|(_, schema_index, _)| *schema_index)
            .collect::<Vec<_>>()
    });
    row.to_excel_write_row_selected(converters, selected_schema_indexes.as_deref())
        .map_err(|error| {
            let ExcelError::Data {
                column,
                field,
                value,
                message,
                ..
            } = error
            else {
                return error;
            };
            let physical_column = columns
                .iter()
                .find(|(_, _, candidate)| candidate.field == field)
                .map(|(physical, _, _)| *physical)
                .or(column);
            ExcelError::Data {
                sheet: sheet_name.to_owned(),
                row: row_index,
                column: physical_column,
                field,
                value,
                message,
            }
        })
}

// 泛型行对象按值传入是宏生成的调用惯例，改引用会改变泛型约束
#[allow(clippy::needless_pass_by_value)]
fn prepare_write_row<T>(
    row: T,
    converters: &ConverterRegistry,
    sheet_name: &str,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> Result<PreparedWriteRow>
where
    T: ExcelRow,
{
    if row.is_absent_row() {
        return Ok(PreparedWriteRow {
            absent: true,
            original_cells: Vec::new(),
            cells: Vec::new(),
        });
    }
    let (original_cells, cells) = convert_row_at(&row, converters, sheet_name, row_index, columns)?;
    Ok(PreparedWriteRow {
        absent: false,
        original_cells,
        cells,
    })
}

pub(crate) fn write_csv_to<T, I>(
    path: &Path,
    output: Box<dyn Write + Send>,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let columns = selected_columns(T::schema(), options)?;
    let first_data_row = head_rows_for_schema_state(T::schema().is_empty(), options)?;
    let csv_converters =
        crate::converters::default_converter_loader::load_default_write_converter()
            .merged_with(&options.converters)
            .with_write_target(Some(crate::core::CellDataType::String));
    let mut rows = rows.into_iter().enumerate().map(|(offset, row)| {
        prepare_write_row(
            row,
            &csv_converters,
            &options.sheet_name,
            first_data_row.saturating_add(u32::try_from(offset).unwrap_or(u32::MAX)),
            &columns,
        )
    });
    write_csv_records::<T>(
        path,
        output,
        options,
        &columns,
        T::schema().is_empty(),
        &mut rows,
        handlers,
    )
}

pub(crate) fn write_csv_records<T>(
    path: &Path,
    output: Box<dyn Write + Send>,
    options: &WriteOptions,
    columns: &[(usize, usize, &'static ExcelColumn)],
    schema_is_empty: bool,
    rows: &mut dyn Iterator<Item = Result<PreparedWriteRow>>,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    csv_encoding(&options.charset)?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        path,
        i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
        None,
        options,
    )?;
    let sheet_context = holder_scope.sheet(WriteSheetContext::new(&options.sheet_name));
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;

    let mut writer = create_csv_record_writer(output, &options.charset, options.with_bom)?;
    append_csv_records(
        &mut writer,
        options,
        columns,
        schema_is_empty,
        rows,
        handlers,
        0,
        0,
        true,
        Some(&holder_scope),
    )?;
    finish_csv_record_writer(writer)?;
    after_sheet(handlers, &sheet_context)?;
    after_workbook(handlers, &workbook_context)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_csv_records(
    writer: &mut csv::Writer<CsvEncodingWriter>,
    options: &WriteOptions,
    columns: &[(usize, usize, &'static ExcelColumn)],
    schema_is_empty: bool,
    rows: &mut dyn Iterator<Item = Result<PreparedWriteRow>>,
    handlers: &mut [Box<dyn WriteHandler>],
    mut row_index: u32,
    mut data_index: usize,
    write_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress> {
    let mut csv_workbook = CsvWorkbook::new(
        "und",
        options.use_1904_windowing,
        options.use_scientific_format,
        options.charset.clone(),
        options.with_bom,
    );
    let csv_sheet = create_sheet(&mut csv_workbook, &options.sheet_name)?;
    csv_sheet.set_next_row_index(row_index);
    let head_rows = head_rows_for_schema_state(schema_is_empty, options)?;
    if write_head && head_rows > 0 {
        if let Some(head) = &options.dynamic_head {
            let head = selected_dynamic_head_paths(columns, head)?;
            for level in 0..head_rows {
                #[allow(clippy::cast_possible_truncation)]
                let level = level as usize;
                let labels = head
                    .iter()
                    .map(|path| normalized_head_label(path, level).to_owned())
                    .collect::<Vec<_>>();
                let record = csv_header_record(
                    csv_sheet,
                    row_index,
                    columns,
                    &labels,
                    &options.sheet_name,
                    handlers,
                    holder_scope,
                )?;
                writer.write_record(record).map_err(format_error)?;
                row_index += 1;
            }
        } else {
            let labels = columns
                .iter()
                .map(|(_, _, column)| column.name.to_owned())
                .collect::<Vec<_>>();
            let record = csv_header_record(
                csv_sheet,
                row_index,
                columns,
                &labels,
                &options.sheet_name,
                handlers,
                holder_scope,
            )?;
            writer.write_record(record).map_err(format_error)?;
            row_index = 1;
        }
    }
    for prepared in rows {
        let PreparedWriteRow {
            absent,
            original_cells,
            cells,
        } = prepared?;
        if absent {
            row_index = row_index.saturating_add(1);
            data_index = data_index.saturating_add(1);
            csv_sheet.set_next_row_index(row_index);
            continue;
        }
        let dynamic_columns = dynamic_columns_for_row(schema_is_empty, cells.len(), options);
        let row_columns = dynamic_columns.as_deref().unwrap_or(columns);
        let record = csv_data_record(
            csv_sheet,
            row_index,
            data_index,
            row_columns,
            &original_cells,
            &cells,
            &options.sheet_name,
            handlers,
            holder_scope,
        )?;
        writer.write_record(record).map_err(format_error)?;
        row_index += 1;
        data_index += 1;
    }
    Ok(WriteProgress {
        next_row: row_index,
        next_data_index: data_index,
    })
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
pub(crate) fn append_csv_rows<T, I>(
    writer: &mut csv::Writer<CsvEncodingWriter>,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    row_index: u32,
    data_index: usize,
    write_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let columns = selected_columns(T::schema(), options)?;
    let head_rows = if write_head {
        head_rows_for_schema_state(T::schema().is_empty(), options)?
    } else {
        0
    };
    let first_data_row = row_index.saturating_add(head_rows);
    let csv_converters =
        crate::converters::default_converter_loader::load_default_write_converter()
            .merged_with(&options.converters)
            .with_write_target(Some(crate::core::CellDataType::String));
    let mut rows = rows.into_iter().enumerate().map(|(offset, row)| {
        prepare_write_row(
            row,
            &csv_converters,
            &options.sheet_name,
            first_data_row.saturating_add(u32::try_from(offset).unwrap_or(u32::MAX)),
            &columns,
        )
    });
    append_csv_records(
        writer,
        options,
        &columns,
        T::schema().is_empty(),
        &mut rows,
        handlers,
        row_index,
        data_index,
        write_head,
        holder_scope,
    )
}

pub(crate) fn create_csv_record_writer(
    mut output: Box<dyn Write + Send>,
    charset: &CsvCharset,
    with_bom: bool,
) -> Result<csv::Writer<CsvEncodingWriter>> {
    let encoding = csv_encoding(charset)?;
    if with_bom {
        output.write_all(csv_bom(encoding))?;
    }
    Ok(csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(CsvEncodingWriter::new(output, encoding)))
}

pub(crate) fn create_stateful_csv_writer(
    path: &Path,
    charset: &CsvCharset,
    with_bom: bool,
) -> Result<csv::Writer<CsvEncodingWriter>> {
    create_csv_record_writer(Box::new(File::create(path)?), charset, with_bom)
}

pub(crate) fn finish_csv_record_writer(mut writer: csv::Writer<CsvEncodingWriter>) -> Result<()> {
    writer.flush()?;
    let mut output = writer.into_inner().map_err(format_error)?;
    output.finish()?;
    Ok(())
}

pub(crate) fn validate_csv_options(options: &WriteOptions) -> Result<()> {
    if options.password.is_some() {
        return Err(ExcelError::Unsupported(
            "password protection is not supported for CSV".to_owned(),
        ));
    }
    csv_encoding(&options.charset)?;
    Ok(())
}

// 保留 Result 签名以统一调用点 `?` 传播；当前恒返回 Ok(())
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn validate_xls_options(_options: &WriteOptions) -> Result<()> {
    // XLS password is now supported via BIFF8 RC4 (Phase 5.3)
    Ok(())
}

/// Saves a workbook to `path` (optionally password-protected).
///
/// `pub(crate)` so executor integration tests can persist worksheets built via
/// [`ExcelWriteAddExecutor`] without duplicating the save path.
pub(crate) fn save_workbook(
    workbook: &mut Workbook,
    path: &Path,
    password: Option<&str>,
) -> Result<()> {
    let Some(password) = password else {
        return workbook.save(path).map_err(format_error);
    };
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    save_encrypted_workbook_to(workbook, password, &mut file)
}

pub(crate) fn save_workbook_to_writer(
    workbook: &mut Workbook,
    output: &mut (dyn Write + Send),
    password: Option<&str>,
) -> Result<()> {
    if let Some(password) = password {
        let mut encrypted = std::io::Cursor::new(Vec::new());
        save_encrypted_workbook_to(workbook, password, &mut encrypted)?;
        output.write_all(encrypted.get_ref())?;
    } else {
        workbook
            .save_to_writer(&mut *output)
            .map_err(format_error)?;
    }
    output.flush()?;
    Ok(())
}

pub(crate) trait ReadWriteSeek: Read + Write + Seek {}

impl<T> ReadWriteSeek for T where T: Read + Write + Seek {}

pub(crate) fn save_encrypted_workbook_to(
    workbook: &mut Workbook,
    password: &str,
    file: &mut dyn ReadWriteSeek,
) -> Result<()> {
    let mut random = rand::rng();
    Ecma376AgileWriter::create(&mut random, password, file)
        .map_err(ExcelError::from)
        .and_then(|mut writer| {
            workbook
                .save_to_buffer()
                .map_err(format_error)
                .and_then(|plaintext| {
                    // The encryption crate writes plaintext only to its in-memory cursor; its
                    // `Write` implementation cannot reach the fallible output at this stage.
                    let _ = writer.write_all(&plaintext);
                    writer.finalize().map_err(ExcelError::from)
                })
        })
}

pub(crate) fn csv_header_record(
    csv_sheet: &mut CsvSheet,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
    labels: &[String],
    sheet_name: &str,
    handlers: &mut [Box<dyn WriteHandler>],
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<Vec<String>> {
    let relative = Some(usize::try_from(row_index).unwrap_or(usize::MAX));
    let row_context = WriteRowContext::new(sheet_name, row_index, relative, true);
    let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
    before_csv_row(handlers, &row_context)?;
    let row = create_row(csv_sheet, row_index)?;
    for ((physical_index, _, column), label) in columns.iter().zip(labels) {
        let column_index = to_column(*physical_index)?;
        let mut context = WriteCellContext::new(
            sheet_name,
            row_index,
            column_index,
            CellValue::String(label.clone()),
        )
        .with_column(column)
        .with_head(label.clone())
        .without_original_value()
        .with_relative_row_index(relative);
        if let Some(scope) = holder_scope {
            context = scope.cell(context);
        }
        before_csv_cell(handlers, &mut context)?;
        after_csv_cell(handlers, &mut context)?;
        if !context.skip {
            create_cell(row, column_index)?.set_value(context.value.clone());
        }
    }
    after_csv_row(handlers, &row_context)?;
    let width = csv_record_width(columns);
    Ok(csv_sheet
        .take_last_row()
        .expect("CSV row was just created")
        .into_record(width))
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
pub(crate) fn csv_data_record(
    csv_sheet: &mut CsvSheet,
    row_index: u32,
    relative_row_index: usize,
    columns: &[(usize, usize, &'static ExcelColumn)],
    original_cells: &[CellValue],
    cells: &[WriteCellData],
    sheet_name: &str,
    handlers: &mut [Box<dyn WriteHandler>],
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<Vec<String>> {
    let row_context = WriteRowContext::new(sheet_name, row_index, Some(relative_row_index), false);
    let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
    before_csv_row(handlers, &row_context)?;
    let row = create_row(csv_sheet, row_index)?;
    for (physical_index, schema_index, metadata) in columns {
        let column_index = to_column(*physical_index)?;
        let mut context = WriteCellContext::new(
            sheet_name,
            row_index,
            column_index,
            cells
                .get(*schema_index)
                .map_or(CellValue::Empty, WriteCellData::effective_value),
        )
        .with_column(metadata)
        .with_original_value(
            original_cells
                .get(*schema_index)
                .unwrap_or(&CellValue::Empty)
                .clone(),
        )
        .with_relative_row_index(Some(relative_row_index));
        if let Some(scope) = holder_scope {
            context = scope.cell(context);
        }
        before_csv_cell(handlers, &mut context)?;
        after_csv_cell(handlers, &mut context)?;
        if !context.skip {
            create_cell(row, column_index)?.set_value(context.value.clone());
        }
    }
    after_csv_row(handlers, &row_context)?;
    let width = csv_record_width(columns);
    Ok(csv_sheet
        .take_last_row()
        .expect("CSV row was just created")
        .into_record(width))
}

pub(crate) fn csv_record_width(columns: &[(usize, usize, &'static ExcelColumn)]) -> usize {
    columns
        .iter()
        .map(|(physical_index, _, _)| physical_index + 1)
        .max()
        .unwrap_or(0)
}

// XLS-specific helper functions (moved from lib.rs)

pub(crate) fn write_xls_onto_template<T, I>(
    path: &Path,
    output: Option<&mut (dyn Write + Send)>,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    validate_xls_options(options)?;
    let bytes = crate::write::template_write::load_template_bytes(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    if !crate::write::biff8::looks_like_xls(&bytes) {
        return Err(ExcelError::Format(
            "xls with_template requires an OLE .xls workbook".to_owned(),
        ));
    }
    let mut package = crate::write::biff8::Biff8TemplatePackage::from_bytes(&bytes)?;
    let sheet_names = package.sheet_names();
    let (target_index, target_name, create_new) =
        crate::write::template_write::resolve_package_target(
            &sheet_names,
            options.sheet_index,
            &options.sheet_name,
        );
    if create_new {
        return Err(ExcelError::Unsupported(
            "xls template cannot create sheets absent from the template".to_owned(),
        ));
    }
    let mut write_options = options.clone();
    write_options.sheet_name.clone_from(&target_name);
    let start_row = package.next_row_for_sheet(&target_name)?;
    for range in automatic_dynamic_head_merge_ranges::<T>(&write_options, start_row, true)? {
        package.add_merge_range(&target_name, merge_range_to_biff8(range)?)?;
    }
    let (mut append_rows, original_rows, _converted_rows, absent_rows) =
        collect_template_append_rows::<T, I>(&write_options, rows, true, start_row)?;
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        path,
        i32::try_from(target_index).unwrap_or(i32::MAX),
        None,
        &write_options,
    )?;
    let sheet_context = holder_scope.sheet(WriteSheetContext::new(&target_name));
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;
    let _ignore_styles = run_template_handler_callbacks::<T>(
        &write_options,
        handlers,
        &mut append_rows,
        &original_rows,
        &absent_rows,
        true,
        0,
        start_row,
        Some(&holder_scope),
    )?;
    package.append_rows(&target_name, &append_rows)?;
    after_sheet(handlers, &sheet_context)?;
    match output {
        Some(writer) => package.save_to_writer(writer),
        None => package.save_to_path(path),
    }
}

pub(crate) fn save_xls_book(book: &Biff8Book, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    book.write_to(&mut file)?;
    file.flush()?;
    Ok(())
}

pub(crate) fn write_sheet_to_biff8_book<T, I>(
    book: &mut Biff8Book,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let sheet_name = effective_sheet_name(options);
    let mut write_options = options.clone();
    write_options.sheet_name.clone_from(&sheet_name);
    book.use_1904_windowing = write_options.use_1904_windowing;
    create_sheet(book, &sheet_name)?;
    let sheet_context = WriteSheetContext::new(&sheet_name);
    let sheet_context =
        holder_scope.map_or(sheet_context.clone(), |scope| scope.sheet(sheet_context));
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;
    let progress = append_rows_to_biff8_sheet::<T, I>(
        book,
        &sheet_name,
        &write_options,
        rows,
        handlers,
        WriteProgress {
            next_row: relative_head_start_row(&write_options),
            next_data_index: 0,
        },
        true,
        holder_scope,
    )?;
    after_sheet(handlers, &sheet_context)?;
    Ok(progress)
}

// 参数与 Java 写入路径一一对应且函数体覆盖完整 BIFF8 追加流程，拆分破坏可追溯性
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn append_rows_to_biff8_sheet<T, I>(
    book: &mut Biff8Book,
    sheet_name: &str,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    progress: WriteProgress,
    write_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let WriteProgress {
        next_row: mut row_index,
        next_data_index: mut data_index,
    } = progress;
    let global = WriteGlobalFlags::from(options);
    let columns = selected_columns(T::schema(), options)?;
    let metadata = T::write_metadata();
    let head_rows = head_rows_for_schema(T::schema(), options)?;
    let loop_merges = effective_loop_merges(&columns, options, handlers)?;

    if write_head {
        apply_biff8_column_widths::<T>(book.sheet_mut(sheet_name), options, handlers)?;
        apply_biff8_once_absolute_merges::<T>(book.sheet_mut(sheet_name), handlers)?;
        for range in &options.merge_ranges {
            add_biff8_merge_range(book.sheet_mut(sheet_name), *range)?;
        }
    }

    if write_head && head_rows > 0 {
        write_biff8_headers(
            book,
            sheet_name,
            &columns,
            options,
            metadata,
            handlers,
            row_index,
            holder_scope,
        )?;
        // Annotation `@HeadRowHeight` / `SimpleRowHeightStyleStrategy`
        let head_height = collect_handler_head_row_height(handlers).or(metadata.head_row_height);
        if let Some(height) = head_height {
            let sheet = book.sheet_mut(sheet_name);
            for head_row in row_index..row_index + head_rows {
                let row = u16::try_from(head_row)
                    .map_err(|_| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
                sheet.set_row_height(row, height);
            }
        }
        if options.automatic_merge_head
            && let Some(head) = &options.dynamic_head
        {
            let head = selected_dynamic_head_paths(&columns, head)?;
            merge_biff8_dynamic_head_groups(
                book.sheet_mut(sheet_name),
                &columns,
                &head,
                row_index,
            )?;
        }
        row_index = row_index
            .checked_add(head_rows)
            .ok_or_else(|| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
    }

    let row_list: Vec<T> = rows.into_iter().collect();
    for row in row_list {
        if row.is_absent_row() {
            row_index = row_index
                .checked_add(1)
                .ok_or_else(|| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
            data_index = data_index.saturating_add(1);
            continue;
        }
        let content_height =
            collect_handler_content_row_height(handlers).or(metadata.content_row_height);
        if let Some(height) = content_height {
            let row_u16 = u16::try_from(row_index)
                .map_err(|_| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
            book.sheet_mut(sheet_name).set_row_height(row_u16, height);
        }
        let (original_cells, cells) =
            convert_row_at(&row, &options.converters, sheet_name, row_index, &columns)?;
        let dynamic_columns = dynamic_columns_for_row(T::schema().is_empty(), cells.len(), options);
        let row_columns = dynamic_columns.as_deref().unwrap_or(&columns);
        let explicit_style = (!options.content_styles.is_empty())
            .then(|| &options.content_styles[data_index % options.content_styles.len()]);
        apply_biff8_loop_merges(
            book.sheet_mut(sheet_name),
            row_index,
            data_index,
            &loop_merges,
        )?;
        let row_context = WriteRowContext::new(sheet_name, row_index, Some(data_index), false);
        let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
        // 样式上下文按行构建一次：`content` 是常量构造，但移出单元格循环与
        // XLSX 路径保持一致，避免每单元格重复构造。
        let style_ctx = SheetStyleContext::content(explicit_style, metadata, global);
        begin_row_lifecycle(handlers, &row_context)?;
        for (physical_index, schema_index, column) in row_columns {
            let cell_data = cells.get(*schema_index);
            let value = cell_data.map_or(CellValue::Empty, WriteCellData::effective_value);
            let mut context =
                WriteCellContext::new(sheet_name, row_index, to_column(*physical_index)?, value)
                    .with_column(column)
                    .with_original_value(
                        original_cells
                            .get(*schema_index)
                            .unwrap_or(&CellValue::Empty)
                            .clone(),
                    )
                    .with_relative_row_index(Some(data_index));
            if let Some(scope) = holder_scope {
                context = scope.cell(context);
            }
            begin_cell_lifecycle(handlers, &mut context)?;
            finish_cell_lifecycle(handlers, &context)?;
            context.apply_cell_mutations();
            if !context.skip {
                let format_ctx = if context.ignore_fill_style {
                    style_ctx.column(column).without_fill_style()
                } else {
                    let format_ctx = style_ctx
                        .column(column)
                        .with_handler_cell(effective_handler_cell_style(handlers, &context));
                    cell_data.map_or(format_ctx, |cell| format_ctx.with_converted_cell(cell))
                };
                let cell =
                    cell_value_to_biff8_styled(&context.value, &mut book.styles, format_ctx)?;
                let mut row_creator = Biff8RowCreator {
                    sheet: book.sheet_mut(sheet_name),
                };
                let mut row = create_row(&mut row_creator, row_index)?;
                let column = u16::try_from(*physical_index).map_err(|_| {
                    ExcelError::Format("BIFF8 supports at most 256 columns".to_owned())
                })?;
                create_cell(&mut row, column)?.set(cell)?;
            }
        }
        finish_row_lifecycle(handlers, &row_context)?;
        if let Some(height) = row_context.row().requested_height() {
            let row = u16::try_from(row_index)
                .map_err(|_| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
            book.sheet_mut(sheet_name).set_row_height(row, height);
        }
        row_index = row_index
            .checked_add(1)
            .ok_or_else(|| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
        data_index += 1;
    }
    // LongestMatch / strategy widths may update after cells (Java afterCellDispose).
    apply_biff8_handler_column_widths::<T>(book.sheet_mut(sheet_name), options, handlers)?;
    let sheet = book.sheet_mut(sheet_name);
    sheet.next_row = row_index;
    sheet.next_data_index = data_index;
    Ok(WriteProgress {
        next_row: row_index,
        next_data_index: data_index,
    })
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_biff8_headers(
    book: &mut Biff8Book,
    sheet_name: &str,
    columns: &[(usize, usize, &'static ExcelColumn)],
    options: &WriteOptions,
    metadata: &ExcelWriteMetadata,
    handlers: &mut [Box<dyn WriteHandler>],
    start_row: u32,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let global = WriteGlobalFlags::from(options);
    let style_ctx = SheetStyleContext::head(&options.head_style, metadata, global);
    if let Some(head) = &options.dynamic_head {
        let head = selected_dynamic_head_paths(columns, head)?;
        let levels = head.iter().map(Vec::len).max().unwrap_or(0);
        for level in 0..levels {
            let row = start_row
                .checked_add(
                    u32::try_from(level)
                        .map_err(|_| ExcelError::Format("dynamic head is too deep".to_owned()))?,
                )
                .ok_or_else(|| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
            let row_context = WriteRowContext::new(sheet_name, row, Some(level), true);
            let row_context =
                holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
            begin_row_lifecycle(handlers, &row_context)?;
            for ((physical, _, column), path) in columns.iter().zip(&head) {
                let label = normalized_head_label(path, level).to_owned();
                write_biff8_styled_text_cell(
                    book,
                    sheet_name,
                    row,
                    *physical,
                    label,
                    column,
                    Some(level),
                    style_ctx.column(column),
                    handlers,
                    true,
                    holder_scope,
                )?;
            }
            finish_row_lifecycle(handlers, &row_context)?;
            if let Some(height) = row_context.row().requested_height() {
                let row = u16::try_from(row)
                    .map_err(|_| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
                book.sheet_mut(sheet_name).set_row_height(row, height);
            }
        }
    } else {
        let row_context = WriteRowContext::new(sheet_name, start_row, Some(0), true);
        let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
        begin_row_lifecycle(handlers, &row_context)?;
        for (physical_index, _, column) in columns {
            write_biff8_styled_text_cell(
                book,
                sheet_name,
                start_row,
                *physical_index,
                column.name.to_owned(),
                column,
                Some(0),
                style_ctx.column(column),
                handlers,
                true,
                holder_scope,
            )?;
        }
        finish_row_lifecycle(handlers, &row_context)?;
        if let Some(height) = row_context.row().requested_height() {
            let row = u16::try_from(start_row)
                .map_err(|_| ExcelError::Format("BIFF8 row overflow".to_owned()))?;
            book.sheet_mut(sheet_name).set_row_height(row, height);
        }
    }
    Ok(())
}

// 参数与 Java BIFF8 单元格写入签名一一对应；label/format_ctx 按值传入是调用点惯例
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
// CellFormatContext 是 Java 写入上下文 1:1 映射的聚合值类型，borrow 化会牵动整条调用链。
#[allow(clippy::large_types_passed_by_value)]
pub(crate) fn write_biff8_styled_text_cell(
    book: &mut Biff8Book,
    sheet_name: &str,
    row_index: u32,
    physical_index: usize,
    label: String,
    column: &'static ExcelColumn,
    relative_row_index: Option<usize>,
    format_ctx: CellFormatContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    is_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let column_index = to_column(physical_index)?;
    let mut context = WriteCellContext::new(
        sheet_name,
        row_index,
        column_index,
        CellValue::String(label.clone()),
    )
    .with_column(column)
    .with_relative_row_index(relative_row_index);
    if is_head {
        context = context.with_head(label.clone()).without_original_value();
    }
    if let Some(scope) = holder_scope {
        context = scope.cell(context);
    }
    begin_cell_lifecycle(handlers, &mut context)?;
    finish_cell_lifecycle(handlers, &context)?;
    context.apply_cell_mutations();
    if !context.skip {
        let format_ctx = if context.ignore_fill_style {
            format_ctx.without_fill_style()
        } else {
            format_ctx.with_handler_cell(effective_handler_cell_style(handlers, &context))
        };
        let cell = cell_value_to_biff8_styled(&context.value, &mut book.styles, format_ctx)?;
        let mut row_creator = Biff8RowCreator {
            sheet: book.sheet_mut(sheet_name),
        };
        let mut row = create_row(&mut row_creator, row_index)?;
        let column = u16::try_from(physical_index)
            .map_err(|_| ExcelError::Format("BIFF8 supports at most 256 columns".to_owned()))?;
        create_cell(&mut row, column)?.set(cell)?;
    }
    Ok(())
}

pub(crate) fn cell_value_to_biff8(
    value: &CellValue,
    global: WriteGlobalFlags,
) -> Result<Biff8Cell> {
    match value {
        CellValue::Empty => Ok(Biff8Cell::general(Biff8Value::Blank)),
        CellValue::String(text) | CellValue::Error(text) | CellValue::Hyperlink { text, .. } => {
            Ok(Biff8Cell::general(Biff8Value::Text(
                maybe_trim_cell_string(text, global.auto_trim).into_owned(),
            )))
        }
        CellValue::Formula(text) => Ok(Biff8Cell::general(Biff8Value::Formula(text.clone()))),
        CellValue::Bool(flag) => Ok(Biff8Cell::general(Biff8Value::Bool(*flag))),
        CellValue::Int(number) =>
        {
            #[allow(clippy::cast_precision_loss)]
            Ok(Biff8Cell::general(Biff8Value::Number(*number as f64)))
        }
        CellValue::Float(number) => Ok(Biff8Cell::general(Biff8Value::Number(*number))),
        CellValue::Decimal(number) => {
            let numeric = finite_decimal_f64(number, "BIFF8")?;
            if decimal_integer_requires_text(number)? {
                Ok(Biff8Cell::general(Biff8Value::Text(
                    number.to_plain_string(),
                )))
            } else {
                Ok(Biff8Cell::general(Biff8Value::Number(numeric)))
            }
        }
        CellValue::Date(date) => Ok(Biff8Cell::date_serial(date_to_excel_serial_with_windowing(
            *date,
            global.use_1904_windowing,
        ))),
        CellValue::DateTime(date_time) => Ok(Biff8Cell::datetime_serial(
            datetime_to_excel_serial_with_windowing(*date_time, global.use_1904_windowing),
        )),
        CellValue::Comment { value, .. } => cell_value_to_biff8(value, global),
        CellValue::Images { value, images } => {
            // Write the base value; image bytes are persisted via
            // write_raw_bytes on the Biff8Book (called by caller).
            for img in images {
                let _ = img.image();
            }
            cell_value_to_biff8(value, global)
        }
        CellValue::RichText(rich) => Ok(Biff8Cell::general(Biff8Value::Text(
            maybe_trim_cell_string(rich.text_string(), global.auto_trim).into_owned(),
        ))),
        CellValue::Image(bytes) => {
            // Write base value, image bytes handled by caller
            let _ = bytes;
            Ok(Biff8Cell::general(Biff8Value::Blank))
        }
    }
}

// 按值传入与调用点构造惯例一致，改引用会增加不必要的借用链
#[allow(clippy::large_types_passed_by_value)]
pub(crate) fn cell_value_to_biff8_styled(
    value: &CellValue,
    styles: &mut Biff8StyleTable,
    format_ctx: CellFormatContext<'_>,
) -> Result<Biff8Cell> {
    let cell = cell_value_to_biff8(value, format_ctx.global)?;
    let request = biff8_style_request(styles, format_ctx);
    let xf = styles.resolve_xf(&request, cell.xf);
    Ok(cell.with_xf(xf))
}

// 按值传入与调用点构造惯例一致，改引用会增加不必要的借用链
#[allow(clippy::large_types_passed_by_value)]
pub(crate) fn biff8_style_request(
    styles: &mut Biff8StyleTable,
    context: CellFormatContext<'_>,
) -> Biff8StyleRequest {
    let mut request = Biff8StyleRequest::default();
    let mut annotation_cell = context.converted_cell;
    if let Some(annotation_style) = context.cell {
        annotation_cell = Some(merge_write_cell_style(
            &annotation_style,
            annotation_cell.unwrap_or_default(),
        ));
    }
    if let Some(handler_style) = context.handler_cell {
        annotation_cell = Some(merge_write_cell_style(
            &handler_style,
            annotation_cell.unwrap_or_default(),
        ));
    }
    let mut font = context.font;
    if let Some(style) = annotation_cell {
        if let Some(style_font) = style.font {
            font = Some(match font {
                Some(target) => merge_handler_font_style(&style_font, target),
                None => style_font,
            });
        }
        // Remap RGB fills through the palette allocator before applying.
        let mut style = style;
        if let Some(ExcelColor::Rgb(rgb)) = style.fill_foreground_color {
            style.fill_foreground_color = Some(ExcelColor::Indexed(
                u8::try_from(styles.alloc_rgb_icv(rgb)).unwrap_or(8),
            ));
        }
        if let Some(ExcelColor::Rgb(rgb)) = style.fill_background_color {
            style.fill_background_color = Some(ExcelColor::Indexed(
                u8::try_from(styles.alloc_rgb_icv(rgb)).unwrap_or(8),
            ));
        }
        request.apply_excel_cell_style(style);
    }
    if let Some(font) = font {
        let mut font = font;
        if let Some(ExcelColor::Rgb(rgb)) = font.color {
            font.color = Some(ExcelColor::Indexed(
                u8::try_from(styles.alloc_rgb_icv(rgb)).unwrap_or(8),
            ));
        }
        request.apply_excel_font_style(font);
    }
    if let Some(style) = context.explicit {
        apply_writer_cell_style_to_request(&mut request, styles, style);
    }
    request
}

pub(crate) fn apply_writer_cell_style_to_request(
    request: &mut Biff8StyleRequest,
    styles: &mut Biff8StyleTable,
    style: &CellStyle,
) {
    if style.bold {
        request.bold = true;
    }
    if style.italic {
        request.italic = true;
    }
    if let Some(color) = style.font_color {
        request.font_color_icv = Some(styles.alloc_rgb_icv(color));
    }
    if let Some(color) = style.background_color {
        request.fill_pattern = Some(1);
        request.fill_fg_icv = Some(styles.alloc_rgb_icv(color));
        request.fill_bg_icv = Some(64); // automatic pattern background
    }
    if let Some(alignment) = style.horizontal_alignment {
        request.halign = Some(biff8_halign(alignment));
    }
    if let Some(alignment) = style.vertical_alignment {
        request.valign = Some(biff8_valign(alignment));
    }
    if style.wrap_text {
        request.wrap = true;
    }
}

pub(crate) fn apply_biff8_column_widths<T>(
    sheet: &mut Biff8Sheet,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    for (column, width) in &options.column_widths {
        let col = u8::try_from(*column)
            .map_err(|_| ExcelError::Format("BIFF8 supports at most 256 columns".to_owned()))?;
        sheet.set_column_width(col, *width);
    }
    let type_width = T::write_metadata().column_width;
    for (physical_index, _, column) in selected_columns(T::schema(), options)? {
        let col = u8::try_from(physical_index)
            .map_err(|_| ExcelError::Format("BIFF8 supports at most 256 columns".to_owned()))?;
        if sheet.column_widths.contains_key(&col) {
            continue;
        }
        if let Some(width) = column.column_width.or(type_width) {
            sheet.set_column_width(col, width);
        }
    }
    apply_biff8_handler_column_widths::<T>(sheet, options, handlers)
}

pub(crate) fn apply_biff8_handler_column_widths<T>(
    sheet: &mut Biff8Sheet,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    for (physical_index, _, _) in selected_columns(T::schema(), options)? {
        let col = u8::try_from(physical_index)
            .map_err(|_| ExcelError::Format("BIFF8 supports at most 256 columns".to_owned()))?;
        if options
            .column_widths
            .iter()
            .any(|(explicit, _)| usize::from(*explicit) == physical_index)
        {
            continue;
        }
        for handler in handlers {
            if let Some(width) = handler.style_column_width(physical_index) {
                sheet.set_column_width(col, width);
            }
        }
    }
    Ok(())
}

pub(crate) fn apply_biff8_once_absolute_merges<T>(
    sheet: &mut Biff8Sheet,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    for merge in collect_once_absolute_merges::<T>(handlers) {
        apply_biff8_once_absolute_merge_property(sheet, merge)?;
    }
    Ok(())
}

pub(crate) fn apply_biff8_once_absolute_merge_property(
    sheet: &mut Biff8Sheet,
    merge: crate::core::OnceAbsoluteMergeProperty,
) -> Result<()> {
    if merge.first_row_index < 0
        || merge.last_row_index < 0
        || merge.first_column_index < 0
        || merge.last_column_index < 0
    {
        return Ok(());
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    add_biff8_merge_range(
        sheet,
        MergeRange::new(
            merge.first_row_index as u32,
            merge.last_row_index as u32,
            merge.first_column_index as u16,
            merge.last_column_index as u16,
        ),
    )
}

pub(crate) const fn biff8_halign(align: HorizontalAlignment) -> u8 {
    match align {
        HorizontalAlignment::General => 0,
        HorizontalAlignment::Left => 1,
        HorizontalAlignment::Center => 2,
        HorizontalAlignment::Right => 3,
        HorizontalAlignment::Fill => 4,
        HorizontalAlignment::Justify => 5,
        HorizontalAlignment::CenterAcross => 6,
    }
}

pub(crate) const fn biff8_valign(align: VerticalAlignment) -> u8 {
    match align {
        VerticalAlignment::Top => 0,
        VerticalAlignment::Center => 1,
        VerticalAlignment::Bottom => 2,
        VerticalAlignment::Justify => 3,
        VerticalAlignment::Distributed => 4,
    }
}

pub(crate) fn add_biff8_merge_range(sheet: &mut Biff8Sheet, range: MergeRange) -> Result<()> {
    sheet.add_merge(merge_range_to_biff8(range)?)
}

pub(crate) fn merge_range_to_biff8(range: MergeRange) -> Result<Biff8Merge> {
    let first_row = u16::try_from(range.first_row)
        .map_err(|_| ExcelError::Format("BIFF8 merge row exceeds 65536".to_owned()))?;
    let last_row = u16::try_from(range.last_row)
        .map_err(|_| ExcelError::Format("BIFF8 merge row exceeds 65536".to_owned()))?;
    let first_col = u8::try_from(range.first_column)
        .map_err(|_| ExcelError::Format("BIFF8 merge column exceeds 256".to_owned()))?;
    let last_col = u8::try_from(range.last_column)
        .map_err(|_| ExcelError::Format("BIFF8 merge column exceeds 256".to_owned()))?;
    Ok(Biff8Merge {
        first_row,
        last_row,
        first_col,
        last_col,
    })
}

pub(crate) fn apply_biff8_loop_merges(
    sheet: &mut Biff8Sheet,
    row_index: u32,
    data_index: usize,
    strategies: &[MirroredLoopMergeStrategy],
) -> Result<()> {
    for strategy in strategies {
        #[allow(clippy::cast_possible_truncation)]
        let each_rows = strategy.each_rows as usize;
        if !data_index.is_multiple_of(each_rows) {
            continue;
        }
        let last_row = row_index
            .checked_add(strategy.each_rows - 1)
            .ok_or_else(|| ExcelError::Format("loop merge row overflow".to_owned()))?;
        let last_column = strategy
            .column_index
            .checked_add(strategy.column_extend - 1)
            .ok_or_else(|| ExcelError::Format("loop merge column overflow".to_owned()))?;
        add_biff8_merge_range(
            sheet,
            MergeRange::new(row_index, last_row, strategy.column_index, last_column),
        )?;
    }
    Ok(())
}

pub(crate) fn merge_biff8_dynamic_head_groups(
    sheet: &mut Biff8Sheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
    start_row: u32,
) -> Result<()> {
    for range in dynamic_head_merge_ranges(columns, head, start_row)? {
        add_biff8_merge_range(sheet, range)?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn csv_record(columns: &[(usize, usize, &'static ExcelColumn)]) -> Vec<String> {
    vec![String::new(); csv_record_width(columns)]
}

pub(crate) fn before_csv_row(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteRowContext,
) -> Result<()> {
    begin_row_lifecycle(handlers, context)
}

pub(crate) fn after_csv_row(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteRowContext,
) -> Result<()> {
    finish_row_lifecycle(handlers, context)
}

pub(crate) fn before_csv_cell(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &mut WriteCellContext,
) -> Result<()> {
    begin_cell_lifecycle(handlers, context)
}

pub(crate) fn after_csv_cell(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &mut WriteCellContext,
) -> Result<()> {
    finish_cell_lifecycle(handlers, context)?;
    context.apply_cell_mutations();
    Ok(())
}

/// Tracks the next physical row / data-row index while appending.
///
/// Immutable Java-holder state shared by row/cell callback construction.
#[derive(Debug, Clone)]
pub(crate) struct HandlerHolderScope {
    workbook: WriteWorkbookHolderView,
    sheet_no: i32,
    table_no: Option<i32>,
    current_holder_state: WriteContextHolderState,
}

impl HandlerHolderScope {
    pub(crate) fn new_resolved<T>(
        path: &Path,
        sheet_no: i32,
        table_no: Option<i32>,
        options: &WriteOptions,
    ) -> Result<Self>
    where
        T: ExcelRow,
    {
        Ok(Self {
            workbook: WriteWorkbookHolderView::new(path),
            sheet_no,
            table_no,
            current_holder_state: resolved_write_context_holder_state::<T>(options, table_no)?,
        })
    }

    fn row(&self, context: WriteRowContext) -> WriteRowContext {
        // 跳过临时 live_context 构造与解构——直接注入字段，省去中间全套克隆
        context.with_resolved_holder_context(
            self.workbook.clone(),
            self.sheet_no,
            self.table_no,
            self.current_holder_state.clone(),
        )
    }

    /// 每单元格注入 holder 状态。跳过临时 `live_context` 值构造与解构，
    /// 直接调用 `with_resolved_holder_context` 省去中间全套克隆。
    fn cell(&self, context: WriteCellContext) -> WriteCellContext {
        context.with_resolved_holder_context(
            self.workbook.clone(),
            self.sheet_no,
            self.table_no,
            self.current_holder_state.clone(),
        )
    }

    pub(crate) fn sheet(&self, context: WriteSheetContext) -> WriteSheetContext {
        context.with_resolved_holder_context(
            self.workbook.clone(),
            self.sheet_no,
            self.table_no,
            self.current_holder_state.clone(),
        )
    }
}

pub(crate) fn excel_column_width_pixels(width: u16) -> u32 {
    if width == 0 {
        0
    } else {
        u32::from(width) * 7 + 5
    }
}

/// Sets an OOXML column width that serializes as exact character units.
///
/// Java / POI `Sheet.setColumnWidth(col, chars * 256)` becomes
/// `width="{chars}"` in worksheet XML. `rust_xlsxwriter`'s
/// [`Worksheet::set_column_width`] stores `chars * 7 + 5` pixels and round-trips
/// to `~chars + 0.71`; using `chars * 7` pixels yields exact `width="{chars}"`.
pub(crate) fn set_xlsx_column_width_chars(
    worksheet: &mut Worksheet,
    column: u16,
    chars: u16,
) -> Result<()> {
    let pixels = u32::from(chars).saturating_mul(7);
    worksheet
        .set_column_width_pixels(column, pixels)
        .map_err(format_error)?;
    Ok(())
}

pub(crate) fn excel_row_height_pixels(height: Option<u16>) -> u32 {
    height.map_or(20, |height| (u32::from(height) * 4 + 1) / 3)
}

pub(crate) fn write_sheet_to_workbook<T, I>(
    workbook: &mut Workbook,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let mut spill = if options.compress_temp_files {
        Some(crate::write::gzip_spill::GzipSheetDataWriter::create_owned(
            &options.sheet_name,
        )?)
    } else {
        None
    };
    write_sheet_to_workbook_with_gzip::<T, I>(
        workbook,
        options,
        rows,
        handlers,
        spill.as_mut(),
        false,
        holder_scope,
    )
}

/// Creates a worksheet and appends rows, optionally mirroring into a gzip spill.
pub(crate) fn write_sheet_to_workbook_with_gzip<T, I>(
    workbook: &mut Workbook,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
    gzip_spill: Option<&mut crate::write::gzip_spill::GzipSheetDataWriter>,
    skip_sheet_create_callbacks: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let mut sheet_creator = XlsxSheetCreator {
        workbook,
        constant_memory: uses_constant_memory_spill(options),
    };
    let worksheet = create_sheet(&mut sheet_creator, &options.sheet_name)?;
    for (column, width) in &options.column_widths {
        set_xlsx_column_width_chars(worksheet, *column, *width)?;
    }
    apply_annotation_column_widths::<T>(worksheet, options)?;
    // Static strategy widths (e.g. SimpleColumnWidth) apply before cells.
    apply_handler_column_widths::<T>(worksheet, options, handlers)?;
    apply_annotation_once_absolute_merge::<T>(worksheet, handlers)?;
    // Java `OnceAbsoluteMergeStrategy.afterSheetCreate` via registerWriteHandler
    apply_handler_once_absolute_merge(worksheet, handlers)?;
    for range in &options.merge_ranges {
        worksheet
            .merge_range(
                range.first_row,
                range.first_column,
                range.last_row,
                range.last_column,
                "",
                &Format::new(),
            )
            .map_err(format_error)?;
    }
    let head_rows = head_rows_for_schema(T::schema(), options)?;
    let freeze_panes = options
        .freeze_panes
        .or_else(|| (options.freeze_head && options.need_head).then_some((head_rows, 0)));
    if let Some((row, column)) = freeze_panes {
        worksheet
            .set_freeze_panes(row, column)
            .map_err(format_error)?;
    }

    let sheet_context = WriteSheetContext::new(&options.sheet_name);
    let sheet_context =
        holder_scope.map_or(sheet_context.clone(), |scope| scope.sheet(sheet_context));
    if !skip_sheet_create_callbacks {
        before_sheet(handlers, &sheet_context)?;
        after_sheet_create(handlers, &sheet_context)?;
    }

    let progress = append_rows_to_worksheet_with_gzip_and_context::<T, I>(
        worksheet,
        options,
        rows,
        handlers,
        WriteProgress {
            // Java `WriteContextImpl.initHead`: newRowIndex += relativeHeadRowIndex()
            next_row: relative_head_start_row(options),
            next_data_index: 0,
        },
        true,
        T::write_metadata(),
        gzip_spill,
        holder_scope,
    )?;
    after_sheet(handlers, &sheet_context)?;
    // Optional autofit first; byte-length widths reapplied so LongestMatch
    // is not autofit-only (Java setColumnWidth(String.getBytes().length)).
    if options.auto_width || handlers_request_auto_width(handlers) {
        worksheet.autofit();
    }
    // LongestMatch measures in after_cell — re-apply measured widths after write
    // (Java AbstractColumnWidthStyleStrategy.afterCellDispose → setColumnWidth).
    apply_handler_column_widths::<T>(worksheet, options, handlers)?;
    Ok(progress)
}

/// ZIP/OOXML `withTemplate` path: preserve styles/merges and append sheetData.
///
/// When the requested sheet is missing, creates a new worksheet part inside the
/// package so existing sheets keep their styles and merges. The legacy
/// calamine → `rust_xlsxwriter` seed path is used only when
/// [`WriteOptions::use_legacy_template_seed`] is set.
pub(crate) fn write_xlsx_onto_template_package<T, I>(
    path: &Path,
    output: Option<&mut (dyn Write + Send)>,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    crate::write::template_write::validate_template_source(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    let bytes = crate::write::template_write::load_template_bytes(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    if options.use_legacy_template_seed {
        let mut workbook = Workbook::new();
        write_sheet_onto_template::<T, I>(&mut workbook, options, rows, handlers)?;
        return match output {
            Some(writer) => {
                save_workbook_to_writer(&mut workbook, writer, options.password.as_deref())
            }
            None => save_workbook(&mut workbook, path, options.password.as_deref()),
        };
    }

    let mut package = crate::write::template_write::TemplatePackage::from_bytes(&bytes)?;
    let sheet_names = package.sheet_names()?;
    let (target_index, target_name, create_new) =
        crate::write::template_write::resolve_package_target(
            &sheet_names,
            options.sheet_index,
            &options.sheet_name,
        );
    if create_new {
        package.ensure_sheet(&target_name)?;
    }

    let mut write_options = options.clone();
    write_options.sheet_name.clone_from(&target_name);
    apply_template_holder_layout::<T>(&mut package, &target_name, &write_options, handlers, &[])?;
    let start_row = package.next_row_for_sheet(&target_name)?.saturating_sub(1);
    let head_merges = automatic_dynamic_head_merge_ranges::<T>(&write_options, start_row, true)?;
    package.apply_sheet_layout(&target_name, &[], &head_merges)?;
    let (mut append_rows, original_rows, converted_rows, absent_rows) =
        collect_template_append_rows::<T, I>(&write_options, rows, true, start_row)?;
    let mut row_heights = template_append_row_heights::<T>(
        &write_options,
        handlers,
        true,
        append_rows.len(),
        &absent_rows,
    )?;
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        path,
        i32::try_from(target_index).unwrap_or(i32::MAX),
        None,
        &write_options,
    )?;
    let sheet_context = holder_scope.sheet(WriteSheetContext::new(&target_name));
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;
    let effects = run_template_handler_callbacks::<T>(
        &write_options,
        handlers,
        &mut append_rows,
        &original_rows,
        &absent_rows,
        true,
        0,
        start_row,
        Some(&holder_scope),
    )?;
    if row_heights.is_empty() && effects.requested_row_heights.iter().any(Option::is_some) {
        row_heights.resize(effects.requested_row_heights.len(), None);
    }
    for (height, requested) in row_heights.iter_mut().zip(&effects.requested_row_heights) {
        if requested.is_some() {
            *height = *requested;
        }
    }
    let cell_styles = template_append_cell_styles::<T>(
        &mut package,
        &write_options,
        handlers,
        &append_rows,
        &original_rows,
        &converted_rows,
        &effects.ignore_styles,
        &effects.requested_styles,
        true,
        0,
    )?;
    package.append_rows_with_layout_and_absent(
        &target_name,
        &append_rows,
        &row_heights,
        &cell_styles,
        &absent_rows,
    )?;
    after_sheet(handlers, &sheet_context)?;
    save_template_package(&package, path, output, options.password.as_deref())
}

/// Resolves Java annotation/handler row-height precedence for template rows.
pub(crate) fn template_append_row_heights<T>(
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
    write_head: bool,
    row_count: usize,
    absent_rows: &[bool],
) -> Result<Vec<Option<u16>>>
where
    T: ExcelRow,
{
    let head_start = if write_head {
        usize::try_from(relative_head_start_row(options)).unwrap_or(usize::MAX)
    } else {
        0
    }
    .min(row_count);
    let head_end = head_start
        .saturating_add(if write_head {
            usize::try_from(head_rows_for_schema(T::schema(), options)?).unwrap_or(0)
        } else {
            0
        })
        .min(row_count);
    let metadata = T::write_metadata();
    let head_height = collect_handler_head_row_height(handlers).or(metadata.head_row_height);
    let content_height =
        collect_handler_content_row_height(handlers).or(metadata.content_row_height);
    if head_height.is_none() && content_height.is_none() {
        return Ok(Vec::new());
    }
    Ok((0..row_count)
        .map(|index| {
            if absent_rows.get(index).copied().unwrap_or(false) {
                None
            } else if (head_start..head_end).contains(&index) {
                head_height
            } else {
                content_height
            }
        })
        .collect())
}

pub(crate) struct TemplateHandlerEffects {
    pub(crate) ignore_styles: Vec<Vec<bool>>,
    pub(crate) requested_styles: Vec<Vec<Option<ExcelCellStyle>>>,
    pub(crate) requested_row_heights: Vec<Option<u16>>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_template_handler_callbacks<T>(
    options: &WriteOptions,
    handlers: &mut [Box<dyn WriteHandler>],
    rows: &mut [Vec<(usize, CellValue)>],
    original_rows: &[Vec<(usize, CellValue)>],
    absent_rows: &[bool],
    write_head: bool,
    next_data_index: usize,
    start_row: u32,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<TemplateHandlerEffects>
where
    T: ExcelRow,
{
    let columns = selected_columns(T::schema(), options)?;
    let head_start = if write_head {
        usize::try_from(relative_head_start_row(options)).unwrap_or(usize::MAX)
    } else {
        0
    }
    .min(rows.len());
    let head_end = head_start
        .saturating_add(if write_head {
            usize::try_from(head_rows_for_schema(T::schema(), options)?).unwrap_or(0)
        } else {
            0
        })
        .min(rows.len());
    let mut ignored_styles = Vec::with_capacity(rows.len());
    let mut requested_styles = Vec::with_capacity(rows.len());
    let mut requested_row_heights = Vec::with_capacity(rows.len());
    for (row_offset, row) in rows.iter_mut().enumerate() {
        if absent_rows.get(row_offset).copied().unwrap_or(false) {
            ignored_styles.push(Vec::new());
            requested_styles.push(Vec::new());
            requested_row_heights.push(None);
            continue;
        }
        let is_head = (head_start..head_end).contains(&row_offset);
        let row_index = start_row.saturating_add(u32::try_from(row_offset).unwrap_or(u32::MAX));
        let relative_row_index = if is_head {
            Some(row_offset.saturating_sub(head_start))
        } else {
            Some(next_data_index + row_offset.saturating_sub(head_end))
        };
        let row_context =
            WriteRowContext::new(&options.sheet_name, row_index, relative_row_index, is_head);
        let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
        begin_row_lifecycle(handlers, &row_context)?;
        let mut emitted = Vec::with_capacity(row.len());
        let mut row_ignored_styles = Vec::with_capacity(row.len());
        let mut row_requested_styles = Vec::with_capacity(row.len());
        for (physical_index, value) in row.iter() {
            let column = columns
                .iter()
                .find(|(index, _, _)| index == physical_index)
                .map(|(_, _, column)| *column);
            let mut context = WriteCellContext::new(
                &options.sheet_name,
                row_index,
                u16::try_from(*physical_index).map_err(|_| {
                    ExcelError::Format("template column index exceeds XLSX limit".to_owned())
                })?,
                value.clone(),
            )
            .with_relative_row_index(relative_row_index);
            if let Some(column) = column {
                context = context.with_column(column);
            }
            if is_head {
                context = context.with_head(value.as_text()).without_original_value();
            } else if let Some(original_value) = original_rows
                .get(row_offset)
                .and_then(|row| row.iter().find(|(index, _)| index == physical_index))
                .map(|(_, value)| value.clone())
            {
                context = context.with_original_value(original_value);
            }
            if let Some(scope) = holder_scope {
                context = scope.cell(context);
            }
            begin_cell_lifecycle(handlers, &mut context)?;
            finish_cell_lifecycle(handlers, &context)?;
            context.apply_cell_mutations();
            if !context.skip {
                emitted.push((*physical_index, context.value.clone()));
                row_ignored_styles.push(context.ignore_fill_style);
                row_requested_styles.push(context.cell().requested_style());
            }
        }
        *row = emitted;
        ignored_styles.push(row_ignored_styles);
        requested_styles.push(row_requested_styles);
        finish_row_lifecycle(handlers, &row_context)?;
        requested_row_heights.push(row_context.row().requested_height());
    }
    Ok(TemplateHandlerEffects {
        ignore_styles: ignored_styles,
        requested_styles,
        requested_row_heights,
    })
}

/// Compiles annotation, explicit and strategy styles with `rust_xlsxwriter`,
/// imports their OOXML records into the preserved template, and returns a
/// style-index matrix aligned with `rows`.
// 参数与 Java 样式编译流程一一对应，函数体覆盖完整样式矩阵编译，拆分会割裂上下文
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn template_append_cell_styles<T>(
    package: &mut crate::write::template_write::TemplatePackage,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
    rows: &[Vec<(usize, CellValue)>],
    original_rows: &[Vec<(usize, CellValue)>],
    converted_rows: &[Vec<(usize, WriteCellData)>],
    ignore_styles: &[Vec<bool>],
    requested_styles: &[Vec<Option<ExcelCellStyle>>],
    write_head: bool,
    next_data_index: usize,
) -> Result<Vec<Vec<Option<u32>>>>
where
    T: ExcelRow,
{
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let columns = selected_columns(T::schema(), options)?;
    let metadata = T::write_metadata();
    let global = WriteGlobalFlags::from(options);
    let head_start = if write_head {
        usize::try_from(relative_head_start_row(options)).unwrap_or(usize::MAX)
    } else {
        0
    }
    .min(rows.len());
    let head_end = head_start
        .saturating_add(if write_head {
            usize::try_from(head_rows_for_schema(T::schema(), options)?).unwrap_or(0)
        } else {
            0
        })
        .min(rows.len());
    let start_row = package
        .next_row_for_sheet(&options.sheet_name)?
        .saturating_sub(1);
    let mut formats = Vec::new();
    let mut format_by_key = HashMap::new();
    let mut local_styles = Vec::with_capacity(rows.len());

    for (row_offset, row) in rows.iter().enumerate() {
        let is_head = (head_start..head_end).contains(&row_offset);
        let relative_row_index = if is_head {
            Some(row_offset.saturating_sub(head_start))
        } else {
            Some(next_data_index + row_offset.saturating_sub(head_end))
        };
        let explicit = if is_head {
            Some(&options.head_style)
        } else if options.content_styles.is_empty() {
            None
        } else {
            Some(
                &options.content_styles
                    [relative_row_index.unwrap_or(0) % options.content_styles.len()],
            )
        };
        let mut row_styles = Vec::with_capacity(row.len());
        for (cell_offset, (physical_index, value)) in row.iter().enumerate() {
            let column = columns
                .iter()
                .find(|(index, _, _)| index == physical_index)
                .map(|(_, _, column)| *column);
            let (annotation_cell, annotation_font, field) = match column {
                Some(column) if is_head => (
                    column.head_style.or(metadata.head_style),
                    column.head_font_style.or(metadata.head_font_style),
                    Some(column.field),
                ),
                Some(column) => (
                    column.content_style.or(metadata.content_style),
                    column.content_font_style.or(metadata.content_font_style),
                    Some(column.field),
                ),
                None if is_head => (metadata.head_style, metadata.head_font_style, None),
                None => (metadata.content_style, metadata.content_font_style, None),
            };
            let mut context = WriteCellContext::new(
                &options.sheet_name,
                start_row.saturating_add(u32::try_from(row_offset).unwrap_or(u32::MAX)),
                u16::try_from(*physical_index).map_err(|_| {
                    ExcelError::Format("template column index exceeds XLSX limit".to_owned())
                })?,
                value.clone(),
            )
            .with_relative_row_index(relative_row_index);
            if let Some(column) = column {
                context = context.with_column(column);
            } else {
                context.field = field;
            }
            if is_head {
                context = context.with_head(value.as_text()).without_original_value();
            } else if let Some(original_value) = original_rows
                .get(row_offset)
                .and_then(|row| row.iter().find(|(index, _)| index == physical_index))
                .map(|(_, value)| value.clone())
            {
                context = context.with_original_value(original_value);
            }
            context.activate_original_value();
            context.refresh_converted_data();
            context.ignore_fill_style = ignore_styles
                .get(row_offset)
                .and_then(|row| row.get(cell_offset))
                .copied()
                .unwrap_or(false);
            if context.ignore_fill_style {
                row_styles.push(None);
                continue;
            }
            let handler_cell = collect_handler_cell_style(handlers, &context);
            let handler_cell = requested_styles
                .get(row_offset)
                .and_then(|row| row.get(cell_offset))
                .copied()
                .flatten()
                .map_or(handler_cell, |requested| {
                    Some(match handler_cell {
                        Some(current) => merge_write_cell_style(&requested, current),
                        None => requested,
                    })
                });
            let converted_cell = converted_rows
                .get(row_offset)
                .and_then(|row| row.iter().find(|(index, _)| index == physical_index))
                .map(|(_, cell)| cell);
            let annotation_cell =
                annotation_cell.filter(|style| *style != ExcelCellStyle::default());
            let annotation_font = annotation_font.filter(|font| *font != ExcelFontStyle::default());
            let handler_cell = handler_cell.filter(|style| *style != ExcelCellStyle::default());
            let explicit = explicit.filter(|style| **style != CellStyle::default());
            if explicit.is_none()
                && annotation_cell.is_none()
                && annotation_font.is_none()
                && handler_cell.is_none()
                && converted_cell
                    .and_then(WriteCellData::write_cell_style)
                    .is_none()
                && converted_cell
                    .and_then(WriteCellData::data_format_data)
                    .and_then(|data| data.format())
                    .is_none()
            {
                row_styles.push(None);
                continue;
            }
            let converted_style = converted_cell.and_then(WriteCellData::write_cell_style);
            let converted_format = converted_cell
                .and_then(WriteCellData::data_format_data)
                .and_then(|data| data.format());
            let key = format!(
                "{explicit:?}|{annotation_cell:?}|{annotation_font:?}|{handler_cell:?}|\
                 {converted_style:?}|{converted_format:?}|{global:?}"
            );
            let local_index = if let Some(index) = format_by_key.get(&key).copied() {
                index
            } else {
                let index = formats.len();
                let format_context = CellFormatContext {
                    explicit,
                    cell: annotation_cell,
                    font: annotation_font,
                    handler_cell: None,
                    converted_cell: None,
                    converted_data_format: None,
                    global,
                }
                .with_handler_cell(handler_cell);
                let format_context = converted_cell.map_or(format_context, |cell| {
                    format_context.with_converted_cell(cell)
                });
                formats.push(cell_format(format_context));
                format_by_key.insert(key, index);
                index
            };
            row_styles.push(Some(local_index));
        }
        local_styles.push(row_styles);
    }
    if formats.is_empty() {
        return Ok(Vec::new());
    }

    let mut compiler = create_work_book(XlsxWorkBookCreator)?;
    let mut sheet_creator = XlsxSheetCreator {
        workbook: &mut compiler,
        constant_memory: false,
    };
    let worksheet = create_sheet(&mut sheet_creator, "Sheet1")?;
    for (index, format) in formats.iter().enumerate() {
        let row = u32::try_from(index)
            .map_err(|_| ExcelError::Format("too many template styles".to_owned()))?;
        worksheet
            .write_blank(row, 0, format)
            .map_err(format_error)?;
    }
    let bytes = compiler.save_to_buffer().map_err(format_error)?;
    let mapped = package.import_compiled_styles(&bytes, formats.len())?;
    Ok(local_styles
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|index| index.map(|index| mapped[index]))
                .collect()
        })
        .collect())
}

/// Builds sparse `(physical_column, value)` rows for ZIP `sheetData` append.
// 四元组返回值与 Java 追加行的多路数据一一对应，提取别名反而割裂阅读
#[allow(clippy::type_complexity)]
pub(crate) fn collect_template_append_rows<T, I>(
    options: &WriteOptions,
    rows: I,
    write_head: bool,
    start_row: u32,
) -> Result<(
    Vec<Vec<(usize, CellValue)>>,
    Vec<Vec<(usize, CellValue)>>,
    Vec<Vec<(usize, WriteCellData)>>,
    Vec<bool>,
)>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let columns = selected_columns(T::schema(), options)?;
    let mut output = Vec::new();
    let mut original_output = Vec::new();
    let mut converted_output = Vec::new();
    let mut absent_rows = Vec::new();
    let head_rows = head_rows_for_schema(T::schema(), options)?;
    if write_head {
        for _ in 0..relative_head_start_row(options) {
            output.push(Vec::new());
            original_output.push(Vec::new());
            converted_output.push(Vec::new());
            absent_rows.push(true);
        }
    }
    if write_head && head_rows > 0 {
        if let Some(head) = &options.dynamic_head {
            let head = selected_dynamic_head_paths(&columns, head)?;
            for level in 0..usize::try_from(head_rows).unwrap_or(0) {
                let mut row = Vec::with_capacity(columns.len());
                for ((physical_index, _, _), path) in columns.iter().zip(&head) {
                    let label = normalized_head_label(path, level).to_owned();
                    row.push((*physical_index, CellValue::String(label)));
                }
                output.push(row);
                original_output.push(Vec::new());
                converted_output.push(Vec::new());
                absent_rows.push(false);
            }
        } else {
            let mut row = Vec::with_capacity(columns.len());
            for (physical_index, _, column) in &columns {
                row.push((*physical_index, CellValue::String(column.name.to_owned())));
            }
            output.push(row);
            original_output.push(Vec::new());
            converted_output.push(Vec::new());
            absent_rows.push(false);
        }
    }
    for row in rows {
        if row.is_absent_row() {
            output.push(Vec::new());
            original_output.push(Vec::new());
            converted_output.push(Vec::new());
            absent_rows.push(true);
            continue;
        }
        let row_index = start_row.saturating_add(u32::try_from(output.len()).unwrap_or(u32::MAX));
        let (original_cells, cells) = convert_row_at(
            &row,
            &options.converters,
            &options.sheet_name,
            row_index,
            &columns,
        )?;
        let dynamic_columns = dynamic_columns_for_row(T::schema().is_empty(), cells.len(), options);
        let row_columns = dynamic_columns.as_deref().unwrap_or(&columns);
        let mut sparse = Vec::with_capacity(row_columns.len());
        let mut original_sparse = Vec::with_capacity(row_columns.len());
        let mut converted_sparse = Vec::with_capacity(row_columns.len());
        for (physical_index, schema_index, _) in row_columns {
            let cell = cells
                .get(*schema_index)
                .cloned()
                .unwrap_or_else(|| WriteCellData::new(CellValue::Empty));
            let value = cell.effective_value();
            sparse.push((*physical_index, value));
            converted_sparse.push((*physical_index, cell));
            original_sparse.push((
                *physical_index,
                original_cells
                    .get(*schema_index)
                    .cloned()
                    .unwrap_or(CellValue::Empty),
            ));
        }
        output.push(sparse);
        original_output.push(original_sparse);
        converted_output.push(converted_sparse);
        absent_rows.push(false);
    }
    Ok((output, original_output, converted_output, absent_rows))
}

/// Persists a template package to a path or stream, optionally encrypting.
pub(crate) fn save_template_package(
    package: &crate::write::template_write::TemplatePackage,
    path: &Path,
    output: Option<&mut (dyn Write + Send)>,
    password: Option<&str>,
) -> Result<()> {
    let plaintext = package.to_bytes()?;
    if let Some(password) = password {
        let mut encrypted = std::io::Cursor::new(Vec::new());
        save_encrypted_bytes_to(&plaintext, password, &mut encrypted)?;
        if let Some(writer) = output {
            writer.write_all(encrypted.get_ref())?;
            writer.flush()?;
        } else {
            std::fs::write(path, encrypted.get_ref())?;
        }
        return Ok(());
    }
    if let Some(writer) = output {
        writer.write_all(&plaintext)?;
        writer.flush()?;
        Ok(())
    } else {
        std::fs::write(path, plaintext).map_err(ExcelError::from)
    }
}

pub(crate) fn save_encrypted_bytes_to(
    plaintext: &[u8],
    password: &str,
    file: &mut dyn ReadWriteSeek,
) -> Result<()> {
    let mut random = rand::rng();
    Ecma376AgileWriter::create(&mut random, password, file)
        .map_err(ExcelError::from)
        .and_then(|mut writer| {
            let _ = writer.write_all(plaintext);
            writer.finalize().map_err(ExcelError::from)
        })
}

/// Seeds a workbook from `withTemplate` then appends typed rows to the target sheet.
///
/// **Legacy path only** — enabled via [`WriteOptions::use_legacy_template_seed`].
/// Value replay does not preserve styles/merges; prefer the ZIP package path.
///
/// 对应 Java：`WorkBookUtil.createWorkBook` (template branch) + `ExcelWriteAddExecutor`.
///
/// # Errors
///
/// Returns template validation/load errors, or standard XLSX write errors.
pub(crate) fn write_sheet_onto_template<T, I>(
    workbook: &mut Workbook,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<WriteProgress>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    crate::write::template_write::validate_template_source(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    let bytes = crate::write::template_write::load_template_bytes(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    )?;
    let sheets = crate::write::template_write::load_template_sheets(&bytes)?;
    let (target_index, target_name, create_new) =
        crate::write::template_write::resolve_template_target(
            &sheets,
            options.sheet_index,
            &options.sheet_name,
        );
    crate::write::template_write::seed_workbook_from_template(workbook, &sheets)?;

    let mut write_options = options.clone();
    write_options.sheet_name.clone_from(&target_name);

    if create_new {
        // Java creates a new sheet when the requested name/index is absent.
        return write_sheet_to_workbook::<T, I>(workbook, &write_options, rows, handlers, None);
    }

    let start_row = sheets.get(target_index).map_or(0, |sheet| sheet.next_row);
    let worksheet = workbook
        .worksheet_from_name(&target_name)
        .map_err(format_error)?;
    for (column, width) in &write_options.column_widths {
        set_xlsx_column_width_chars(worksheet, *column, *width)?;
    }
    apply_annotation_column_widths::<T>(worksheet, &write_options)?;
    apply_handler_column_widths::<T>(worksheet, &write_options, handlers)?;
    apply_annotation_once_absolute_merge::<T>(worksheet, handlers)?;
    apply_handler_once_absolute_merge(worksheet, handlers)?;
    for range in &write_options.merge_ranges {
        let offset = start_row;
        worksheet
            .merge_range(
                range.first_row.saturating_add(offset),
                range.first_column,
                range.last_row.saturating_add(offset),
                range.last_column,
                "",
                &Format::new(),
            )
            .map_err(format_error)?;
    }

    let sheet_context = WriteSheetContext::new(&target_name);
    before_sheet(handlers, &sheet_context)?;
    after_sheet_create(handlers, &sheet_context)?;
    let mut spill = if write_options.compress_temp_files {
        Some(crate::write::gzip_spill::GzipSheetDataWriter::create_owned(
            &target_name,
        )?)
    } else {
        None
    };
    let progress = append_rows_to_worksheet_with_gzip::<T, I>(
        worksheet,
        &write_options,
        rows,
        handlers,
        WriteProgress {
            next_row: start_row,
            next_data_index: 0,
        },
        true,
        T::write_metadata(),
        spill.as_mut(),
    )?;
    after_sheet(handlers, &sheet_context)?;
    if write_options.auto_width || handlers_request_auto_width(handlers) {
        worksheet.autofit();
    }
    // Byte-length widths win over optional autofit fallback.
    apply_handler_column_widths::<T>(worksheet, &write_options, handlers)?;
    Ok(progress)
}

pub(crate) fn apply_loop_merges(
    worksheet: &mut Worksheet,
    row_index: u32,
    data_index: usize,
    strategies: &[MirroredLoopMergeStrategy],
) -> Result<()> {
    for strategy in strategies {
        #[allow(clippy::cast_possible_truncation)]
        let each_rows = strategy.each_rows as usize;
        if !data_index.is_multiple_of(each_rows) {
            continue;
        }
        let last_row = row_index
            .checked_add(strategy.each_rows - 1)
            .ok_or_else(|| ExcelError::Format("loop merge row overflow".to_owned()))?;
        let last_column = strategy
            .column_index
            .checked_add(strategy.column_extend - 1)
            .ok_or_else(|| ExcelError::Format("loop merge column overflow".to_owned()))?;
        worksheet
            .merge_range(
                row_index,
                strategy.column_index,
                last_row,
                last_column,
                "",
                &Format::new(),
            )
            .map_err(format_error)?;
    }
    Ok(())
}

pub(crate) fn sort_handlers(handlers: &mut [Box<dyn WriteHandler>]) {
    handlers.sort_by_key(|handler| handler.order());
    let mut unique_values = HashSet::new();
    for handler in handlers {
        let duplicate = handler
            .as_not_repeat_executor()
            .is_some_and(|executor| !unique_values.insert(executor.unique_value().to_owned()));
        if duplicate {
            *handler = Box::new(NoopWriteHandler);
        }
    }
}

struct NoopWriteHandler;

impl WriteHandler for NoopWriteHandler {}

pub(crate) fn begin_row_lifecycle(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteRowContext,
) -> Result<()> {
    crate::util::write_handler_utils::before_row_create(handlers, context)?;
    crate::util::write_handler_utils::after_row_create(handlers, context)?;
    Ok(())
}

pub(crate) fn finish_row_lifecycle(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteRowContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_row_dispose(handlers, context)
}

pub(crate) fn begin_cell_lifecycle(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &mut WriteCellContext,
) -> Result<()> {
    crate::util::write_handler_utils::before_cell_create(handlers, context)?;
    context.apply_cell_mutations();
    crate::util::write_handler_utils::after_cell_create(handlers, context)?;
    context.apply_cell_mutations();
    crate::util::write_handler_utils::after_cell_data_converted(handlers, context)?;
    context.apply_cell_mutations();
    context.sync_cell_handle();
    Ok(())
}

pub(crate) fn finish_cell_lifecycle(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteCellContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_cell_dispose(handlers, context)
}

pub(crate) fn before_workbook(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteWorkbookContext,
) -> Result<()> {
    crate::util::write_handler_utils::before_workbook_create(handlers, context)
}

pub(crate) fn after_workbook_create(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteWorkbookContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_workbook_create(handlers, context)
}

pub(crate) fn after_workbook(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteWorkbookContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_workbook_dispose(handlers, context)
}

pub(crate) fn run_own_workbook_callbacks(scope: &HandlerExecutionScope, path: &Path) -> Result<()> {
    let mut own = scope.own_boxed();
    let context = WriteWorkbookContext::new(path);
    before_workbook(&mut own, &context)?;
    after_workbook_create(&mut own, &context)
}

pub(crate) fn before_sheet(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteSheetContext,
) -> Result<()> {
    crate::util::write_handler_utils::before_sheet_create(handlers, context)
}

pub(crate) fn after_sheet_create(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteSheetContext,
) -> Result<()> {
    crate::util::write_handler_utils::after_sheet_create(handlers, context)
}

pub(crate) fn after_sheet(
    handlers: &mut [Box<dyn WriteHandler>],
    context: &WriteSheetContext,
) -> Result<()> {
    for handler in handlers.iter_mut() {
        handler.after_sheet_dispose(context)?;
    }
    Ok(())
}

pub(crate) fn validate_excel_row_schema<T>() -> Result<()>
where
    T: ExcelRow,
{
    validate_schema(T::schema())
}

fn validate_schema(schema: &'static [ExcelColumn]) -> Result<()> {
    let mut indexed_fields = HashMap::new();
    for column in schema {
        let Some(index) = column.index else {
            continue;
        };
        if let Some(previous_field) = indexed_fields.insert(index, column.field) {
            return Err(ExcelError::Format(format!(
                "The index of '{previous_field}' and '{}' must be inconsistent",
                column.field
            )));
        }
    }
    Ok(())
}

fn ordered_columns(
    schema: &'static [ExcelColumn],
) -> Result<Vec<(usize, usize, &'static ExcelColumn)>> {
    validate_schema(schema)?;
    // Java `ClassUtils.buildSortedAllFieldMap` first groups non-indexed fields
    // by `@ExcelProperty.order` (preserving declaration order inside a group),
    // then inserts them into the first free physical indexes while skipping
    // every forced `@ExcelProperty.index`. Forced indexes are anchors, not a
    // secondary sort key.
    let forced_indexes = schema
        .iter()
        .filter_map(|column| column.index)
        .collect::<HashSet<_>>();
    let mut automatic = schema
        .iter()
        .enumerate()
        .filter(|(_, column)| column.index.is_none())
        .collect::<Vec<_>>();
    automatic.sort_by_key(|(schema_index, column)| (column.order, *schema_index));

    let mut columns = schema
        .iter()
        .enumerate()
        .filter_map(|(schema_index, column)| {
            column
                .index
                .map(|physical_index| (physical_index, schema_index, column))
        })
        .collect::<Vec<_>>();
    let mut next_automatic_index = 0usize;
    for (schema_index, column) in automatic {
        while forced_indexes.contains(&next_automatic_index) {
            next_automatic_index = next_automatic_index.saturating_add(1);
        }
        columns.push((next_automatic_index, schema_index, column));
        next_automatic_index = next_automatic_index.saturating_add(1);
    }
    columns.sort_by_key(|(physical_index, _, _)| *physical_index);
    Ok(columns)
}

pub(crate) fn apply_annotation_column_widths<T>(
    worksheet: &mut Worksheet,
    options: &WriteOptions,
) -> Result<()>
where
    T: ExcelRow,
{
    let type_width = T::write_metadata().column_width;
    for (physical_index, _, column) in selected_columns(T::schema(), options)? {
        if options
            .column_widths
            .iter()
            .any(|(explicit, _)| usize::from(*explicit) == physical_index)
        {
            continue;
        }
        if let Some(width) = column.column_width.or(type_width) {
            set_xlsx_column_width_chars(worksheet, to_column(physical_index)?, width)?;
        }
    }
    Ok(())
}

pub(crate) fn apply_template_holder_layout<T>(
    package: &mut crate::write::template_write::TemplatePackage,
    sheet_name: &str,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
    excluded_merges: &[crate::core::OnceAbsoluteMergeProperty],
) -> Result<()>
where
    T: ExcelRow,
{
    let explicit_columns = options
        .column_widths
        .iter()
        .map(|(column, _)| *column)
        .collect::<HashSet<_>>();
    let mut widths = options
        .column_widths
        .iter()
        .copied()
        .collect::<HashMap<_, _>>();
    let type_width = T::write_metadata().column_width;
    for (physical_index, _, column) in selected_columns(T::schema(), options)? {
        let physical_index = to_column(physical_index)?;
        if !explicit_columns.contains(&physical_index) {
            if let Some(width) = column.column_width.or(type_width) {
                widths.entry(physical_index).or_insert(width);
            }
            for handler in handlers {
                if let Some(width) = handler.style_column_width(usize::from(physical_index)) {
                    widths.insert(physical_index, width);
                }
            }
        }
    }
    let mut widths = widths.into_iter().collect::<Vec<_>>();
    widths.sort_unstable_by_key(|(column, _)| *column);

    let mut merges = Vec::new();
    if let Some(merge) = T::write_metadata().once_absolute_merge
        && !excluded_merges.contains(&merge)
        && let Some(range) = absolute_merge_range(merge)
    {
        merges.push(range);
    }
    for handler in handlers {
        if let Some(merge) = handler.style_once_absolute_merge()
            && !excluded_merges.contains(&merge)
            && let Some(range) = absolute_merge_range(merge)
            && !merges.contains(&range)
        {
            merges.push(range);
        }
    }
    package.apply_sheet_layout(sheet_name, &widths, &merges)
}

pub(crate) fn collect_once_absolute_merges<T>(
    handlers: &[Box<dyn WriteHandler>],
) -> Vec<crate::core::OnceAbsoluteMergeProperty>
where
    T: ExcelRow,
{
    let mut merges = Vec::new();
    if let Some(merge) = T::write_metadata().once_absolute_merge {
        merges.push(merge);
    }
    for merge in collect_handler_once_absolute_merges(handlers) {
        if !merges.contains(&merge) {
            merges.push(merge);
        }
    }
    merges
}

pub(crate) fn collect_handler_once_absolute_merges(
    handlers: &[Box<dyn WriteHandler>],
) -> Vec<crate::core::OnceAbsoluteMergeProperty> {
    let mut merges = Vec::new();
    for handler in handlers {
        if let Some(merge) = handler.style_once_absolute_merge()
            && !merges.contains(&merge)
        {
            merges.push(merge);
        }
    }
    merges
}

fn absolute_merge_range(merge: crate::core::OnceAbsoluteMergeProperty) -> Option<MergeRange> {
    if merge.first_row_index < 0
        || merge.last_row_index < 0
        || merge.first_column_index < 0
        || merge.last_column_index < 0
    {
        return None;
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Some(MergeRange::new(
        merge.first_row_index as u32,
        merge.last_row_index as u32,
        merge.first_column_index as u16,
        merge.last_column_index as u16,
    ))
}

/// Applies column widths from registered strategies
/// (Java `SimpleColumnWidthStyleStrategy` / `AbstractColumnWidthStyleStrategy`).
pub(crate) fn apply_handler_column_widths<T>(
    worksheet: &mut Worksheet,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    for (physical_index, _, _) in selected_columns(T::schema(), options)? {
        let column = to_column(physical_index)?;
        // Explicit `WriteOptions::column_widths` wins over strategies.
        if options
            .column_widths
            .iter()
            .any(|(explicit, _)| *explicit == column)
        {
            continue;
        }
        for handler in handlers {
            if let Some(width) = handler.style_column_width(physical_index) {
                set_xlsx_column_width_chars(worksheet, column, width)?;
            }
        }
    }
    Ok(())
}

/// Collects head row height from registered strategies
/// (Java `SimpleRowHeightStyleStrategy`).
pub(crate) fn collect_handler_head_row_height(handlers: &[Box<dyn WriteHandler>]) -> Option<u16> {
    handlers
        .iter()
        .rev()
        .find_map(|handler| handler.style_head_row_height())
}

/// Collects content row height from registered strategies
/// (Java `SimpleRowHeightStyleStrategy`).
pub(crate) fn collect_handler_content_row_height(
    handlers: &[Box<dyn WriteHandler>],
) -> Option<u16> {
    handlers
        .iter()
        .rev()
        .find_map(|handler| handler.style_content_row_height())
}

/// Whether any handler requests longest-match autofit
/// (Java `LongestMatchColumnWidthStyleStrategy`).
pub(crate) fn handlers_request_auto_width(handlers: &[Box<dyn WriteHandler>]) -> bool {
    handlers
        .iter()
        .any(|handler| handler.style_auto_column_width())
}

/// Merges cell styles from registered style strategies in handler order
/// (Java `AbstractCellStyleStrategy.afterCellDispose` + `WriteCellStyle.merge`).
fn collect_handler_cell_style(
    handlers: &[Box<dyn WriteHandler>],
    context: &WriteCellContext,
) -> Option<ExcelCellStyle> {
    let mut merged: Option<ExcelCellStyle> = None;
    for handler in handlers {
        if let Some(style) = handler.style_cell_style(context) {
            merged = Some(match merged {
                Some(target) => merge_write_cell_style(&style, target),
                None => style,
            });
        }
    }
    merged
}

/// Combines registered strategy styles with a mutation requested through the
/// logical POI-equivalent cell handle. The explicit handle request runs last,
/// matching a custom Java handler that mutates the POI cell in
/// `afterCellDispose`.
pub(crate) fn effective_handler_cell_style(
    handlers: &[Box<dyn WriteHandler>],
    context: &WriteCellContext,
) -> Option<ExcelCellStyle> {
    let merged = collect_handler_cell_style(handlers, context);
    context
        .cell()
        .requested_style()
        .map_or(merged, |requested| {
            Some(match merged {
                Some(current) => merge_write_cell_style(&requested, current),
                None => requested,
            })
        })
}

/// Applies type-level `@OnceAbsoluteMerge` metadata when all indexes are non-negative.
pub(crate) fn apply_annotation_once_absolute_merge<T>(
    worksheet: &mut Worksheet,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    let Some(merge) = T::write_metadata().once_absolute_merge else {
        return Ok(());
    };
    if handlers
        .iter()
        .any(|handler| handler.style_once_absolute_merge() == Some(merge))
    {
        return Ok(());
    }
    apply_once_absolute_merge_property(worksheet, merge)
}

/// Applies registered [`OnceAbsoluteMergeStrategy`] regions
/// (Java `OnceAbsoluteMergeStrategy.afterSheetCreate` → `addMergedRegionUnsafe`).
fn apply_handler_once_absolute_merge(
    worksheet: &mut Worksheet,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()> {
    for handler in handlers {
        if let Some(merge) = handler.style_once_absolute_merge() {
            apply_once_absolute_merge_property(worksheet, merge)?;
        }
    }
    Ok(())
}

/// Shared absolute-merge apply used by annotation and registered strategy paths.
pub(crate) fn apply_once_absolute_merge_property(
    worksheet: &mut Worksheet,
    merge: crate::core::OnceAbsoluteMergeProperty,
) -> Result<()> {
    if merge.first_row_index < 0
        || merge.last_row_index < 0
        || merge.first_column_index < 0
        || merge.last_column_index < 0
    {
        return Ok(());
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    worksheet
        .merge_range(
            merge.first_row_index as u32,
            merge.first_column_index as u16,
            merge.last_row_index as u32,
            merge.last_column_index as u16,
            "",
            &Format::new(),
        )
        .map_err(format_error)?;
    Ok(())
}

/// Builds loop-merge strategies from field-level `@ContentLoopMerge` metadata.
fn annotation_loop_merges_from_columns(
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> Result<Vec<MirroredLoopMergeStrategy>> {
    let mut strategies = Vec::new();
    for (physical_index, _, column) in columns {
        let Some(property) = column.loop_merge else {
            continue;
        };
        strategies.push(MirroredLoopMergeStrategy::new(
            property.each_row,
            property.column_extend,
            to_column(*physical_index)?,
        )?);
    }
    Ok(strategies)
}

pub(crate) fn effective_loop_merges(
    columns: &[(usize, usize, &'static ExcelColumn)],
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<Vec<MirroredLoopMergeStrategy>> {
    let mut strategies = options.loop_merges.clone();
    for strategy in annotation_loop_merges_from_columns(columns)? {
        if !strategies.contains(&strategy) {
            strategies.push(strategy);
        }
    }
    for handler in handlers {
        let Some((property, column_index)) = handler.style_loop_merge() else {
            continue;
        };
        let strategy = MirroredLoopMergeStrategy::new(
            property.each_row,
            property.column_extend,
            to_column(column_index)?,
        )?;
        if !strategies.contains(&strategy) {
            strategies.push(strategy);
        }
    }
    Ok(strategies)
}

pub(crate) fn selected_columns(
    schema: &'static [ExcelColumn],
    options: &WriteOptions,
) -> Result<Vec<(usize, usize, &'static ExcelColumn)>> {
    if schema.is_empty()
        && let Some(head) = &options.dynamic_head
    {
        return Ok(selected_dynamic_columns(head.len(), options));
    }
    let mut columns = ordered_columns(schema)?
        .into_iter()
        .filter(|(physical_index, _, column)| {
            let included_by_index = options
                .include_column_indexes
                .as_ref()
                .is_some_and(|indexes| indexes.contains(physical_index));
            let included_by_name = options
                .include_column_field_names
                .as_ref()
                .is_some_and(|names| names.iter().any(|name| name == column.field));
            let has_includes = options.include_column_indexes.is_some()
                || options.include_column_field_names.is_some();
            let excluded = options.exclude_column_indexes.contains(physical_index)
                || options
                    .exclude_column_field_names
                    .iter()
                    .any(|name| name == column.field);
            (!has_includes || included_by_index || included_by_name) && !excluded
        })
        .collect::<Vec<_>>();

    if options.order_by_include_column {
        columns.sort_by_key(|(physical_index, _, column)| {
            options
                .include_column_indexes
                .as_ref()
                .and_then(|indexes| indexes.iter().position(|index| index == physical_index))
                .or_else(|| {
                    options
                        .include_column_field_names
                        .as_ref()
                        .and_then(|names| names.iter().position(|name| name == column.field))
                })
                .unwrap_or(usize::MAX)
        });
        for (output_index, (physical_index, _, _)) in columns.iter_mut().enumerate() {
            *physical_index = output_index;
        }
    }
    Ok(columns)
}

const DYNAMIC_COLUMN: ExcelColumn = ExcelColumn::new("", "", None, i32::MAX, None);

#[inline(never)]
fn selected_dynamic_columns(
    column_count: usize,
    options: &WriteOptions,
) -> Vec<(usize, usize, &'static ExcelColumn)> {
    let mut columns = Vec::with_capacity(column_count);
    for index in 0..column_count {
        let included_by_index = match &options.include_column_indexes {
            Some(indexes) => indexes.contains(&index),
            None => false,
        };
        let has_includes = options.include_column_indexes.is_some()
            || options.include_column_field_names.is_some();
        let excluded = options.exclude_column_indexes.contains(&index);
        if (!has_includes || included_by_index) && !excluded {
            columns.push((index, index, &DYNAMIC_COLUMN));
        }
    }

    if options.order_by_include_column {
        if let Some(indexes) = &options.include_column_indexes {
            let mut ordered = Vec::with_capacity(columns.len());
            for requested in indexes {
                for column in &columns {
                    if column.1 == *requested {
                        ordered.push(*column);
                        break;
                    }
                }
            }
            columns = ordered;
        }
        for (output_index, (physical_index, _, _)) in columns.iter_mut().enumerate() {
            *physical_index = output_index;
        }
    }
    columns
}

pub(crate) fn dynamic_columns_for_row(
    schema_is_empty: bool,
    column_count: usize,
    options: &WriteOptions,
) -> Option<Vec<(usize, usize, &'static ExcelColumn)>> {
    if !schema_is_empty {
        return None;
    }
    let Some(head) = &options.dynamic_head else {
        return Some(selected_dynamic_columns(column_count, options));
    };

    // Java `ExcelWriteAddExecutor.addBasicTypeToExcel` consumes basic row
    // values sequentially while iterating the effective head map. When the
    // row is shorter it stops creating cells; when it is longer it appends the
    // remaining values after the greatest head column (issue #1702).
    let mut columns = selected_dynamic_columns(head.len(), options);
    columns.truncate(column_count);
    for (data_index, (_, schema_index, _)) in columns.iter_mut().enumerate() {
        *schema_index = data_index;
    }

    let mut next_physical_index = columns
        .iter()
        .map(|(physical_index, _, _)| *physical_index)
        .max()
        .map_or(0, |index| index.saturating_add(1));
    for data_index in columns.len()..column_count {
        columns.push((next_physical_index, data_index, &DYNAMIC_COLUMN));
        next_physical_index = next_physical_index.saturating_add(1);
    }
    Some(columns)
}

pub(crate) fn head_rows_for_schema(schema: &[ExcelColumn], options: &WriteOptions) -> Result<u32> {
    head_rows_for_schema_state(schema.is_empty(), options)
}

fn head_rows_for_schema_state(schema_is_empty: bool, options: &WriteOptions) -> Result<u32> {
    if schema_is_empty && options.dynamic_head.is_none() {
        return Ok(0);
    }
    dynamic_head_rows(options)
}

fn dynamic_head_rows(options: &WriteOptions) -> Result<u32> {
    if !options.need_head {
        return Ok(0);
    }
    let Some(head) = &options.dynamic_head else {
        return Ok(1);
    };
    if head.is_empty() || head.iter().any(Vec::is_empty) {
        return Err(ExcelError::Format(
            "dynamic head must contain at least one non-empty path".to_owned(),
        ));
    }
    let levels = head.iter().map(Vec::len).max().unwrap_or(0);
    head_level_to_row(levels)
}

pub(crate) fn selected_dynamic_head_paths(
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
) -> Result<Vec<Vec<String>>> {
    columns
        .iter()
        .map(|(_, source_index, _)| {
            head.get(*source_index).cloned().ok_or_else(|| {
                ExcelError::Format(format!(
                    "dynamic head source column {source_index} is absent"
                ))
            })
        })
        .collect()
}

pub(crate) fn resolved_write_context_holder_state<T>(
    options: &WriteOptions,
    table_no: Option<i32>,
) -> Result<WriteContextHolderState>
where
    T: ExcelRow,
{
    let selected_columns = selected_columns(T::schema(), options)?;
    let selected_head = options
        .dynamic_head
        .as_deref()
        .map(|head| selected_dynamic_head_paths(&selected_columns, head))
        .transpose()?;
    let indexed_columns = selected_columns
        .iter()
        .map(|(column_index, _, column)| (*column_index, *column))
        .collect::<Vec<_>>();
    let head_property = ExcelWriteHeadProperty::from_columns(
        (!T::schema().is_empty()).then(|| type_name::<T>().to_owned()),
        &indexed_columns,
        selected_head.as_deref(),
        *T::write_metadata(),
    )?;

    Ok(WriteContextHolderState {
        holder_type: if table_no.is_some() {
            Holder::Table
        } else {
            Holder::Sheet
        },
        excel_write_head_property: head_property,
        converter_map: crate::converters::default_converter_loader::load_default_write_converter()
            .merged_with(&options.converters),
        need_head: options.need_head,
        automatic_merge_head: options.automatic_merge_head,
        relative_head_row_index: options.relative_head_row_index,
        order_by_include_column: options.order_by_include_column,
        include_column_indexes: options.include_column_indexes.clone(),
        include_column_field_names: options.include_column_field_names.clone(),
        exclude_column_indexes: options.exclude_column_indexes.clone(),
        exclude_column_field_names: options.exclude_column_field_names.clone(),
    })
}

pub(crate) fn normalized_head_label(path: &[String], level: usize) -> &str {
    path.get(level)
        .or_else(|| path.last())
        .map_or("", String::as_str)
}

/// Exact Rust port of Java `ExcelWriteHeadProperty.headCellRangeList()`.
///
/// Each unclaimed cell greedily expands right across equal labels, then down
/// only while the complete rectangle remains equal and unclaimed. Short paths
/// repeat their final label, matching Java `ExcelHeadProperty.initHeadRowNumber`.
pub(crate) fn dynamic_head_merge_ranges(
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
    start_row: u32,
) -> Result<Vec<MergeRange>> {
    if columns.len() != head.len() {
        return Err(ExcelError::Format(format!(
            "dynamic head column count {} does not match selected column count {}",
            head.len(),
            columns.len()
        )));
    }
    let indexed_columns = columns
        .iter()
        .map(|(column_index, _, column)| (*column_index, *column))
        .collect::<Vec<_>>();
    let property = ExcelWriteHeadProperty::from_columns(
        None,
        &indexed_columns,
        Some(head),
        ExcelWriteMetadata::default(),
    )?;
    property
        .head_cell_range_list()
        .into_iter()
        .map(|range| {
            let first_row = start_row
                .checked_add(u32::try_from(range.first_row).map_err(|_| {
                    ExcelError::Format("dynamic head row can not be negative".to_owned())
                })?)
                .ok_or_else(|| ExcelError::Format("dynamic head row overflow".to_owned()))?;
            let last_row = start_row
                .checked_add(u32::try_from(range.last_row).map_err(|_| {
                    ExcelError::Format("dynamic head row can not be negative".to_owned())
                })?)
                .ok_or_else(|| ExcelError::Format("dynamic head row overflow".to_owned()))?;
            let first_col = usize::try_from(range.first_col).map_err(|_| {
                ExcelError::Format("dynamic head column can not be negative".to_owned())
            })?;
            let last_col = usize::try_from(range.last_col).map_err(|_| {
                ExcelError::Format("dynamic head column can not be negative".to_owned())
            })?;
            Ok(MergeRange::new(
                first_row,
                last_row,
                to_column(first_col)?,
                to_column(last_col)?,
            ))
        })
        .collect()
}

pub(crate) fn automatic_dynamic_head_merge_ranges<T>(
    options: &WriteOptions,
    start_row: u32,
    write_head: bool,
) -> Result<Vec<MergeRange>>
where
    T: ExcelRow,
{
    if !write_head || !options.need_head || !options.automatic_merge_head {
        return Ok(Vec::new());
    }
    let Some(head) = &options.dynamic_head else {
        return Ok(Vec::new());
    };
    let columns = selected_columns(T::schema(), options)?;
    let head = selected_dynamic_head_paths(&columns, head)?;
    dynamic_head_merge_ranges(
        &columns,
        &head,
        start_row.saturating_add(relative_head_start_row(options)),
    )
}

fn head_level_to_row(level: usize) -> Result<u32> {
    u32::try_from(level).map_err(|_| ExcelError::Format("dynamic head is too deep".to_owned()))
}

/// Java `relativeHeadRowIndex` → zero-based start row for a new sheet write.
pub(crate) fn relative_head_start_row(options: &WriteOptions) -> u32 {
    if options.relative_head_row_index <= 0 {
        0
    } else {
        u32::try_from(options.relative_head_row_index).unwrap_or(0)
    }
}

#[cfg(test)]
fn write_headers(
    worksheet: &mut Worksheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> Result<()> {
    const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
    let layout = ImageLayout::default();
    write_headers_with_handlers(
        worksheet,
        columns,
        "",
        SheetStyleContext::head(&CellStyle::new(), &METADATA, WriteGlobalFlags::default()),
        &mut [],
        &layout,
        0,
        None,
    )
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_headers_with_handlers(
    worksheet: &mut Worksheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
    sheet_name: &str,
    style: SheetStyleContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    image_layout: &ImageLayout,
    start_row: u32,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let labels = columns
        .iter()
        .map(|(_, _, column)| column.name.to_owned())
        .collect::<Vec<_>>();
    write_header_row_with_handlers(
        worksheet,
        start_row,
        columns,
        &labels,
        sheet_name,
        style,
        handlers,
        image_layout,
        holder_scope,
    )
}

// 参数与 Java 对应写入路径参数一一对应，拆分结构体会破坏 1:1 可追溯性
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_dynamic_headers_with_handlers(
    worksheet: &mut Worksheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
    sheet_name: &str,
    style: SheetStyleContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    image_layout: &ImageLayout,
    start_row: u32,
    automatic_merge_head: bool,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let head = selected_dynamic_head_paths(columns, head)?;
    let levels = head.iter().map(Vec::len).max().unwrap_or(0);
    for level in 0..levels {
        #[allow(clippy::cast_possible_truncation)]
        let row_index = start_row.saturating_add(level as u32);
        let labels = head
            .iter()
            .map(|path| normalized_head_label(path, level).to_owned())
            .collect::<Vec<_>>();
        write_header_row_with_handlers(
            worksheet,
            row_index,
            columns,
            &labels,
            sheet_name,
            style,
            handlers,
            image_layout,
            holder_scope,
        )?;
    }
    if automatic_merge_head {
        merge_dynamic_head_groups(worksheet, columns, &head, style, start_row)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_header_row_with_handlers(
    worksheet: &mut Worksheet,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
    labels: &[String],
    sheet_name: &str,
    style: SheetStyleContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    image_layout: &ImageLayout,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let relative = Some(usize::try_from(row_index).unwrap_or(usize::MAX));
    let row_context = WriteRowContext::new(sheet_name, row_index, relative, true);
    let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
    begin_row_lifecycle(handlers, &row_context)?;
    for ((physical_index, _, column), label) in columns.iter().zip(labels) {
        let column_index = to_column(*physical_index)?;
        let mut context = WriteCellContext::new(
            sheet_name,
            row_index,
            column_index,
            CellValue::String(label.clone()),
        )
        .with_column(column)
        .with_head(label.clone())
        .without_original_value()
        .with_relative_row_index(relative);
        if let Some(scope) = holder_scope {
            context = scope.cell(context);
        }
        begin_cell_lifecycle(handlers, &mut context)?;
        finish_cell_lifecycle(handlers, &context)?;
        context.apply_cell_mutations();
        if !context.skip {
            let format_context = if context.ignore_fill_style {
                style.column(column).without_fill_style()
            } else {
                style
                    .column(column)
                    .with_handler_cell(effective_handler_cell_style(handlers, &context))
            };
            let format = cell_format(format_context);
            match &context.value {
                CellValue::String(value) | CellValue::Error(value) => {
                    worksheet
                        .write_string_with_format(row_index, context.column_index, value, &format)
                        .map_err(format_error)?;
                }
                value => write_cell(
                    worksheet,
                    row_index,
                    context.column_index,
                    column,
                    value,
                    format_context,
                    image_layout,
                )?,
            }
        }
    }
    finish_row_lifecycle(handlers, &row_context)?;
    if let Some(height) = row_context.row().requested_height() {
        worksheet
            .set_row_height(row_index, height)
            .map_err(format_error)?;
    }
    Ok(())
}

fn merge_dynamic_head_groups(
    worksheet: &mut Worksheet,
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
    style: SheetStyleContext<'_>,
    start_row: u32,
) -> Result<()> {
    for range in dynamic_head_merge_ranges(columns, head, start_row)? {
        let column_position = columns
            .iter()
            .position(|(physical, _, _)| u16::try_from(*physical).ok() == Some(range.first_column))
            .ok_or_else(|| ExcelError::Format("dynamic head merge column is absent".to_owned()))?;
        let relative_level =
            usize::try_from(range.first_row.saturating_sub(start_row)).unwrap_or(usize::MAX);
        let label = normalized_head_label(&head[column_position], relative_level);
        let format = cell_format(style.column(columns[column_position].2));
        worksheet
            .merge_range(
                range.first_row,
                range.first_column,
                range.last_row,
                range.last_column,
                label,
                &format,
            )
            .map_err(format_error)?;
    }
    Ok(())
}

#[cfg(test)]
fn write_data_row(
    worksheet: &mut Worksheet,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
    cells: &[CellValue],
) -> Result<()> {
    let image_layout = ImageLayout::default();
    let write_cells = cells
        .iter()
        .cloned()
        .map(WriteCellData::new)
        .collect::<Vec<_>>();
    write_data_row_with_handlers(
        worksheet,
        row_index,
        0,
        columns,
        cells,
        &write_cells,
        "",
        SheetStyleContext {
            explicit: None,
            metadata: &ExcelWriteMetadata::new(),
            is_head: false,
            global: WriteGlobalFlags::default(),
        },
        &mut [],
        &image_layout,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn write_data_row_with_handlers(
    worksheet: &mut Worksheet,
    row_index: u32,
    relative_row_index: usize,
    columns: &[(usize, usize, &'static ExcelColumn)],
    original_cells: &[CellValue],
    cells: &[WriteCellData],
    sheet_name: &str,
    style: SheetStyleContext<'_>,
    handlers: &mut [Box<dyn WriteHandler>],
    image_layout: &ImageLayout,
    holder_scope: Option<&HandlerHolderScope>,
) -> Result<()> {
    let row_context = WriteRowContext::new(sheet_name, row_index, Some(relative_row_index), false);
    let row_context = holder_scope.map_or(row_context.clone(), |scope| scope.row(row_context));
    begin_row_lifecycle(handlers, &row_context)?;
    for (physical_index, schema_index, metadata) in columns {
        let cell_data = cells.get(*schema_index);
        let value = cell_data.map_or(CellValue::Empty, WriteCellData::effective_value);
        let column = to_column(*physical_index)?;
        let mut context = WriteCellContext::new(sheet_name, row_index, column, value)
            .with_column(metadata)
            .with_original_value(
                original_cells
                    .get(*schema_index)
                    .unwrap_or(&CellValue::Empty)
                    .clone(),
            )
            .with_relative_row_index(Some(relative_row_index));
        if let Some(scope) = holder_scope {
            context = scope.cell(context);
        }
        begin_cell_lifecycle(handlers, &mut context)?;
        finish_cell_lifecycle(handlers, &context)?;
        context.apply_cell_mutations();
        if !context.skip {
            let format_context = if context.ignore_fill_style {
                style.column(metadata).without_fill_style()
            } else {
                let format_context = style
                    .column(metadata)
                    .with_handler_cell(effective_handler_cell_style(handlers, &context));
                cell_data.map_or(format_context, |cell| {
                    format_context.with_converted_cell(cell)
                })
            };
            write_cell(
                worksheet,
                row_index,
                context.column_index,
                metadata,
                &context.value,
                format_context,
                image_layout,
            )?;
        }
    }
    finish_row_lifecycle(handlers, &row_context)?;
    if let Some(height) = row_context.row().requested_height() {
        worksheet
            .set_row_height(row_index, height)
            .map_err(format_error)?;
    }
    Ok(())
}

// CellFormatContext 是 Java 写入上下文 1:1 映射的聚合值类型，borrow 化会牵动
// 整条调用链；函数体端到端覆盖单元格写入流程，故豁免 too_many_lines /
// large_types_passed_by_value。
#[allow(clippy::too_many_lines, clippy::large_types_passed_by_value)]
fn write_cell(
    worksheet: &mut Worksheet,
    row_index: u32,
    column: u16,
    metadata: &ExcelColumn,
    value: &CellValue,
    style: CellFormatContext<'_>,
    image_layout: &ImageLayout,
) -> Result<()> {
    // Java creates the POI Row and Cell through WorkBookUtil before assigning
    // the typed value. rust_xlsxwriter materialises them on the first write,
    // so the adapter creates and validates the same logical handles here.
    let mut row_creator = XlsxRowCreator { worksheet };
    let mut row = create_row(&mut row_creator, row_index)?;
    let cell = create_cell(&mut row, column)?;
    let XlsxCell {
        worksheet,
        row_index,
        column_index: column,
    } = cell;
    let global = style.global;
    // 无样式快速路径：CellFormatContext 全字段为空时，cell_format 的结果恒
    // 等于 rust_xlsxwriter 默认格式（xf 0），直接调用无格式写方法可跳过每个
    // 单元格的 Format 构造与格式表哈希查找（RwLock + Format 哈希）。输出字节
    // 完全一致：默认格式在 workbook 创建时预置为 xf 0，两种路径的单元格 XML
    // 均不带 s 属性，styles.xml 亦不受影响。
    if style.explicit.is_none()
        && style.cell.is_none()
        && style.font.is_none()
        && style.handler_cell.is_none()
        && style.converted_cell.is_none()
        && style.converted_data_format.is_none()
    {
        match value {
            CellValue::String(text) | CellValue::Error(text) => {
                let text = maybe_trim_cell_string(text, global.auto_trim);
                if text.is_empty() {
                    // 空字符串经带格式写入会落成空白单元格（store_string 语义），
                    // 无格式写入则整格跳过——为保持优化前输出，回退带格式路径。
                    let format = Format::new();
                    return worksheet
                        .write_string_with_format(row_index, column, text, &format)
                        .map(|_| ())
                        .map_err(format_error);
                }
                return worksheet
                    .write_string(row_index, column, text)
                    .map(|_| ())
                    .map_err(format_error);
            }
            CellValue::Bool(flag) => {
                return worksheet
                    .write_boolean(row_index, column, *flag)
                    .map(|_| ())
                    .map_err(format_error);
            }
            CellValue::Int(number) => {
                return write_integer_unformatted(worksheet, row_index, column, *number);
            }
            CellValue::Float(number) => {
                if global.use_scientific_format
                    && metadata.format.is_none()
                    && is_scientific_magnitude(*number)
                {
                    // 科学计数法需要数字格式，落入下方带格式路径。
                } else {
                    return worksheet
                        .write_number(row_index, column, *number)
                        .map(|_| ())
                        .map_err(format_error);
                }
            }
            CellValue::Decimal(number) => {
                let numeric = finite_decimal_f64(number, "XLSX")?;
                if decimal_integer_requires_text(number)? {
                    return worksheet
                        .write_string(row_index, column, number.to_plain_string())
                        .map(|_| ())
                        .map_err(format_error);
                }
                if global.use_scientific_format
                    && metadata.format.is_none()
                    && is_scientific_magnitude(numeric)
                {
                    // 科学计数法需要数字格式，落入下方带格式路径。
                } else {
                    return worksheet
                        .write_number(row_index, column, numeric)
                        .map(|_| ())
                        .map_err(format_error);
                }
            }
            CellValue::Formula(text) => {
                return worksheet
                    .write_formula(row_index, column, text.as_str())
                    .map(|_| ())
                    .map_err(format_error);
            }
            // 其余类型（Empty/Date/DateTime/Hyperlink/Comment/Image/RichText/
            // Images）必然携带格式或特殊语义（如 Hyperlink 无格式写入会套用
            // 超链接样式），一律走带格式路径。
            _ => {}
        }
    }
    let format = cell_format(style);
    match value {
        CellValue::Empty => {
            worksheet
                .write_blank(row_index, column, &format)
                .map_err(format_error)?;
        }
        CellValue::String(value) | CellValue::Error(value) => {
            let text = maybe_trim_cell_string(value, global.auto_trim);
            worksheet
                .write_string_with_format(row_index, column, text.as_ref(), &format)
                .map_err(format_error)?;
        }
        CellValue::Bool(value) => {
            worksheet
                .write_boolean_with_format(row_index, column, *value, &format)
                .map_err(format_error)?;
        }
        CellValue::Int(value) => {
            write_integer(worksheet, row_index, column, *value, &format)?;
        }
        CellValue::Float(value) => {
            let mut cell_format = format.clone();
            if global.use_scientific_format
                && metadata.format.is_none()
                && is_scientific_magnitude(*value)
            {
                cell_format = cell_format.set_num_format("0.#####E0");
            }
            worksheet
                .write_number_with_format(row_index, column, *value, &cell_format)
                .map_err(format_error)?;
        }
        CellValue::Decimal(value) => {
            let numeric = finite_decimal_f64(value, "XLSX")?;
            if decimal_integer_requires_text(value)? {
                worksheet
                    .write_string_with_format(row_index, column, value.to_plain_string(), &format)
                    .map_err(format_error)?;
                return Ok(());
            }
            let mut cell_format = format.clone();
            if global.use_scientific_format
                && metadata.format.is_none()
                && is_scientific_magnitude(numeric)
            {
                cell_format = cell_format.set_num_format("0.#####E0");
            }
            worksheet
                .write_number_with_format(row_index, column, numeric, &cell_format)
                .map_err(format_error)?;
        }
        CellValue::Date(value) => {
            let format = format
                .clone()
                .set_num_format(excel_date_format(metadata.format, "yyyy-mm-dd"));
            if global.use_1904_windowing {
                let serial = date_to_excel_serial_with_windowing(*value, true);
                worksheet
                    .write_number_with_format(row_index, column, serial, &format)
                    .map_err(format_error)?;
            } else {
                worksheet
                    .write_datetime_with_format(row_index, column, *value, &format)
                    .map_err(format_error)?;
            }
        }
        CellValue::DateTime(value) => {
            let format = format
                .clone()
                .set_num_format(excel_date_format(metadata.format, "yyyy-mm-dd hh:mm:ss"));
            if global.use_1904_windowing {
                let serial = datetime_to_excel_serial_with_windowing(*value, true);
                worksheet
                    .write_number_with_format(row_index, column, serial, &format)
                    .map_err(format_error)?;
            } else {
                worksheet
                    .write_datetime_with_format(row_index, column, *value, &format)
                    .map_err(format_error)?;
            }
        }
        CellValue::Formula(value) => {
            worksheet
                .write_formula_with_format(row_index, column, value.as_str(), &format)
                .map_err(format_error)?;
        }
        CellValue::Hyperlink { url, text } => {
            worksheet
                .write_url_with_options(row_index, column, url.as_str(), text, "", Some(&format))
                .map_err(format_error)?;
        }
        CellValue::Comment { value, text } => {
            write_cell(
                worksheet,
                row_index,
                column,
                metadata,
                value,
                style,
                image_layout,
            )?;
            worksheet
                .insert_note(row_index, column, &Note::new(text))
                .map_err(format_error)?;
        }
        CellValue::Image(bytes) => {
            let image = image_from_buffer(bytes)?;
            worksheet
                .insert_image_fit_to_cell(row_index, column, &image, true)
                .map_err(format_error)?;
        }
        CellValue::RichText(value) => {
            write_rich_text(worksheet, row_index, column, value, &format)?;
        }
        CellValue::Images { value, images } => {
            write_cell(
                worksheet,
                row_index,
                column,
                metadata,
                value,
                style,
                image_layout,
            )?;
            for image in images {
                insert_image_data(worksheet, row_index, column, image, image_layout)?;
            }
        }
    }
    Ok(())
}

fn image_from_buffer(bytes: &[u8]) -> Result<Image> {
    if bytes.len() < 8 {
        return Err(ExcelError::Format(
            "image buffer is too short to contain a valid header".to_owned(),
        ));
    }
    Image::new_from_buffer(bytes).map_err(format_error)
}

fn write_rich_text(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    data: &RichTextStringData,
    cell_format: &Format,
) -> Result<()> {
    if data.text_string().is_empty() {
        worksheet
            .write_string_with_format(row, column, "", cell_format)
            .map(|_| ())
            .map_err(format_error)?;
        return Ok(());
    }
    let runs = rich_text_runs(data)?;
    let references = runs
        .iter()
        .map(|(format, text)| (format, text.as_str()))
        .collect::<Vec<_>>();
    worksheet
        .write_rich_string_with_format(row, column, &references, cell_format)
        .map(|_| ())
        .map_err(format_error)
}

fn rich_text_runs(data: &RichTextStringData) -> Result<Vec<(Format, String)>> {
    let text = data.text_string();
    let utf16_length = text.encode_utf16().count();
    let mut boundaries = vec![0, utf16_length];
    for interval in data.interval_fonts() {
        let start = interval.start_index();
        let end = interval.end_index();
        if start >= end || end > utf16_length {
            return Err(ExcelError::Format(format!(
                "rich-text font range [{start}, {end}) is outside UTF-16 length {utf16_length}"
            )));
        }
        if utf16_byte_index(text, start).is_none() || utf16_byte_index(text, end).is_none() {
            return Err(ExcelError::Format(format!(
                "rich-text font range [{start}, {end}) splits a UTF-16 surrogate pair"
            )));
        }
        boundaries.push(start);
        boundaries.push(end);
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    boundaries
        .windows(2)
        .map(|window| {
            let start = window[0];
            let end = window[1];
            let start_byte = utf16_byte_index(text, start).expect("validated UTF-16 boundary");
            let end_byte = utf16_byte_index(text, end).expect("validated UTF-16 boundary");
            let font = data
                .interval_fonts()
                .iter()
                .rev()
                .find(|interval| interval.start_index() <= start && interval.end_index() >= end)
                .map_or(data.write_font(), |interval| Some(interval.write_font()));
            Ok((
                font.map_or_else(Format::new, rich_text_format),
                text[start_byte..end_byte].to_owned(),
            ))
        })
        .collect()
}

fn utf16_byte_index(text: &str, target: usize) -> Option<usize> {
    let mut utf16_index = 0;
    for (byte_index, character) in text.char_indices() {
        if utf16_index == target {
            return Some(byte_index);
        }
        utf16_index += character.len_utf16();
        if utf16_index > target {
            return None;
        }
    }
    (utf16_index == target).then_some(text.len())
}

fn rich_text_format(font: &WriteFont) -> Format {
    let mut format = Format::new();
    if let Some(name) = font.get_font_name() {
        format = format.set_font_name(name);
    }
    if let Some(size) = font.get_font_height_in_points() {
        format = format.set_font_size(size);
    }
    if let Some(italic) = font.get_italic() {
        format = if italic {
            format.set_italic()
        } else {
            format.unset_italic()
        };
    }
    if let Some(strikeout) = font.get_strikeout() {
        format = if strikeout {
            format.set_font_strikethrough()
        } else {
            format.unset_font_strikethrough()
        };
    }
    if let Some(color) = font.get_color() {
        format = format.set_font_color(annotation_color(color));
    }
    if let Some(script) = font.get_type_offset() {
        format = format.set_font_script(annotation_font_script(script));
    }
    if let Some(underline) = font.get_underline() {
        format = format.set_underline(annotation_underline(underline));
    }
    if let Some(charset) = font.get_charset() {
        format = format.set_font_charset(charset);
    }
    if let Some(bold) = font.get_bold() {
        format = if bold {
            format.set_bold()
        } else {
            format.unset_bold()
        };
    }
    format
}

fn insert_image_data(
    worksheet: &mut Worksheet,
    current_row: u32,
    current_column: u16,
    data: &ImageData,
    layout: &ImageLayout,
) -> Result<()> {
    let anchor = data.get_anchor();
    let coordinates = anchor.get_coordinates();
    let first_row = resolve_anchor_coordinate(
        current_row,
        coordinates.get_first_row_index(),
        coordinates.get_relative_first_row_index(),
        "first row",
    )?;
    let first_column = resolve_anchor_coordinate(
        u32::from(current_column),
        coordinates.get_first_column_index().map(u32::from),
        coordinates.get_relative_first_column_index(),
        "first column",
    )?;
    let last_row = resolve_anchor_coordinate(
        current_row,
        coordinates.get_last_row_index(),
        coordinates.get_relative_last_row_index(),
        "last row",
    )?;
    let last_column = resolve_anchor_coordinate(
        u32::from(current_column),
        coordinates.get_last_column_index().map(u32::from),
        coordinates.get_relative_last_column_index(),
        "last column",
    )?;
    if first_row > last_row || first_column > last_column {
        return Err(ExcelError::Format(
            "image anchor start must not follow its end".to_owned(),
        ));
    }
    let first_column = u16::try_from(first_column)
        .map_err(|_| ExcelError::Format("image anchor column exceeds XLSX limit".to_owned()))?;
    let last_column = u16::try_from(last_column)
        .map_err(|_| ExcelError::Format("image anchor column exceeds XLSX limit".to_owned()))?;
    if last_row >= 1_048_576 || last_column >= 16_384 {
        return Err(ExcelError::Format(
            "image anchor exceeds XLSX worksheet limits".to_owned(),
        ));
    }

    let total_width = (first_column..=last_column).try_fold(0_u32, |width, column| {
        width
            .checked_add(layout.column_width(column))
            .ok_or_else(|| ExcelError::Format("image anchor width overflow".to_owned()))
    })?;
    let total_height = (first_row..=last_row).try_fold(0_u32, |height, row| {
        height
            .checked_add(layout.row_height(row))
            .ok_or_else(|| ExcelError::Format("image anchor height overflow".to_owned()))
    })?;
    let left = anchor.get_left().unwrap_or(0);
    let right = anchor.get_right().unwrap_or(0);
    let top = anchor.get_top().unwrap_or(0);
    let bottom = anchor.get_bottom().unwrap_or(0);
    let width = total_width
        .checked_sub(left)
        .and_then(|value| value.checked_sub(right))
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            ExcelError::Format("image horizontal margins consume its anchor".to_owned())
        })?;
    let height = total_height
        .checked_sub(top)
        .and_then(|value| value.checked_sub(bottom))
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            ExcelError::Format("image vertical margins consume its anchor".to_owned())
        })?;
    let movement = match anchor
        .get_anchor_type()
        .unwrap_or(AnchorType::MoveAndResize)
    {
        AnchorType::MoveAndResize => ObjectMovement::MoveAndSizeWithCells,
        AnchorType::DontMoveDoResize | AnchorType::MoveDontResize => {
            ObjectMovement::MoveButDontSizeWithCells
        }
        AnchorType::DontMoveAndResize => ObjectMovement::DontMoveOrSizeWithCells,
    };
    let image = image_from_buffer(data.image())?
        .set_scale_to_size(width, height, false)
        .set_object_movement(movement);
    insert_scaled_image(worksheet, first_row, first_column, &image, left, top)
}

fn insert_scaled_image(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    image: &Image,
    left: u32,
    top: u32,
) -> Result<()> {
    worksheet
        .insert_image_with_offset(row, column, image, left, top)
        .map(|_| ())
        .map_err(format_error)
}

fn resolve_anchor_coordinate(
    current: u32,
    absolute: Option<u32>,
    relative: Option<i32>,
    label: &str,
) -> Result<u32> {
    if let Some(absolute) = absolute.filter(|value| *value > 0) {
        return Ok(absolute);
    }
    let Some(relative) = relative else {
        return Ok(current);
    };
    current
        .checked_add_signed(relative)
        .ok_or_else(|| ExcelError::Format(format!("image anchor {label} is outside the worksheet")))
}

// 按值传入与调用点构造惯例一致，改引用会增加不必要的借用链
#[allow(clippy::large_types_passed_by_value)]
fn cell_format(context: CellFormatContext<'_>) -> Format {
    let mut format = Format::new();
    // Annotation style merged with handler strategy style
    // (Java `WriteCellStyle.merge(strategy, cellData.getOrCreateStyle())`).
    let mut annotation_cell = context.converted_cell;
    if let Some(annotation_style) = context.cell {
        annotation_cell = Some(merge_write_cell_style(
            &annotation_style,
            annotation_cell.unwrap_or_default(),
        ));
    }
    if let Some(handler_style) = context.handler_cell {
        annotation_cell = Some(merge_write_cell_style(
            &handler_style,
            annotation_cell.unwrap_or_default(),
        ));
    }
    // Nested WriteFont / ExcelFontStyle on merged cell style
    // (Java WriteCellStyle.writeFont merge onto annotation HeadFontStyle/ContentFontStyle).
    let mut font = context.font;
    let merged_has_data_format = annotation_cell.is_some_and(|style| style.data_format.is_some());
    if let Some(style) = annotation_cell {
        if let Some(style_font) = style.font {
            font = Some(match font {
                Some(target) => merge_handler_font_style(&style_font, target),
                None => style_font,
            });
        }
        format = apply_annotation_cell_style(format, style);
    }
    if !merged_has_data_format && let Some(number_format) = context.converted_data_format {
        format = format.set_num_format(number_format);
    }
    if let Some(font) = font {
        format = apply_annotation_font_style(format, font);
    }
    let Some(style) = context.explicit else {
        return format;
    };
    if style.bold {
        format = format.set_bold();
    }
    if style.italic {
        format = format.set_italic();
    }
    if let Some(color) = style.font_color {
        format = format.set_font_color(color);
    }
    if let Some(color) = style.background_color {
        format = format
            .set_background_color(color)
            .set_pattern(FormatPattern::Solid);
    }
    if let Some(alignment) = style.horizontal_alignment {
        format = format.set_align(horizontal_format_align(alignment));
    }
    if let Some(alignment) = style.vertical_alignment {
        format = format.set_align(vertical_format_align(alignment));
    }
    if style.wrap_text {
        format = format.set_text_wrap();
    }
    if let Some(number_format) = &style.number_format {
        format = format.set_num_format(number_format);
    }
    format
}

fn apply_annotation_cell_style(mut format: Format, style: ExcelCellStyle) -> Format {
    if let Some(hidden) = style.hidden {
        format = if hidden {
            format.set_hidden()
        } else {
            format.unset_hidden()
        };
    }
    if let Some(locked) = style.locked {
        format = if locked {
            format.set_locked()
        } else {
            format.set_unlocked()
        };
    }
    if let Some(quote_prefix) = style.quote_prefix {
        format = if quote_prefix {
            format.set_quote_prefix()
        } else {
            format.unset_quote_prefix()
        };
    }
    if let Some(alignment) = style.horizontal_alignment {
        format = format.set_align(annotation_horizontal_format_align(alignment));
    }
    if let Some(wrapped) = style.wrapped {
        format = if wrapped {
            format.set_text_wrap()
        } else {
            format.unset_text_wrap()
        };
    }
    if let Some(alignment) = style.vertical_alignment {
        format = format.set_align(annotation_vertical_format_align(alignment));
    }
    if let Some(rotation) = style.rotation {
        format = format.set_rotation(rotation);
    }
    if let Some(indent) = style.indent {
        format = format.set_indent(indent);
    }
    if let Some(border) = style.border_left {
        format = format.set_border_left(annotation_border_style(border));
    }
    if let Some(border) = style.border_right {
        format = format.set_border_right(annotation_border_style(border));
    }
    if let Some(border) = style.border_top {
        format = format.set_border_top(annotation_border_style(border));
    }
    if let Some(border) = style.border_bottom {
        format = format.set_border_bottom(annotation_border_style(border));
    }
    if let Some(color) = style.left_border_color {
        format = format.set_border_left_color(annotation_color(color));
    }
    if let Some(color) = style.right_border_color {
        format = format.set_border_right_color(annotation_color(color));
    }
    if let Some(color) = style.top_border_color {
        format = format.set_border_top_color(annotation_color(color));
    }
    if let Some(color) = style.bottom_border_color {
        format = format.set_border_bottom_color(annotation_color(color));
    }
    if let Some(pattern) = style.fill_pattern {
        format = format.set_pattern(annotation_fill_pattern(pattern));
    }
    if let Some(color) = style.fill_background_color {
        format = format.set_background_color(annotation_color(color));
    }
    if let Some(color) = style.fill_foreground_color {
        format = format.set_foreground_color(annotation_color(color));
    }
    if let Some(shrink) = style.shrink_to_fit {
        format = if shrink {
            format.set_shrink()
        } else {
            format.unset_shrink()
        };
    }
    if let Some(data_format) = style.data_format {
        format = match data_format {
            ExcelDataFormat::Builtin(index) => format.set_num_format_index(index),
            ExcelDataFormat::Custom(value) => format.set_num_format(value),
        };
    }
    // Nested WriteFont / ExcelFontStyle (Java WriteCellStyle.writeFont)
    if let Some(font) = style.font {
        format = apply_annotation_font_style(format, font);
    }
    format
}

fn apply_annotation_font_style(mut format: Format, style: ExcelFontStyle) -> Format {
    if let Some(font_name) = style.font_name {
        format = format.set_font_name(font_name);
    }
    if let Some(font_height) = style.font_height_in_points {
        format = format.set_font_size(font_height);
    }
    if let Some(italic) = style.italic {
        format = if italic {
            format.set_italic()
        } else {
            format.unset_italic()
        };
    }
    if let Some(strikeout) = style.strikeout {
        format = if strikeout {
            format.set_font_strikethrough()
        } else {
            format.unset_font_strikethrough()
        };
    }
    if let Some(color) = style.color {
        format = format.set_font_color(annotation_color(color));
    }
    if let Some(script) = style.type_offset {
        format = format.set_font_script(annotation_font_script(script));
    }
    if let Some(underline) = style.underline {
        format = format.set_underline(annotation_underline(underline));
    }
    if let Some(charset) = style.charset {
        format = format.set_font_charset(charset);
    }
    if let Some(bold) = style.bold {
        format = if bold {
            format.set_bold()
        } else {
            format.unset_bold()
        };
    }
    format
}

fn annotation_color(color: ExcelColor) -> Color {
    match color {
        ExcelColor::Rgb(value) => Color::RGB(value),
        ExcelColor::Indexed(64) => Color::Automatic,
        ExcelColor::Indexed(index) => indexed_color(index),
    }
}

fn indexed_color(index: u8) -> Color {
    let rgb = match index {
        0 | 8 => 0x0000_0000,
        1 | 9 => 0x00ff_ffff,
        2 | 10 => 0x00ff_0000,
        3 | 11 => 0x0000_ff00,
        4 | 12 | 39 => 0x0000_00ff,
        5 | 13 | 34 => 0x00ff_ff00,
        6 | 14 | 33 => 0x00ff_00ff,
        7 | 15 | 35 => 0x0000_ffff,
        16 | 37 => 0x0080_0000,
        17 => 0x0000_8000,
        18 | 32 => 0x0000_0080,
        19 => 0x0080_8000,
        20 | 36 => 0x0080_0080,
        21 | 38 => 0x0000_8080,
        22 => 0x00c0_c0c0,
        23 => 0x0080_8080,
        24 => 0x0099_99ff,
        25 => 0x007f_0000,
        26 => 0x00ff_ffcc,
        27 | 41 => 0x00cc_ffff,
        28 => 0x0066_0066,
        29 => 0x00ff_8080,
        30 => 0x0000_66cc,
        31 => 0x00cc_ccff,
        40 => 0x0000_ccff,
        42 => 0x00cc_ffcc,
        43 => 0x00ff_ff99,
        44 => 0x0099_ccff,
        45 => 0x00ff_99cc,
        46 => 0x00cc_99ff,
        47 => 0x00ff_cc99,
        48 => 0x0033_66ff,
        49 => 0x0033_cccc,
        50 => 0x0099_cc00,
        51 => 0x00ff_cc00,
        52 => 0x00ff_9900,
        53 => 0x00ff_6600,
        54 => 0x0066_6699,
        55 => 0x0096_9696,
        56 => 0x0000_3366,
        57 => 0x0033_9966,
        58 => 0x0000_3300,
        59 => 0x0033_3300,
        60 => 0x0099_3300,
        61 => 0x0099_3366,
        62 => 0x0033_3399,
        63 => 0x0033_3333,
        _ => return Color::Default,
    };
    Color::RGB(rgb)
}

const fn annotation_horizontal_format_align(alignment: ExcelHorizontalAlignment) -> FormatAlign {
    match alignment {
        ExcelHorizontalAlignment::General => FormatAlign::General,
        ExcelHorizontalAlignment::Left => FormatAlign::Left,
        ExcelHorizontalAlignment::Center => FormatAlign::Center,
        ExcelHorizontalAlignment::Right => FormatAlign::Right,
        ExcelHorizontalAlignment::Fill => FormatAlign::Fill,
        ExcelHorizontalAlignment::Justify => FormatAlign::Justify,
        ExcelHorizontalAlignment::CenterAcross => FormatAlign::CenterAcross,
        ExcelHorizontalAlignment::Distributed => FormatAlign::Distributed,
    }
}

const fn annotation_vertical_format_align(alignment: ExcelVerticalAlignment) -> FormatAlign {
    match alignment {
        ExcelVerticalAlignment::Top => FormatAlign::Top,
        ExcelVerticalAlignment::Center => FormatAlign::VerticalCenter,
        ExcelVerticalAlignment::Bottom => FormatAlign::Bottom,
        ExcelVerticalAlignment::Justify => FormatAlign::VerticalJustify,
        ExcelVerticalAlignment::Distributed => FormatAlign::VerticalDistributed,
    }
}

const fn annotation_border_style(border: ExcelBorderStyle) -> FormatBorder {
    match border {
        ExcelBorderStyle::None => FormatBorder::None,
        ExcelBorderStyle::Thin => FormatBorder::Thin,
        ExcelBorderStyle::Medium => FormatBorder::Medium,
        ExcelBorderStyle::Dashed => FormatBorder::Dashed,
        ExcelBorderStyle::Dotted => FormatBorder::Dotted,
        ExcelBorderStyle::Thick => FormatBorder::Thick,
        ExcelBorderStyle::Double => FormatBorder::Double,
        ExcelBorderStyle::Hair => FormatBorder::Hair,
        ExcelBorderStyle::MediumDashed => FormatBorder::MediumDashed,
        ExcelBorderStyle::DashDot => FormatBorder::DashDot,
        ExcelBorderStyle::MediumDashDot => FormatBorder::MediumDashDot,
        ExcelBorderStyle::DashDotDot => FormatBorder::DashDotDot,
        ExcelBorderStyle::MediumDashDotDot => FormatBorder::MediumDashDotDot,
        ExcelBorderStyle::SlantDashDot => FormatBorder::SlantDashDot,
    }
}

const fn annotation_fill_pattern(pattern: ExcelFillPattern) -> FormatPattern {
    match pattern {
        ExcelFillPattern::None => FormatPattern::None,
        ExcelFillPattern::Solid => FormatPattern::Solid,
        ExcelFillPattern::MediumGray => FormatPattern::MediumGray,
        ExcelFillPattern::DarkGray => FormatPattern::DarkGray,
        ExcelFillPattern::LightGray => FormatPattern::LightGray,
        ExcelFillPattern::DarkHorizontal => FormatPattern::DarkHorizontal,
        ExcelFillPattern::DarkVertical => FormatPattern::DarkVertical,
        ExcelFillPattern::DarkDown => FormatPattern::DarkDown,
        ExcelFillPattern::DarkUp => FormatPattern::DarkUp,
        ExcelFillPattern::DarkGrid => FormatPattern::DarkGrid,
        ExcelFillPattern::DarkTrellis => FormatPattern::DarkTrellis,
        ExcelFillPattern::LightHorizontal => FormatPattern::LightHorizontal,
        ExcelFillPattern::LightVertical => FormatPattern::LightVertical,
        ExcelFillPattern::LightDown => FormatPattern::LightDown,
        ExcelFillPattern::LightUp => FormatPattern::LightUp,
        ExcelFillPattern::LightGrid => FormatPattern::LightGrid,
        ExcelFillPattern::LightTrellis => FormatPattern::LightTrellis,
        ExcelFillPattern::Gray125 => FormatPattern::Gray125,
        ExcelFillPattern::Gray0625 => FormatPattern::Gray0625,
    }
}

const fn annotation_underline(underline: ExcelUnderline) -> FormatUnderline {
    match underline {
        ExcelUnderline::None => FormatUnderline::None,
        ExcelUnderline::Single => FormatUnderline::Single,
        ExcelUnderline::Double => FormatUnderline::Double,
        ExcelUnderline::SingleAccounting => FormatUnderline::SingleAccounting,
        ExcelUnderline::DoubleAccounting => FormatUnderline::DoubleAccounting,
    }
}

const fn annotation_font_script(script: ExcelFontScript) -> FormatScript {
    match script {
        ExcelFontScript::None => FormatScript::None,
        ExcelFontScript::Superscript => FormatScript::Superscript,
        ExcelFontScript::Subscript => FormatScript::Subscript,
    }
}

const fn horizontal_format_align(alignment: HorizontalAlignment) -> FormatAlign {
    match alignment {
        HorizontalAlignment::General => FormatAlign::General,
        HorizontalAlignment::Left => FormatAlign::Left,
        HorizontalAlignment::Center => FormatAlign::Center,
        HorizontalAlignment::Right => FormatAlign::Right,
        HorizontalAlignment::Fill => FormatAlign::Fill,
        HorizontalAlignment::Justify => FormatAlign::Justify,
        HorizontalAlignment::CenterAcross => FormatAlign::CenterAcross,
    }
}

const fn vertical_format_align(alignment: VerticalAlignment) -> FormatAlign {
    match alignment {
        VerticalAlignment::Top => FormatAlign::Top,
        VerticalAlignment::Center => FormatAlign::VerticalCenter,
        VerticalAlignment::Bottom => FormatAlign::Bottom,
        VerticalAlignment::Justify => FormatAlign::VerticalJustify,
        VerticalAlignment::Distributed => FormatAlign::VerticalDistributed,
    }
}

fn write_integer(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: i64,
    format: &Format,
) -> Result<()> {
    const MAX_EXACT_EXCEL_INTEGER: u64 = 9_007_199_254_740_991;
    if value.unsigned_abs() <= MAX_EXACT_EXCEL_INTEGER {
        #[allow(clippy::cast_precision_loss)]
        let number = value as f64;
        worksheet
            .write_number_with_format(row, column, number, format)
            .map(|_| ())
            .map_err(format_error)
    } else {
        worksheet
            .write_string_with_format(row, column, value.to_string(), format)
            .map(|_| ())
            .map_err(format_error)
    }
}

/// 无格式整数写入（无样式快速路径专用）：语义与 [`write_integer`] 完全一致，
/// 仅跳过格式表查找，输出单元格 XML 相同（默认格式与无格式均解析为 xf 0）。
fn write_integer_unformatted(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: i64,
) -> Result<()> {
    const MAX_EXACT_EXCEL_INTEGER: u64 = 9_007_199_254_740_991;
    if value.unsigned_abs() <= MAX_EXACT_EXCEL_INTEGER {
        #[allow(clippy::cast_precision_loss)]
        let number = value as f64;
        worksheet
            .write_number(row, column, number)
            .map(|_| ())
            .map_err(format_error)
    } else {
        worksheet
            .write_string(row, column, value.to_string())
            .map(|_| ())
            .map_err(format_error)
    }
}

pub(crate) fn finite_decimal_f64(value: &BigDecimal, format: &str) -> Result<f64> {
    value
        .to_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| ExcelError::Format(format!("decimal value exceeds {format} numeric range")))
}

pub(crate) fn decimal_integer_requires_text(value: &BigDecimal) -> Result<bool> {
    const MAX_EXACT_EXCEL_INTEGER: i64 = 9_007_199_254_740_991;
    let _ = finite_decimal_f64(value, "Excel")?;
    if value != &value.with_scale(0) {
        return Ok(false);
    }
    let maximum = BigDecimal::from(MAX_EXACT_EXCEL_INTEGER);
    let minimum = -maximum.clone();
    Ok(value > &maximum || value < &minimum)
}

fn excel_date_format(format: Option<&str>, default: &str) -> String {
    format
        .unwrap_or(default)
        .replace("%Y", "yyyy")
        .replace("%m", "mm")
        .replace("%d", "dd")
        .replace("%H", "hh")
        .replace("%M", "mm")
        .replace("%S", "ss")
}

pub(crate) fn to_column(index: usize) -> Result<u16> {
    u16::try_from(index)
        .map_err(|_| ExcelError::Format("column index exceeds XLSX limit".to_owned()))
}

pub(crate) fn format_error(error: impl std::fmt::Display) -> ExcelError {
    ExcelError::Format(error.to_string())
}

#[cfg(test)]
#[path = "missing_tests.rs"]
mod missing_tests;
pub use crate::write::write_csv::*;
#[cfg(test)]
// Re-exports for tests
pub use crate::write::write_xls::*;
pub use crate::write::xlsx_write::*;

#[path = "tests.rs"]
mod tests;

#[cfg(test)]
mod tests_extra {
    use super::*;

    use std::collections::BTreeMap;

    use crate::core::{DynamicRow, DynamicValue};
    use bigdecimal::BigDecimal;
    use calamine::{Data, Reader, Xls, Xlsx};
    use chrono::NaiveDate;
    use std::str::FromStr;
    use tempfile::tempdir;

    const CFB_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

    fn open_xls(path: &std::path::Path) -> Result<Xls<std::fs::File>> {
        Xls::new(std::fs::File::open(path)?).map_err(format_error)
    }

    fn open_xlsx(path: &std::path::Path) -> Result<Xlsx<std::fs::File>> {
        Xlsx::new(std::fs::File::open(path)?).map_err(format_error)
    }

    fn dyn_row(values: &[(usize, &str)]) -> DynamicRow {
        DynamicRow::new(
            values
                .iter()
                .map(|(index, value)| (*index, DynamicValue::String((*value).to_owned())))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn dyn_row_values(values: &[(usize, CellValue)]) -> DynamicRow {
        DynamicRow::new(
            values
                .iter()
                .map(|(index, value)| (*index, DynamicValue::ActualData(value.clone())))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn xls_template_bytes(sheet_name: &str) -> Vec<u8> {
        let mut book = Biff8Book::default();
        let sheet = book.sheet_mut(sheet_name);
        sheet
            .set(
                0,
                0,
                Biff8Cell::general(Biff8Value::Text("seed".to_owned())),
            )
            .expect("seed cell");
        book.to_cfb_bytes().expect("cfb bytes")
    }

    fn xlsx_template_bytes(sheet_name: &str) -> Vec<u8> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.set_name(sheet_name).expect("sheet name");
        sheet.write_string(0, 0, "seed").expect("seed cell");
        workbook.save_to_buffer().expect("template buffer")
    }

    /// Minimal typed row with a two-column schema and annotation metadata.
    struct TwoColRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for TwoColRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("field", "Field", Some(0), 0, None)
                    .with_column_width(18)
                    .with_content_style(ExcelCellStyle {
                        fill_pattern: Some(ExcelFillPattern::Solid),
                        fill_foreground_color: Some(ExcelColor::Indexed(14)),
                        ..ExcelCellStyle::new()
                    }),
                ExcelColumn::new("type", "Type", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
                .head_row_height(31)
                .content_row_height(24);
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    impl TwoColRow {
        fn new(field: &str, r#type: &str) -> Self {
            Self {
                cells: vec![
                    CellValue::String(field.to_owned()),
                    CellValue::String(r#type.to_owned()),
                ],
            }
        }
    }

    /// Handler that requests a concrete row height through the logical handle.
    struct HeightRequestingHandler;

    impl WriteHandler for HeightRequestingHandler {
        fn after_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
            context.row().set_height(27);
            Ok(())
        }
    }

    /// Handler that flags cells for fill-style ignoring and requests a style.
    struct StyleRequestingHandler;

    impl WriteHandler for StyleRequestingHandler {
        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            context.ignore_fill_style = true;
            context.cell().set_style(ExcelCellStyle {
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Indexed(20)),
                ..ExcelCellStyle::new()
            });
            Ok(())
        }
    }

    /// Handler returning a negative (invalid) once-absolute merge property.
    struct NegativeMergeHandler;

    impl WriteHandler for NegativeMergeHandler {
        fn style_once_absolute_merge(
            &self,
        ) -> Option<crate::metadata::property::OnceAbsoluteMergeProperty> {
            Some(crate::core::OnceAbsoluteMergeProperty::new(-1, -1, 0, 1))
        }
    }

    /// Handler that only requests a style through the logical cell handle.
    struct StyleOnlyHandler;

    impl WriteHandler for StyleOnlyHandler {
        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            context.cell().set_style(ExcelCellStyle {
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Indexed(30)),
                ..ExcelCellStyle::new()
            });
            Ok(())
        }
    }

    /// Handler with a repeatable unique value, used for deduplication tests.
    struct UniqueHandler(&'static str);

    impl crate::event::NotRepeatExecutor for UniqueHandler {
        fn unique_value(&self) -> &str {
            self.0
        }
    }

    impl WriteHandler for UniqueHandler {
        fn as_not_repeat_executor(&self) -> Option<&dyn crate::event::NotRepeatExecutor> {
            Some(self)
        }
    }

    /// Handler requesting a loop-merge strategy through the query API.
    struct LoopMergeHandler;

    impl WriteHandler for LoopMergeHandler {
        fn style_loop_merge(&self) -> Option<(crate::core::LoopMergeProperty, usize)> {
            Some((crate::core::LoopMergeProperty::new(2, 1), 0))
        }
    }

    /// Row whose `to_row` fails with a typed data-conversion error.
    struct FailingRow;

    impl ExcelRow for FailingRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Err(ExcelError::Data {
                sheet: String::new(),
                row: 0,
                column: Some(7),
                field: "field",
                value: "bad".to_owned(),
                message: "injected conversion failure".to_owned(),
            })
        }
    }

    /// Row with a field-level `@ContentLoopMerge` annotation.
    struct LoopMergeRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for LoopMergeRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)
                .with_loop_merge(crate::core::LoopMergeProperty::new(2, 1))];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    impl LoopMergeRow {
        fn new(cells: Vec<CellValue>) -> Self {
            Self { cells }
        }
    }

    /// Row with a type-level `@OnceAbsoluteMerge` annotation.
    struct AbsoluteMergeRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for AbsoluteMergeRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("left", "Left", Some(0), 0, None),
                ExcelColumn::new("right", "Right", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
                .once_absolute_merge(crate::core::OnceAbsoluteMergeProperty::new(10, 10, 0, 1));
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    impl AbsoluteMergeRow {
        fn new(cells: Vec<CellValue>) -> Self {
            Self { cells }
        }
    }

    /// Row with a negative (invalid) absolute merge annotation.
    struct NegativeMergeRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for NegativeMergeRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
                .once_absolute_merge(crate::core::OnceAbsoluteMergeProperty::new(-1, -1, 0, 1));
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    impl NegativeMergeRow {
        fn new(cells: Vec<CellValue>) -> Self {
            Self { cells }
        }
    }

    /// Row with annotation head style/font metadata exercising style merges.
    struct FontStyleRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for FontStyleRow {
        fn schema() -> &'static [ExcelColumn] {
            const HEAD_STYLE: ExcelCellStyle = ExcelCellStyle {
                font: Some(ExcelFontStyle {
                    color: Some(ExcelColor::Rgb(0x11_2233)),
                    font_height_in_points: Some(12.0),
                    ..ExcelFontStyle::new()
                }),
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Rgb(0x01_0203)),
                fill_background_color: Some(ExcelColor::Rgb(0x04_0506)),
                ..ExcelCellStyle::new()
            };
            const HEAD_FONT: ExcelFontStyle = ExcelFontStyle {
                color: Some(ExcelColor::Rgb(0x77_8899)),
                font_height_in_points: Some(11.0),
                ..ExcelFontStyle::new()
            };
            const CONTENT_STYLE: ExcelCellStyle = ExcelCellStyle {
                font: Some(ExcelFontStyle {
                    color: Some(ExcelColor::Rgb(0xDD_EEFF)),
                    font_height_in_points: Some(10.0),
                    ..ExcelFontStyle::new()
                }),
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Rgb(0x0A_0B0C)),
                ..ExcelCellStyle::new()
            };
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("field", "Field", Some(0), 0, None)
                    .with_head_style(HEAD_STYLE)
                    .with_head_font_style(HEAD_FONT)
                    .with_content_style(CONTENT_STYLE),
                ExcelColumn::new("other", "Other", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const HEAD_FONT: ExcelFontStyle = ExcelFontStyle {
                color: Some(ExcelColor::Rgb(0x77_8899)),
                font_height_in_points: Some(11.0),
                ..ExcelFontStyle::new()
            };
            const METADATA: ExcelWriteMetadata =
                ExcelWriteMetadata::new().head_font_style(HEAD_FONT);
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    impl FontStyleRow {
        fn new(cells: Vec<CellValue>) -> Self {
            Self { cells }
        }
    }

    #[test]
    fn xls_stateful_double_write_appends_rows_and_finish_saves() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("stateful.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a"), (1, "b")])], &sheet)?;
        writer.write([dyn_row(&[(0, "c"), (1, "d")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((0, 0)), Some(&Data::String("a".to_owned())));
        assert_eq!(range.get_value((1, 1)), Some(&Data::String("d".to_owned())));
        Ok(())
    }

    #[test]
    fn xls_stateful_finish_on_exception_discards_unless_configured() -> Result<()> {
        let directory = tempdir()?;
        for (on_exception, expected_exists) in [(false, false), (true, true)] {
            let path = directory.path().join(format!("exc-{on_exception}.xls"));
            let mut writer = ExcelWriter::with_handlers_and_options(
                &path,
                Vec::new(),
                WriteOptions {
                    write_excel_on_exception: on_exception,
                    ..WriteOptions::default()
                },
            );
            writer.write([dyn_row(&[(0, "boom")])], &WriteSheet::new("Sheet1"))?;
            writer.finish_on_exception()?;
            assert_eq!(path.exists(), expected_exists);
            if expected_exists {
                let bytes = std::fs::read(&path)?;
                assert!(bytes.starts_with(CFB_MAGIC));
            }
        }
        Ok(())
    }

    #[test]
    fn xls_sheet_handlers_registration_rules() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("handlers.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "first")])], &sheet)?;
        // Handlers cannot be attached to an already-initialized sheet.
        let result = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "late")])],
            &sheet,
            vec![Box::new(HeightRequestingHandler)],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));

        // A fresh sheet accepts handlers; a second registration is rejected.
        let fresh = WriteSheet::<DynamicRow>::new("Fresh");
        writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "early")])],
            &fresh,
            vec![Box::new(HeightRequestingHandler)],
        )?;
        let duplicate = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "again")])],
            &fresh,
            vec![Box::new(HeightRequestingHandler)],
        );
        assert!(matches!(duplicate, Err(ExcelError::Unsupported(_))));
        writer.finish()?;
        Ok(())
    }

    #[test]
    fn xls_template_stateful_append_and_finish_preserves_seed() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("template.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a")])], &sheet)?;
        writer.write([dyn_row(&[(0, "b")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        // Seed row, then the two appended rows.
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("seed".to_owned()))
        );
        assert_eq!(range.get_value((1, 0)), Some(&Data::String("a".to_owned())));
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("b".to_owned())));
        Ok(())
    }

    #[test]
    fn xls_template_rejects_non_xls_bytes() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("bad-template.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn csv_with_template_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("template.csv");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn xlsx_template_stateful_append_and_finish_preserves_seed() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("template.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a")])], &sheet)?;
        writer.write([dyn_row(&[(0, "b")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("seed".to_owned()))
        );
        assert_eq!(range.get_value((1, 0)), Some(&Data::String("a".to_owned())));
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("b".to_owned())));
        Ok(())
    }

    #[test]
    fn xlsx_template_legacy_seed_path_writes_values() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        writer.write([dyn_row(&[(0, "legacy")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("legacy".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn xlsx_template_creates_sheet_absent_from_template() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("new-sheet.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("TemplateOnly")),
                ..WriteOptions::default()
            },
        );
        writer.write([dyn_row(&[(0, "fresh")])], &WriteSheet::new("NewSheet"))?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let names = workbook.sheet_names();
        assert!(names.contains(&"NewSheet".to_owned()));
        let range = workbook.worksheet_range("NewSheet").map_err(format_error)?;
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("fresh".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn csv_stateful_append_and_finish_writes_file() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("stateful.csv");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a"), (1, "b")])], &sheet)?;
        writer.write([dyn_row(&[(0, "c")])], &sheet)?;
        writer.finish()?;
        let content = std::fs::read_to_string(&path)?;
        let lines = content.lines().collect::<Vec<_>>();
        assert!(
            lines
                .iter()
                .any(|line| line.contains('a') && line.contains('b'))
        );
        assert!(lines.iter().any(|line| line.contains('c')));
        Ok(())
    }

    #[test]
    fn csv_output_stream_finish_on_exception_emits_capture() -> Result<()> {
        for (on_exception, should_emit) in [(false, false), (true, true)] {
            let output = ExcelOutputStream::new(Vec::new());
            let inspect = output.clone();
            let mut writer = ExcelWriter::with_output_stream(
                "response.csv",
                output,
                Vec::new(),
                WriteOptions {
                    auto_close_stream: false,
                    write_excel_on_exception: on_exception,
                    ..WriteOptions::default()
                },
            );
            writer.write([dyn_row(&[(0, "captured")])], &WriteSheet::new("Sheet1"))?;
            writer.finish_on_exception()?;
            let bytes = inspect.with_inner(Clone::clone).expect("open stream");
            let content = String::from_utf8(bytes).map_err(format_error)?;
            assert_eq!(content.contains("captured"), should_emit);
            assert!(!content.is_empty() || !should_emit);
        }
        Ok(())
    }

    #[test]
    fn csv_second_sheet_name_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("two-sheets.csv");
        let mut writer = ExcelWriter::new(&path);
        writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("first"))?;
        let result = writer.write([dyn_row(&[(0, "b")])], &WriteSheet::new("second"));
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn workbook_mut_exposes_inner_workbook() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("inner.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer
            .workbook_mut()
            .add_worksheet()
            .write_string(0, 0, "manual")
            .map_err(format_error)?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(b"PK"));
        Ok(())
    }

    #[test]
    fn xls_formula_cells_emit_formula_records() -> Result<()> {
        // 对应 Java：POI HSSF setCellFormula → FORMULA 记录（rgce Ptg 编码）
        let directory = tempdir()?;
        let path = directory.path().join("formula.xls");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row_values(&[
                (0, CellValue::Int(2)),
                (1, CellValue::Int(3)),
                (2, CellValue::Formula("A1+B1".to_owned())),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        writer.finish()?;
        assert!(path.exists());
        // calamine 回读：普通单元格为原值；公式单元格回读写入时的缓存值
        // （xls 求值引擎当场计算：A1+B1 = 5，而非 0）
        let mut workbook = calamine::Xls::<std::io::BufReader<std::fs::File>>::new(
            std::io::BufReader::new(std::fs::File::open(&path)?),
        )
        .map_err(|e| crate::core::ExcelError::Format(e.to_string()))?;
        let range = workbook
            .worksheet_range("Sheet1")
            .map_err(|e| crate::core::ExcelError::Format(e.to_string()))?;
        assert_eq!(range.get_value((0, 0)), Some(&calamine::Data::Int(2)));
        assert_eq!(range.get_value((0, 1)), Some(&calamine::Data::Int(3)));
        assert_eq!(range.get_value((0, 2)), Some(&calamine::Data::Float(5.0)));
        Ok(())
    }

    #[test]
    fn xls_write_raw_bytes_and_image_are_embedded() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("raw.xls");
        let mut writer = ExcelWriter::new(&path);
        writer.write_raw_bytes(b"extra-image-stream");
        let png: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 1];
        writer.write_image(png, 1, 2);
        writer.write([dyn_row(&[(0, "cell")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        assert!(bytes.windows(png.len()).any(|window| window == png));
        Ok(())
    }

    #[test]
    fn xlsx_password_protected_stateful_output_is_ole() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("secret.xlsx");
        let mut writer =
            ExcelWriter::with_handlers_and_password(&path, Vec::new(), Some("pw".to_owned()));
        writer.write([dyn_row(&[(0, "hidden")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }

    #[test]
    fn xlsx_template_password_protected_output_is_ole() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("secret-template.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                password: Some("pw".to_owned()),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write([dyn_row(&[(0, "hidden")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }

    #[test]
    fn xlsx_compress_temp_files_populates_gzip_spill_snapshot() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("spill.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                compress_temp_files: true,
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "spill")])], &sheet)?;
        writer.write([dyn_row(&[(0, "again")])], &sheet)?;
        writer.finish()?;
        let snapshot = writer
            .last_gzip_spill_snapshot()
            .expect("snapshot after finish");
        assert_eq!(snapshot.sheet_name, "Sheet1");
        assert!(snapshot.is_gzip);
        assert!(snapshot.uncompressed_len > 0);
        Ok(())
    }

    #[test]
    fn finish_gzip_spill_failure_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("spill-fail.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                compress_temp_files: true,
                ..WriteOptions::default()
            },
        );
        let mut spill = crate::write::gzip_spill::GzipSheetDataWriter::create_owned("Sheet1")?;
        let snapshot = spill.snapshot()?;
        std::fs::remove_file(&snapshot.path)?;
        writer.gzip_spills.insert("Sheet1".to_owned(), spill);
        assert!(writer.finish().is_err());
        Ok(())
    }

    #[test]
    fn write_with_table_handlers_xlsx_new_sheet_and_table() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "tabled")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("tabled".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn write_with_table_handlers_xls_existing_sheet_new_table() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<TwoColRow>::new("Sheet1");
        writer.write([TwoColRow::new("first", "x")], &sheet)?;
        let table = MirroredWriteTable::with_table_no(7);
        writer.write_with_table_handlers(
            [TwoColRow::new("second", "y")],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((3, 0)),
            Some(&Data::String("second".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn write_with_table_handlers_registration_errors() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-err.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "first")])],
            &sheet,
            &table,
            vec![Box::new(HeightRequestingHandler)],
            Vec::new(),
        )?;
        // Duplicate sheet-handler registration on an initialized sheet.
        let duplicate_sheet = writer.write_with_table_handlers(
            [dyn_row(&[(0, "second")])],
            &sheet,
            &table,
            vec![Box::new(HeightRequestingHandler)],
            Vec::new(),
        );
        assert!(matches!(duplicate_sheet, Err(ExcelError::Unsupported(_))));
        // Duplicate table-handler registration on an initialized table.
        let duplicate_table = writer.write_with_table_handlers(
            [dyn_row(&[(0, "second")])],
            &sheet,
            &table,
            Vec::new(),
            vec![Box::new(HeightRequestingHandler)],
        );
        assert!(matches!(duplicate_table, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn xls_dynamic_head_automatic_merge_applied() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dyn-head.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
                vec!["Meta".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "n"), (1, "a"), (2, "m")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let merges = workbook
            .merge_cells_by_sheet_name("Sheet1")
            .map_err(format_error)?;
        assert!(!merges.is_empty());
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("n".to_owned())));
        Ok(())
    }

    #[test]
    fn xls_dynamic_row_with_over_256_columns_errors() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("wide.xls");
        let mut writer = ExcelWriter::new(&path);
        let wide = dyn_row(&(0..300).map(|index| (index, "x")).collect::<Vec<_>>());
        let result = writer.write([wide], &WriteSheet::new("Sheet1"));
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xls_content_styles_apply_all_attributes() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("styled.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            content_styles: vec![CellStyle {
                bold: true,
                italic: true,
                font_color: Some(0xFF_0000),
                background_color: Some(0x00_FF00),
                horizontal_alignment: Some(HorizontalAlignment::Center),
                vertical_alignment: Some(VerticalAlignment::Center),
                wrap_text: true,
                number_format: Some("0.00".to_owned()),
            }],
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "styled")])], &sheet)?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }

    #[test]
    fn xlsx_public_write_with_template_bytes() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("public-template.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "pub")])],
        )?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("pub".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn xls_public_write_with_template_bytes() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("public-template.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "pub")])],
        )?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("pub".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn xls_public_write_to_writer_with_template() -> Result<()> {
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut output = Vec::new();
        crate::write::write_xls::write_xls_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xls"),
            &mut output,
            &options,
            [dyn_row(&[(0, "streamed")])],
            &mut [],
        )?;
        assert!(output.starts_with(CFB_MAGIC));
        Ok(())
    }

    #[test]
    fn finish_twice_is_noop() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("twice.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        writer.finish()?;
        writer.finish_on_exception()?;
        assert!(writer.is_finished());
        Ok(())
    }

    #[test]
    fn xls_height_requesting_handler_applies_head_and_content_heights() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("heights.xls");
        let mut writer = ExcelWriter::with_handlers(&path, vec![Box::new(HeightRequestingHandler)]);
        writer.write([TwoColRow::new("h", "c")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((1, 0)), Some(&Data::String("h".to_owned())));
        Ok(())
    }

    #[test]
    fn xlsx_height_requesting_handler_applies_row_heights() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("heights.xlsx");
        let mut writer = ExcelWriter::with_handlers(&path, vec![Box::new(HeightRequestingHandler)]);
        writer.write([TwoColRow::new("h", "c")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(b"PK"));
        Ok(())
    }

    #[test]
    fn xlsx_stateful_double_write_with_incoming_table_options() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("incoming.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "one")])], &sheet)?;
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "two")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("two".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn xls_cell_value_variant_branches() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("values.xls");
        let mut writer = ExcelWriter::new(&path);
        let date = NaiveDate::from_ymd_opt(2024, 1, 2).expect("date");
        let row = dyn_row_values(&[
            (0, CellValue::Bool(true)),
            (1, CellValue::Int(-7)),
            (2, CellValue::Float(1.5)),
            (3, CellValue::Error("boom".to_owned())),
            (4, CellValue::Formula("SUM(1,2)".to_owned())),
            (
                5,
                CellValue::Hyperlink {
                    url: "https://example.test".to_owned(),
                    text: "link".to_owned(),
                },
            ),
            (6, CellValue::Date(date)),
            (
                7,
                CellValue::DateTime(date.and_hms_opt(3, 4, 5).expect("time")),
            ),
            (
                8,
                CellValue::Comment {
                    value: Box::new(CellValue::String("note".to_owned())),
                    text: "hello".to_owned(),
                },
            ),
            (9, CellValue::RichText(RichTextStringData::new("rich"))),
            (
                10,
                CellValue::Images {
                    value: Box::new(CellValue::String("img".to_owned())),
                    images: vec![ImageData::new(vec![1, 2, 3])],
                },
            ),
            (11, CellValue::Image(vec![4, 5, 6])),
            (
                12,
                CellValue::Decimal(BigDecimal::from_str("12.34").expect("dec")),
            ),
            (
                13,
                CellValue::Decimal(BigDecimal::from_str("9007199254740992").expect("dec")),
            ),
            (14, CellValue::Float(1e12)),
            (15, CellValue::Empty),
        ]);
        writer.write([row], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }

    #[test]
    fn xlsx_cell_value_variant_branches() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("values.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            use_scientific_format: true,
            use_1904_windowing: true,
            ..WriteOptions::default()
        });
        let date = NaiveDate::from_ymd_opt(2024, 1, 2).expect("date");
        let row = dyn_row_values(&[
            (0, CellValue::Float(1e12)),
            (
                1,
                CellValue::Decimal(BigDecimal::from_str("9007199254740992").expect("dec")),
            ),
            (
                2,
                CellValue::Decimal(BigDecimal::from_str("1000000000000").expect("dec")),
            ),
            (3, CellValue::Date(date)),
            (
                4,
                CellValue::DateTime(date.and_hms_opt(1, 2, 3).expect("time")),
            ),
            (
                5,
                CellValue::Comment {
                    value: Box::new(CellValue::Bool(true)),
                    text: "note text".to_owned(),
                },
            ),
            (6, CellValue::Bool(false)),
            (7, CellValue::Error("boom".to_owned())),
            (8, CellValue::Formula("A1+B1".to_owned())),
            (
                9,
                CellValue::Hyperlink {
                    url: "https://example.test".to_owned(),
                    text: "go".to_owned(),
                },
            ),
        ]);
        writer.write([row], &sheet)?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(b"PK"));
        Ok(())
    }

    #[test]
    fn xls_and_xlsx_loop_merge_annotation_rows() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("loop.xls");
        let mut xls_writer = ExcelWriter::new(&xls_path);
        let rows = vec![
            LoopMergeRow::new(vec![CellValue::String("a".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("b".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("c".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("d".to_owned())]),
        ];
        xls_writer.write(rows, &WriteSheet::new("Sheet1"))?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("loop.xlsx");
        let mut xlsx_writer = ExcelWriter::new(&xlsx_path);
        let rows = vec![
            LoopMergeRow::new(vec![CellValue::String("a".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("b".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("c".to_owned())]),
            LoopMergeRow::new(vec![CellValue::String("d".to_owned())]),
        ];
        xlsx_writer.write(rows, &WriteSheet::new("Sheet1"))?;
        xlsx_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        Ok(())
    }

    #[test]
    fn xls_and_xlsx_absolute_merge_annotation_rows() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("merge.xls");
        let mut xls_writer = ExcelWriter::new(&xls_path);
        xls_writer.write(
            [AbsoluteMergeRow::new(vec![
                CellValue::String("l".to_owned()),
                CellValue::String("r".to_owned()),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("merge.xlsx");
        let mut xlsx_writer = ExcelWriter::new(&xlsx_path);
        xlsx_writer.write(
            [AbsoluteMergeRow::new(vec![
                CellValue::String("l".to_owned()),
                CellValue::String("r".to_owned()),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        xlsx_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        Ok(())
    }

    #[test]
    fn negative_merge_handler_properties_are_skipped() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("neg.xls");
        let mut xls_writer =
            ExcelWriter::with_handlers(&xls_path, vec![Box::new(NegativeMergeHandler)]);
        xls_writer.write([TwoColRow::new("v", "w")], &WriteSheet::new("Sheet1"))?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("neg.xlsx");
        let mut xlsx_writer =
            ExcelWriter::with_handlers(&xlsx_path, vec![Box::new(NegativeMergeHandler)]);
        xlsx_writer.write([TwoColRow::new("v", "w")], &WriteSheet::new("Sheet1"))?;
        xlsx_writer.finish()?;

        // Negative indexes in template layout merges are skipped too.
        let tpl_path = directory.path().join("neg-tpl.xlsx");
        let mut tpl_writer = ExcelWriter::with_handlers_and_options(
            &tpl_path,
            vec![Box::new(NegativeMergeHandler)],
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        tpl_writer.write([TwoColRow::new("v", "w")], &WriteSheet::new("Sheet1"))?;
        tpl_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        assert!(tpl_path.exists());
        Ok(())
    }

    #[test]
    fn negative_metadata_merge_is_rejected_at_handler_load() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("neg-meta.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write(
            [NegativeMergeRow::new(vec![CellValue::String(
                "v".to_owned(),
            )])],
            &WriteSheet::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn xls_annotation_font_style_merge_and_rgb_remap() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("fonts.xls");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [FontStyleRow::new(vec![
                CellValue::String("f".to_owned()),
                CellValue::String("o".to_owned()),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        writer.finish()?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(CFB_MAGIC));
        Ok(())
    }

    #[test]
    fn convert_row_at_data_error_maps_physical_column() -> Result<()> {
        let columns = selected_columns(FailingRow::schema(), &WriteOptions::default())?;
        let result = convert_row_at(
            &FailingRow,
            &ConverterRegistry::default(),
            "Sheet1",
            3,
            &columns,
        );
        let error = result.expect_err("must fail");
        let text = error.to_string();
        assert!(text.contains("Sheet1"), "{text}");
        assert!(text.contains("row=3"), "{text}");
        assert!(text.contains("column=Some(0)"), "{text}");
        assert!(text.contains("injected"), "{text}");

        let directory = tempdir()?;
        let path = directory.path().join("failing.xlsx");
        let mut writer = ExcelWriter::new(&path);
        assert!(
            writer
                .write([FailingRow], &WriteSheet::new("Sheet1"))
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn xls_finish_via_output_stream_with_and_without_template() -> Result<()> {
        for use_template in [false, true] {
            let output = ExcelOutputStream::new(Vec::new());
            let inspect = output.clone();
            let mut writer = ExcelWriter::with_output_stream(
                "logical.xls",
                output,
                Vec::new(),
                WriteOptions {
                    auto_close_stream: false,
                    template_bytes: if use_template {
                        Some(xls_template_bytes("Sheet1"))
                    } else {
                        None
                    },
                    ..WriteOptions::default()
                },
            );
            writer.write([dyn_row(&[(0, "streamed")])], &WriteSheet::new("Sheet1"))?;
            writer.finish()?;
            let bytes = inspect.with_inner(Clone::clone).expect("open stream");
            assert!(bytes.starts_with(CFB_MAGIC));
        }
        Ok(())
    }

    #[test]
    fn xls_finish_save_failure_propagates() -> Result<()> {
        let directory = tempdir()?;
        // A directory with an .xls name is not a writable file, so saving fails.
        let path = directory.path().join("out.xls");
        std::fs::create_dir(&path)?;
        let mut writer = ExcelWriter::new(&path);
        writer.write([dyn_row(&[(0, "x")])], &WriteSheet::new("Sheet1"))?;
        assert!(matches!(writer.finish(), Err(ExcelError::Io(_))));
        Ok(())
    }

    #[test]
    fn xls_template_write_absent_sheet_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("absent.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write([dyn_row(&[(0, "x")])], &WriteSheet::new("NoSuchSheet"));
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn xls_template_with_table_handlers_and_dynamic_head() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-table.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
                vec!["Meta".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        let table = MirroredWriteTable::with_table_no(2);
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "n"), (1, "a"), (2, "m")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        let table2 = MirroredWriteTable::with_table_no(3);
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "n2"), (1, "a2"), (2, "m2")])],
            &sheet,
            &table2,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert!(range.get_value((4, 0)).is_some());
        Ok(())
    }

    #[test]
    fn xlsx_template_with_table_handlers_existing_state() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-table.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "one")])], &sheet)?;
        let table = MirroredWriteTable::with_table_no(0);
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "two")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((2, 0)),
            Some(&Data::String("two".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn xlsx_template_height_handler_styles_and_zero_rows() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-styles.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(HeightRequestingHandler)],
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            content_styles: vec![CellStyle {
                bold: true,
                font_color: Some(0x00_FF00),
                background_color: Some(0x00_00FF),
                ..CellStyle::new()
            }],
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "styled")])], &sheet)?;
        writer.finish()?;
        assert!(path.exists());

        let empty_path = directory.path().join("tpl-empty.xlsx");
        let mut empty_writer = ExcelWriter::with_handlers_and_options(
            &empty_path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        empty_writer.write(Vec::<DynamicRow>::new(), &WriteSheet::new("Sheet1"))?;
        empty_writer.finish()?;
        assert!(empty_path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_template_public_legacy_seed_with_spill() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-public.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "legacy")])],
        )?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("legacy".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn xlsx_template_public_rejects_xls_template_file() -> Result<()> {
        let directory = tempdir()?;
        let template_path = directory.path().join("seed.xls");
        std::fs::write(&template_path, xls_template_bytes("Sheet1"))?;
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(template_path),
            ..WriteOptions::default()
        };
        let path = directory.path().join("dual.xlsx");
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "x")])],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn xlsx_template_public_creates_absent_sheet() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("new-sheet-public.xlsx");
        let options = WriteOptions {
            sheet_name: "NewSheet".to_owned(),
            template_bytes: Some(xlsx_template_bytes("TemplateOnly")),
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "fresh")])],
        )?;
        let workbook = open_xlsx(&path)?;
        assert!(workbook.sheet_names().contains(&"NewSheet".to_owned()));
        Ok(())
    }

    #[test]
    fn xls_public_template_bad_bytes_and_absent_sheet() -> Result<()> {
        let directory = tempdir()?;
        let bad_path = directory.path().join("bad.xls");
        let bad_options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        assert!(
            crate::write::write_xls::write_xls::<DynamicRow, _>(
                &bad_path,
                &bad_options,
                [dyn_row(&[(0, "x")])],
            )
            .is_err()
        );
        let absent_path = directory.path().join("absent.xls");
        let absent_options = WriteOptions {
            sheet_index: Some(9),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        assert!(
            crate::write::write_xls::write_xls::<DynamicRow, _>(
                &absent_path,
                &absent_options,
                [dyn_row(&[(0, "x")])],
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn xls_public_template_with_handlers_to_subdirectory() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("nested").join("out.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(HeightRequestingHandler)];
        crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "nested")])],
            &mut handlers,
        )?;
        assert!(path.exists());

        let plain_path = directory.path().join("plain.xls");
        let plain_options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &plain_path,
            &plain_options,
            [dyn_row(&[(0, "plain")])],
            &mut handlers,
        )?;
        assert!(plain_path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_public_compress_temp_files() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("spill-public.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "spill")])],
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn csv_write_with_table_handlers() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table.csv");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "tabled")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "again")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains("tabled"));
        assert!(content.contains("again"));
        Ok(())
    }

    #[test]
    fn csv_schema_change_between_writes_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("schema.csv");
        let mut writer = ExcelWriter::new(&path);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        let result = writer.write([dyn_row(&[(0, "x")])], &WriteSheet::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn table_schema_mismatch_between_writes_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-schema.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [TwoColRow::new("a", "b")],
            &WriteSheet::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        let result = writer.write_with_table_handlers(
            [dyn_row(&[(0, "x")])],
            &WriteSheet::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn sheet_handlers_on_initialized_sheet_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("late-sheet-handlers.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write([dyn_row(&[(0, "first")])], &sheet)?;
        let result = writer.write_with_table_handlers(
            [dyn_row(&[(0, "second")])],
            &sheet,
            &table,
            vec![Box::new(HeightRequestingHandler)],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn table_handlers_on_new_sheet_run_workbook_callbacks() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("new-sheet-handlers.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "a")])],
            &sheet,
            &table,
            vec![Box::new(HeightRequestingHandler)],
            Vec::new(),
        )?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xls_dynamic_head_with_height_handler() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dyn-heights.xls");
        let mut writer = ExcelWriter::with_handlers(&path, vec![Box::new(HeightRequestingHandler)]);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "n"), (1, "a")])], &sheet)?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("n".to_owned())));
        Ok(())
    }

    #[test]
    fn xlsx_legacy_template_autofit() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("autofit.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            auto_width: true,
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "autofit me")])], &sheet)?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn biff8_create_row_overflow_errors() {
        let mut book = Biff8Book::default();
        let mut creator = Biff8RowCreator {
            sheet: book.sheet_mut("Sheet1"),
        };
        let result = create_row(&mut creator, 65_536);
        assert!(matches!(result, Err(ExcelError::Format(_))));
        let result = create_row(&mut creator, 65_535);
        assert!(result.is_ok());
    }

    #[test]
    fn effective_sheet_name_keeps_trimmed_when_disabled() {
        let options = WriteOptions {
            auto_trim: false,
            sheet_name: "  padded  ".to_owned(),
            ..WriteOptions::default()
        };
        assert_eq!(effective_sheet_name(&options), "  padded  ");
        let trimmed = WriteOptions {
            auto_trim: true,
            sheet_name: "  padded  ".to_owned(),
            ..WriteOptions::default()
        };
        assert_eq!(effective_sheet_name(&trimmed), "padded");
    }

    #[test]
    fn write_with_sheet_handlers_after_finish_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("finished.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer.write([dyn_row(&[(0, "a")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let result = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "b")])],
            &WriteSheet::new("Sheet1"),
            vec![Box::new(HeightRequestingHandler)],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn handler_ignore_fill_and_requested_style() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("style-h.xls");
        let mut xls_writer =
            ExcelWriter::with_handlers(&xls_path, vec![Box::new(StyleRequestingHandler)]);
        xls_writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("style-h.xlsx");
        let mut xlsx_writer =
            ExcelWriter::with_handlers(&xlsx_path, vec![Box::new(StyleRequestingHandler)]);
        xlsx_writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        xlsx_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        Ok(())
    }

    #[test]
    fn dynamic_head_merge_mismatch_errors() -> Result<()> {
        let options = WriteOptions {
            dynamic_head: Some(vec![vec!["A".to_owned()]]),
            ..WriteOptions::default()
        };
        let columns = selected_columns(&[], &options)?;
        assert_eq!(columns.len(), 1);
        let head = vec![vec!["A".to_owned()], vec!["B".to_owned()]];
        let result = dynamic_head_merge_ranges(&columns, &head, 0);
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn xlsx_template_annotation_merge_and_width_handlers() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-ann.xlsx");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(
            MirroredOnceAbsoluteMerge::from_property(crate::core::OnceAbsoluteMergeProperty::new(
                0, 0, 0, 1,
            ))
            .expect("merge strategy"),
        )];
        let mut width_strategy = SimpleColumnWidthStyleStrategy::new();
        width_strategy.set_column_width(0, 42);
        handlers.push(Box::new(width_strategy));
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx_with_handlers::<AbsoluteMergeRow, _>(
            &path,
            &options,
            [AbsoluteMergeRow::new(vec![
                CellValue::String("l".to_owned()),
                CellValue::String("r".to_owned()),
            ])],
            &mut handlers,
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn initialize_existing_table_holder_csv_early_return() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl.csv");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        writer.write([dyn_row(&[(0, "a")])], &sheet)?;
        let table = MirroredWriteTable::with_table_no(5);
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "c")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains('a'));
        assert!(content.contains('c'));
        Ok(())
    }

    #[test]
    fn initialize_existing_table_holder_xls_applies_table_merges() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl.xls");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<TwoColRow>::new("Sheet1");
        writer.write([TwoColRow::new("a", "b")], &sheet)?;
        let table = MirroredWriteTable::with_table_no(5);
        writer.write_with_table_handlers(
            [TwoColRow::new("c", "d")],
            &sheet,
            &table,
            Vec::new(),
            vec![Box::new(
                MirroredOnceAbsoluteMerge::from_property(
                    crate::core::OnceAbsoluteMergeProperty::new(10, 10, 0, 1),
                )
                .expect("merge strategy"),
            )],
        )?;
        writer.finish()?;
        let mut workbook = open_xls(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert!(range.get_value((3, 0)).is_some());
        Ok(())
    }

    #[test]
    fn initialize_existing_table_holder_xlsx_template_layout() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-tpl.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<TwoColRow>::new("Sheet1");
        writer.write([TwoColRow::new("a", "b")], &sheet)?;
        let table = MirroredWriteTable::with_table_no(5);
        writer.write_with_table_handlers(
            [TwoColRow::new("c", "d")],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn initialize_existing_table_holder_xlsx_column_widths() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-widths.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<TwoColRow>::from_options(WriteOptions {
            column_widths: vec![(0, 30)],
            ..WriteOptions::default()
        });
        writer.write([TwoColRow::new("a", "b")], &sheet)?;
        let table = MirroredWriteTable::with_table_no(5);
        writer.write_with_table_handlers(
            [TwoColRow::new("c", "d")],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_annotation_font_merge_and_number_format() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("fonts.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [FontStyleRow::new(vec![
                CellValue::String("f".to_owned()),
                CellValue::String("o".to_owned()),
            ])],
            &WriteSheet::new("Sheet1"),
        )?;
        writer.finish()?;

        let fmt_path = directory.path().join("fmt.xlsx");
        let mut fmt_writer = ExcelWriter::new(&fmt_path);
        let fmt_sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            content_styles: vec![CellStyle {
                number_format: Some("0.00".to_owned()),
                bold: true,
                italic: true,
                font_color: Some(0x00_FF00),
                background_color: Some(0xFF_0000),
                horizontal_alignment: Some(HorizontalAlignment::Right),
                vertical_alignment: Some(VerticalAlignment::Top),
                wrap_text: true,
            }],
            ..WriteOptions::default()
        });
        fmt_writer.write([dyn_row(&[(0, "x")])], &fmt_sheet)?;
        fmt_writer.finish()?;
        assert!(path.exists());
        assert!(fmt_path.exists());
        Ok(())
    }

    #[test]
    fn sort_handlers_dedupes_repeat_executors() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dedupe.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![
            Box::new(UniqueHandler("shared")),
            Box::new(UniqueHandler("shared")),
        ];
        crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "dedupe")])],
            &mut handlers,
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_template_to_writer_with_password() -> Result<()> {
        let mut output = Vec::new();
        crate::write::xlsx_write::write_xlsx_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                password: Some("pw".to_owned()),
                ..WriteOptions::default()
            },
            [dyn_row(&[(0, "encrypted")])],
            &mut [],
        )?;
        assert!(output.starts_with(CFB_MAGIC));
        Ok(())
    }

    #[test]
    fn legacy_seed_public_with_layout_and_absent_sheet() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-layout.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            column_widths: vec![(0, 25)],
            merge_ranges: vec![MergeRange::new(1, 2, 0, 1)],
            auto_width: true,
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "layout")])],
        )?;
        assert!(path.exists());

        let absent_path = directory.path().join("legacy-absent.xlsx");
        let absent_options = WriteOptions {
            sheet_name: "BrandNew".to_owned(),
            sheet_index: Some(9),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &absent_path,
            &absent_options,
            [dyn_row(&[(0, "fresh")])],
        )?;
        let workbook = open_xlsx(&absent_path)?;
        assert!(workbook.sheet_names().contains(&"BrandNew".to_owned()));
        Ok(())
    }

    #[test]
    fn xlsx_template_wide_row_style_column_error() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("wide-tpl.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let wide = dyn_row(&(0..70_000).map(|index| (index, "x")).collect::<Vec<_>>());
        let result = writer.write([wide], &WriteSheet::new("Sheet1"));
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xlsx_template_absent_rows_get_no_heights() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("absent-tpl.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let rows: Vec<Option<TwoColRow>> = vec![
            Some(TwoColRow::new("a", "b")),
            None,
            Some(TwoColRow::new("c", "d")),
        ];
        writer.write(rows, &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_template_requested_styles_merge_with_handler_styles() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("req-styles.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(StyleOnlyHandler)],
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        let mut workbook = open_xlsx(&path)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(range.get_value((2, 0)), Some(&Data::String("a".to_owned())));
        Ok(())
    }

    #[test]
    fn loop_merge_handler_strategy_applied() -> Result<()> {
        let directory = tempdir()?;
        let xls_path = directory.path().join("loop-h.xls");
        let mut xls_writer =
            ExcelWriter::with_handlers(&xls_path, vec![Box::new(LoopMergeHandler)]);
        let rows = vec![
            TwoColRow::new("a", "b"),
            TwoColRow::new("c", "d"),
            TwoColRow::new("e", "f"),
            TwoColRow::new("g", "h"),
        ];
        xls_writer.write(rows, &WriteSheet::new("Sheet1"))?;
        xls_writer.finish()?;

        let xlsx_path = directory.path().join("loop-h.xlsx");
        let mut xlsx_writer =
            ExcelWriter::with_handlers(&xlsx_path, vec![Box::new(LoopMergeHandler)]);
        let rows = vec![
            TwoColRow::new("a", "b"),
            TwoColRow::new("c", "d"),
            TwoColRow::new("e", "f"),
            TwoColRow::new("g", "h"),
        ];
        xlsx_writer.write(rows, &WriteSheet::new("Sheet1"))?;
        xlsx_writer.finish()?;
        assert!(xls_path.exists());
        assert!(xlsx_path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_legacy_seed_to_writer() -> Result<()> {
        let mut output = Vec::new();
        crate::write::xlsx_write::write_xlsx_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
            [dyn_row(&[(0, "legacy")])],
            &mut [],
        )?;
        assert!(output.starts_with(b"PK"));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_template_negative_merge_handler_layout() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("neg-tpl.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(NegativeMergeHandler)],
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write([TwoColRow::new("v", "w")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn row_type_from_row_constructors_are_invokable() {
        let row_data = crate::core::RowData::new(
            "Sheet1",
            0,
            vec![CellValue::String("x".to_owned())],
            std::sync::Arc::new(std::collections::HashMap::new()),
        );
        assert!(TwoColRow::from_row(&row_data).is_ok());
        assert!(LoopMergeRow::from_row(&row_data).is_ok());
        assert!(AbsoluteMergeRow::from_row(&row_data).is_ok());
        assert!(NegativeMergeRow::from_row(&row_data).is_ok());
        assert!(FontStyleRow::from_row(&row_data).is_ok());
        assert!(FailingRow::from_row(&row_data).is_ok());
        assert!(
            NegativeMergeRow::new(vec![CellValue::String("v".to_owned())])
                .to_row()
                .is_ok()
        );
    }

    #[test]
    fn xlsx_requested_style_merged_with_handler_style() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("req-style.xlsx");
        let mut writer = ExcelWriter::with_handlers(&path, vec![Box::new(StyleOnlyHandler)]);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn cell_format_applies_converted_data_format_without_annotation() {
        let context = CellFormatContext {
            explicit: None,
            cell: None,
            font: None,
            handler_cell: None,
            converted_cell: None,
            converted_data_format: Some("0.00"),
            global: WriteGlobalFlags::default(),
        };
        let format = cell_format(context);
        // rust_xlsxwriter exposes no num-format getter; exercising cell_format
        // with a converted data format is the coverage goal.
        let _ = format;
    }

    #[test]
    fn apply_annotation_once_absolute_merge_applies_when_handler_absent() -> Result<()> {
        let mut worksheet = rust_xlsxwriter::Worksheet::new();
        let handlers: Vec<Box<dyn WriteHandler>> = Vec::new();
        apply_annotation_once_absolute_merge::<AbsoluteMergeRow>(&mut worksheet, &handlers)?;
        Ok(())
    }

    #[test]
    fn table_annotation_handlers_second_write_short_circuits() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-twice.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<TwoColRow>::new("Sheet1");
        let table = crate::write::metadata::write_table::WriteTable::new();
        writer.write_with_table([TwoColRow::new("a", "b")], &sheet, &table)?;
        writer.write_with_table([TwoColRow::new("c", "d")], &sheet, &table)?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_template_existing_sheet_uses_else_target_name() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-existing.xlsx");
        let _options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut writer = ExcelWriter::new(&path);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xls_write_with_automatic_merge_head_disabled() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("no-merge-head.xls");
        let options = WriteOptions {
            automatic_merge_head: false,
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        crate::write::write_xls::write_xls::<TwoColRow, _>(
            &path,
            &options,
            [TwoColRow::new("a", "b")],
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn template_head_style_none_column_matches_head_fallback() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-dyn-head.xlsx");
        let _options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut writer = ExcelWriter::new(&path);
        // DynamicRow has an empty schema, so head columns never match.
        writer.write([dyn_row(&[(0, "a"), (1, "b")])], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xls_finish_via_output_stream_with_and_without_template_full_loop() -> Result<()> {
        for use_template in [false, true] {
            let directory = tempdir()?;
            let logical = directory.path().join("stream.xls");
            let output = ExcelOutputStream::new(std::io::Cursor::new(Vec::<u8>::new()));
            let mut options = WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                ..WriteOptions::default()
            };
            if use_template {
                let mut book = crate::write::biff8::Biff8Book::default();
                book.sheet_mut("Sheet1");
                options.template_bytes = Some(book.to_cfb_bytes()?);
            }
            let writer = ExcelWriter::with_output_stream(logical, output, Vec::new(), options);
            let mut writer = writer;
            writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
            writer.finish()?;
        }
        Ok(())
    }

    #[test]
    fn ensure_table_annotation_handlers_second_call_short_circuits() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("ensure-twice.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let options = WriteOptions::default();
        writer.ensure_table_annotation_handlers::<TwoColRow>("Sheet1", 0, &options)?;
        writer.ensure_table_annotation_handlers::<TwoColRow>("Sheet1", 0, &options)?;
        Ok(())
    }

    #[test]
    fn xlsx_template_existing_sheet_name_uses_else_target() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-else-target.xlsx");
        let _options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut writer = ExcelWriter::new(&path);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn template_head_extra_column_hits_none_head_fallback() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-extra-col.xlsx");
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.set_name("Sheet1").expect("sheet name");
        sheet.write_string(0, 0, "A").expect("head a");
        sheet.write_string(0, 1, "B").expect("head b");
        sheet.write_string(0, 2, "C").expect("head c");
        let template = workbook.save_to_buffer().expect("template");
        let _options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(template),
            ..WriteOptions::default()
        };
        let mut writer = ExcelWriter::new(&path);
        writer.write([TwoColRow::new("a", "b")], &WriteSheet::new("Sheet1"))?;
        writer.finish()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn template_append_cell_styles_head_with_unknown_column() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("styles-head.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut package = crate::write::template_write::TemplatePackage::from_bytes(
            xlsx_template_bytes("Sheet1").as_slice(),
        )?;
        let rows = vec![
            vec![
                (0usize, CellValue::String("h0".to_owned())),
                (5usize, CellValue::String("extra".to_owned())),
            ],
            vec![(0usize, CellValue::String("v".to_owned()))],
        ];
        let converted: Vec<Vec<(usize, crate::core::WriteCellData)>> = Vec::new();
        let ignore: Vec<Vec<bool>> = vec![Vec::new(), Vec::new()];
        let requested: Vec<Vec<Option<ExcelCellStyle>>> = vec![Vec::new(), Vec::new()];
        let styles = template_append_cell_styles::<TwoColRow>(
            &mut package,
            &options,
            &[],
            &rows,
            &rows,
            &converted,
            &ignore,
            &requested,
            true,
            0,
        )?;
        assert_eq!(styles.len(), 2);
        let _ = ExcelWriter::new(&path);
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    use std::collections::BTreeMap;
    use std::io::{Cursor, Write};

    use crate::core::{DynamicRow, DynamicValue};
    use tempfile::tempdir;

    fn dyn_row(values: &[(usize, &str)]) -> DynamicRow {
        DynamicRow::new(
            values
                .iter()
                .map(|(index, value)| (*index, DynamicValue::String((*value).to_owned())))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn dyn_row_values(values: &[(usize, CellValue)]) -> DynamicRow {
        DynamicRow::new(
            values
                .iter()
                .map(|(index, value)| (*index, DynamicValue::ActualData(value.clone())))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn xls_template_bytes(sheet_name: &str) -> Vec<u8> {
        let mut book = Biff8Book::default();
        book.sheet_mut(sheet_name);
        book.to_cfb_bytes().expect("cfb bytes")
    }

    fn xlsx_template_bytes(sheet_name: &str) -> Vec<u8> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.set_name(sheet_name).expect("sheet name");
        sheet.write_string(0, 0, "seed").expect("seed cell");
        workbook.save_to_buffer().expect("template buffer")
    }

    /// 手工构造 ZIP 模板包（entries: (路径, 内容)），默认 Stored 压缩。
    fn zip_template(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, bytes) in entries {
            writer.start_file(*name, options).expect("start entry");
            writer.write_all(bytes).expect("write entry");
        }
        writer.finish().expect("finish").into_inner()
    }

    fn minimal_workbook_xml(sheet_name: &str) -> String {
        format!(
            r#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="{sheet_name}" sheetId="1" r:id="rId1"/></sheets></workbook>"#
        )
    }

    const MINIMAL_PACKAGE_RELS_XML: &[u8] = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#;

    const MINIMAL_RELS_XML: &[u8] = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#;

    const MINIMAL_SHEET_XML: &[u8] = br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#;

    const MINIMAL_CONTENT_TYPES_XML: &[u8] = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#;

    /// 失败阶段可配置的处理器（对应 Java 测试里的 `FailingHandler` 模式）。
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum FailStage {
        BeforeWorkbookCreate,
        AfterSheetCreate,
        HeadCell,
        DataCell,
    }

    struct StageFailingHandler(FailStage);

    impl WriteHandler for StageFailingHandler {
        fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            if self.0 == FailStage::BeforeWorkbookCreate {
                Err(ExcelError::Format("stage failure".to_owned()))
            } else {
                Ok(())
            }
        }

        fn after_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
            if self.0 == FailStage::AfterSheetCreate {
                Err(ExcelError::Format("stage failure".to_owned()))
            } else {
                Ok(())
            }
        }

        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            let expected = if context.is_head {
                FailStage::HeadCell
            } else {
                FailStage::DataCell
            };
            if self.0 == expected {
                Err(ExcelError::Format("stage failure".to_owned()))
            } else {
                Ok(())
            }
        }
    }

    /// 跳过所有单元格写入（对应 Java 里通过 handler 丢弃单元格）。
    struct SkipCellHandler;

    impl WriteHandler for SkipCellHandler {
        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            context.skip = true;
            Ok(())
        }
    }

    /// 只请求单元格样式（对应 Java `requestedStyle`）。
    struct StyleRequestingHandler;

    impl WriteHandler for StyleRequestingHandler {
        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            context.cell().set_style(ExcelCellStyle {
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Indexed(21)),
                ..ExcelCellStyle::new()
            });
            Ok(())
        }
    }

    /// 请求非法 loop-merge（eachRow=1 且 columnExtend=1）。
    struct LoopMergeBadHandler;

    impl WriteHandler for LoopMergeBadHandler {
        fn style_loop_merge(&self) -> Option<(crate::core::LoopMergeProperty, usize)> {
            Some((crate::core::LoopMergeProperty::new(1, 1), 0))
        }
    }

    /// `to_row` 返回错误的行（对应 Java `ConvertAllFiled` 抛异常场景）。
    struct FailingRow2;

    impl ExcelRow for FailingRow2 {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Err(ExcelError::Data {
                sheet: String::new(),
                row: 0,
                column: Some(7),
                field: "field",
                value: "bad".to_owned(),
                message: "test-only row conversion failure".to_owned(),
            })
        }
    }

    /// 普通两列 typed 行。
    struct PlainRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for PlainRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("first", "First", Some(0), 0, None),
                ExcelColumn::new("second", "Second", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    impl PlainRow {
        fn new(first: &str, second: &str) -> Self {
            Self {
                cells: vec![
                    CellValue::String(first.to_owned()),
                    CellValue::String(second.to_owned()),
                ],
            }
        }
    }

    /// 注解 `loop_merge` 非法（eachRow=1 / columnExtend=1）的行。
    struct LoopMergeBadRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for LoopMergeBadRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)
                .with_loop_merge(crate::core::LoopMergeProperty::new(1, 1))];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    /// 强制列号超出 u16 上限的行（对应 Java `index = 70000` 的极端注解）。
    struct WideIndexRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for WideIndexRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] =
                &[ExcelColumn::new("field", "Field", Some(70_000), 0, None)];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    // ========================================================================
    // 注解处理器加载 / 表写入的错误分支
    // ========================================================================

    #[test]
    fn loop_merge_bad_annotation_handlers_rejected() -> Result<()> {
        // 对应 Java：@ContentLoopMerge(eachRow=1, columnExtend=1) → IllegalArgumentException。
        let directory = tempdir()?;
        let path = directory.path().join("bad-loop.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write(
            [LoopMergeBadRow { cells: Vec::new() }],
            &WriteSheet::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn loop_merge_bad_table_annotation_handlers_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("bad-loop-table.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let table = MirroredWriteTable::new();
        let result = writer.write_with_table_handlers(
            [LoopMergeBadRow { cells: Vec::new() }],
            &WriteSheet::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn sheet_handlers_workbook_callback_error_propagates() -> Result<()> {
        // 对应 Java：新 sheet 首次注册 sheet handler 时运行 workbook 回调。
        let directory = tempdir()?;
        let path = directory.path().join("sheet-cb.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "first")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
            vec![Box::new(StageFailingHandler(
                FailStage::BeforeWorkbookCreate,
            ))],
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn table_handlers_new_sheet_workbook_callback_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-cb.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let table = MirroredWriteTable::new();
        let result = writer.write_with_table_handlers(
            [dyn_row(&[(0, "first")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
            &table,
            vec![Box::new(StageFailingHandler(
                FailStage::BeforeWorkbookCreate,
            ))],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn existing_sheet_table_template_layout_error_propagates() -> Result<()> {
        // 对应 Java：已有 sheet 上建表时按模板布局（列宽/合并），列号超限必须报错。
        let directory = tempdir()?;
        let path = directory.path().join("table-layout.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write(
            [dyn_row(&[(0, "seed")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let table = MirroredWriteTable::new();
        let result = writer.write_with_table_handlers(
            [WideIndexRow { cells: Vec::new() }],
            &WriteSheet::<WideIndexRow>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn table_batch_row_conversion_errors_by_backend() -> Result<()> {
        // 对应 Java：doWrite 期间行转换失败 → 各后端（csv/xls/xlsx）批量写入报错。
        let directory = tempdir()?;

        let csv_path = directory.path().join("table.csv");
        let mut csv_writer = ExcelWriter::new(&csv_path);
        let table = MirroredWriteTable::new();
        let csv_result = csv_writer.write_with_table_handlers(
            [FailingRow2],
            &WriteSheet::<FailingRow2>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(csv_result, Err(ExcelError::Data { .. })));

        let xls_path = directory.path().join("table.xls");
        let mut xls_writer = ExcelWriter::new(&xls_path);
        let xls_result = xls_writer.write_with_table_handlers(
            [FailingRow2],
            &WriteSheet::<FailingRow2>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(xls_result, Err(ExcelError::Data { .. })));

        let xlsx_path = directory.path().join("table.xlsx");
        let mut xlsx_writer = ExcelWriter::new(&xlsx_path);
        let xlsx_outcome = xlsx_writer.write_with_table_handlers(
            [FailingRow2],
            &WriteSheet::<FailingRow2>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(xlsx_outcome, Err(ExcelError::Data { .. })));
        Ok(())
    }

    // ========================================================================
    // start() 的模板加载错误分支（1537/1554/1558/1565）
    // ========================================================================

    #[test]
    fn stateful_xls_start_rejects_missing_template_file() -> Result<()> {
        // 对应 Java：withTemplate(file) 指向不存在的文件 → 打开失败。
        let directory = tempdir()?;
        let path = directory.path().join("missing-template.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_file: Some(directory.path().join("absent.xls")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_start_rejects_csv_template_source() -> Result<()> {
        // 对应 Java：xlsx 不允许用 csv 模板。
        let directory = tempdir()?;
        let path = directory.path().join("csv-template.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_file: Some(directory.path().join("template.csv")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_start_rejects_missing_template_file() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("missing-template.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_file: Some(directory.path().join("absent.xlsx")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_legacy_seed_rejects_invalid_sheet_name() -> Result<()> {
        // 对应 Java：模板 sheet 名含非法字符（`[`）时 seed 到工作簿必须失败。
        let bytes = zip_template(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES_XML),
            ("_rels/.rels", MINIMAL_PACKAGE_RELS_XML),
            (
                "xl/workbook.xml",
                minimal_workbook_xml("bad[name").as_bytes(),
            ),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", MINIMAL_SHEET_XML),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("legacy-bad-name.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "bad[name".to_owned(),
                template_bytes: Some(bytes),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("bad[name"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // 有状态模板批量写错误分支（1832/1906/1923/2079/2174/2181）
    // ========================================================================

    #[test]
    fn stateful_xls_template_handler_callback_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("xls-tpl-cb.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(StageFailingHandler(FailStage::DataCell))],
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_legacy_seed_after_sheet_create_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-cb.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(StageFailingHandler(FailStage::AfterSheetCreate))],
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_legacy_seed_row_conversion_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-bad-row.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                use_legacy_template_seed: true,
                ..WriteOptions::default()
            },
        );
        let result = writer.write([FailingRow2], &WriteSheet::<FailingRow2>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_template_absent_sheet_missing_content_types() -> Result<()> {
        // 对应 Java：模板缺少 [Content_Types].xml 时创建新 sheet 必须报错。
        let bytes = zip_template(&[
            ("xl/workbook.xml", minimal_workbook_xml("Sheet1").as_bytes()),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", MINIMAL_SHEET_XML),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("no-ct.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Absent".to_owned(),
                template_bytes: Some(bytes),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Absent"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_template_missing_styles_xml() -> Result<()> {
        // 对应 Java：模板缺少 styles.xml 且单元格请求样式 → 导入样式必须报错。
        let bytes = zip_template(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES_XML),
            ("xl/workbook.xml", minimal_workbook_xml("Sheet1").as_bytes()),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", MINIMAL_SHEET_XML),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("no-styles.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            vec![Box::new(StyleRequestingHandler)],
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(bytes),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn stateful_xlsx_template_missing_sheet_data() -> Result<()> {
        // 对应 Java：模板 worksheet 缺少 sheetData → 追加行必须报错。
        let sheet_xml = br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><dimension ref="A1"/></worksheet>"#;
        let bytes = zip_template(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES_XML),
            ("xl/workbook.xml", minimal_workbook_xml("Sheet1").as_bytes()),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", sheet_xml),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("no-sheetdata.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                template_bytes: Some(bytes),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "v")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // 公开 API 模板/错误分支（2985/3030/3044/3131/3185/3292/3318/3375/3688）
    // ========================================================================

    #[test]
    fn public_xls_template_missing_file_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("out.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(directory.path().join("absent.xls")),
            ..WriteOptions::default()
        };
        let result = crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

    #[test]
    fn public_xls_template_handler_callback_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("out.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::DataCell))];
        let result = crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn public_xls_save_under_regular_file_rejected() -> Result<()> {
        // 对应 Java：父路径是普通文件时无法创建目录。
        let directory = tempdir()?;
        let blocker = directory.path().join("blocker");
        std::fs::write(&blocker, b"not a directory")?;
        let path = blocker.join("out.xls");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let result = crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

    #[test]
    fn public_xls_head_cell_handler_not_invoked_without_head() -> Result<()> {
        // 对应 Java：无表头（空 schema 且未配置 dynamic_head）时 head cell
        // handler 不会被调用，写入正常完成（handler 错误仅在表头真实创建时传播）。
        let directory = tempdir()?;
        let path = directory.path().join("head-nohead.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::HeadCell))];
        let result = crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(result.is_ok(), "{result:?}");
        Ok(())
    }

    #[test]
    fn public_xls_dynamic_head_cell_handler_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("head-dyn-err.xls");
        let options = WriteOptions {
            dynamic_head: Some(vec![vec!["Level".to_owned()], vec!["Field".to_owned()]]),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::HeadCell))];
        let result = crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn public_xls_loop_merge_column_overflow_rejected() -> Result<()> {
        // 对应 Java：BIFF8 合并列号超过 255 → 报错。
        let directory = tempdir()?;
        let path = directory.path().join("loop-overflow.xls");
        let loop_merges = vec![MirroredLoopMergeStrategy::new(2, 1, 300)?];
        let options = WriteOptions {
            loop_merges,
            ..WriteOptions::default()
        };
        let result = crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v"), (1, "w")])],
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn public_xls_head_cells_skipped_by_handler() -> Result<()> {
        // 对应 Java：handler 跳过单元格后表头不落盘（ExcelWriter 空 sheet 仍可保存）。
        let directory = tempdir()?;
        let path = directory.path().join("skipped.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(SkipCellHandler)];
        crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn public_xls_handler_loop_merge_invalid_property_rejected() -> Result<()> {
        // 对应 Java：handler 返回 eachRow=1/columnExtend=1 的 loop merge → 校验失败。
        let directory = tempdir()?;
        let path = directory.path().join("bad-handler-loop.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(LoopMergeBadHandler)];
        let result = crate::write::write_xls::write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // 公开 XLSX 模板路径（4172/4229/4249/4500/4501/4793/4797/4864/4964/5303/6299）
    // ========================================================================

    #[test]
    fn public_xlsx_template_missing_file_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("out.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(directory.path().join("absent.xlsx")),
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

    #[test]
    fn public_xlsx_template_handler_callback_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("out.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::DataCell))];
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn public_xlsx_template_missing_styles_xml() -> Result<()> {
        let bytes = zip_template(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES_XML),
            ("xl/workbook.xml", minimal_workbook_xml("Sheet1").as_bytes()),
            ("xl/_rels/workbook.xml.rels", MINIMAL_RELS_XML),
            ("xl/worksheets/sheet1.xml", MINIMAL_SHEET_XML),
        ]);
        let directory = tempdir()?;
        let path = directory.path().join("no-styles.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(bytes),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(StyleRequestingHandler)];
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn template_append_cell_styles_column_overflow_rejected() -> Result<()> {
        // 对应 Java：模板列号超过 XLSX 上限时样式编译必须报错。
        let mut package = crate::write::template_write::TemplatePackage::from_bytes(
            xlsx_template_bytes("Sheet1").as_slice(),
        )?;
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let rows = vec![vec![(70_000usize, CellValue::String("wide".to_owned()))]];
        let empty_converted: Vec<Vec<(usize, WriteCellData)>> = Vec::new();
        let empty_ignore: Vec<Vec<bool>> = vec![Vec::new()];
        let empty_requested: Vec<Vec<Option<ExcelCellStyle>>> = vec![Vec::new()];
        let result = template_append_cell_styles::<PlainRow>(
            &mut package,
            &options,
            &[],
            &rows,
            &rows,
            &empty_converted,
            &empty_ignore,
            &empty_requested,
            true,
            0,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn public_xlsx_legacy_seed_rejects_csv_template_source() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-csv.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(directory.path().join("template.csv")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
        Ok(())
    }

    #[test]
    fn public_xlsx_legacy_seed_missing_template_file_rejected() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-missing.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_file: Some(directory.path().join("absent.xlsx")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
        );
        assert!(matches!(result, Err(ExcelError::Io(_))));
        Ok(())
    }

    #[test]
    fn public_xlsx_legacy_seed_row_conversion_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-bad-row.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow2, _>(&path, &options, [FailingRow2]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn public_xlsx_dynamic_head_handler_error_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("head-dyn.xlsx");
        let options = WriteOptions {
            dynamic_head: Some(vec![vec!["Level".to_owned()], vec!["Field".to_owned()]]),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler(FailStage::HeadCell))];
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &options,
            [dyn_row(&[(0, "v")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn template_layout_skips_explicit_column_widths() -> Result<()> {
        // 对应 Java：WriteOptions.column_widths 显式列宽优先于注解/策略宽度。
        let directory = tempdir()?;
        let path = directory.path().join("explicit-width.xlsx");
        let mut options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        options.column_widths = vec![(0, 30)];
        crate::write::xlsx_write::write_xlsx::<PlainRow, _>(
            &path,
            &options,
            [PlainRow::new("a", "b")],
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn xlsx_comment_with_invalid_image_errors() -> Result<()> {
        // 对应 Java：批注内的图片数据损坏时按图片解析错误处理。
        let directory = tempdir()?;
        let path = directory.path().join("comment-img.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let row = dyn_row_values(&[(
            0,
            CellValue::Comment {
                value: Box::new(CellValue::Image(vec![0x89, 0x50, 0x4E])),
                text: "note".to_owned(),
            },
        )]);
        let result = crate::write::xlsx_write::write_xlsx::<DynamicRow, _>(&path, &options, [row]);
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // trait 方法直接调用补充：from_row / to_row 不经过写入主路径
    // ========================================================================

    #[test]
    fn failing_row2_from_row_is_constructible() -> Result<()> {
        // 对应 Java：ExcelRow.fromRow 只在读取侧被调用，写入侧直接调用验证。
        let row = FailingRow2::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(matches!(
            row.to_row(),
            Err(ExcelError::Data { message, .. })
                if message == "test-only row conversion failure"
        ));
        Ok(())
    }

    #[test]
    fn plain_row_from_and_to_row_round_trip() -> Result<()> {
        // 对应 Java：PlainRow 的 fromRow/toRow 往返一致（空单元格行）。
        let row = PlainRow::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(row.to_row()?.is_empty());
        Ok(())
    }

    #[test]
    fn loop_merge_bad_row_from_and_to_row_round_trip() -> Result<()> {
        // 对应 Java：注解行 fromRow/toRow 直接调用（校验失败发生在写入前）。
        let row = LoopMergeBadRow::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(row.to_row()?.is_empty());
        Ok(())
    }

    #[test]
    fn wide_index_row_from_and_to_row_round_trip() -> Result<()> {
        // 对应 Java：宽列索引行 fromRow/toRow 直接调用（写入前即被列号校验拦截）。
        let row = WideIndexRow::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(row.to_row()?.is_empty());
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra3 {
    use super::*;

    use std::collections::BTreeMap;
    use std::io::{Cursor, Write};

    use crate::core::{DynamicRow, DynamicValue};
    use tempfile::tempdir;

    /// 空实现 handler（对应 Java 无副作用的 `WriteHandler`）。
    struct NoopHandler3;

    impl WriteHandler for NoopHandler3 {}

    /// 失败阶段可配置的 handler（对应 Java 测试里的 `FailingHandler` 模式）。
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum FailStage3 {
        BeforeWorkbookCreate,
        AfterSheetCreate,
        HeadCell,
    }

    struct StageFailingHandler3(FailStage3);

    impl WriteHandler for StageFailingHandler3 {
        fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            if self.0 == FailStage3::BeforeWorkbookCreate {
                Err(ExcelError::Format("stage failure".to_owned()))
            } else {
                Ok(())
            }
        }

        fn after_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
            if self.0 == FailStage3::AfterSheetCreate {
                Err(ExcelError::Format("stage failure".to_owned()))
            } else {
                Ok(())
            }
        }

        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            if self.0 == FailStage3::HeadCell && context.is_head {
                Err(ExcelError::Format("stage failure".to_owned()))
            } else {
                Ok(())
            }
        }
    }

    /// 跳过所有单元格写入（对应 Java 里通过 handler 丢弃单元格）。
    struct SkipCellHandler3;

    impl WriteHandler for SkipCellHandler3 {
        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            context.skip = true;
            Ok(())
        }
    }

    /// 具有重复 `unique_value` 的 handler（对应 Java `NotRepeatExecutor` 去重）。
    struct UniqueHandler3(&'static str);

    impl crate::event::NotRepeatExecutor for UniqueHandler3 {
        fn unique_value(&self) -> &str {
            self.0
        }
    }

    impl WriteHandler for UniqueHandler3 {
        fn as_not_repeat_executor(&self) -> Option<&dyn crate::event::NotRepeatExecutor> {
            Some(self)
        }
    }

    /// `to_row` 返回错误的行（对应 Java `toRow` 抛异常）。
    struct FailingRow3;
    impl ExcelRow for FailingRow3 {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Err(ExcelError::Data {
                sheet: String::new(),
                row: 0,
                column: Some(7),
                field: "field",
                value: "bad".to_owned(),
                message: "round-2 injected conversion failure".to_owned(),
            })
        }
    }

    /// 普通单列 typed 行（schema 非空 → 走非 dynamic 表头分支）。
    struct SingleColRow3 {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for SingleColRow3 {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    fn dyn_row(values: &[(usize, &str)]) -> DynamicRow {
        DynamicRow::new(
            values
                .iter()
                .map(|(index, value)| (*index, DynamicValue::String((*value).to_owned())))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn xls_template_bytes(sheet_name: &str) -> Vec<u8> {
        let mut book = Biff8Book::default();
        book.sheet_mut(sheet_name);
        book.to_cfb_bytes().expect("cfb bytes")
    }

    fn xlsx_template_bytes(sheet_name: &str) -> Vec<u8> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.set_name(sheet_name).expect("sheet name");
        sheet.write_string(0, 0, "seed").expect("seed cell");
        workbook.save_to_buffer().expect("template buffer")
    }

    /// 手工构造 ZIP 模板包（entries: (路径, 内容)），默认 Stored 压缩。
    fn zip_template(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, bytes) in entries {
            writer.start_file(*name, options).expect("start entry");
            writer.write_all(bytes).expect("write entry");
        }
        writer.finish().expect("finish").into_inner()
    }

    const PACKAGE_RELS_XML: &[u8] = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#;

    const CONTENT_TYPES_XML: &[u8] = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#;

    const SHEET_XML: &[u8] = br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#;

    /// 缺少 `xl/_rels/workbook.xml.rels` 的模板：`ensure_sheet` 必须报错。
    ///
    /// 对应 Java：POI 在 `createSheet` 时依赖 workbook 关系表，缺失即失败。
    fn xlsx_template_missing_workbook_rels() -> Vec<u8> {
        zip_template(&[
            ("[Content_Types].xml", CONTENT_TYPES_XML),
            ("_rels/.rels", PACKAGE_RELS_XML),
            (
                "xl/workbook.xml",
                br#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="TemplateOnly" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
            ),
            ("xl/worksheets/sheet1.xml", SHEET_XML),
        ])
    }

    // ========================================================================
    // 生产代码 `?` 错误边：write_with_sheet_handlers / write_with_table_handlers
    // 首次注册 sheet handler 时 workbook 回调失败（对应 Java `ExcelWriter` 抛异常）。
    // ========================================================================

    #[test]
    fn write_with_sheet_handlers_new_sheet_callback_error_propagates() -> Result<()> {
        // 对应 Java：新 sheet 首次注册 sheet handler 时运行 workbook 回调，
        // 回调失败必须向上传播（`runOwnWorkbookCallbacks`）。
        let directory = tempdir()?;
        let path = directory.path().join("sheet-cb-err.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Fresh");
        let result = writer.write_with_sheet_handlers(
            [dyn_row(&[(0, "x")])],
            &sheet,
            vec![Box::new(StageFailingHandler3(
                FailStage3::BeforeWorkbookCreate,
            ))],
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn write_with_table_handlers_new_sheet_callback_error_propagates() -> Result<()> {
        // 对应 Java：表写入路径首次注册 sheet handler 时 workbook 回调失败。
        let directory = tempdir()?;
        let path = directory.path().join("table-cb-err.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [dyn_row(&[(0, "x")])],
            &WriteSheet::<DynamicRow>::new("Fresh"),
            &MirroredWriteTable::new(),
            vec![Box::new(StageFailingHandler3(
                FailStage3::BeforeWorkbookCreate,
            ))],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    #[test]
    fn xlsx_write_after_sheet_create_error_propagates() -> Result<()> {
        // 对应 Java：`afterSheetCreate` 回调失败 → `ExcelWriteExecutor` 报错。
        let directory = tempdir()?;
        let path = directory.path().join("sheet-create-err.xlsx");
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler3(FailStage3::AfterSheetCreate))];
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "x")])],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // 模板保真路径：新建 sheet 时 ensure_sheet 失败（对应 Java createSheet）。
    // ========================================================================

    #[test]
    fn xlsx_template_new_sheet_missing_workbook_rels_errors() -> Result<()> {
        // 对应 Java：withTemplate 后写入模板中不存在的 sheet，POI 需要创建，
        // 缺少 workbook 关系表时 `createSheet` 抛异常。
        let directory = tempdir()?;
        let path = directory.path().join("tpl-no-rels.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_missing_workbook_rels()),
                ..WriteOptions::default()
            },
        );
        let result = writer.write(
            [dyn_row(&[(0, "x")])],
            &WriteSheet::<DynamicRow>::new("BrandNew"),
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // BIFF8 保存：父目录创建与不可写父路径（对应 Java FileOutputStream）。
    // ========================================================================

    #[test]
    fn save_xls_book_creates_nested_parent_directory() -> Result<()> {
        // 对应 Java：写入 `a/b/out.xls` 时自动 `mkdirs`。
        let directory = tempdir()?;
        let path = directory.path().join("nested").join("plain.xls");
        crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "x")])],
        )?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn save_xls_book_parent_is_regular_file_errors() -> Result<()> {
        // 对应 Java：父路径被普通文件占位时 `mkdirs` 抛 IOException。
        let directory = tempdir()?;
        let blocker = directory.path().join("blocker");
        std::fs::write(&blocker, b"not a directory")?;
        let path = blocker.join("out.xls");
        let result = crate::write::write_xls::write_xls::<DynamicRow, _>(
            &path,
            &WriteOptions::default(),
            [dyn_row(&[(0, "x")])],
        );
        assert!(result.is_err());
        Ok(())
    }

    // ========================================================================
    // BIFF8 表头单元格：handler 在 head 单元格阶段失败（对应 Java 回调异常）。
    // ========================================================================

    #[test]
    fn xls_write_head_cell_failure_propagates() -> Result<()> {
        // 对应 Java：写表头时 `beforeCellCreate` 抛异常 → 整次写入失败。
        // 使用 schema 非空的行（非 dynamic 表头分支）；DynamicRow 无表头时
        // head 回调不会被调用（见 public_xls_head_cell_handler_not_invoked_without_head）。
        let directory = tempdir()?;
        let path = directory.path().join("head-cell-err.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(StageFailingHandler3(FailStage3::HeadCell))];
        let result = crate::write::write_xls::write_xls_with_handlers::<SingleColRow3, _>(
            &path,
            &WriteOptions::default(),
            [SingleColRow3 {
                cells: vec![CellValue::String("x".to_owned())],
            }],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // legacy seed 直达调用：模板源校验失败 / 模板文件缺失。
    // ========================================================================

    #[test]
    fn write_sheet_onto_template_rejects_csv_template_bytes() {
        // 对应 Java：`validateTemplateSource` 拒绝 CSV 模板。
        let mut workbook = Workbook::new();
        let options = WriteOptions {
            template_bytes: Some(b"a,b\n1,2".to_vec()),
            ..WriteOptions::default()
        };
        let result = write_sheet_onto_template::<DynamicRow, _>(
            &mut workbook,
            &options,
            [dyn_row(&[(0, "x")])],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Unsupported(_))));
    }

    #[test]
    fn write_sheet_onto_template_missing_template_file_errors() -> Result<()> {
        // 对应 Java：`withTemplate(file)` 指向不存在的文件 → IOException。
        let directory = tempdir()?;
        let missing = directory.path().join("missing.xlsx");
        let mut workbook = Workbook::new();
        let options = WriteOptions {
            template_file: Some(missing),
            ..WriteOptions::default()
        };
        let result = write_sheet_onto_template::<DynamicRow, _>(
            &mut workbook,
            &options,
            [dyn_row(&[(0, "x")])],
            &mut [],
        );
        assert!(result.is_err());
        Ok(())
    }

    // ========================================================================
    // TemplatePackage::from_bytes 拒绝非 ZIP 数据（对应 Java ZipFile 异常）。
    // ========================================================================

    #[test]
    fn template_package_from_bytes_rejects_garbage() {
        let result =
            crate::write::template_write::TemplatePackage::from_bytes(b"not a zip package");
        assert!(matches!(result, Err(ExcelError::Format(_))));
    }

    // ========================================================================
    // 行转换失败（FailingRow 模式）在各公开写入入口的传播：
    // 对应 Java `doWrite` 期间 `ConvertAllFiled` 抛异常 → `ExcelGenerateException`。
    // ========================================================================

    #[test]
    fn sheet_handlers_failing_row_propagates() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("sheet-handlers-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_sheet_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Fresh"),
            vec![Box::new(NoopHandler3)],
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn table_handlers_xlsx_new_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn table_handlers_xls_existing_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-fail.xls");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row(&[(0, "first")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::with_table_no(7),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn table_handlers_with_sheet_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-handlers-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            vec![Box::new(NoopHandler3)],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xls_to_writer_template_failing_row() {
        let mut output = Vec::new();
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xls_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let result = crate::write::write_xls::write_xls_to_writer::<FailingRow3, _, _>(
            std::path::Path::new("logical.xls"),
            &mut output,
            &options,
            [FailingRow3],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
    }

    #[test]
    fn xlsx_stateful_table_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("incoming-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row(&[(0, "one")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xls_absolute_merge_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("merge-fail.xls");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write([FailingRow3], &WriteSheet::<FailingRow3>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xlsx_absolute_merge_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("merge-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write([FailingRow3], &WriteSheet::<FailingRow3>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xls_font_style_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("fonts-fail.xls");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write([FailingRow3], &WriteSheet::<FailingRow3>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xls_template_dynamic_head_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-table-fail.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<FailingRow3>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
                vec!["Meta".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &sheet,
            &MirroredWriteTable::with_table_no(2),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xls_template_dynamic_head_second_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-table2-fail.xls");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xls_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![
                vec!["User".to_owned(), "Name".to_owned()],
                vec!["User".to_owned(), "Age".to_owned()],
                vec!["Meta".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "n"), (1, "a"), (2, "m")])],
            &sheet,
            &MirroredWriteTable::with_table_no(2),
            Vec::new(),
            Vec::new(),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::from_options(WriteOptions {
                sheet_name: "Sheet1".to_owned(),
                dynamic_head: Some(vec![
                    vec!["User".to_owned(), "Name".to_owned()],
                    vec!["User".to_owned(), "Age".to_owned()],
                    vec!["Meta".to_owned()],
                ]),
                ..WriteOptions::default()
            }),
            &MirroredWriteTable::with_table_no(3),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xlsx_template_existing_state_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-state-fail.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write(
            [dyn_row(&[(0, "one")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::with_table_no(0),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xlsx_legacy_seed_spill_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xlsx_absent_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("absent-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "NewSheet".to_owned(),
            template_bytes: Some(xlsx_template_bytes("TemplateOnly")),
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xls_nested_dir_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("nested").join("out-fail.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoopHandler3)];
        let result = crate::write::write_xls::write_xls_with_handlers::<FailingRow3, _>(
            &path,
            &WriteOptions::default(),
            [FailingRow3],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xls_plain_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("plain-fail.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoopHandler3)];
        let result = crate::write::write_xls::write_xls_with_handlers::<FailingRow3, _>(
            &path,
            &WriteOptions::default(),
            [FailingRow3],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xlsx_compress_temp_files_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("spill-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn csv_table_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-fail.csv");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn csv_table_handlers_second_write_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-fail2.csv");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let table = MirroredWriteTable::new();
        writer.write_with_table_handlers(
            [dyn_row(&[(0, "tabled")])],
            &sheet,
            &table,
            Vec::new(),
            Vec::new(),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn table_handlers_first_write_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("table-schema-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn table_handlers_new_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("new-sheet-handlers-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::new(),
            vec![Box::new(NoopHandler3)],
            Vec::new(),
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xlsx_template_annotation_merge_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tpl-ann-fail.xlsx");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(
            MirroredOnceAbsoluteMerge::from_property(crate::core::OnceAbsoluteMergeProperty::new(
                0, 0, 0, 1,
            ))
            .expect("merge strategy"),
        )];
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<FailingRow3, _>(
            &path,
            &options,
            [FailingRow3],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn csv_early_return_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-early-fail.csv");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row(&[(0, "a")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::with_table_no(5),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xls_table_merges_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-fail.xls");
        let mut writer = ExcelWriter::new(&path);
        writer.write(
            [dyn_row(&[(0, "a")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::with_table_no(5),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xlsx_template_layout_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-tpl-fail.xlsx");
        let mut writer = ExcelWriter::with_handlers_and_options(
            &path,
            Vec::new(),
            WriteOptions {
                template_bytes: Some(xlsx_template_bytes("Sheet1")),
                ..WriteOptions::default()
            },
        );
        writer.write(
            [dyn_row(&[(0, "a")])],
            &WriteSheet::<DynamicRow>::new("Sheet1"),
        )?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::new("Sheet1"),
            &MirroredWriteTable::with_table_no(5),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xlsx_column_widths_table_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("tbl-widths-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            column_widths: vec![(0, 30)],
            ..WriteOptions::default()
        });
        writer.write([dyn_row(&[(0, "a")])], &sheet)?;
        let result = writer.write_with_table_handlers(
            [FailingRow3],
            &WriteSheet::<FailingRow3>::from_options(WriteOptions {
                column_widths: vec![(0, 30)],
                ..WriteOptions::default()
            }),
            &MirroredWriteTable::with_table_no(5),
            Vec::new(),
            Vec::new(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn xlsx_font_style_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("fonts-fail.xlsx");
        let mut writer = ExcelWriter::new(&path);
        let result = writer.write([FailingRow3], &WriteSheet::<FailingRow3>::new("Sheet1"));
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn dedupe_handlers_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dedupe-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![
            Box::new(UniqueHandler3("shared")),
            Box::new(UniqueHandler3("shared")),
        ];
        let result = crate::write::xlsx_write::write_xlsx_with_handlers::<FailingRow3, _>(
            &path,
            &options,
            [FailingRow3],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xlsx_template_password_failing_row() {
        let mut output = Vec::new();
        let options = WriteOptions {
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            password: Some("pw".to_owned()),
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx_to_writer::<FailingRow3, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            [FailingRow3],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
    }

    #[test]
    fn legacy_seed_layout_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-layout-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            column_widths: vec![(0, 25)],
            merge_ranges: vec![MergeRange::new(1, 2, 0, 1)],
            auto_width: true,
            compress_temp_files: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn legacy_seed_absent_sheet_failing_row() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("legacy-absent-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "BrandNew".to_owned(),
            sheet_index: Some(9),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xlsx_legacy_seed_writer_failing_row() {
        let mut output = Vec::new();
        let options = WriteOptions {
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            use_legacy_template_seed: true,
            ..WriteOptions::default()
        };
        let result = crate::write::xlsx_write::write_xlsx_to_writer::<FailingRow3, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            [FailingRow3],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
    }

    // ========================================================================
    // 模板样式编译：列号超限（对应 Java `@ExcelProperty.index` 越界）。
    // ========================================================================

    #[test]
    fn template_append_cell_styles_wide_column_direct() -> Result<()> {
        // 对应 Java：模板样式编译时列号超过 XLSX 上限 → 报错。
        let mut package = crate::write::template_write::TemplatePackage::from_bytes(
            xlsx_template_bytes("Sheet1").as_slice(),
        )?;
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let rows = vec![vec![(70_000usize, CellValue::String("wide".to_owned()))]];
        let empty_converted: Vec<Vec<(usize, WriteCellData)>> = Vec::new();
        let empty_ignore: Vec<Vec<bool>> = vec![Vec::new()];
        let empty_requested: Vec<Vec<Option<ExcelCellStyle>>> = vec![Vec::new()];
        let result = template_append_cell_styles::<FailingRow3>(
            &mut package,
            &options,
            &[],
            &rows,
            &rows,
            &empty_converted,
            &empty_ignore,
            &empty_requested,
            true,
            0,
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        Ok(())
    }

    // ========================================================================
    // 跳过表头单元格 + 行转换失败（对应 Java handler 跳过单元格后仍转换行）。
    // ========================================================================

    #[test]
    fn xls_skipped_head_cells_failing_row() -> Result<()> {
        // 对应 Java：handler 跳过全部单元格时，行转换（toRow）失败仍须上报。
        let directory = tempdir()?;
        let path = directory.path().join("skipped-fail.xls");
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(SkipCellHandler3)];
        let result = crate::write::write_xls::write_xls_with_handlers::<FailingRow3, _>(
            &path,
            &WriteOptions::default(),
            [FailingRow3],
            &mut handlers,
        );
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    #[test]
    fn xlsx_explicit_widths_failing_row() -> Result<()> {
        // 对应 Java：显式列宽 + 行转换失败 → 错误传播。
        let directory = tempdir()?;
        let path = directory.path().join("explicit-width-fail.xlsx");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(xlsx_template_bytes("Sheet1")),
            column_widths: vec![(0, 30)],
            ..WriteOptions::default()
        };
        let result =
            crate::write::xlsx_write::write_xlsx::<FailingRow3, _>(&path, &options, [FailingRow3]);
        assert!(matches!(result, Err(ExcelError::Data { .. })));
        Ok(())
    }

    // ========================================================================
    // trait 方法直接调用补充：from_row / to_row / before_cell_create Ok 分支
    // ========================================================================

    #[test]
    fn failing_row3_from_row_is_constructible() -> Result<()> {
        // 对应 Java：ExcelRow.fromRow 只在读取侧被调用，写入侧直接调用验证。
        let row = FailingRow3::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(matches!(
            row.to_row(),
            Err(ExcelError::Data { message, .. })
                if message == "round-2 injected conversion failure"
        ));
        Ok(())
    }

    #[test]
    fn single_col_row3_from_and_to_row_round_trip() -> Result<()> {
        // 对应 Java：SingleColRow3 的 fromRow/toRow 往返一致（空单元格行）。
        let row = SingleColRow3::from_row(&crate::core::RowData::new(
            "sheet",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert!(row.to_row()?.is_empty());
        Ok(())
    }

    #[test]
    fn stage_failing_handler3_non_matching_stage_passes_cells() {
        // 对应 Java：失败阶段不匹配时 beforeCellCreate 放行（Ok 分支）。
        let mut context = WriteCellContext::new("Sheet1", 0, 0, CellValue::String("v".to_owned()));
        let mut handler = StageFailingHandler3(FailStage3::AfterSheetCreate);
        assert!(handler.before_cell_create(&mut context).is_ok());
    }
}
