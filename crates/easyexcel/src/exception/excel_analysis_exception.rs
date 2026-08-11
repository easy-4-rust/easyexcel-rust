//! 对应 Java：`com.alibaba.excel.exception.ExcelAnalysisException`。
use super::ExcelRuntimeException;
/// Excel 解析异常。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExcelAnalysisException {
    inner: ExcelRuntimeException,
}
impl ExcelAnalysisException {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: ExcelRuntimeException::new(),
        }
    }
    #[must_use]
    pub fn with_message(v: impl Into<String>) -> Self {
        Self {
            inner: ExcelRuntimeException::with_message(v),
        }
    }
    #[must_use]
    pub fn with_message_and_cause(v: impl Into<String>, cause: impl ToString) -> Self {
        Self {
            inner: ExcelRuntimeException::with_message_and_cause(v, cause),
        }
    }
    #[must_use]
    pub fn with_cause(cause: impl ToString) -> Self {
        Self {
            inner: ExcelRuntimeException::with_cause(cause),
        }
    }
    #[must_use]
    pub const fn runtime_exception(&self) -> &ExcelRuntimeException {
        &self.inner
    }
}
impl std::fmt::Display for ExcelAnalysisException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl std::error::Error for ExcelAnalysisException {}
impl From<ExcelAnalysisException> for crate::ExcelError {
    fn from(v: ExcelAnalysisException) -> Self {
        crate::ExcelError::Format(v.to_string())
    }
}
