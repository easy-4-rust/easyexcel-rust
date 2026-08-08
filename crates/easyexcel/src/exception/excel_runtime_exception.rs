//! 对应 Java：`com.alibaba.excel.exception.ExcelRuntimeException`。

/// 全部 EasyExcel 兼容异常的基础对象。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExcelRuntimeException { message: Option<String>, cause: Option<String> }
impl ExcelRuntimeException {
    /// Java 无参构造器。
    #[must_use] pub const fn new() -> Self { Self { message: None, cause: None } }
    /// Java `ExcelRuntimeException(String)`。
    #[must_use] pub fn with_message(message: impl Into<String>) -> Self { Self { message: Some(message.into()), cause: None } }
    /// Java `ExcelRuntimeException(String, Throwable)` 的后端中立映射。
    #[must_use] pub fn with_message_and_cause(message: impl Into<String>, cause: impl ToString) -> Self { Self { message: Some(message.into()), cause: Some(cause.to_string()) } }
    /// Java `ExcelRuntimeException(Throwable)`。
    #[must_use] pub fn with_cause(cause: impl ToString) -> Self { let cause = cause.to_string(); Self { message: Some(cause.clone()), cause: Some(cause) } }
    /// 返回异常消息。
    #[must_use] pub fn message(&self) -> Option<&str> { self.message.as_deref() }
    /// 返回 cause 的稳定文本。
    #[must_use] pub fn cause(&self) -> Option<&str> { self.cause.as_deref() }
}
impl std::fmt::Display for ExcelRuntimeException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.message.as_deref().unwrap_or("")) }
}
impl std::error::Error for ExcelRuntimeException {}
impl From<ExcelRuntimeException> for crate::ExcelError {
    fn from(value: ExcelRuntimeException) -> Self { crate::ExcelError::Format(value.to_string()) }
}
