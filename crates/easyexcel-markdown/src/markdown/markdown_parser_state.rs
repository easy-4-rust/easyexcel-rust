use easyexcel_io::{Error, Result};
use easyexcel_model::{CellError, CellValue, TabularCell, TabularDocument, TabularTable};
use pulldown_cmark::{Event, Tag, TagEnd};

use super::{
    MarkdownConversionMode, MarkdownConversionReport, MarkdownImportOptions, MarkdownReadResult,
    MarkdownTableSelection, MarkdownTypeInference,
};

/// 将 pulldown-cmark 事件归约为中立表格模型的状态机。
pub(crate) struct MarkdownParserState<'a> {
    options: &'a MarkdownImportOptions,
    heading: String,
    pending_heading: Option<String>,
    current_table: Option<TabularTable>,
    current_row: Vec<TabularCell>,
    current_cell: String,
    in_heading: bool,
    in_table_head: bool,
    tables: Vec<TabularTable>,
    rows: u64,
    cells: u64,
}

impl<'a> MarkdownParserState<'a> {
    pub(crate) fn new(options: &'a MarkdownImportOptions) -> Self {
        Self {
            options,
            heading: String::new(),
            pending_heading: None,
            current_table: None,
            current_row: Vec::new(),
            current_cell: String::new(),
            in_heading: false,
            in_table_head: false,
            tables: Vec::new(),
            rows: 0,
            cells: 0,
        }
    }

    pub(crate) fn accept(&mut self, event: Event<'_>) -> Result<()> {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                self.heading.clear();
                self.in_heading = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                self.in_heading = false;
                let heading = self.heading.trim();
                if !heading.is_empty() {
                    self.pending_heading = Some(heading.to_owned());
                }
            }
            Event::Start(Tag::Table(_)) => {
                let name = self
                    .pending_heading
                    .take()
                    .unwrap_or_else(|| format!("Table{}", self.tables.len().saturating_add(1)));
                self.current_table = Some(TabularTable::new(name));
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = self.current_table.take() {
                    self.tables.push(table);
                    self.check_table_limit()?;
                }
            }
            Event::Start(Tag::TableHead) => self.in_table_head = true,
            Event::End(TagEnd::TableHead) => {
                // pulldown-cmark 将 GFM 表头直接置于 TableHead 下，不额外发送 TableRow。
                if !self.current_row.is_empty() {
                    self.finish_row()?;
                }
                self.in_table_head = false;
            }
            Event::Start(Tag::TableRow) => self.current_row.clear(),
            Event::End(TagEnd::TableRow) => self.finish_row()?,
            Event::Start(Tag::TableCell) => self.current_cell.clear(),
            Event::End(TagEnd::TableCell) => self.finish_cell()?,
            Event::Text(text) | Event::Code(text) => {
                if self.in_heading {
                    self.heading.push_str(&text);
                } else if self.current_table.is_some() {
                    self.current_cell.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak if self.current_table.is_some() => {
                self.current_cell.push('\n');
            }
            _ => {}
        }
        Ok(())
    }

    fn finish_cell(&mut self) -> Result<()> {
        let chars = self.current_cell.chars().count();
        if chars > self.options.limits().max_cell_chars() {
            return Err(Error::ResourceLimit {
                resource: "cell_chars",
                limit: u64::try_from(self.options.limits().max_cell_chars()).unwrap_or(u64::MAX),
                actual: u64::try_from(chars).unwrap_or(u64::MAX),
            });
        }
        let value = infer_cell(&self.current_cell, self.options.type_inference());
        let cell = if self.in_table_head {
            TabularCell::header(value)
        } else {
            TabularCell::new(value)
        };
        self.current_row.push(cell);
        self.cells = self.cells.saturating_add(1);
        if self.current_row.len() > self.options.limits().max_columns() {
            return Err(Error::ResourceLimit {
                resource: "columns",
                limit: u64::try_from(self.options.limits().max_columns()).unwrap_or(u64::MAX),
                actual: u64::try_from(self.current_row.len()).unwrap_or(u64::MAX),
            });
        }
        Ok(())
    }

    fn finish_row(&mut self) -> Result<()> {
        self.rows = self.rows.saturating_add(1);
        if self.rows > self.options.limits().max_rows() {
            return Err(Error::ResourceLimit {
                resource: "rows",
                limit: self.options.limits().max_rows(),
                actual: self.rows,
            });
        }
        if let Some(table) = self.current_table.as_mut() {
            table.push_row(std::mem::take(&mut self.current_row));
        }
        Ok(())
    }

    fn check_table_limit(&self) -> Result<()> {
        if self.tables.len() > self.options.limits().max_sheets() {
            return Err(Error::ResourceLimit {
                resource: "tables",
                limit: u64::try_from(self.options.limits().max_sheets()).unwrap_or(u64::MAX),
                actual: u64::try_from(self.tables.len()).unwrap_or(u64::MAX),
            });
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<MarkdownReadResult> {
        if self.tables.is_empty() {
            return Err(Error::Markdown {
                line: None,
                message: "no GitHub Flavored Markdown table found".to_owned(),
            });
        }
        let tables = select_tables(self.tables, self.options.tables())?;
        let mut report = MarkdownConversionReport::new(MarkdownConversionMode::Workbook);
        report.tables_processed = tables.len();
        report.sheets_processed = tables.len();
        report.rows_processed = tables
            .iter()
            .map(|table| u64::try_from(table.rows().len()).unwrap_or(u64::MAX))
            .sum();
        report.cells_processed = tables
            .iter()
            .flat_map(TabularTable::rows)
            .map(|row| u64::try_from(row.len()).unwrap_or(u64::MAX))
            .sum();
        Ok(MarkdownReadResult {
            document: TabularDocument::from_tables(tables),
            report,
        })
    }
}

fn select_tables(
    tables: Vec<TabularTable>,
    selection: &MarkdownTableSelection,
) -> Result<Vec<TabularTable>> {
    match selection {
        MarkdownTableSelection::All => Ok(tables),
        MarkdownTableSelection::Index(index) => tables
            .into_iter()
            .nth(*index)
            .map(|table| vec![table])
            .ok_or_else(|| Error::Markdown {
                line: None,
                message: format!("table index {index} is out of range"),
            }),
        MarkdownTableSelection::Name(name) => tables
            .into_iter()
            .find(|table| table.name().eq_ignore_ascii_case(name))
            .map(|table| vec![table])
            .ok_or_else(|| Error::Markdown {
                line: None,
                message: format!("table not found: {name}"),
            }),
    }
}

fn infer_cell(value: &str, mode: MarkdownTypeInference) -> CellValue {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return CellValue::Empty;
    }
    if mode == MarkdownTypeInference::Text {
        return CellValue::Text(value.to_owned());
    }
    if trimmed.eq_ignore_ascii_case("true") {
        return CellValue::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return CellValue::Bool(false);
    }
    if let Some(error) = CellError::parse(trimmed) {
        return CellValue::Error(error);
    }
    let number = if mode == MarkdownTypeInference::Aggressive {
        easyexcel_model::value::parse_number_text(trimmed)
    } else if is_canonical_number(trimmed) {
        trimmed.parse::<f64>().ok()
    } else {
        None
    };
    number.map_or_else(|| CellValue::Text(value.to_owned()), CellValue::Number)
}

fn is_canonical_number(value: &str) -> bool {
    let unsigned = value.strip_prefix(['-', '+']).unwrap_or(value);
    if unsigned.len() > 1 && unsigned.starts_with('0') && !unsigned.starts_with("0.") {
        return false;
    }
    value.parse::<f64>().is_ok()
}
