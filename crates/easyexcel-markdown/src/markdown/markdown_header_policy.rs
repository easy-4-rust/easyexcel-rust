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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_first_row() {
        assert_eq!(MarkdownHeaderPolicy::default(), MarkdownHeaderPolicy::FirstRow);
    }

    #[test]
    fn serialize_roundtrip() {
        let json = serde_json::to_string(&MarkdownHeaderPolicy::Generated).unwrap();
        assert_eq!(json, "\"generated\"");
        let restored: MarkdownHeaderPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, MarkdownHeaderPolicy::Generated);
    }
}
