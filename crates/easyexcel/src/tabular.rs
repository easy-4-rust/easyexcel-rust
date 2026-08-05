//! Markdown、HTML、JSON 表格转换门面。

pub use easyexcel_tabular::{
    TabularCell, TabularDocument, TabularFormat, TabularTable, parse_document, parse_html,
    parse_json, parse_markdown, render_html, render_json, render_markdown,
};
