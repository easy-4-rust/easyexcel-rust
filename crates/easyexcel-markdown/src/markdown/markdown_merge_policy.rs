/// 合并单元格投影策略。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MarkdownMergePolicy {
    /// 只保留锚点值并报告损失。
    #[default]
    #[serde(rename = "anchor")]
    AnchorWithWarning,
    /// 将锚点值重复到覆盖区域。
    #[serde(rename = "repeat")]
    RepeatAnchor,
    /// 对包含合并区域的工作表生成安全内嵌 HTML table。
    #[serde(rename = "html")]
    HtmlFallback,
    /// 发现合并区域即失败。
    #[serde(rename = "error")]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_anchor_with_warning() {
        assert_eq!(
            MarkdownMergePolicy::default(),
            MarkdownMergePolicy::AnchorWithWarning
        );
    }

    #[test]
    fn serialize_roundtrip() {
        let variants = [
            MarkdownMergePolicy::AnchorWithWarning,
            MarkdownMergePolicy::RepeatAnchor,
            MarkdownMergePolicy::HtmlFallback,
            MarkdownMergePolicy::Error,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let restored: MarkdownMergePolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(v, restored);
        }
    }
}
