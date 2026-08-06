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
