use std::fs::File;
use std::io::Write;
use std::path::Path;

use easyexcel_io::{Format, RowSource};
use easyexcel_markdown::{
    MarkdownConversionMode, MarkdownConversionReport, MarkdownExportOptions, MarkdownFormulaPolicy,
    MarkdownMergePolicy, MarkdownSheetSelection, MarkdownWarning, MarkdownWarningCode,
    MarkdownWriter,
};
use easyexcel_model::{Visibility, Workbook};

use crate::{ExcelError, Result};

/// 将工作簿路径导出到 Markdown 路径。
///
/// # Errors
///
/// 输入格式不受支持、资源超限或读写失败时返回错误。
pub fn export_path(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: &MarkdownExportOptions,
) -> Result<MarkdownConversionReport> {
    export_path_with_password(input, output, options, None)
}

/// 使用可选密码将工作簿路径导出到 Markdown 路径。
///
/// # Errors
///
/// 密码错误、输入格式不受支持、资源超限或读写失败时返回错误。
pub fn export_path_with_password(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: &MarkdownExportOptions,
    password: Option<&str>,
) -> Result<MarkdownConversionReport> {
    let file = File::create(output)?;
    let (_, report) = export_to_writer_with_password(input, file, options, password)?;
    Ok(report)
}

/// 将工作簿路径导出到任意 writer。
///
/// # Errors
///
/// 输入格式不受支持、资源超限或底层 writer 写入失败时返回错误。
pub fn export_to_writer<W: Write>(
    input: impl AsRef<Path>,
    writer: W,
    options: &MarkdownExportOptions,
) -> Result<(W, MarkdownConversionReport)> {
    export_to_writer_with_password(input, writer, options, None)
}

/// 使用可选密码将工作簿路径导出到任意 writer。
///
/// # Errors
///
/// 密码错误、模式与策略不兼容、资源超限或读写失败时返回错误。
pub fn export_to_writer_with_password<W: Write>(
    input: impl AsRef<Path>,
    writer: W,
    options: &MarkdownExportOptions,
    password: Option<&str>,
) -> Result<(W, MarkdownConversionReport)> {
    let input = input.as_ref();
    validate_input_size(input, options)?;
    let format = Format::detect_path(input)?;
    let mode = resolve_mode(format, options, password)?;
    match mode {
        MarkdownConversionMode::Event => export_event(input, format, writer, options),
        MarkdownConversionMode::Workbook | MarkdownConversionMode::Auto => {
            let workbook = read_workbook(input, format, password)?;
            easyexcel_markdown::write_workbook(&workbook, writer, options).map_err(ExcelError::from)
        }
    }
}

fn validate_input_size(path: &Path, options: &MarkdownExportOptions) -> Result<()> {
    let actual = path.metadata()?.len();
    let limit = options.limits().max_file_bytes();
    if actual > limit {
        return Err(ExcelError::from(easyexcel_io::Error::ResourceLimit {
            resource: "file_bytes",
            limit,
            actual,
        }));
    }
    Ok(())
}

fn resolve_mode(
    format: Format,
    options: &MarkdownExportOptions,
    password: Option<&str>,
) -> Result<MarkdownConversionMode> {
    let xlsx_event_compatible = options.formulas() == MarkdownFormulaPolicy::CachedValue
        && options.merges() == MarkdownMergePolicy::AnchorWithWarning
        && password.is_none();
    match options.mode() {
        MarkdownConversionMode::Workbook => Ok(MarkdownConversionMode::Workbook),
        MarkdownConversionMode::Auto => Ok(match format {
            Format::Xlsx if xlsx_event_compatible => MarkdownConversionMode::Event,
            Format::Csv => MarkdownConversionMode::Event,
            Format::Xlsx | Format::Xls => MarkdownConversionMode::Workbook,
            _ => {
                return Err(ExcelError::Unsupported(
                    "format is not supported for Markdown export".to_owned(),
                ));
            }
        }),
        MarkdownConversionMode::Event => match format {
            Format::Csv => Ok(MarkdownConversionMode::Event),
            Format::Xlsx if xlsx_event_compatible => Ok(MarkdownConversionMode::Event),
            Format::Xlsx => Err(ExcelError::Unsupported(
                "XLSX Event Mode only supports cached formulas, anchor merge projection, and unencrypted input".to_owned(),
            )),
            Format::Xls => Err(ExcelError::Unsupported(
                "XLS Event Mode is not implemented; use Workbook Mode".to_owned(),
            )),
            _ => Err(ExcelError::Unsupported(
                "format is not supported for Markdown Event Mode".to_owned(),
            )),
        },
    }
}

