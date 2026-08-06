/// 单元格值的文本投影策略。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownValuePolicy {
    /// 应用 number format 和日期系统。
    #[default]
    Formatted,
    /// 输出未格式化的标量值。
    Raw,
}
