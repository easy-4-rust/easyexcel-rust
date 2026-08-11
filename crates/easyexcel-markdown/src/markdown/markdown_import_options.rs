use easyexcel_io::ResourceLimits;

use super::{MarkdownTableSelection, MarkdownTypeInference};

/// Markdown 导入选项。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarkdownImportOptions {
    tables: MarkdownTableSelection,
    type_inference: MarkdownTypeInference,
    apply_header_style: bool,
    limits: ResourceLimits,
}

impl MarkdownImportOptions {
    /// 返回表格选择。
    #[must_use]
    pub const fn tables(&self) -> &MarkdownTableSelection {
        &self.tables
    }
    /// 返回类型推断策略。
    #[must_use]
    pub const fn type_inference(&self) -> MarkdownTypeInference {
        self.type_inference
    }
    /// 返回是否应用表头样式。
    #[must_use]
    pub const fn apply_header_style(&self) -> bool {
        self.apply_header_style
    }
    /// 返回资源限制。
    #[must_use]
    pub const fn limits(&self) -> ResourceLimits {
        self.limits
    }
    /// 设置表格选择。
    #[must_use]
    pub fn with_tables(mut self, value: MarkdownTableSelection) -> Self {
        self.tables = value;
        self
    }
    /// 设置类型推断策略。
    #[must_use]
    pub const fn with_type_inference(mut self, value: MarkdownTypeInference) -> Self {
        self.type_inference = value;
        self
    }
    /// 设置是否应用表头样式。
    #[must_use]
    pub const fn with_apply_header_style(mut self, value: bool) -> Self {
        self.apply_header_style = value;
        self
    }
    /// 设置资源限制。
    #[must_use]
    pub const fn with_limits(mut self, value: ResourceLimits) -> Self {
        self.limits = value;
        self
    }
}

impl Default for MarkdownImportOptions {
    fn default() -> Self {
        Self {
            tables: MarkdownTableSelection::All,
            type_inference: MarkdownTypeInference::Conservative,
            apply_header_style: true,
            limits: ResourceLimits::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let opts = MarkdownImportOptions::default();
        assert_eq!(opts.tables(), &MarkdownTableSelection::All);
        assert_eq!(opts.type_inference(), MarkdownTypeInference::Conservative);
        assert!(opts.apply_header_style());
    }

    #[test]
    fn with_tables() {
        let opts = MarkdownImportOptions::default().with_tables(MarkdownTableSelection::Index(0));
        assert_eq!(opts.tables(), &MarkdownTableSelection::Index(0));
    }

    #[test]
    fn with_type_inference() {
        let opts =
            MarkdownImportOptions::default().with_type_inference(MarkdownTypeInference::Aggressive);
        assert_eq!(opts.type_inference(), MarkdownTypeInference::Aggressive);
    }

    #[test]
    fn with_apply_header_style() {
        let opts = MarkdownImportOptions::default().with_apply_header_style(false);
        assert!(!opts.apply_header_style());
    }

    #[test]
    fn builder_chaining() {
        let opts = MarkdownImportOptions::default()
            .with_tables(MarkdownTableSelection::Name("Sheet1".into()))
            .with_type_inference(MarkdownTypeInference::Text)
            .with_apply_header_style(false);
        assert_eq!(
            opts.tables(),
            &MarkdownTableSelection::Name("Sheet1".into())
        );
        assert_eq!(opts.type_inference(), MarkdownTypeInference::Text);
        assert!(!opts.apply_header_style());
    }
}
