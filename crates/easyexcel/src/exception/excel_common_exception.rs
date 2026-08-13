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

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_has_no_message() {
        let ex = ExcelCommonException::new();
        assert!(ex.inner.message().is_none());
    }

    #[test]
    fn with_message() {
        let ex = ExcelCommonException::with_message("bad input");
        assert_eq!(ex.inner.message(), Some("bad input"));
    }

    #[test]
    fn with_message_and_cause() {
        let ex = ExcelCommonException::with_message_and_cause("wrap", "root");
        assert_eq!(ex.inner.cause(), Some("root"));
    }

    #[test]
    fn with_cause() {
        let ex = ExcelCommonException::with_cause("timeout");
        assert_eq!(ex.inner.cause(), Some("timeout"));
    }

    #[test]
    fn display_shows_message() {
        let ex = ExcelCommonException::with_message("oops");
        assert_eq!(format!("{ex}"), "oops");
    }

    #[test]
    fn error_trait() {
        let ex = ExcelCommonException::with_message("err");
        let err: &dyn std::error::Error = &ex;
        assert!(err.to_string().contains("err"));
    }

    #[test]
    fn from_converts_to_excel_error() {
        let ex = ExcelCommonException::with_message("bad");
        let err: crate::ExcelError = ex.into();
        match &err {
            crate::ExcelError::Format(msg) => assert!(msg.contains("bad")),
            other => panic!("expected Format, got {:?}", other),
        }
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(ExcelCommonException::default(), ExcelCommonException::new());
    }
}
