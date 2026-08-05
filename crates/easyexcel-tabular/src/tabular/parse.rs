use std::collections::{BTreeSet, HashSet};

use easyexcel_io::{Error, Result};
use easyexcel_model::{CellAddress, CellError, CellRange, CellValue};
use scraper::{Html, Selector};
use serde_json::Value as JsonValue;

use super::{TabularCell, TabularDocument, TabularFormat, TabularTable};

/// 按指定格式解析中立表格文档。
pub fn parse_document(input: &str, format: TabularFormat) -> Result<TabularDocument> {
    match format {
        TabularFormat::Markdown => parse_markdown(input),
        TabularFormat::Html => parse_html(input),
        TabularFormat::Json => parse_json(input),
    }
}

/// 解析一个或多个 GitHub Flavored Markdown 表格。
pub fn parse_markdown(input: &str) -> Result<TabularDocument> {
    let lines: Vec<&str> = input.lines().collect();
    let mut tables = Vec::new();
    let mut index = 0usize;
    while index + 1 < lines.len() {
        let header = split_markdown_row(lines[index]);
        let separator = split_markdown_row(lines[index + 1]);
        if header.is_empty()
            || separator.len() != header.len()
            || !separator.iter().all(|cell| is_markdown_separator(cell))
        {
            index += 1;
            continue;
        }

        let mut table = TabularTable::new(format!("Table{}", tables.len() + 1));
        table.push_row(
            header
                .into_iter()
                .map(|cell| TabularCell::header(infer_text_cell(&cell)))
                .collect(),
        );
        index += 2;
        while index < lines.len() {
            let cells = split_markdown_row(lines[index]);
            if cells.is_empty() {
                break;
            }
            let mut row: Vec<TabularCell> = cells
                .into_iter()
                .map(|cell| TabularCell::new(infer_text_cell(&cell)))
                .collect();
            while row.len() < separator.len() {
                row.push(TabularCell::new(CellValue::Empty));
            }
            table.push_row(row);
            index += 1;
        }
        tables.push(table);
    }
    if tables.is_empty() {
        return Err(Error::Other("no Markdown table found".to_owned()));
    }
    Ok(TabularDocument::from_tables(tables))
}

/// 解析静态 HTML 中的所有 `<table>`，并转换 rowspan/colspan 为合并区域。
pub fn parse_html(input: &str) -> Result<TabularDocument> {
    let document = Html::parse_document(input);
    let table_selector = parse_selector("table")?;
    let row_selector = parse_selector("tr")?;
    let cell_selector = parse_selector("th, td")?;
    let caption_selector = parse_selector("caption")?;
    let mut tables = Vec::new();

    for (table_index, element) in document.select(&table_selector).enumerate() {
        let caption = element
            .select(&caption_selector)
            .next()
            .map(|caption| normalize_text(caption.text()))
            .filter(|value| !value.is_empty());
        let name = caption
            .or_else(|| element.value().attr("id").map(str::to_owned))
            .unwrap_or_else(|| format!("Table{}", table_index + 1));
        let mut table = TabularTable::new(name);
        let mut grid: Vec<Vec<TabularCell>> = Vec::new();
        let mut occupied = BTreeSet::new();

        for (row_index, row_element) in element.select(&row_selector).enumerate() {
            ensure_grid_cell(&mut grid, row_index, 0);
            let mut column_index = 0usize;
            for cell_element in row_element.select(&cell_selector) {
                while occupied.contains(&(row_index, column_index)) {
                    column_index += 1;
                }
                let row_span = parse_span(cell_element.value().attr("rowspan"));
                let column_span = parse_span(cell_element.value().attr("colspan"));
                let is_header = cell_element.value().name().eq_ignore_ascii_case("th");
                let value = infer_text_cell(&normalize_text(cell_element.text()));
                for covered_row in row_index..row_index + row_span {
                    ensure_grid_cell(&mut grid, covered_row, column_index + column_span - 1);
                    for covered_column in column_index..column_index + column_span {
                        if covered_row != row_index || covered_column != column_index {
                            occupied.insert((covered_row, covered_column));
                        }
                    }
                }
                grid[row_index][column_index] = if is_header {
                    TabularCell::header(value)
                } else {
                    TabularCell::new(value)
                };
                if row_span > 1 || column_span > 1 {
                    table.push_merge(CellRange::new(
                        CellAddress::new(row_index as u32, column_index as u32),
                        CellAddress::new(
                            (row_index + row_span - 1) as u32,
                            (column_index + column_span - 1) as u32,
                        ),
                    ));
                }
                column_index += column_span;
            }
        }
        for row in grid {
            table.push_row(row);
        }
        if !table.rows().is_empty() {
            tables.push(table);
        }
    }

    if tables.is_empty() {
        return Err(Error::Other("no HTML table found".to_owned()));
    }
    Ok(TabularDocument::from_tables(tables))
}

