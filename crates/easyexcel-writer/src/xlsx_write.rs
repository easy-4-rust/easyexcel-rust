//! XLSX 写入功能。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter` 的 XLSX 写入路径。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/ExcelWriter.java

use std::io::Write;
use std::path::Path;

use easyexcel_core::{ExcelRow, Result, WriteHandler, WriteWorkbookContext};
use rust_xlsxwriter::Workbook;
use crate::write_options::WriteOptions;
use crate::excel_writer_core::{
    before_workbook, after_workbook, after_workbook_create, save_workbook, save_workbook_to_writer,
    sort_handlers, validate_excel_row_schema, with_default_write_converters,
    write_sheet_to_workbook, write_xlsx_onto_template_package,
    HandlerHolderScope,
};

/// 将类型化行写入 XLSX 文件。
///
/// # Errors
///
/// 返回转换、校验、XLSX 格式或 I/O 错误。
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
        let holder_scope = HandlerHolderScope::new_resolved::<T>(logical_path, i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX), None, options)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;
    use std::io::Cursor;
    use easyexcel_core::{DynamicRow, DynamicValue};
    use crate::write_options::WriteOptions;

    fn dynamic_row() -> DynamicRow {
        let mut values = BTreeMap::new();
        values.insert(0, DynamicValue::String("alice".to_owned()));
        DynamicRow::new(values)
    }

    #[test]
    fn write_xlsx_to_writer_emits_xlsx_bytes() {
        let mut options = WriteOptions::default();
        options.sheet_name = "Sheet1".to_owned();
        let mut output = Cursor::new(Vec::<u8>::new());
        write_xlsx_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            vec![dynamic_row()],
            &mut [],
        )
        .expect("write must succeed");
        assert!(output.get_ref().len() > 0);
    }

}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use std::collections::BTreeMap;
    use std::io::Cursor;
    use easyexcel_core::{DynamicRow, DynamicValue};
    use crate::write_options::WriteOptions;

    fn dynamic_row() -> DynamicRow {
        let mut values = BTreeMap::new();
        values.insert(0, DynamicValue::String("alice".to_owned()));
        DynamicRow::new(values)
    }

    #[test]
    fn write_xlsx_to_writer_uses_template_when_provided() {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.write_string(0, 0, "h").expect("write");
        let template = workbook.save_to_buffer().expect("buffer");
        let mut options = WriteOptions::default();
        options.sheet_name = "Sheet1".to_owned();
        options.template_bytes = Some(template);
        let mut output = Cursor::new(Vec::<u8>::new());
        write_xlsx_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            vec![dynamic_row()],
            &mut [],
        )
        .expect("template write must succeed");
        assert!(output.get_ref().len() > 0);
    }

}
