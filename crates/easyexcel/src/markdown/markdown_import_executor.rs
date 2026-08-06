use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;

use easyexcel_io::Format;
use easyexcel_markdown::{MarkdownConversionReport, MarkdownImportOptions};

use crate::{ExcelError, Result};

/// 将 Markdown 路径导入 XLS、XLSX 或 CSV 路径。
///
/// # Errors
///
/// Markdown 无效、目标格式不受支持、资源超限或读写失败时返回错误。
pub fn import_path(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: &MarkdownImportOptions,
) -> Result<MarkdownConversionReport> {
    let output = output.as_ref();
    let format = Format::from_path(output).ok_or_else(|| {
        ExcelError::Unsupported(format!("unsupported workbook output: {}", output.display()))
    })?;
    let file = File::create(output)?;
    let (_, report) = import_to_writer(input, format, file, options)?;
    Ok(report)
}

/// 将 Markdown 路径导入指定工作簿 writer。
///
/// # Errors
///
/// Markdown 无效、目标格式不受支持、资源超限或底层 writer 写入失败时返回错误。
pub fn import_to_writer<W: Read + Write + Seek>(
    input: impl AsRef<Path>,
    format: Format,
    mut writer: W,
    options: &MarkdownImportOptions,
) -> Result<(W, MarkdownConversionReport)> {
    let input = input.as_ref();
    let actual = input.metadata()?.len();
    if actual > options.limits().max_file_bytes() {
        return Err(ExcelError::from(easyexcel_io::Error::ResourceLimit {
            resource: "file_bytes",
            limit: options.limits().max_file_bytes(),
            actual,
        }));
    }
    let read = easyexcel_markdown::read_markdown(File::open(input)?, options)?;
    if format == Format::Csv && read.document.tables().len() != 1 {
        return Err(ExcelError::Unsupported(
            "Markdown to CSV requires exactly one selected table".to_owned(),
        ));
    }
    let workbook = read
        .document
        .to_workbook_with_header_style(options.apply_header_style());
    match format {
        Format::Xlsx => easyexcel_xlsx::write(&workbook, &mut writer)?,
        Format::Xls => easyexcel_xls::write(&workbook, &mut writer)?,
        Format::Csv => easyexcel_csv::write_csv(
            &workbook,
            0,
            &mut writer,
            &easyexcel_csv::CsvWriteOptions::default(),
        )?,
        _ => {
            return Err(ExcelError::Unsupported(
                "format is not supported for Markdown import".to_owned(),
            ));
        }
    }
    let mut report = read.report;
    report.output_bytes = writer.stream_position()?;
    Ok((writer, report))
}
