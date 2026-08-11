//! 对应 Java：`com.alibaba.excel.read.listener.IgnoreExceptionReadListener`.
//!
//! 拆分后仅保留 `IgnoreExceptionReadListener` trait；
//! `IgnoreExceptionListenerAdapter` 位于同级
//! `ignore_exception_read_listener/ignore_exception_listener_adapter.rs`。

use std::collections::HashMap;

use crate::core::{AnalysisContext, CellExtra, ExcelError, ReadListener, Result};

include!("ignore_exception_read_listener/ignore_exception_listener_adapter.rs");

/// 对应 Java：`IgnoreExceptionReadListener extends ReadListener<T>`.
///
/// Java overrides `onException` to swallow the error and `hasNext` to
/// return `true`. The Rust port implements the same defaults on the
/// trait.
pub trait IgnoreExceptionReadListener<T>: ReadListener<T> {
    /// Default exception handler that returns `ErrorAction::Continue`
    /// instead of the trait's `Stop` default. (Java `onException` empty body)
    fn on_exception_silent(
        &mut self,
        _error: &crate::core::ExcelError,
        _context: &AnalysisContext,
    ) -> crate::core::ErrorAction {
        crate::core::ErrorAction::Continue
    }

    /// 转为可直接注册到读取管线的监听器。
    ///
    /// Rust 的父 trait 默认方法不能被子 trait 自动覆盖；该适配器确保生产管线
    /// 通过 `ReadListener` 动态分派时仍执行 Java 的"忽略异常并继续"语义。
    fn ignoring_exceptions(self) -> IgnoreExceptionListenerAdapter<Self>
    where
        Self: Sized,
    {
        IgnoreExceptionListenerAdapter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ExcelError, Result};

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
        assert_eq!(action, crate::core::ErrorAction::Continue);
    }
}
