//! 对应 Java：`com.alibaba.excel.read.listener.IgnoreExceptionReadListener`.

use std::collections::HashMap;

use crate::core::{AnalysisContext, CellExtra, ExcelError, ReadListener, Result};

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
    /// 通过 `ReadListener` 动态分派时仍执行 Java 的“忽略异常并继续”语义。
    fn ignoring_exceptions(self) -> IgnoreExceptionListenerAdapter<Self>
    where
        Self: Sized,
    {
        IgnoreExceptionListenerAdapter::new(self)
    }
}

/// 将 `IgnoreExceptionReadListener` 的子接口默认语义接入真实 `ReadListener` vtable。
pub struct IgnoreExceptionListenerAdapter<L> {
    inner: L,
}

impl<L> IgnoreExceptionListenerAdapter<L> {
    /// 创建忽略异常的读取监听器适配器。
    #[must_use]
    pub const fn new(inner: L) -> Self {
        Self { inner }
    }

    /// 返回内部监听器。
    #[must_use]
    pub const fn inner(&self) -> &L {
        &self.inner
    }

    /// 返回内部监听器的可变引用。
    pub const fn inner_mut(&mut self) -> &mut L {
        &mut self.inner
    }

    /// 消费适配器并返回内部监听器。
    pub fn into_inner(self) -> L {
        self.inner
    }
}

impl<T, L> ReadListener<T> for IgnoreExceptionListenerAdapter<L>
where
    L: IgnoreExceptionReadListener<T>,
{
    fn on_exception(&mut self, error: &ExcelError, context: &AnalysisContext) -> crate::core::ErrorAction {
        self.inner.on_exception_silent(error, context)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        ReadListener::invoke_head(&mut self.inner, head, context)
    }

    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        ReadListener::invoke(&mut self.inner, data, context)
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<()> {
        ReadListener::extra(&mut self.inner, extra, context)
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        ReadListener::do_after_all_analysed(&mut self.inner, context)
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        ReadListener::has_next(&mut self.inner, context)
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
