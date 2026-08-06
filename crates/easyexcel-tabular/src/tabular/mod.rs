//! 中立表格模型、解析器与渲染器。

mod parse;
mod render;
mod tabular_format;

pub use easyexcel_model::{TabularCell, TabularDocument, TabularTable};
pub use parse::{parse_document, parse_html, parse_json};
pub use render::{render_document, render_html, render_json};
pub use tabular_format::TabularFormat;
