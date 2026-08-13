//! 对应 Java：`com.alibaba.excel.exception.ExcelRuntimeException`。

/// 全部 EasyExcel 兼容异常的基础对象。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExcelRuntimeException {
    message: Option<String>,
    cause: Option<String>,
}
impl ExcelRuntimeException {
    /// Java 无参构造器。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            message: None,
            cause: None,
        }
    }
    /// Java `ExcelRuntimeException(String)`。
    #[must_use]
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            cause: None,
        }
    }
    /// Java `ExcelRuntimeException(String, Throwable)` 的后端中立映射。
    #[must_use]
    pub fn with_message_and_cause(message: impl Into<String>, cause: impl ToString) -> Self {
        Self {
            message: Some(message.into()),
            cause: Some(cause.to_string()),
        }
    }
    /// Java `ExcelRuntimeException(Throwable)`。
    #[must_use]
    pub fn with_cause(cause: impl ToString) -> Self {
        let cause = cause.to_string();
        Self {
            message: Some(cause.clone()),
            cause: Some(cause),
        }
    }
    /// 返回异常消息。
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    /// 返回 cause 的稳定文本。
    #[must_use]
    pub fn cause(&self) -> Option<&str> {
        self.cause.as_deref()
    }
}
impl std::fmt::Display for ExcelRuntimeException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_deref().unwrap_or(""))
    }
}
impl std::error::Error for ExcelRuntimeException {}
impl From<ExcelRuntimeException> for crate::ExcelError {
    fn from(value: ExcelRuntimeException) -> Self {
        crate::ExcelError::Format(value.to_string())
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_has_no_message_or_cause() {
        let ex = ExcelRuntimeException::new();
        assert!(ex.message().is_none());
        assert!(ex.cause().is_none());
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(
            ExcelRuntimeException::default(),
            ExcelRuntimeException::new()
        );
    }

    #[test]
    fn with_message_sets_message() {
        let ex = ExcelRuntimeException::with_message("something failed");
        assert_eq!(ex.message(), Some("something failed"));
        assert!(ex.cause().is_none());
    }

    #[test]
    fn with_message_and_cause_sets_both() {
        let ex = ExcelRuntimeException::with_message_and_cause("wrap", "root cause");
        assert_eq!(ex.message(), Some("wrap"));
        assert_eq!(ex.cause(), Some("root cause"));
    }

    #[test]
    fn with_cause_sets_cause_and_message() {
        let ex = ExcelRuntimeException::with_cause("io error");
        assert_eq!(ex.cause(), Some("io error"));
        // with_cause sets message = cause text
        assert_eq!(ex.message(), Some("io error"));
    }

    #[test]
    fn display_shows_message() {
        let ex = ExcelRuntimeException::with_message("hello");
        assert_eq!(format!("{ex}"), "hello");
    }

    #[test]
    fn display_empty_for_no_message() {
        let ex = ExcelRuntimeException::new();
        assert_eq!(format!("{ex}"), "");
    }

    #[test]
    fn error_trait() {
        let ex = ExcelRuntimeException::with_message("err");
        let err: &dyn std::error::Error = &ex;
        assert!(err.to_string().contains("err"));
    }

    #[test]
    fn from_converts_to_excel_error() {
        let ex = ExcelRuntimeException::with_message("bad");
        let err: crate::ExcelError = ex.into();
        match &err {
            crate::ExcelError::Format(msg) => assert!(msg.contains("bad")),
            other => panic!("expected Format variant, got {:?}", other),
        }
    }

    #[test]
    fn clone_eq() {
        let a = ExcelRuntimeException::with_message("test");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let ex = ExcelRuntimeException::new();
        assert!(format!("{ex:?}").contains("ExcelRuntimeException"));
    }
}
