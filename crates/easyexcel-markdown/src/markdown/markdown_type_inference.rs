/// Markdown 导入的标量类型推断策略。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownTypeInference {
    /// 所有非空值保持文本。
    Text,
    /// 只推断无歧义布尔值、错误值和规范数字。
    #[default]
    Conservative,
    /// 额外接受空白、千位分隔符、百分号等数字文本。
    Aggressive,
}
