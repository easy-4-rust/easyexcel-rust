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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_auto() {
        assert_eq!(
            MarkdownConversionMode::default(),
            MarkdownConversionMode::Auto
        );
    }

    #[test]
    fn serialize_roundtrip() {
        let variants = [
            MarkdownConversionMode::Auto,
            MarkdownConversionMode::Event,
            MarkdownConversionMode::Workbook,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let restored: MarkdownConversionMode = serde_json::from_str(&json).unwrap();
            assert_eq!(v, restored);
        }
    }
}
