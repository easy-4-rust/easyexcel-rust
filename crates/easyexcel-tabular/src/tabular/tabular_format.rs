/// 对应 Java：无直接对应对象；Rust 架构扩展。 中立表格文档的文本格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TabularFormat {
    /// GitHub Flavored Markdown 表格。
    Markdown,
    /// 静态 HTML 表格。
    Html,
    /// JSON 数组、对象数组或 tables 文档。
    Json,
}
