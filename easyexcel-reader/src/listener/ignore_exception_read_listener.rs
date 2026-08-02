//! Mirrors Java `com.alibaba.excel.read.listener.IgnoreExceptionReadListener`.

use easyexcel_core::{AnalysisContext, ReadListener};

/// Mirrors Java `IgnoreExceptionReadListener extends ReadListener<T>`.
///
/// Java overrides `onException` to swallow the error and `hasNext` to
/// return `true`. The Rust port implements the same defaults on the
/// trait.
pub trait IgnoreExceptionReadListener<T>: ReadListener<T> {
    /// Default exception handler that returns `ErrorAction::Continue`
    /// instead of the trait's `Stop` default. (Java `onException` empty body)
    fn on_exception_silent(
        &mut self,
        _error: &easyexcel_core::ExcelError,
        _context: &AnalysisContext,
    ) -> easyexcel_core::ErrorAction {
        easyexcel_core::ErrorAction::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_core::{ExcelError, Result};

    struct SilentListener;

    impl ReadListener<i32> for SilentListener {
        fn invoke(&mut self, _data: i32, _context: &AnalysisContext) -> Result<()> {
            Ok(())
        }
    }

    impl IgnoreExceptionReadListener<i32> for SilentListener {}

    #[test]
    fn default_silent_handler_returns_continue() {
        // 对应 Java：IgnoreExceptionReadListener.onException 空实现（Continue）
        let mut listener = SilentListener;
        let action = IgnoreExceptionReadListener::<i32>::on_exception_silent(
            &mut listener,
            &ExcelError::Format("boom".to_owned()),
            &AnalysisContext::new("", 0, 0),
        );
        assert_eq!(action, easyexcel_core::ErrorAction::Continue);
    }
}
