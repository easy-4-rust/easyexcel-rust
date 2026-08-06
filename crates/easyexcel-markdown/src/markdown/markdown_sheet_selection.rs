/// Markdown 导出时的工作表选择。
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum MarkdownSheetSelection {
    /// 全部可见工作表。
    #[default]
    All,
    /// 第一张工作表。
    First,
    /// 按零基下标选择。
    Index(usize),
    /// 按名称选择。
    Name(String),
}
