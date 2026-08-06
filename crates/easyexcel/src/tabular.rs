//! HTML、JSON 与通用表格格式分派门面。

pub use easyexcel_tabular::{
    TabularCell, TabularDocument, TabularFormat, TabularTable, parse_document, parse_html,
    parse_json, render_document, render_html, render_json,
};
