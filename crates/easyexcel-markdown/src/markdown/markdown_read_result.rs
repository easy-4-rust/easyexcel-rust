use easyexcel_model::TabularDocument;

use super::MarkdownConversionReport;

/// Markdown 解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownReadResult {
    /// 解析得到的中立表格文档。
    pub document: TabularDocument,
    /// 解析统计和 warning。
    pub report: MarkdownConversionReport,
}
