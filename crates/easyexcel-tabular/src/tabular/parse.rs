use std::collections::{BTreeSet, HashSet};

use easyexcel_io::{Error, Result};
use easyexcel_model::{CellAddress, CellError, CellRange, CellValue};
use scraper::{Html, Selector};
use serde_json::Value as JsonValue;

use super::{TabularCell, TabularDocument, TabularFormat, TabularTable};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按指定格式解析中立表格文档。
///
/// # Errors
///
/// 输入不符合指定格式，或文档中不存在可用表格时返回错误。
pub fn parse_document(input: &str, format: TabularFormat) -> Result<TabularDocument> {
    match format {
        TabularFormat::Markdown => easyexcel_markdown::read_markdown(
            input.as_bytes(),
            &easyexcel_markdown::MarkdownImportOptions::default(),
        )
        .map(|result| result.document),
        TabularFormat::Html => parse_html(input),
        TabularFormat::Json => parse_json(input),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析静态 HTML 中的所有 `<table>`，并转换 rowspan/colspan 为合并区域。
///
/// # Errors
///
/// CSS 选择器无法构造、HTML 中没有表格或合并区域超出 `u32` 坐标范围时返回错误。
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
                    let end_row = row_index
                        .checked_add(row_span - 1)
                        .ok_or_else(|| Error::Other("HTML rowspan overflow".to_owned()))?;
                    let end_column = column_index
                        .checked_add(column_span - 1)
                        .ok_or_else(|| Error::Other("HTML colspan overflow".to_owned()))?;
                    table.push_merge(CellRange::new(
                        CellAddress::new(
                            u32::try_from(row_index).map_err(|_| {
                                Error::Other("HTML row index exceeds u32".to_owned())
                            })?,
                            u32::try_from(column_index).map_err(|_| {
                                Error::Other("HTML column index exceeds u32".to_owned())
                            })?,
                        ),
                        CellAddress::new(
                            u32::try_from(end_row).map_err(|_| {
                                Error::Other("HTML merged row exceeds u32".to_owned())
                            })?,
                            u32::try_from(end_column).map_err(|_| {
                                Error::Other("HTML merged column exceeds u32".to_owned())
                            })?,
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 JSON 数组、对象数组或 `{ "tables": [...] }` 文档。
///
/// # Errors
///
/// JSON 语法错误、tables/rows 结构错误或单元格值无法映射时返回错误。
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

    /// 对应 Java：无直接对应；Rust 扩展。 测试 parse_document 各格式分支。
    #[test]
    fn parse_document_dispatches_to_markdown_html_json() {
        // HTML 分支
        let html = "<table><tr><td>x</td></tr></table>";
        let doc = parse_document(html, TabularFormat::Html).expect("html");
        assert_eq!(doc.tables().len(), 1);

        // JSON 数组分支
        let json = r#"[{"a":1}]"#;
        let doc = parse_document(json, TabularFormat::Json).expect("json");
        assert_eq!(doc.tables().len(), 1);

        // Markdown 分支（简单表格）
        let md = "| A |\n|---|\n| 1 |\n";
        let doc = parse_document(md, TabularFormat::Markdown).expect("md");
        assert!(!doc.tables().is_empty());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 HTML caption 标签提取为表名。
    #[test]
    fn parse_html_uses_caption_as_table_name() {
        let html = "<table><caption>My Table</caption><tr><td>1</td></tr></table>";
        let doc = parse_html(html).expect("caption");
        assert_eq!(doc.tables()[0].name(), "My Table");
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 HTML colspan 合并。
    #[test]
    fn parse_html_colspan_creates_merge() {
        let html = "<table><tr><td colspan='2'>wide</td></tr><tr><td>a</td><td>b</td></tr></table>";
        let doc = parse_html(html).expect("colspan");
        assert_eq!(doc.tables()[0].merges().len(), 1);
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试无表格 HTML 返回错误。
    #[test]
    fn parse_html_no_table_returns_error() {
        let result = parse_html("<div>no table</div>");
        assert!(result.is_err());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 对象含 tables 字段。
    #[test]
    fn parse_json_tables_object() {
        let json = r#"{"tables":[{"name":"T1","rows":[["a","b"],[1,2]]}]}"#;
        let doc = parse_json(json).expect("tables obj");
        assert_eq!(doc.tables().len(), 1);
        assert_eq!(doc.tables()[0].name(), "T1");
        assert_eq!(doc.tables()[0].rows().len(), 2);
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 对象缺 tables 字段报错。
    #[test]
    fn parse_json_object_without_tables_errors() {
        let result = parse_json(r#"{"data":[1]}"#);
        assert!(result.is_err());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON tables 非数组报错。
    #[test]
    fn parse_json_tables_not_array_errors() {
        let result = parse_json(r#"{"tables":"oops"}"#);
        assert!(result.is_err());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON tables 元素非对象报错。
    #[test]
    fn parse_json_table_element_not_object_errors() {
        let result = parse_json(r#"{"tables":[123]}"#);
        assert!(result.is_err());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 表缺少 rows 报错。
    #[test]
    fn parse_json_table_missing_rows_errors() {
        let result = parse_json(r#"{"tables":[{"name":"T"}]}"#);
        assert!(result.is_err());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 顶层非数组/非对象报错。
    #[test]
    fn parse_json_invalid_top_level_errors() {
        let result = parse_json(r#""just a string""#);
        assert!(result.is_err());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 混合行类型报错。
    #[test]
    fn parse_json_mixed_row_types_errors() {
        let result = parse_json(r#"[{"a":1}, [2, 3]]"#);
        assert!(result.is_err());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 数组行格式。
    #[test]
    fn parse_json_array_of_arrays() {
        let json = r#"[["name","age"],["Alice",30]]"#;
        let doc = parse_json(json).expect("array of arrays");
        assert_eq!(doc.tables()[0].rows().len(), 2);
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 数组行含非数组报错。
    #[test]
    fn parse_json_array_row_not_array_errors() {
        let result = parse_json(r#"[[1,2],"not_array"]"#);
        assert!(result.is_err());
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 json_cell 各种类型映射。
    #[test]
    fn json_cell_maps_all_types() {
        assert_eq!(json_cell(&JsonValue::Null), CellValue::Empty);
        assert_eq!(json_cell(&JsonValue::Bool(true)), CellValue::Bool(true));
        assert_eq!(
            json_cell(&JsonValue::Number(42.into())),
            CellValue::Number(42.0)
        );
        assert_eq!(
            json_cell(&JsonValue::String("hello".to_owned())),
            CellValue::Text("hello".to_owned())
        );
        // 数组和对象转为文本字符串
        let arr = json_cell(&JsonValue::Array(vec![JsonValue::Number(1.into())]));
        assert!(matches!(arr, CellValue::Text(_)));
        let obj = json_cell(&JsonValue::Object(serde_json::Map::new()));
        assert!(matches!(obj, CellValue::Text(_)));
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 infer_text_cell 各种推断。
    #[test]
    fn infer_text_cell_classifies_values() {
        assert_eq!(infer_text_cell(""), CellValue::Empty);
        assert_eq!(infer_text_cell("  "), CellValue::Empty);
        assert_eq!(infer_text_cell("true"), CellValue::Bool(true));
        assert_eq!(infer_text_cell("FALSE"), CellValue::Bool(false));
        assert_eq!(infer_text_cell("hello"), CellValue::Text("hello".to_owned()));
        // 数字文本
        let v = infer_text_cell("3.14");
        assert_eq!(v, CellValue::Number(3.14));
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 parse_span 处理边界情况。
    #[test]
    fn parse_span_handles_edge_cases() {
        assert_eq!(parse_span(None), 1);
        assert_eq!(parse_span(Some("0")), 1); // 0 treated as 1
        assert_eq!(parse_span(Some("3")), 3);
        assert_eq!(parse_span(Some("abc")), 1); // non-numeric
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 HTML 多表格解析。
    #[test]
    fn parse_html_multiple_tables() {
        let html = "<table><tr><td>a</td></tr></table><table><tr><td>b</td></tr></table>";
        let doc = parse_html(html).expect("multi table");
        assert_eq!(doc.tables().len(), 2);
        assert_eq!(doc.tables()[0].name(), "Table1");
        assert_eq!(doc.tables()[1].name(), "Table2");
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试空 JSON 对象默认表名。
    #[test]
    fn parse_json_tables_default_name() {
        let json = r#"{"tables":[{"rows":[["x"]]}]}"#;
        let doc = parse_json(json).expect("default name");
        assert_eq!(doc.tables()[0].name(), "Table1");
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 JSON 中 null、bool、嵌套值。
    #[test]
    fn parse_json_object_with_mixed_types() {
        let json = r#"[{"name":"A","active":true,"score":null,"tags":"tag"}]"#;
        let doc = parse_json(json).expect("mixed");
        let table = &doc.tables()[0];
        assert_eq!(table.rows().len(), 2); // header + 1 data row
    }

    /// 对应 Java：无直接对应；Rust 扩展。 测试 normalize_text 合并空白。
    #[test]
    fn normalize_text_joins_whitespace() {
        let parts = vec!["hello", "  world  ", "foo"];
        assert_eq!(normalize_text(parts.into_iter()), "hello world foo");
    }
}
