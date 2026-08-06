use super::{MarkdownConversionMode, MarkdownWarning};

/// Markdown 转换的结构化统计与损失报告。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarkdownConversionReport {
    /// 实际使用的执行模式。
    pub mode_used: MarkdownConversionMode,
    /// 已处理工作表数。
    pub sheets_processed: usize,
    /// 已处理表格数。
    pub tables_processed: usize,
    /// 已处理总行数。
    pub rows_processed: u64,
    /// 已处理总单元格数。
    pub cells_processed: u64,
    /// 已写出字节数。
    pub output_bytes: u64,
    /// 非致命损失和降级。
    pub warnings: Vec<MarkdownWarning>,
}

impl MarkdownConversionReport {
    /// 创建指定执行模式的空报告。
    #[must_use]
    pub const fn new(mode_used: MarkdownConversionMode) -> Self {
        Self {
            mode_used,
            sheets_processed: 0,
            tables_processed: 0,
            rows_processed: 0,
            cells_processed: 0,
            output_bytes: 0,
            warnings: Vec::new(),
        }
    }
}

impl Default for MarkdownConversionReport {
    fn default() -> Self {
        Self::new(MarkdownConversionMode::Workbook)
    }
}
