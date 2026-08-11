use std::collections::BTreeSet;
use std::fmt::Write as _;

use easyexcel_model::{CellRange, CellValue};
use serde_json::{Map, Value as JsonValue, json};

use super::{TabularDocument, TabularFormat};

/// 按指定文本格式渲染中立表格文档。
///
/// # Errors
///
/// Markdown 输出超过资源限制、写入失败或结果不是 UTF-8 时返回错误。
pub fn render_document(
    document: &TabularDocument,
    format: TabularFormat,
) -> easyexcel_io::Result<String> {
    match format {
        TabularFormat::Markdown => {
            let (bytes, _) = easyexcel_markdown::write_document(
                document,
                Vec::new(),
                &easyexcel_markdown::MarkdownExportOptions::default(),
            )?;
            String::from_utf8(bytes).map_err(|error| easyexcel_io::Error::Other(error.to_string()))
        }
        TabularFormat::Html => Ok(render_html(document)),
        TabularFormat::Json => Ok(render_json(document)),
    }
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

    /// 对应 Java：无直接对应；Rust 扩展。 测试 render_document 各格式分支。
    #[test]
    fn render_document_dispatches_formats() {
        let mut table = TabularTable::new("T");
        table.push_row(vec![TabularCell::new(CellValue::Number(1.0))]);
        let doc = TabularDocument::from_tables(vec![table]);

        // JSON
        let json_out = render_document(&doc, TabularFormat::Json).expect("json");
        assert!(json_out.contains("\"schemaVersion\""));

        // HTML
        let html_out = render_document(&doc, TabularFormat::Html).expect("html");
        assert!(html_out.contains("<table>"));

        // Markdown
        let md_out = render_document(&doc, TabularFormat::Markdown).expect("md");
        assert!(!md_out.is_empty());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 json_cell 各种 CellValue 类型。
    #[test]
    fn json_cell_handles_all_value_types() {
        assert_eq!(json_cell(&CellValue::Empty), JsonValue::Null);
        assert_eq!(json_cell(&CellValue::Bool(true)), JsonValue::Bool(true));
        assert_eq!(
            json_cell(&CellValue::Text("hi".to_owned())),
            JsonValue::String("hi".to_owned())
        );
        // 正常浮点数
        let n = json_cell(&CellValue::Number(3.14));
        assert!(n.is_number());
        // NaN/Inf 转为字符串
        let nan = json_cell(&CellValue::Number(f64::NAN));
        assert!(nan.is_string());
        let inf = json_cell(&CellValue::Number(f64::INFINITY));
        assert!(inf.is_string());
        // Error
        let err = json_cell(&CellValue::Error(easyexcel_model::CellError::Value));
        assert!(err.is_string());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 HTML 转义。
    #[test]
    fn escape_html_replaces_special_chars() {
        assert_eq!(escape_html("a&b<c>d\"e'f"), "a&amp;b&lt;c&gt;d&quot;e&#39;f");
        assert_eq!(escape_html("no special"), "no special");
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 merge_at 查找。
    #[test]
    fn merge_at_finds_correct_range() {
        let ranges = vec![
            CellRange::new(CellAddress::new(0, 0), CellAddress::new(1, 1)),
            CellRange::new(CellAddress::new(2, 0), CellAddress::new(2, 1)),
        ];
        assert!(merge_at(&ranges, 0, 0).is_some());
        assert!(merge_at(&ranges, 0, 1).is_none()); // 不是起始位置
        assert!(merge_at(&ranges, 2, 0).is_some());
        assert!(merge_at(&ranges, 5, 0).is_none());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 covered_cells 计算。
    #[test]
    fn covered_cells_excludes_start_cell() {
        let ranges = vec![CellRange::new(CellAddress::new(0, 0), CellAddress::new(1, 1))];
        let covered = covered_cells(&ranges);
        // (0,0) 是起始单元格，不包含在 covered 中
        assert!(!covered.contains(&(0, 0)));
        assert!(covered.contains(&(0, 1)));
        assert!(covered.contains(&(1, 0)));
        assert!(covered.contains(&(1, 1)));
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试渲染含合并的表格输出 rowspan/colspan。
    #[test]
    fn render_html_includes_span_attributes() {
        let mut table = TabularTable::new("T");
        table.push_row(vec![
            TabularCell::new(CellValue::Text("a".to_owned())),
            TabularCell::new(CellValue::Text("b".to_owned())),
        ]);
        table.push_row(vec![
            TabularCell::new(CellValue::Text("c".to_owned())),
            TabularCell::new(CellValue::Text("d".to_owned())),
        ]);
        table.push_merge(CellRange::new(
            CellAddress::new(0, 0),
            CellAddress::new(1, 1),
        ));
        let doc = TabularDocument::from_tables(vec![table]);
        let html = render_html(&doc);
        assert!(html.contains("rowspan=\"2\""));
        assert!(html.contains("colspan=\"2\""));
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 渲染含合并区域。
    #[test]
    fn render_json_includes_merges() {
        let mut table = TabularTable::new("T");
        table.push_row(vec![TabularCell::new(CellValue::Number(1.0))]);
        table.push_merge(CellRange::new(
            CellAddress::new(0, 0),
            CellAddress::new(0, 1),
        ));
        let doc = TabularDocument::from_tables(vec![table]);
        let json_str = render_json(&doc);
        assert!(json_str.contains("\"merges\""));
        assert!(json_str.contains("A1:B1"));
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试渲染空文档。
    #[test]
    fn render_json_empty_tables() {
        let doc = TabularDocument::from_tables(vec![]);
        let json_str = render_json(&doc);
        assert!(json_str.contains("\"tables\":[]"));
    }
}
