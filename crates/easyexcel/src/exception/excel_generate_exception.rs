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

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn with_message() {
        let ex = ExcelGenerateException::with_message("gen fail");
        assert_eq!(ex.inner.message(), Some("gen fail"));
    }

    #[test]
    fn with_message_and_cause() {
        let ex = ExcelGenerateException::with_message_and_cause("wrap", "root");
        assert_eq!(ex.inner.cause(), Some("root"));
    }

    #[test]
    fn with_cause() {
        let ex = ExcelGenerateException::with_cause("disk full");
        assert_eq!(ex.inner.cause(), Some("disk full"));
    }

    #[test]
    fn display_shows_message() {
        let ex = ExcelGenerateException::with_message("oops");
        assert_eq!(format!("{ex}"), "oops");
    }

    #[test]
    fn error_trait() {
        let ex = ExcelGenerateException::with_message("err");
        let err: &dyn std::error::Error = &ex;
        assert!(err.to_string().contains("err"));
    }

    #[test]
    fn from_converts_to_excel_error() {
        let ex = ExcelGenerateException::with_message("bad");
        let err: crate::ExcelError = ex.into();
        match &err {
            crate::ExcelError::Format(msg) => assert!(msg.contains("bad")),
            other => panic!("expected Format, got {:?}", other),
        }
    }

    #[test]
    fn clone_eq() {
        let a = ExcelGenerateException::with_message("x");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let ex = ExcelGenerateException::with_message("test");
        assert!(format!("{ex:?}").contains("ExcelGenerateException"));
    }
}
