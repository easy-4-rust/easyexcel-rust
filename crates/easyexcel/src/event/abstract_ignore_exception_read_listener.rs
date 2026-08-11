//! 对应 Java：`com.alibaba.excel.event.AbstractIgnoreExceptionReadListener`.
//!
//! 拆分后仅保留 `AbstractIgnoreExceptionReadListener` trait；
//! `AbstractIgnoreExceptionListenerAdapter` 位于同级
//! `abstract_ignore_exception_read_listener/abstract_ignore_exception_listener_adapter.rs`。

use crate::core::analysis_context::AnalysisContext;
use crate::core::cell_extra::CellExtra;
use crate::core::read_listener::ReadListener;
use std::collections::HashMap;

include!("abstract_ignore_exception_read_listener/abstract_ignore_exception_listener_adapter.rs");

/// 忽略异常的读取监听器：吞掉异常并继续处理，对应 Java `AbstractIgnoreExceptionReadListener`。
pub trait AbstractIgnoreExceptionReadListener<T>: ReadListener<T> {
    /// 静默处理读取过程中发生的异常，默认实现不做任何事。
    fn on_exception_silent(
        &mut self,
        error: &crate::core::excel_error::ExcelError,
        context: &AnalysisContext,
    ) {
        let _ = (error, context);
    }
    /// 静默处理单元格附加信息，默认实现不做任何事。
    fn extra_silent(&mut self, extra: &CellExtra, context: &AnalysisContext) {
        let _ = (extra, context);
    }

    /// 转为真实读取管线可消费的忽略异常适配器。
    ///
    /// Java 抽象基类直接覆盖父接口默认方法；Rust 需要显式适配以保证经
    /// `dyn ReadListener` 调用时不会重新落回 `Stop`。
    fn ignoring_exceptions(self) -> AbstractIgnoreExceptionListenerAdapter<Self>
    where
        Self: Sized,
    {
        AbstractIgnoreExceptionListenerAdapter::new(self)
    }
}

#[allow(dead_code)]
fn import_marker(m: &HashMap<usize, String>) {
    let _ = m;
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    /// 对应 Java：仅实现必需方法的监听器，验证静默默认实现
    struct SilentListener;

    impl ReadListener<crate::CellValue> for SilentListener {
        fn invoke(
            &mut self,
            _data: crate::CellValue,
            _context: &AnalysisContext,
        ) -> crate::core::analysis_context::Result<()> {
            Ok(())
        }
    }

    impl AbstractIgnoreExceptionReadListener<crate::CellValue> for SilentListener {}

    #[test]
    fn silent_defaults_are_noops() {
        // 对应 Java：AbstractIgnoreExceptionReadListener 默认静默处理
        let mut listener = SilentListener;
        let context = AnalysisContext::new("Sheet1", 0, 0);
        listener.on_exception_silent(
            &crate::core::excel_error::ExcelError::Format("boom".to_owned()),
            &context,
        );
        let extra = CellExtra::new(
            crate::core::enum_cell_extra_type::CellExtraType::Merge,
            None,
            0,
            1,
            0,
            1,
        );
        listener.extra_silent(&extra, &context);
        import_marker(&HashMap::from([(0, "Name".to_owned())]));
    }

    #[test]
    fn invoke_callback_returns_ok() {
        // 对应 Java：ReadListener.invoke 数据行回调返回 Ok
        let mut listener = SilentListener;
        let context = AnalysisContext::new("Sheet1", 0, 0);
        listener
            .invoke(crate::CellValue::Int(1), &context)
            .expect("invoke ok");
    }
}
