use std::collections::HashSet;
use std::io::Write;

use easyexcel_io::{Error, Result, RowSink, StreamCell, StreamInfo};
use easyexcel_model::{Cell, CellRange, CellValue, Sheet, Visibility, Workbook};

use super::gfm_escape::escape_html;
use super::{
    MarkdownConversionMode, MarkdownConversionReport, MarkdownExportOptions, MarkdownFormulaPolicy,
    MarkdownMergePolicy, MarkdownSheetSelection, MarkdownValuePolicy, MarkdownWarning,
    MarkdownWarningCode, MarkdownWriter,
};

/// 直接从完整 Workbook 投影公式、格式、隐藏状态和合并区域。
pub struct MarkdownWorkbookWriter<'a, W: Write> {
    workbook: &'a Workbook,
    writer: MarkdownWriter<W>,
}

impl<'a, W: Write> MarkdownWorkbookWriter<'a, W> {
    /// 创建 Workbook Mode writer。
    #[must_use]
    pub fn new(workbook: &'a Workbook, writer: W, options: MarkdownExportOptions) -> Self {
        Self {
            workbook,
            writer: MarkdownWriter::with_mode(writer, options, MarkdownConversionMode::Workbook),
        }
    }

    /// 写出选定工作表。
    ///
    /// # Errors
    ///
    /// 工作表选择无效、策略不支持当前工作簿或写入超过资源限制时返回错误。
    pub fn write(mut self) -> Result<(W, MarkdownConversionReport)> {
        let indexes = select_sheet_indexes(self.workbook, self.writer_options().sheets())?;
        for index in indexes {
            let sheet = &self.workbook.sheets[index];
            if sheet.visibility != Visibility::Visible && !self.writer_options().include_hidden() {
                self.writer.push_warning(
                    MarkdownWarning::new(
                        MarkdownWarningCode::HiddenSheetSkipped,
                        "hidden worksheet was skipped",
                    )
                    .with_sheet(&sheet.name),
                );
                continue;
            }
            self.write_sheet(index, sheet)?;
        }
        self.writer.finish()
    }

    fn writer_options(&self) -> &MarkdownExportOptions {
        self.writer.options()
    }

    fn write_sheet(&mut self, sheet_index: usize, sheet: &Sheet) -> Result<()> {
        if !sheet.merged.is_empty() {
            match self.writer_options().merges() {
                MarkdownMergePolicy::Error => {
                    return Err(Error::Unsupported(format!(
                        "worksheet {} contains merged cells",
                        sheet.name
                    )));
                }
                MarkdownMergePolicy::HtmlFallback => {
                    return self.write_html_sheet(sheet_index, sheet);
                }
                MarkdownMergePolicy::AnchorWithWarning => {
                    for range in &sheet.merged {
                        self.writer.push_warning(
                            MarkdownWarning::new(
                                MarkdownWarningCode::MergeFlattened,
                                "merged range was projected as its anchor value",
                            )
                            .with_sheet(&sheet.name)
                            .with_range(range.to_a1()),
                        );
                    }
                }
                MarkdownMergePolicy::RepeatAnchor => {}
            }
        }
        if !sheet.styles.is_empty() {
            self.writer.push_warning(
                MarkdownWarning::new(
                    MarkdownWarningCode::StyleDropped,
                    "cell styles are not representable in GFM",
                )
                .with_sheet(&sheet.name),
            );
        }
        if !sheet.opaque.is_empty() || !sheet.tables.is_empty() || !self.workbook.opaque.is_empty()
        {
            self.writer.push_warning(
                MarkdownWarning::new(
                    MarkdownWarningCode::UnsupportedObjectDropped,
                    "workbook objects are not representable in GFM",
                )
                .with_sheet(&sheet.name),
            );
        }

        self.writer.begin(&StreamInfo {
            sheet_name: sheet.name.clone(),
            date_system: self.workbook.date_system,
        })?;
        let (rows, columns) = projection_dimensions(sheet);
        if rows == 0 || columns == 0 {
            self.writer.push_warning(
                MarkdownWarning::new(
                    MarkdownWarningCode::EmptySheet,
                    "empty worksheet produced a heading only",
                )
                .with_sheet(&sheet.name),
            );
        }
        for row in 0..rows {
            let mut cells = Vec::new();
            for column in 0..columns {
                let value = self.project_cell(sheet_index, sheet, row, column);
                if !matches!(value, CellValue::Empty) {
                    cells.push(StreamCell {
                        col: column,
                        value,
                        number_format: self.number_format(sheet, row, column),
                    });
                }
            }
            if columns > 0 && cells.last().is_none_or(|cell| cell.col + 1 < columns) {
                cells.push(StreamCell {
                    col: columns - 1,
                    value: CellValue::Empty,
                    number_format: String::new(),
                });
            }
            self.writer.row(row, &cells)?;
        }
        self.writer.end()
    }

    fn project_cell(&self, sheet_index: usize, sheet: &Sheet, row: u32, column: u32) -> CellValue {
        if self.writer_options().merges() == MarkdownMergePolicy::RepeatAnchor
            && let Some(range) = merge_covering(&sheet.merged, row, column)
        {
            return self.project_unmerged_cell(
                sheet_index,
                sheet,
                range.start.row,
                range.start.col,
            );
        }
        self.project_unmerged_cell(sheet_index, sheet, row, column)
    }