fn export_event<W: Write>(
    input: &Path,
    format: Format,
    writer: W,
    options: &MarkdownExportOptions,
) -> Result<(W, MarkdownConversionReport)> {
    let mut markdown = MarkdownWriter::new(writer, options.clone());
    match format {
        Format::Xlsx => {
            let entries = easyexcel_xlsx::stream_sheet_entries(File::open(input)?)?;
            let selected = select_event_sheets(
                entries,
                options.sheets(),
                options.include_hidden(),
                &mut markdown,
            )?;
            for name in selected {
                markdown.push_warning(
                    MarkdownWarning::new(
                        MarkdownWarningCode::MergeMetadataUnavailable,
                        "XLSX Event Mode projects cached row values without merge metadata",
                    )
                    .with_sheet(&name),
                );
                easyexcel_xlsx::stream(File::open(input)?, Some(&name), &mut markdown)?;
            }
        }
        Format::Csv => {
            let mut source = easyexcel_csv::CsvRowSource::new(
                File::open(input)?,
                easyexcel_csv::CsvReadOptions::default(),
                easyexcel_csv::CsvCharset::default(),
            );
            source.stream(&mut markdown)?;
        }
        Format::Xls => {
            return Err(ExcelError::Unsupported(
                "XLS Event Mode is not implemented".to_owned(),
            ));
        }
        _ => {
            return Err(ExcelError::Unsupported(
                "format is not supported for Markdown Event Mode".to_owned(),
            ));
        }
    }
    markdown.finish().map_err(ExcelError::from)
}

fn select_event_sheets<W: Write>(
    entries: Vec<(String, Visibility)>,
    selection: &MarkdownSheetSelection,
    include_hidden: bool,
    writer: &mut MarkdownWriter<W>,
) -> Result<Vec<String>> {
    let visible = entries
        .into_iter()
        .filter_map(|(name, visibility)| {
            if visibility == Visibility::Visible || include_hidden {
                Some(name)
            } else {
                writer.push_warning(
                    MarkdownWarning::new(
                        MarkdownWarningCode::HiddenSheetSkipped,
                        "hidden worksheet was skipped",
                    )
                    .with_sheet(name),
                );
                None
            }
        })
        .collect::<Vec<_>>();
    match selection {
        MarkdownSheetSelection::All => Ok(visible),
        MarkdownSheetSelection::First => visible
            .first()
            .cloned()
            .map(|name| vec![name])
            .ok_or_else(|| ExcelError::SheetNotFound("0".to_owned())),
        MarkdownSheetSelection::Index(index) => visible
            .get(*index)
            .cloned()
            .map(|name| vec![name])
            .ok_or_else(|| ExcelError::SheetNotFound(index.to_string())),
        MarkdownSheetSelection::Name(name) => visible
            .into_iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(name))
            .map(|name| vec![name])
            .ok_or_else(|| ExcelError::SheetNotFound(name.clone())),
    }
}

fn read_workbook(path: &Path, format: Format, password: Option<&str>) -> Result<Workbook> {
    match format {
        Format::Xlsx => {
            easyexcel_xlsx::read_path_with_password(path, password).map_err(ExcelError::from)
        }
        Format::Xls => easyexcel_xls::read_path(path).map_err(ExcelError::from),
        Format::Csv => {
            easyexcel_csv::read_csv(File::open(path)?, &easyexcel_csv::CsvReadOptions::default())
                .map_err(ExcelError::from)
        }
        _ => Err(ExcelError::Unsupported(
            "format is not supported for Markdown export".to_owned(),
        )),
    }
}
