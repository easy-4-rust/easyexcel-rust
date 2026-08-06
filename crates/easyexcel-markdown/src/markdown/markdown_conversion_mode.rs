/// Markdown 转换的读取模式。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownConversionMode {
    /// 按格式和策略选择安全的执行模式。
    #[default]
    Auto,
    /// 逐行事件模式。
    Event,
    /// 完整工作簿模式。
    Workbook,
}
