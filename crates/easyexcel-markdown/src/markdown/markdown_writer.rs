use std::io::Write;

use easyexcel_io::{Error, Result, RowSink, StreamCell, StreamInfo};
use easyexcel_model::{CellValue, DateSystem, TabularDocument};

use super::gfm_escape::{escape_cell, escape_heading};
use super::markdown_output_guard::MarkdownOutputGuard;
use super::{
    MarkdownConversionMode, MarkdownConversionReport, MarkdownExportOptions, MarkdownHeaderPolicy,
    MarkdownWarning,
};

/// 将一个或多个工作表行流写为确定性 GFM table。
pub struct MarkdownWriter<W: Write> {
    output: MarkdownOutputGuard<W>,
    options: MarkdownExportOptions,
    report: MarkdownConversionReport,
    date_system: DateSystem,
    pending_header: Option<Vec<String>>,
    columns: Option<usize>,
    header_written: bool,
    sheets_started: usize,
}

impl<W: Write> MarkdownWriter<W> {
    /// 创建 Event Mode writer。
    #[must_use]
    pub fn new(writer: W, options: MarkdownExportOptions) -> Self {
        Self::with_mode(writer, options, MarkdownConversionMode::Event)
    }

    pub(crate) fn with_mode(
        writer: W,
        options: MarkdownExportOptions,
        mode: MarkdownConversionMode,
    ) -> Self {
        let limit = options.limits().max_output_bytes();
        Self {
            output: MarkdownOutputGuard::new(writer, limit),
            options,
            report: MarkdownConversionReport::new(mode),
            date_system: DateSystem::Date1900,
            pending_header: None,
            columns: None,
            header_written: false,
            sheets_started: 0,
        }
    }

    /// 返回当前转换报告。
    #[must_use]
    pub const fn report(&self) -> &MarkdownConversionReport {
        &self.report
    }

    pub(crate) fn report_mut(&mut self) -> &mut MarkdownConversionReport {
        &mut self.report
    }

    pub(crate) const fn options(&self) -> &MarkdownExportOptions {
        &self.options
    }

    /// 添加由格式编排层发现的 warning。
    pub fn push_warning(&mut self, warning: MarkdownWarning) {
        self.report.warnings.push(warning);
    }

    /// 刷新输出并返回底层 writer 与最终报告。
    ///
    /// # Errors
    ///
    /// 刷新底层 writer 失败时返回 I/O 错误。
    pub fn finish(mut self) -> Result<(W, MarkdownConversionReport)> {
        self.output.flush()?;
        self.report.output_bytes = self.output.written();
        Ok((self.output.into_inner(), self.report))
    }

    pub(crate) fn write_raw(&mut self, value: &str) -> Result<()> {
        self.output.write_text(value)
    }

    fn begin_sheet(&mut self, name: &str, date_system: DateSystem) -> Result<()> {
        if self.sheets_started > 0 {
            self.output.write_text("\n\n")?;
        }
        self.output.write_text("## ")?;
        self.output.write_text(&escape_heading(name))?;
        self.output.write_text("\n\n")?;
        self.date_system = date_system;
        self.pending_header = None;
        self.columns = None;
        self.header_written = false;
        self.sheets_started = self.sheets_started.saturating_add(1);
        self.report.sheets_processed = self.report.sheets_processed.saturating_add(1);
        self.report.tables_processed = self.report.tables_processed.saturating_add(1);
        Ok(())
    }

    fn densify(&self, cells: &[StreamCell]) -> Result<Vec<String>> {
        let width = cells.last().map_or(0usize, |cell| {
            usize::try_from(cell.col)
                .unwrap_or(usize::MAX)
                .saturating_add(1)
        });
        if width > self.options.limits().max_columns() {
            return Err(Error::ResourceLimit {
                resource: "columns",
                limit: u64::try_from(self.options.limits().max_columns()).unwrap_or(u64::MAX),
                actual: u64::try_from(width).unwrap_or(u64::MAX),
            });
        }
        let mut row = vec![String::new(); width];
        for cell in cells {
            let index = usize::try_from(cell.col).unwrap_or(usize::MAX);
            if index >= row.len() {
                return Err(Error::Markdown {
                    line: None,
                    message: "stream cells are not sorted by column".to_owned(),
                });
            }
            let text = match self.options.values() {
                super::MarkdownValuePolicy::Formatted => cell.display(self.date_system),
                super::MarkdownValuePolicy::Raw => cell.value.to_display_string(),
            };
            let chars = text.chars().count();
            if chars > self.options.limits().max_cell_chars() {
                return Err(Error::ResourceLimit {
                    resource: "cell_chars",
                    limit: u64::try_from(self.options.limits().max_cell_chars())
                        .unwrap_or(u64::MAX),
                    actual: u64::try_from(chars).unwrap_or(u64::MAX),
                });
            }
            row[index] = text;
        }
        Ok(row)
    }

