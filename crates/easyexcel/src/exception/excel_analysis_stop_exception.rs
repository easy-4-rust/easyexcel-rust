//! 对应 Java：`com.alibaba.excel.exception.ExcelAnalysisStopException`。
use super::ExcelAnalysisException;
/// 终止整个工作簿解析的控制流异常。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExcelAnalysisStopException { inner: ExcelAnalysisException }
impl ExcelAnalysisStopException {
    #[must_use] pub const fn new() -> Self { Self { inner: ExcelAnalysisException::new() } }
    #[must_use] pub fn with_message(v: impl Into<String>) -> Self { Self { inner: ExcelAnalysisException::with_message(v) } }
    #[must_use] pub fn with_message_and_cause(v: impl Into<String>, cause: impl ToString) -> Self { Self { inner: ExcelAnalysisException::with_message_and_cause(v, cause) } }
    #[must_use] pub fn with_cause(cause: impl ToString) -> Self { Self { inner: ExcelAnalysisException::with_cause(cause) } }
}
impl std::fmt::Display for ExcelAnalysisStopException { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(&self.inner, f) } }
impl std::error::Error for ExcelAnalysisStopException {}
impl From<ExcelAnalysisStopException> for crate::ExcelError { fn from(v: ExcelAnalysisStopException) -> Self { crate::ExcelError::AnalysisStop(v.to_string()) } }
