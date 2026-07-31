//! CSV 写入功能。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter` 的 CSV 写入路径。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/ExcelWriter.java

use std::fs::File;
use std::io::Write;
use std::path::Path;

use easyexcel_core::{ExcelError, ExcelRow, Result, WriteHandler};
use crate::write_options::WriteOptions;
use crate::writer_helpers::CapturedOutput;
use crate::excel_writer_core::{
    write_csv_to, validate_csv_options, validate_excel_row_schema,
};

fn take_captured_output(output: &CapturedOutput) -> Result<Vec<u8>> {
    let mut bytes = output
        .0
        .lock()
        .map_err(|_| ExcelError::Io(std::io::Error::other("CSV capture lock poisoned")))?;
    Ok(std::mem::take(&mut *bytes))
}

/// 使用自定义处理器将类型化行写入 CSV 文件。
///
/// # Errors
///
/// 返回转换、校验、处理器或文件 I/O 错误。
pub fn write_csv_with_handlers<T, I>(
    path: &Path,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    validate_excel_row_schema::<T>()?;
    validate_csv_options(options)?;
    let file = File::create(path)?;
    write_csv_to::<T, I>(path, Box::new(file), options, rows, handlers)
}

/// Writes typed CSV rows to an owned byte stream.
///
/// `logical_path` is used by write-handler contexts and does not need to exist
/// on the filesystem. This is the Rust equivalent of Java `EasyExcel`'s
/// `OutputStream` CSV entry point.
///
/// # Errors
///
/// Returns a conversion, handler, CSV-format, charset, or stream I/O error.
pub fn write_csv_to_writer<T, I, W>(
    logical_path: &Path,
    output: W,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
    W: Write + Send + 'static,
{
    validate_excel_row_schema::<T>()?;
    validate_csv_options(options)?;
    write_csv_to::<T, I>(logical_path, Box::new(output), options, rows, handlers)
}

/// Builds a complete CSV document in memory.
///
/// This is primarily used when a borrowed output stream must not receive a
/// partial document if row conversion or a handler fails.
///
/// # Errors
///
/// Returns a conversion, handler, CSV-format, or charset error.
pub fn write_csv_to_buffer<T, I>(
    logical_path: &Path,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<Vec<u8>>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let output = CapturedOutput::default();
    write_csv_to_writer::<T, I, _>(logical_path, output.clone(), options, rows, handlers)?;
    take_captured_output(&output)
}
