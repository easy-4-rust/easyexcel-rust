use std::collections::BTreeSet;
use std::fmt::Write as _;

use easyexcel_model::{CellRange, CellValue};
use serde_json::{Map, Value as JsonValue, json};

use super::TabularDocument;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将中立表格文档渲染为 Markdown。
#[must_use]
pub fn render_markdown(document: &TabularDocument) -> String {
    let mut output = String::new();
    for (table_index, table) in document.tables().iter().enumerate() {
        if table_index > 0 {
            output.push_str("\n\n");
        }
        output.push_str("## ");
        output.push_str(table.name());
        output.push_str("\n\n");
        let column_count = table.rows().iter().map(Vec::len).max().unwrap_or(0);
        if column_count == 0 {
            continue;
        }
        let first_row = table.rows().first();
        write_markdown_row(&mut output, first_row, column_count);
        output.push('|');
        for _ in 0..column_count {
            output.push_str(" --- |");
        }
        output.push('\n');
        for row in table.rows().iter().skip(1) {
            write_markdown_row(&mut output, Some(row), column_count);
        }
    }
    output.trim_end().to_owned()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将中立表格文档渲染为静态 HTML；不生成脚本、外链或样式资源。
#[must_use]
pub fn render_html(document: &TabularDocument) -> String {
    let mut output = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Spreadsheet tables</title></head><body>",
    );
    for table in document.tables() {
        output.push_str("<table><caption>");
        output.push_str(&escape_html(table.name()));
        output.push_str("</caption>");
        let covered = covered_cells(table.merges());
        for (row_index, row) in table.rows().iter().enumerate() {
            output.push_str("<tr>");
            for (column_index, cell) in row.iter().enumerate() {
                let coordinate = u32::try_from(row_index)
                    .ok()
                    .zip(u32::try_from(column_index).ok());
                if coordinate.is_some_and(|coordinate| covered.contains(&coordinate)) {
                    continue;
                }
                let tag = if cell.is_header() { "th" } else { "td" };
                output.push('<');
                output.push_str(tag);
                if let Some(range) = coordinate.and_then(|(row_index, column_index)| {
                    merge_at(table.merges(), row_index, column_index)
                }) {
                    if range.rows() > 1 {
                        let _ = write!(output, " rowspan=\"{}\"", range.rows());
                    }
                    if range.cols() > 1 {
                        let _ = write!(output, " colspan=\"{}\"", range.cols());
                    }
                }
                output.push('>');
                output.push_str(&escape_html(&cell.value().to_display_string()));
                output.push_str("</");
                output.push_str(tag);
                output.push('>');
            }
            output.push_str("</tr>");
        }
        output.push_str("</table>");
    }
    output.push_str("</body></html>");
    output
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将中立表格文档渲染为稳定的 JSON tables 协议。
#[must_use]
pub fn render_json(document: &TabularDocument) -> String {
    let tables: Vec<JsonValue> = document
        .tables()
        .iter()
        .map(|table| {
            let rows: Vec<JsonValue> = table
                .rows()
                .iter()
                .map(|row| {
                    JsonValue::Array(row.iter().map(|cell| json_cell(cell.value())).collect())
                })
                .collect();
            let merges: Vec<JsonValue> = table
                .merges()
                .iter()
                .map(|range| JsonValue::String(range.to_a1()))
                .collect();
            let mut object = Map::new();
            object.insert(
                "name".to_owned(),
                JsonValue::String(table.name().to_owned()),
            );
            object.insert("rows".to_owned(), JsonValue::Array(rows));
            object.insert("merges".to_owned(), JsonValue::Array(merges));
            JsonValue::Object(object)
        })
        .collect();
    json!({
        "schemaVersion": "1.0",
        "tables": tables,
    })
    .to_string()
}

fn write_markdown_row(
    output: &mut String,
    row: Option<&Vec<crate::TabularCell>>,
    column_count: usize,
) {
    output.push('|');
    for column_index in 0..column_count {
        output.push(' ');
        let text = row
            .and_then(|cells| cells.get(column_index))
            .map(|cell| escape_markdown(&cell.value().to_display_string()))
            .unwrap_or_default();
        output.push_str(&text);
        output.push_str(" |");
    }
    output.push('\n');
}

fn json_cell(value: &CellValue) -> JsonValue {
    match value {
        CellValue::Empty => JsonValue::Null,
        CellValue::Number(number) => serde_json::Number::from_f64(*number)
            .map_or_else(|| JsonValue::String(number.to_string()), JsonValue::Number),
        CellValue::Text(text) => JsonValue::String(text.clone()),
        CellValue::Bool(flag) => JsonValue::Bool(*flag),
        CellValue::Error(error) => JsonValue::String(error.to_string()),
    }
}

fn merge_at(ranges: &[CellRange], row: u32, column: u32) -> Option<&CellRange> {
    ranges
        .iter()
        .find(|range| range.start.row == row && range.start.col == column)
}

fn covered_cells(ranges: &[CellRange]) -> BTreeSet<(u32, u32)> {
    let mut cells = BTreeSet::new();
    for range in ranges {
        for (row, column) in range.iter_cells() {
            if row != range.start.row || column != range.start.col {
                cells.insert((row, column));
            }
        }
    }
    cells
}

fn escape_markdown(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace(['\r', '\n'], " ")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use easyexcel_model::{CellAddress, CellRange};

    use crate::{TabularCell, TabularDocument, TabularTable};

    use super::*;

    #[test]
    fn renders_machine_json_and_safe_html() {
        let mut table = TabularTable::new("<Sales>");
        table.push_row(vec![TabularCell::header(CellValue::Text("A".to_owned()))]);
        table.push_row(vec![TabularCell::new(CellValue::Number(2.0))]);
        table.push_merge(CellRange::new(
            CellAddress::new(0, 0),
            CellAddress::new(0, 1),
        ));
        let document = TabularDocument::from_tables(vec![table]);
        assert!(render_json(&document).contains("\"schemaVersion\":\"1.0\""));
        let html = render_html(&document);
        assert!(html.contains("&lt;Sales&gt;"));
        assert!(!html.contains("<script"));
    }
}
