use super::MarkdownWarningCode;

/// Markdown 转换期间产生的一条结构化非致命诊断。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarkdownWarning {
    /// 稳定 warning 代码。
    pub code: MarkdownWarningCode,
    /// 人类可读说明。
    pub message: String,
    /// 相关工作表。
    pub sheet: Option<String>,
    /// 可用时提供 A1 range。
    pub range: Option<String>,
}

impl MarkdownWarning {
    /// 创建 warning。
    #[must_use]
    pub fn new(code: MarkdownWarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            sheet: None,
            range: None,
        }
    }

    /// 绑定工作表。
    #[must_use]
    pub fn with_sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheet = Some(sheet.into());
        self
    }

    /// 绑定 A1 range。
    #[must_use]
    pub fn with_range(mut self, range: impl Into<String>) -> Self {
        self.range = Some(range.into());
        self
    }
}