/// 解析 JSON 数组、对象数组或 `{ "tables": [...] }` 文档。
pub fn parse_json(input: &str) -> Result<TabularDocument> {
    let value: JsonValue = serde_json::from_str(input)
        .map_err(|error| Error::Other(format!("invalid JSON table: {error}")))?;
    match value {
        JsonValue::Array(values) => Ok(TabularDocument::from_tables(vec![parse_json_table(
            "Table1", &values,
        )?])),
        JsonValue::Object(mut object) => {
            let tables_value = object.remove("tables").ok_or_else(|| {
                Error::Other("JSON object must contain a tables array".to_owned())
            })?;
            let JsonValue::Array(table_values) = tables_value else {
                return Err(Error::Other("JSON tables must be an array".to_owned()));
            };
            let mut tables = Vec::new();
            for (index, table_value) in table_values.into_iter().enumerate() {
                let JsonValue::Object(mut table_object) = table_value else {
                    return Err(Error::Other("each JSON table must be an object".to_owned()));
                };
                let name = table_object
                    .remove("name")
                    .and_then(|name| name.as_str().map(str::to_owned))
                    .unwrap_or_else(|| format!("Table{}", index + 1));
                let rows = table_object
                    .remove("rows")
                    .and_then(|rows| rows.as_array().cloned())
                    .ok_or_else(|| Error::Other("each JSON table requires rows".to_owned()))?;
                tables.push(parse_json_table(&name, &rows)?);
            }
            Ok(TabularDocument::from_tables(tables))
        }
        _ => Err(Error::Other(
            "JSON table input must be an array or tables object".to_owned(),
        )),
    }
}

fn parse_json_table(name: &str, values: &[JsonValue]) -> Result<TabularTable> {
    let mut table = TabularTable::new(name);
    if values.iter().all(JsonValue::is_object) && !values.is_empty() {
        let mut headers = Vec::new();
        let mut seen = HashSet::new();
        for value in values {
            if let Some(object) = value.as_object() {
                for key in object.keys() {
                    if seen.insert(key.clone()) {
                        headers.push(key.clone());
                    }
                }
            }
        }
        table.push_row(
            headers
                .iter()
                .map(|header| TabularCell::header(CellValue::Text(header.clone())))
                .collect(),
        );
        for value in values {
            let object = value
                .as_object()
                .ok_or_else(|| Error::Other("mixed JSON row types are not supported".to_owned()))?;
            table.push_row(
                headers
                    .iter()
                    .map(|header| {
                        TabularCell::new(json_cell(object.get(header).unwrap_or(&JsonValue::Null)))
                    })
                    .collect(),
            );
        }
        return Ok(table);
    }

    for value in values {
        let JsonValue::Array(row) = value else {
            return Err(Error::Other(
                "JSON rows must all be arrays or all be objects".to_owned(),
            ));
        };
        table.push_row(
            row.iter()
                .map(|cell| TabularCell::new(json_cell(cell)))
                .collect(),
        );
    }
    Ok(table)
}

fn json_cell(value: &JsonValue) -> CellValue {
    match value {
        JsonValue::Null => CellValue::Empty,
        JsonValue::Bool(flag) => CellValue::Bool(*flag),
        JsonValue::Number(number) => number
            .as_f64()
            .map_or_else(|| CellValue::Text(number.to_string()), CellValue::Number),
        JsonValue::String(text) => infer_text_cell(text),
        JsonValue::Array(_) | JsonValue::Object(_) => CellValue::Text(value.to_string()),
    }
}

fn infer_text_cell(value: &str) -> CellValue {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        CellValue::Empty
    } else if trimmed.eq_ignore_ascii_case("true") {
        CellValue::Bool(true)
    } else if trimmed.eq_ignore_ascii_case("false") {
        CellValue::Bool(false)
    } else if let Some(error) = CellError::parse(trimmed) {
        CellValue::Error(error)
    } else if let Some(number) = easyexcel_model::value::parse_number_text(trimmed) {
        CellValue::Number(number)
    } else {
        CellValue::Text(trimmed.to_owned())
    }
}

fn split_markdown_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if !trimmed.contains('|') {
        return Vec::new();
    }
    let content = trimmed.trim_matches('|');
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in content.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '|' {
            cells.push(current.trim().to_owned());
            current.clear();
        } else {
            current.push(character);
        }
    }
    cells.push(current.trim().to_owned());
    cells
}

fn is_markdown_separator(value: &str) -> bool {
    let trimmed = value.trim().trim_matches(':');
    trimmed.len() >= 3 && trimmed.chars().all(|character| character == '-')
}

fn parse_selector(selector: &str) -> Result<Selector> {
    Selector::parse(selector)
        .map_err(|error| Error::Other(format!("invalid internal HTML selector: {error:?}")))
}

fn normalize_text<'a>(parts: impl Iterator<Item = &'a str>) -> String {
    parts
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_span(value: Option<&str>) -> usize {
    value
        .and_then(|span| span.parse::<usize>().ok())
        .filter(|span| *span > 0)
        .unwrap_or(1)
}

fn ensure_grid_cell(grid: &mut Vec<Vec<TabularCell>>, row: usize, column: usize) {
    while grid.len() <= row {
        grid.push(Vec::new());
    }
    while grid[row].len() <= column {
        grid[row].push(TabularCell::new(CellValue::Empty));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_markdown_types_and_multiple_tables() {
        let document = parse_markdown(
            "| name | amount | ok |\n|---|---:|:---:|\n| A | 1,200 | true |\n\n|x|\n|---|\n|2|",
        )
        .expect("Markdown tables");
        assert_eq!(document.tables().len(), 2);
        assert_eq!(
            document.tables()[0].rows()[1][1].value(),
            &CellValue::Number(1200.0)
        );
    }

    #[test]
    fn parses_html_spans_without_executing_content() {
        let document = parse_html(
            "<table id='Sales'><tr><th rowspan='2'>Name</th><th>Q1</th></tr><tr><td>2</td></tr></table><script>ignored()</script>",
        )
        .expect("HTML table");
        assert_eq!(document.tables()[0].name(), "Sales");
        assert_eq!(document.tables()[0].merges().len(), 1);
    }

    #[test]
    fn parses_object_array_json() {
        let document = parse_json(r#"[{"name":"A","amount":12},{"name":"B","amount":8}]"#)
            .expect("JSON table");
        assert_eq!(document.tables()[0].rows().len(), 3);
    }
}
