//! 对应 Java：`com.alibaba.excel.exception.ExcelAnalysisStopSheetException`。
use super::ExcelAnalysisException;
/// 仅终止当前 Sheet、仍触发 `doAfterAllAnalysed` 的控制流异常。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExcelAnalysisStopSheetException {
    inner: ExcelAnalysisException,
}
impl ExcelAnalysisStopSheetException {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: ExcelAnalysisException::new(),
        }
    }
    #[must_use]
    pub fn with_message(v: impl Into<String>) -> Self {
        Self {
            inner: ExcelAnalysisException::with_message(v),
        }
    }
    #[must_use]
    pub fn with_message_and_cause(v: impl Into<String>, cause: impl ToString) -> Self {
        Self {
            inner: ExcelAnalysisException::with_message_and_cause(v, cause),
        }
    }
    #[must_use]
    pub fn with_cause(cause: impl ToString) -> Self {
        Self {
            inner: ExcelAnalysisException::with_cause(cause),
        }
    }
}
impl std::fmt::Display for ExcelAnalysisStopSheetException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl std::error::Error for ExcelAnalysisStopSheetException {}
impl From<ExcelAnalysisStopSheetException> for crate::ExcelError {
    fn from(v: ExcelAnalysisStopSheetException) -> Self {
        crate::ExcelError::AnalysisStopSheet(v.to_string())
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_has_no_message() {
        let ex = ExcelAnalysisStopSheetException::new();
        assert!(ex.inner.runtime_exception().message().is_none());
    }

    #[test]
    fn with_message() {
        let ex = ExcelAnalysisStopSheetException::with_message("stop sheet");
        assert_eq!(ex.inner.runtime_exception().message(), Some("stop sheet"));
    }

    #[test]
    fn with_message_and_cause() {
        let ex = ExcelAnalysisStopSheetException::with_message_and_cause("wrap", "root");
        assert_eq!(ex.inner.runtime_exception().cause(), Some("root"));
    }

    #[test]
    fn with_cause() {
        let ex = ExcelAnalysisStopSheetException::with_cause("done");
        assert_eq!(ex.inner.runtime_exception().cause(), Some("done"));
    }

    #[test]
    fn display_shows_message() {
        let ex = ExcelAnalysisStopSheetException::with_message("halt");
        assert_eq!(format!("{ex}"), "halt");
    }

    #[test]
    fn error_trait() {
        let ex = ExcelAnalysisStopSheetException::with_message("err");
        let err: &dyn std::error::Error = &ex;
        assert!(err.to_string().contains("err"));
    }

    #[test]
    fn from_converts_to_analysis_stop_sheet() {
        let ex = ExcelAnalysisStopSheetException::with_message("stop");
        let err: crate::ExcelError = ex.into();
        match &err {
            crate::ExcelError::AnalysisStopSheet(msg) => assert!(msg.contains("stop")),
            other => panic!("expected AnalysisStopSheet, got {:?}", other),
        }
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(
            ExcelAnalysisStopSheetException::default(),
            ExcelAnalysisStopSheetException::new()
        );
    }

    #[test]
    fn clone_eq() {
        let a = ExcelAnalysisStopSheetException::with_message("x");
        let b = a.clone();
        assert_eq!(a, b);
    }
}
