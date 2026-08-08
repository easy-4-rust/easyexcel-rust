//! 对应 Java：`com.alibaba.excel.analysis.ExcelAnalyser` (interface).

use crate::core::{AnalysisContext, ExcelRow, ReadListener, Result};
use crate::read::metadata::ReadSheet;

use super::excel_read_executor::ExcelReadExecutorKind;

/// 对应 Java：`com.alibaba.excel.analysis.ExcelAnalyser`.
///
/// Java declares four methods: `analysis`, `finish`, `excelExecutor`,
/// `analysisContext`. Rust's [`crate::read_xlsx`] / [`crate::read_xls`] /
/// [`crate::read_csv`] functions cover the same contract functionally;
/// [`super::ExcelAnalyserImpl`] is the hot-path dispatcher that selects among
/// them. This trait exists for 1:1 Java package parity.
pub trait ExcelAnalyser {
    /// 按 Java 参数形状解析指定工作表或全部工作表。
    ///
    /// 对应 Java：`analysis(List<ReadSheet>, Boolean)`。`read_all=true` 时忽略
    /// `read_sheet_list`；否则列表不能为空。
    ///
    /// # Errors
    ///
    /// 当工作簿解析（SAX/记录读取）失败时返回 `ExcelError`。
    fn analysis(&mut self, read_sheet_list: Option<&[ReadSheet]>, read_all: bool) -> Result<()>;

    /// 使用 Rust 强类型 listener 运行当前选择的 executor。
    ///
    /// 这是 Java 将 listener 保存于 `ReadWorkbook` 的 Rust 等价入口，供
    /// `ExcelReader<T, L>` 在不擦除类型的情况下复用同一分析器。
    ///
    /// # Errors
    ///
    /// 当工作簿解析或 listener 回调失败时返回 `ExcelError`。
    fn analysis_with_listener<T, L>(&mut self, listener: &mut L) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>;

    /// Completes the read, releasing caches and closing streams. (Java `finish()`)
    fn finish(&mut self);

    /// Returns the selected format-specific executor. (Java `excelExecutor()`)
    fn excel_executor(&self) -> &ExcelReadExecutorKind;

    /// Returns the analysis context. (Java `analysisContext()`)
    fn analysis_context(&self) -> &AnalysisContext;
}
