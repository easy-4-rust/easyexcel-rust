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

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_has_no_message() {
        let ex = ExcelAnalysisException::new();
        assert!(ex.runtime_exception().message().is_none());
    }

    #[test]
    fn with_message() {
        let ex = ExcelAnalysisException::with_message("parse error");
        assert_eq!(ex.runtime_exception().message(), Some("parse error"));
    }

    #[test]
    fn with_message_and_cause() {
        let ex = ExcelAnalysisException::with_message_and_cause("wrap", "io");
        assert_eq!(ex.runtime_exception().message(), Some("wrap"));
        assert_eq!(ex.runtime_exception().cause(), Some("io"));
    }

    #[test]
    fn with_cause() {
        let ex = ExcelAnalysisException::with_cause("io error");
        assert_eq!(ex.runtime_exception().cause(), Some("io error"));
    }

    #[test]
    fn display_delegates() {
        let ex = ExcelAnalysisException::with_message("hello");
        assert_eq!(format!("{ex}"), "hello");
    }

    #[test]
    fn error_trait() {
        let ex = ExcelAnalysisException::with_message("err");
        let err: &dyn std::error::Error = &ex;
        assert!(err.to_string().contains("err"));
    }

    #[test]
    fn from_converts_to_excel_error_format() {
        let ex = ExcelAnalysisException::with_message("bad");
        let err: crate::ExcelError = ex.into();
        match &err {
            crate::ExcelError::Format(msg) => assert!(msg.contains("bad")),
            other => panic!("expected Format, got {:?}", other),
        }
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(
            ExcelAnalysisException::default(),
            ExcelAnalysisException::new()
        );
    }

    #[test]
    fn clone_eq() {
        let a = ExcelAnalysisException::with_message("x");
        let b = a.clone();
        assert_eq!(a, b);
    }
}
