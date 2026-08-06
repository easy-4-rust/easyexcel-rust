use std::path::PathBuf;

use easyexcel_io::ResourceLimits;

use crate::Result;

use super::{
    MarkdownConversionReport, MarkdownImportOptions, MarkdownTableSelection, MarkdownTypeInference,
    import_path,
};

/// Markdown 到 XLS、XLSX 或 CSV 的 `EasyExcel` 风格配置入口。
#[derive(Debug, Clone)]
pub struct MarkdownImportBuilder {
    input: PathBuf,
    output: PathBuf,
    options: MarkdownImportOptions,
}

impl MarkdownImportBuilder {
    pub(crate) fn new(input: PathBuf, output: PathBuf) -> Self {
        Self {
            input,
            output,
            options: MarkdownImportOptions::default(),
        }
    }

    /// 按名称选择一张 Markdown 表格。
    #[must_use]
    pub fn table_name(mut self, value: impl Into<String>) -> Self {
        self.options = self
            .options
            .with_tables(MarkdownTableSelection::Name(value.into()));
        self
    }

    /// 按零基下标选择一张 Markdown 表格。
    #[must_use]
    pub fn table_index(mut self, value: usize) -> Self {
        self.options = self
            .options
            .with_tables(MarkdownTableSelection::Index(value));
        self
    }

    /// 使用保守类型推断，保留前导零标识符。
    #[must_use]
    pub fn conservative_types(mut self) -> Self {
        self.options = self
            .options
            .with_type_inference(MarkdownTypeInference::Conservative);
        self
    }

    /// 指定类型推断策略。
    #[must_use]
    pub fn type_inference(mut self, value: MarkdownTypeInference) -> Self {
        self.options = self.options.with_type_inference(value);
        self
    }

    /// 配置是否为表头应用统一粗体样式。
    #[must_use]
    pub fn apply_header_style(mut self, value: bool) -> Self {
        self.options = self.options.with_apply_header_style(value);
        self
    }

    /// 配置资源限制。
    #[must_use]
    pub fn limits(mut self, value: ResourceLimits) -> Self {
        self.options = self.options.with_limits(value);
        self
    }

    /// 执行导入并返回结构化转换报告。
    ///
    /// # Errors
    ///
    /// Markdown 无效、目标格式不受支持、资源超限或读写失败时返回错误。
    pub fn do_import(self) -> Result<MarkdownConversionReport> {
        import_path(&self.input, &self.output, &self.options)
    }
}
