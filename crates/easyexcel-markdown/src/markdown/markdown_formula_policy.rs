/// 公式单元格投影策略。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MarkdownFormulaPolicy {
    /// 输出公式缓存值。
    #[default]
    #[serde(rename = "cached")]
    CachedValue,
    /// 输出公式表达式。
    #[serde(rename = "expression")]
    Expression,
    /// 同时输出表达式和缓存值。
    #[serde(rename = "both")]
    ExpressionAndCached,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_cached_value() {
        assert_eq!(MarkdownFormulaPolicy::default(), MarkdownFormulaPolicy::CachedValue);
    }

    #[test]
    fn serialize_roundtrip() {
        let variants = [
            MarkdownFormulaPolicy::CachedValue,
            MarkdownFormulaPolicy::Expression,
            MarkdownFormulaPolicy::ExpressionAndCached,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let restored: MarkdownFormulaPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(v, restored);
        }
    }
}
