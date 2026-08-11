//! 对应 Java：`com.alibaba.excel.exception.ExcelCommonException`。
use super::ExcelRuntimeException;
/// EasyExcel 通用运行期异常。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExcelCommonException {
    inner: ExcelRuntimeException,
}
impl ExcelCommonException {
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
}
impl std::fmt::Display for ExcelCommonException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl std::error::Error for ExcelCommonException {}
impl From<ExcelCommonException> for crate::ExcelError {
    fn from(v: ExcelCommonException) -> Self {
        crate::ExcelError::Format(v.to_string())
    }
}
