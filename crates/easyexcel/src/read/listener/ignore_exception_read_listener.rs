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
    use crate::core::{CellExtra, ErrorAction, ExcelError, Result};

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

    // ── IgnoreExceptionListenerAdapter 测试 ──
    // 对应 Java：IgnoreExceptionReadListener 适配器将子接口默认语义接入 ReadListener vtable。

    /// 创建用于测试的 SilentListener（全 no-op）。
    fn make_adapter() -> IgnoreExceptionListenerAdapter<SilentListener> {
        IgnoreExceptionListenerAdapter::new(SilentListener)
    }

    #[test]
    fn adapter_new_and_inner() {
        // 对应 Java：IgnoreExceptionListenerAdapter 构造与 inner 访问
        let adapter = make_adapter();
        // inner() 返回内部监听器引用
        let _inner: &SilentListener = adapter.inner();
    }

    #[test]
    fn adapter_inner_mut() {
        // 对应 Java：inner_mut 可变访问
        let mut adapter = make_adapter();
        let inner: &mut SilentListener = adapter.inner_mut();
        let context = AnalysisContext::new("S1", 0, 0);
        let _ = ReadListener::invoke(inner, 42, &context);
    }

    #[test]
    fn adapter_into_inner() {
        // 对应 Java：into_inner 消费适配器并返回内部监听器
        let adapter = make_adapter();
        let _inner: SilentListener = adapter.into_inner();
    }

    #[test]
    fn adapter_on_exception_returns_continue() {
        // 对应 Java：IgnoreExceptionReadListener.onException 空实现通过适配器转发
        let mut adapter = make_adapter();
        let context = AnalysisContext::new("S1", 0, 0);
        let error = ExcelError::Format("test error".to_owned());
        let action = ReadListener::on_exception(&mut adapter, &error, &context);
        assert_eq!(action, ErrorAction::Continue);
    }

    #[test]
    fn adapter_invoke_head_delegates() {
        // 对应 Java：invoke_head 转发至内部监听器
        let mut adapter = make_adapter();
        let context = AnalysisContext::new("S1", 0, 0);
        let head = std::collections::HashMap::from([("col".to_owned(), 0)]);
        assert!(ReadListener::invoke_head(&mut adapter, &head, &context).is_ok());
    }

    #[test]
    fn adapter_invoke_delegates() {
        // 对应 Java：invoke 转发至内部监听器
        let mut adapter = make_adapter();
        let context = AnalysisContext::new("S1", 0, 0);
        assert!(ReadListener::invoke(&mut adapter, 7, &context).is_ok());
    }

    #[test]
    fn adapter_extra_delegates() {
        // 对应 Java：extra 转发至内部监听器
        let mut adapter = make_adapter();
        let context = AnalysisContext::new("S1", 0, 0);
        let extra = CellExtra::new(
            crate::core::CellExtraType::Comment,
            Some("note".to_owned()),
            0, 0, 1, 1,
        );
        assert!(ReadListener::extra(&mut adapter, &extra, &context).is_ok());
    }

    #[test]
    fn adapter_do_after_all_analysed_delegates() {
        // 对应 Java：do_after_all_analysed 转发至内部监听器
        let mut adapter = make_adapter();
        let context = AnalysisContext::new("S1", 0, 0);
        assert!(ReadListener::do_after_all_analysed(&mut adapter, &context).is_ok());
    }

    #[test]
    fn adapter_has_next_delegates() {
        // 对应 Java：has_next 转发至内部监听器（默认 true）
        let mut adapter = make_adapter();
        let context = AnalysisContext::new("S1", 0, 0);
        assert!(ReadListener::has_next(&mut adapter, &context));
    }

    #[test]
    fn ignoring_exceptions_trait_method_creates_adapter() {
        // 对应 Java：ignoring_exceptions() 将 trait 实例包装为适配器
        let mut adapter = SilentListener.ignoring_exceptions();
        let context = AnalysisContext::new("S1", 0, 0);
        let error = ExcelError::Format("wrap test".to_owned());
        assert_eq!(
            ReadListener::on_exception(&mut adapter, &error, &context),
            ErrorAction::Continue
        );
    }
}
