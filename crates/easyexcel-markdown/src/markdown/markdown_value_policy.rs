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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_formatted() {
        assert_eq!(MarkdownValuePolicy::default(), MarkdownValuePolicy::Formatted);
    }

    #[test]
    fn serialize_roundtrip() {
        let json = serde_json::to_string(&MarkdownValuePolicy::Raw).unwrap();
        assert_eq!(json, "\"raw\"");
        let restored: MarkdownValuePolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, MarkdownValuePolicy::Raw);
    }
}
