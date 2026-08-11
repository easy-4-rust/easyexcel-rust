use std::path::PathBuf;

use easyexcel_io::ResourceLimits;

use crate::Result;

use super::{
    MarkdownConversionMode, MarkdownConversionReport, MarkdownExportOptions, MarkdownFormulaPolicy,
    MarkdownMergePolicy, MarkdownProfile, MarkdownSheetSelection, export_path_with_password,
};

/// XLS、XLSX 或 CSV 到 Markdown 的 `EasyExcel` 风格配置入口。
#[derive(Debug, Clone)]
pub struct MarkdownExportBuilder {
    input: PathBuf,
    output: PathBuf,
    options: MarkdownExportOptions,
    password: Option<String>,
}

impl MarkdownExportBuilder {
    pub(crate) fn new(input: PathBuf, output: PathBuf) -> Self {
        Self {
            input,
            output,
            options: MarkdownExportOptions::default(),
            password: None,
        }
    }

    /// 使用指定输出档案。
    #[must_use]
    pub fn profile(mut self, value: MarkdownProfile) -> Self {
        self.options = self.options.with_profile(value);
        self
    }

    /// 指定自动、事件或完整工作簿模式。
    #[must_use]
    pub fn mode(mut self, value: MarkdownConversionMode) -> Self {
        self.options = self.options.with_mode(value);
        self
    }

    /// 选择全部工作表。
    #[must_use]
    pub fn all_sheets(mut self) -> Self {
        self.options = self.options.with_sheets(MarkdownSheetSelection::All);
        self
    }

    /// 按名称选择工作表。
    #[must_use]
    pub fn sheet_name(mut self, value: impl Into<String>) -> Self {
        self.options = self
            .options
            .with_sheets(MarkdownSheetSelection::Name(value.into()));
        self
    }

    /// 按零基下标选择工作表。
    #[must_use]
    pub fn sheet_index(mut self, value: usize) -> Self {
        self.options = self
            .options
            .with_sheets(MarkdownSheetSelection::Index(value));
        self
    }

    /// 指定公式投影策略。
    #[must_use]
    pub fn formula_policy(mut self, value: MarkdownFormulaPolicy) -> Self {
        self.options = self.options.with_formulas(value);
        self
    }

    /// 指定合并单元格投影策略。
    #[must_use]
    pub fn merge_policy(mut self, value: MarkdownMergePolicy) -> Self {
        self.options = self.options.with_merges(value);
        self
    }

    /// 配置是否包含隐藏工作表。
    #[must_use]
    pub fn include_hidden(mut self, value: bool) -> Self {
        self.options = self.options.with_include_hidden(value);
        self
    }

    /// 配置资源限制。
    #[must_use]
    pub fn limits(mut self, value: ResourceLimits) -> Self {
        self.options = self.options.with_limits(value);
        self
    }

    /// 配置加密 XLSX 的密码。密码只保存在 builder 内存中。
    #[must_use]
    pub fn password(mut self, value: impl Into<String>) -> Self {
        self.password = Some(value.into());
        self
    }

    /// 执行导出并返回结构化损失报告。
    ///
    /// # Errors
    ///
    /// 输入格式不受支持、策略与模式不兼容、资源超限或读写失败时返回错误。
    pub fn do_export(self) -> Result<MarkdownConversionReport> {
        export_path_with_password(
            &self.input,
            &self.output,
            &self.options,
            self.password.as_deref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn builder_chains_options() {
        let builder =
            MarkdownExportBuilder::new(PathBuf::from("/tmp/in.xlsx"), PathBuf::from("/tmp/out.md"))
                .profile(MarkdownProfile::HumanReadable)
                .mode(MarkdownConversionMode::Workbook)
                .all_sheets()
                .formula_policy(MarkdownFormulaPolicy::Expression)
                .merge_policy(MarkdownMergePolicy::RepeatAnchor)
                .include_hidden(true)
                .password("secret");

        // 链式调用后 builder 仍可用 → 所有 setter 正常执行
        let _builder = builder.sheet_name("Sheet1");
    }

    #[test]
    fn builder_sheet_index() {
        let builder =
            MarkdownExportBuilder::new(PathBuf::from("/tmp/in.xlsx"), PathBuf::from("/tmp/out.md"))
                .sheet_index(2);
        let _builder = builder.sheet_name("fallback");
    }

    #[test]
    fn builder_limits() {
        let builder =
            MarkdownExportBuilder::new(PathBuf::from("/tmp/in.xlsx"), PathBuf::from("/tmp/out.md"))
                .limits(ResourceLimits::default());
        let _builder = builder;
    }

    #[test]
    fn builder_do_export_returns_error_for_missing_file() {
        let builder = MarkdownExportBuilder::new(
            PathBuf::from("/nonexistent/input.xlsx"),
            PathBuf::from("/tmp/out.md"),
        );
        assert!(builder.do_export().is_err());
    }
}
