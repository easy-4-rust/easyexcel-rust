/// Markdown 投影的稳定 warning 代码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarkdownWarningCode {
    /// 合并区域已压平。
    MergeFlattened,
    /// Event Mode 无法确认合并元数据。
    MergeMetadataUnavailable,
    /// 隐藏工作表被跳过。
    HiddenSheetSkipped,
    /// 样式无法在 GFM 中表达。
    StyleDropped,
    /// 工作簿对象无法在 GFM 中表达。
    UnsupportedObjectDropped,
    /// 空工作表只输出标题。
    EmptySheet,
}
