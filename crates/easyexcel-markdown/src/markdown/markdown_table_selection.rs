/// Markdown 导入时的表格选择。
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum MarkdownTableSelection {
    /// 选择全部表格。
    #[default]
    All,
    /// 按零基下标选择。
    Index(usize),
    /// 按名称选择。
    Name(String),
}
