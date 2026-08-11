//! 对应 Java：`com.alibaba.excel.exception.ExcelGenerateException`。
use super::ExcelRuntimeException;
/// 工作簿生成异常。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelGenerateException {
    inner: ExcelRuntimeException,
}
impl ExcelGenerateException {
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
}
impl std::fmt::Display for ExcelGenerateException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl std::error::Error for ExcelGenerateException {}
impl From<ExcelGenerateException> for crate::ExcelError {
    fn from(v: ExcelGenerateException) -> Self {
        crate::ExcelError::Format(v.to_string())
    }
}
