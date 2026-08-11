/// Markdown 输出配置档案。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownProfile {
    /// 面向智能体和自动化的确定性 GFM 输出。
    #[default]
    AgentStable,
    /// 面向人类阅读，允许用 HTML 保留合并语义。
    HumanReadable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_agent_stable() {
        assert_eq!(MarkdownProfile::default(), MarkdownProfile::AgentStable);
    }

    #[test]
    fn serialize_roundtrip() {
        let json = serde_json::to_string(&MarkdownProfile::HumanReadable).unwrap();
        assert_eq!(json, "\"human-readable\"");
        let restored: MarkdownProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, MarkdownProfile::HumanReadable);
    }
}
