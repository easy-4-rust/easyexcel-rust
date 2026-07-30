//! XLSX 写入功能。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter` 的 XLSX 写入路径。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/ExcelWriter.java

use std::io::Write;
use std::path::Path;

use easyexcel_core::{ExcelRow, Result, WriteHandler, WriteWorkbookContext};
use rust_xlsxwriter::Workbook;
use crate::handler::default_write_handler_loader::DefaultWriteHandlerLoader;
use crate::write_options::WriteOptions;
use crate::writer_helpers::share_handlers;
use crate::excel_writer_core::{
    before_workbook, after_workbook, after_workbook_create, save_workbook, save_workbook_to_writer,
    sort_handlers, validate_excel_row_schema, with_default_write_converters,
    write_sheet_to_workbook, write_xlsx_onto_template_package,
    HandlerHolderScope,
};

pub fn write_xlsx<T, I>(path: &Path, options: &WriteOptions, rows: I) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    write_xlsx_with_handlers(path, options, rows, &mut [])
}

/// Writes typed rows while invoking ordered write handlers.
///
/// # Errors
///
/// Returns a conversion, handler, worksheet-configuration, XLSX-format, or I/O error.
pub fn write_xlsx_with_handlers<T, I>(
    path: &Path,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let effective_options = with_default_write_converters(options);
    let options = &effective_options;
    validate_excel_row_schema::<T>()?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;

    if crate::template_write::has_template(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    ) {
        write_xlsx_onto_template_package::<T, I>(path, None, options, rows, handlers)?;
    } else {
        let mut workbook = Workbook::new();
        let holder_scope = HandlerHolderScope::new_resolved::<T>(
            path,
            i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
            None,
            options,
        )?;
        write_sheet_to_workbook::<T, I>(
            &mut workbook,
            options,
            rows,
            handlers,
            Some(&holder_scope),
        )?;
        save_workbook(&mut workbook, path, options.password.as_deref())?;
    }
    after_workbook(handlers, &workbook_context)?;
    Ok(())
}

/// Writes typed rows to an arbitrary XLSX byte stream.
///
/// `logical_path` is used only by write-handler contexts. Unlike the path
/// entry point this function writes the OOXML package to `output` itself, so
/// it is suitable for HTTP response bodies and in-memory buffers.
///
/// # Errors
///
/// Returns a conversion, handler, worksheet-configuration, XLSX-format,
/// encryption, or stream I/O error.
pub fn write_xlsx_to_writer<T, I, W>(
    logical_path: &Path,
    mut output: W,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
    W: Write + Send,
{
    let effective_options = with_default_write_converters(options);
    let options = &effective_options;
    validate_excel_row_schema::<T>()?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(logical_path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;

    if crate::template_write::has_template(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    ) {
        write_xlsx_onto_template_package::<T, I>(
            logical_path,
            Some(&mut output),
            options,
            rows,
            handlers,
        )?;
    } else {
        let mut workbook = Workbook::new();
        let holder_scope = HandlerHolderScope::new_resolved::<T>(
            logical_path,
            i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
            None,
            options,
        )?;
        write_sheet_to_workbook::<T, I>(
            &mut workbook,
            options,
            rows,
            handlers,
            Some(&holder_scope),
        )?;
        save_workbook_to_writer(&mut workbook, &mut output, options.password.as_deref())?;
    }
    after_workbook(handlers, &workbook_context)
}
