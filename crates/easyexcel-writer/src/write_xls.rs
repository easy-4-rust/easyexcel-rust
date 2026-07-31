//! XLS 写入功能。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter` 的 XLS 写入路径。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/ExcelWriter.java

use std::io::Write;
use std::path::Path;

use easyexcel_core::{ExcelError, ExcelRow, Result, WriteHandler, WriteWorkbookContext};
use crate::biff8::Biff8Book;
use crate::write_options::WriteOptions;
use crate::excel_writer_core::{
    before_workbook, after_workbook, after_workbook_create, save_xls_book,
    sort_handlers, validate_excel_row_schema, validate_xls_options, with_default_write_converters,
    HandlerHolderScope, write_xls_onto_template, write_sheet_to_biff8_book,
};

/// 将类型化行写入 BIFF8 (`.xls`) 文件。
///
/// # Errors
///
/// 返回转换、校验、BIFF8 格式或 I/O 错误。
pub fn write_xls<T, I>(path: &Path, options: &WriteOptions, rows: I) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    write_xls_with_handlers(path, options, rows, &mut [])
}

/// Writes typed rows to a BIFF8 file while invoking ordered write handlers.
///
/// When [`WriteOptions`] carries a template, uses
/// [`crate::biff8::Biff8TemplatePackage`] (Java `withTemplate` + `doWrite` on HSSF).
/// Password protection remains [`ExcelError::Unsupported`].
///
/// # Errors
///
/// Returns a conversion, handler, BIFF8-format, template, or I/O error.
pub fn write_xls_with_handlers<T, I>(
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
    validate_xls_options(options)?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;

    if crate::template_write::has_template(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    ) {
        write_xls_onto_template::<T, I>(path, None, options, rows, handlers)?;
        after_workbook(handlers, &workbook_context)?;
        return Ok(());
    }

    let mut book = Biff8Book::default();
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        path,
        i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
        None,
        options,
    )?;
    write_sheet_to_biff8_book::<T, I>(&mut book, options, rows, handlers, Some(&holder_scope))?;
    // Phase 5.3: BIFF8 RC4 encryption
    if let Some(password) = &options.password {
        let raw_bytes = book.to_cfb_bytes()?;
        let (encrypted, _salt, _vh) =
            crate::biff8::encrypt::encrypt_biff8_stream(&raw_bytes, password);
        std::fs::write(path, &encrypted).map_err(ExcelError::from)?;
    } else {
        save_xls_book(&book, path)?;
    }
    after_workbook(handlers, &workbook_context)?;
    Ok(())
}

/// Writes typed rows as BIFF8 bytes to an arbitrary writer.
///
/// # Errors
///
/// Returns a conversion, handler, BIFF8-format, or stream I/O error.
pub fn write_xls_to_writer<T, I, W>(
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
    validate_xls_options(options)?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(logical_path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;

    if crate::template_write::has_template(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    ) {
        write_xls_onto_template::<T, I>(logical_path, Some(&mut output), options, rows, handlers)?;
        after_workbook(handlers, &workbook_context)?;
        return Ok(());
    }

    let mut book = Biff8Book::default();
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        logical_path,
        i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
        None,
        options,
    )?;
    write_sheet_to_biff8_book::<T, I>(&mut book, options, rows, handlers, Some(&holder_scope))?;
    book.write_to(&mut output)?;
    output.flush()?;
    after_workbook(handlers, &workbook_context)?;
    Ok(())
}


