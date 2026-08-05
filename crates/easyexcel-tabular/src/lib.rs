//! Markdown、HTML、JSON 与统一工作簿模型之间的安全转换。

pub mod tabular;

pub use tabular::{
    TabularCell, TabularDocument, TabularFormat, TabularTable, parse_document, parse_html,
    parse_json, parse_markdown, render_html, render_json, render_markdown,
};
