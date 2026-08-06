/// GFM 表头生成策略。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownHeaderPolicy {
    /// 使用首行作为表头。
    #[default]
    FirstRow,
    /// 生成 A、B、C 等稳定列名，首行仍作为数据。
    Generated,
}
