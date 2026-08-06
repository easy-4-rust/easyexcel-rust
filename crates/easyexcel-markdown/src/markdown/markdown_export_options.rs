use easyexcel_io::ResourceLimits;

use super::{
    MarkdownConversionMode, MarkdownFormulaPolicy, MarkdownHeaderPolicy, MarkdownMergePolicy,
    MarkdownProfile, MarkdownSheetSelection, MarkdownValuePolicy,
};

/// Markdown 导出选项。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarkdownExportOptions {
    profile: MarkdownProfile,
    mode: MarkdownConversionMode,
    sheets: MarkdownSheetSelection,
    header: MarkdownHeaderPolicy,
    formulas: MarkdownFormulaPolicy,
    merges: MarkdownMergePolicy,
    values: MarkdownValuePolicy,
    include_hidden: bool,
    limits: ResourceLimits,
}

impl MarkdownExportOptions {
    /// 返回输出档案。
    #[must_use]
    pub const fn profile(&self) -> MarkdownProfile {
        self.profile
    }
    /// 返回执行模式。
    #[must_use]
    pub const fn mode(&self) -> MarkdownConversionMode {
        self.mode
    }
    /// 返回工作表选择。
    #[must_use]
    pub const fn sheets(&self) -> &MarkdownSheetSelection {
        &self.sheets
    }
    /// 返回表头策略。
    #[must_use]
    pub const fn header(&self) -> MarkdownHeaderPolicy {
        self.header
    }
    /// 返回公式策略。
    #[must_use]
    pub const fn formulas(&self) -> MarkdownFormulaPolicy {
        self.formulas
    }
    /// 返回合并策略。
    #[must_use]
    pub const fn merges(&self) -> MarkdownMergePolicy {
        self.merges
    }
    /// 返回值策略。
    #[must_use]
    pub const fn values(&self) -> MarkdownValuePolicy {
        self.values
    }
    /// 返回是否包含隐藏工作表。
    #[must_use]
    pub const fn include_hidden(&self) -> bool {
        self.include_hidden
    }
    /// 返回资源限制。
    #[must_use]
    pub const fn limits(&self) -> ResourceLimits {
        self.limits
    }

    /// 设置输出档案，并应用该档案的语义默认值。
    #[must_use]
    pub fn with_profile(mut self, profile: MarkdownProfile) -> Self {
        self.profile = profile;
        if profile == MarkdownProfile::HumanReadable {
            self.merges = MarkdownMergePolicy::HtmlFallback;
        }
        self
    }
    /// 设置执行模式。
    #[must_use]
    pub const fn with_mode(mut self, value: MarkdownConversionMode) -> Self {
        self.mode = value;
        self
    }
    /// 设置工作表选择。
    #[must_use]
    pub fn with_sheets(mut self, value: MarkdownSheetSelection) -> Self {
        self.sheets = value;
        self
    }
    /// 设置表头策略。
    #[must_use]
    pub const fn with_header(mut self, value: MarkdownHeaderPolicy) -> Self {
        self.header = value;
        self
    }
    /// 设置公式策略。
    #[must_use]
    pub const fn with_formulas(mut self, value: MarkdownFormulaPolicy) -> Self {
        self.formulas = value;
        self
    }
    /// 设置合并策略。
    #[must_use]
    pub const fn with_merges(mut self, value: MarkdownMergePolicy) -> Self {
        self.merges = value;
        self
    }
    /// 设置值策略。
    #[must_use]
    pub const fn with_values(mut self, value: MarkdownValuePolicy) -> Self {
        self.values = value;
        self
    }
    /// 设置是否包含隐藏工作表。
    #[must_use]
    pub const fn with_include_hidden(mut self, value: bool) -> Self {
        self.include_hidden = value;
        self
    }
    /// 设置资源限制。
    #[must_use]
    pub const fn with_limits(mut self, value: ResourceLimits) -> Self {
        self.limits = value;
        self
    }
}

impl Default for MarkdownExportOptions {
    fn default() -> Self {
        Self {
            profile: MarkdownProfile::AgentStable,
            mode: MarkdownConversionMode::Auto,
            sheets: MarkdownSheetSelection::All,
            header: MarkdownHeaderPolicy::FirstRow,
            formulas: MarkdownFormulaPolicy::CachedValue,
            merges: MarkdownMergePolicy::AnchorWithWarning,
            values: MarkdownValuePolicy::Formatted,
            include_hidden: false,
            limits: ResourceLimits::default(),
        }
    }
}