    fn project_unmerged_cell(
        &self,
        sheet_index: usize,
        sheet: &Sheet,
        row: u32,
        column: u32,
    ) -> CellValue {
        let Some(cell) = sheet.get(row, column) else {
            return sheet.value(row, column);
        };
        if let Cell::Formula { expr, cached } = cell {
            return match self.writer_options().formulas() {
                MarkdownFormulaPolicy::CachedValue => cached.clone(),
                MarkdownFormulaPolicy::Expression => CellValue::Text(format!("={expr}")),
                MarkdownFormulaPolicy::ExpressionAndCached => CellValue::Text(format!(
                    "={expr} => {}",
                    self.display_value(sheet_index, row, column, cached)
                )),
            };
        }
        cell.value()
    }

    fn display_value(
        &self,
        sheet_index: usize,
        row: u32,
        column: u32,
        value: &CellValue,
    ) -> String {
        match self.writer_options().values() {
            MarkdownValuePolicy::Formatted => self.workbook.display_cell(sheet_index, row, column),
            MarkdownValuePolicy::Raw => value.to_display_string(),
        }
    }

    fn number_format(&self, sheet: &Sheet, row: u32, column: u32) -> String {
        if self.writer_options().values() == MarkdownValuePolicy::Raw {
            return String::new();
        }
        sheet
            .style_at(row, column)
            .and_then(|index| self.workbook.styles.get(index))
            .map_or_else(String::new, |style| style.number_format.clone())
    }

    fn write_html_sheet(&mut self, sheet_index: usize, sheet: &Sheet) -> Result<()> {
        if self.writer.report().sheets_processed > 0 {
            self.writer.write_raw("\n\n")?;
        }
        self.writer.write_raw("## ")?;
        self.writer
            .write_raw(&super::gfm_escape::escape_heading(&sheet.name))?;
        self.writer.write_raw("\n\n<table><caption>")?;
        self.writer.write_raw(&escape_html(&sheet.name))?;
        self.writer.write_raw("</caption>")?;
        let covered = covered_cells(&sheet.merged);
        let (rows, columns) = projection_dimensions(sheet);
        for row in 0..rows {
            self.writer.write_raw("<tr>")?;
            for column in 0..columns {
                if covered.contains(&(row, column)) {
                    continue;
                }
                let tag = if row == 0 { "th" } else { "td" };
                self.writer.write_raw("<")?;
                self.writer.write_raw(tag)?;
                if let Some(range) = merge_anchor(&sheet.merged, row, column) {
                    if range.rows() > 1 {
                        self.writer
                            .write_raw(&format!(" rowspan=\"{}\"", range.rows()))?;
                    }
                    if range.cols() > 1 {
                        self.writer
                            .write_raw(&format!(" colspan=\"{}\"", range.cols()))?;
                    }
                }
                self.writer.write_raw(">")?;
                let value = self.project_unmerged_cell(sheet_index, sheet, row, column);
                self.writer.write_raw(&escape_html(&self.display_value(
                    sheet_index,
                    row,
                    column,
                    &value,
                )))?;
                self.writer.write_raw("</")?;
                self.writer.write_raw(tag)?;
                self.writer.write_raw(">")?;
            }
            self.writer.write_raw("</tr>")?;
        }
        self.writer.write_raw("</table>")?;
        let report = self.writer.report_mut();
        report.sheets_processed = report.sheets_processed.saturating_add(1);
        report.tables_processed = report.tables_processed.saturating_add(1);
        report.rows_processed = report.rows_processed.saturating_add(u64::from(rows));
        report.cells_processed = report
            .cells_processed
            .saturating_add(u64::from(rows) * u64::from(columns));
        Ok(())
    }
}

/// 将完整工作簿写入 Markdown。
///
/// # Errors
///
/// 工作表选择无效、策略不支持当前工作簿或写入超过资源限制时返回错误。
pub fn write_workbook<W: Write>(
    workbook: &Workbook,
    writer: W,
    options: &MarkdownExportOptions,
) -> Result<(W, MarkdownConversionReport)> {
    MarkdownWorkbookWriter::new(workbook, writer, options.clone()).write()
}

fn select_sheet_indexes(
    workbook: &Workbook,
    selection: &MarkdownSheetSelection,
) -> Result<Vec<usize>> {
    match selection {
        MarkdownSheetSelection::All => Ok((0..workbook.sheets.len()).collect()),
        MarkdownSheetSelection::First => (!workbook.sheets.is_empty())
            .then_some(vec![0])
            .ok_or_else(|| Error::SheetNotFound("0".to_owned())),
        MarkdownSheetSelection::Index(index) => workbook
            .sheets
            .get(*index)
            .map(|_| vec![*index])
            .ok_or_else(|| Error::SheetNotFound(index.to_string())),
        MarkdownSheetSelection::Name(name) => workbook
            .sheets
            .iter()
            .position(|sheet| sheet.name.eq_ignore_ascii_case(name))
            .map(|index| vec![index])
            .ok_or_else(|| Error::SheetNotFound(name.clone())),
    }
}

fn merge_covering(ranges: &[CellRange], row: u32, column: u32) -> Option<&CellRange> {
    ranges.iter().find(|range| range.contains(row, column))
}

fn merge_anchor(ranges: &[CellRange], row: u32, column: u32) -> Option<&CellRange> {
    ranges
        .iter()
        .find(|range| range.start.row == row && range.start.col == column)
}

fn covered_cells(ranges: &[CellRange]) -> HashSet<(u32, u32)> {
    let mut covered = HashSet::new();
    for range in ranges {
        for (row, column) in range.iter_cells() {
            if row != range.start.row || column != range.start.col {
                covered.insert((row, column));
            }
        }
    }
    covered
}

fn projection_dimensions(sheet: &Sheet) -> (u32, u32) {
    sheet
        .merged
        .iter()
        .fold(sheet.dimensions(), |(rows, columns), range| {
            (
                rows.max(range.end.row.saturating_add(1)),
                columns.max(range.end.col.saturating_add(1)),
            )
        })
}