    fn ensure_header_written(&mut self) -> Result<()> {
        if self.header_written {
            return Ok(());
        }
        let Some(header) = self.pending_header.take() else {
            return Ok(());
        };
        self.columns = Some(header.len());
        self.write_row_values(&header)?;
        self.output.write_text("|")?;
        for _ in 0..header.len() {
            self.output.write_text(" --- |")?;
        }
        self.output.write_text("\n")?;
        self.header_written = true;
        Ok(())
    }

    fn write_row_values(&mut self, values: &[String]) -> Result<()> {
        let columns = self.columns.unwrap_or(values.len());
        if values.len() > columns {
            return Err(Error::Markdown {
                line: None,
                message: format!("row has {} columns but header has {columns}", values.len()),
            });
        }
        self.output.write_text("|")?;
        for column in 0..columns {
            self.output.write_text(" ")?;
            let value = values.get(column).map_or("", String::as_str);
            self.output.write_text(&escape_cell(value))?;
            self.output.write_text(" |")?;
        }
        self.output.write_text("\n")?;
        Ok(())
    }

    fn generated_header(width: usize) -> Vec<String> {
        (0..width)
            .map(|column| {
                easyexcel_model::addr::col_index_to_letters(
                    u32::try_from(column).unwrap_or(u32::MAX),
                )
            })
            .collect()
    }
}

impl<W: Write> RowSink for MarkdownWriter<W> {
    fn begin(&mut self, info: &StreamInfo) -> Result<()> {
        self.begin_sheet(&info.sheet_name, info.date_system)
    }

    fn row(&mut self, _row_index: u32, cells: &[StreamCell]) -> Result<()> {
        self.report.rows_processed = self.report.rows_processed.saturating_add(1);
        if self.report.rows_processed > self.options.limits().max_rows() {
            return Err(Error::ResourceLimit {
                resource: "rows",
                limit: self.options.limits().max_rows(),
                actual: self.report.rows_processed,
            });
        }
        self.report.cells_processed = self
            .report
            .cells_processed
            .saturating_add(u64::try_from(cells.len()).unwrap_or(u64::MAX));
        let row = self.densify(cells)?;
        if self.pending_header.is_none() && !self.header_written {
            match self.options.header() {
                MarkdownHeaderPolicy::FirstRow => {
                    self.pending_header = Some(row);
                    return Ok(());
                }
                MarkdownHeaderPolicy::Generated => {
                    self.pending_header = Some(Self::generated_header(row.len()));
                    self.ensure_header_written()?;
                    return self.write_row_values(&row);
                }
            }
        }
        self.ensure_header_written()?;
        self.write_row_values(&row)
    }

    fn end(&mut self) -> Result<()> {
        self.ensure_header_written()?;
        Ok(())
    }
}

/// 将中立表格文档写入 Markdown。
///
/// # Errors
///
/// 文档超过资源限制、行列结构不兼容或底层 writer 写入失败时返回错误。
pub fn write_document<W: Write>(
    document: &TabularDocument,
    writer: W,
    options: &MarkdownExportOptions,
) -> Result<(W, MarkdownConversionReport)> {
    let mut markdown =
        MarkdownWriter::with_mode(writer, options.clone(), MarkdownConversionMode::Workbook);
    for table in document.tables() {
        markdown.begin(&StreamInfo {
            sheet_name: table.name().to_owned(),
            date_system: DateSystem::Date1900,
        })?;
        for (row_index, row) in table.rows().iter().enumerate() {
            let cells: Vec<StreamCell> = row
                .iter()
                .enumerate()
                .filter(|(_, cell)| !matches!(cell.value(), CellValue::Empty))
                .map(|(column, cell)| StreamCell {
                    col: u32::try_from(column).unwrap_or(u32::MAX),
                    value: cell.value().clone(),
                    number_format: String::new(),
                })
                .collect();
            markdown.row(u32::try_from(row_index).unwrap_or(u32::MAX), &cells)?;
        }
        markdown.end()?;
    }
    markdown.finish()
}
