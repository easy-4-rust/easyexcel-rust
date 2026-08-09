//! 对应 Java：`com.alibaba.excel.event.AbstractIgnoreExceptionReadListener`.

use crate::core::analysis_context::AnalysisContext;
use crate::core::cell_extra::CellExtra;
use crate::core::read_listener::ReadListener;
use std::collections::HashMap;

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

/// 将 Java 抽象忽略异常监听器的默认方法接入 `ReadListener` 动态分派。
pub struct AbstractIgnoreExceptionListenerAdapter<L> {
    inner: L,
}

impl<L> AbstractIgnoreExceptionListenerAdapter<L> {
    /// 创建适配器。
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

impl<T, L> ReadListener<T> for AbstractIgnoreExceptionListenerAdapter<L>
where
    L: AbstractIgnoreExceptionReadListener<T>,
{
    fn on_exception(
        &mut self,
        error: &crate::core::excel_error::ExcelError,
        context: &AnalysisContext,
    ) -> crate::core::ErrorAction {
        self.inner.on_exception_silent(error, context);
        crate::core::ErrorAction::Continue
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> crate::core::analysis_context::Result<()> {
        ReadListener::invoke_head(&mut self.inner, head, context)
    }

    fn invoke(
        &mut self,
        data: T,
        context: &AnalysisContext,
    ) -> crate::core::analysis_context::Result<()> {
        ReadListener::invoke(&mut self.inner, data, context)
    }

    fn extra(
        &mut self,
        extra: &CellExtra,
        context: &AnalysisContext,
    ) -> crate::core::analysis_context::Result<()> {
        self.inner.extra_silent(extra, context);
        Ok(())
    }

    fn do_after_all_analysed(
        &mut self,
        context: &AnalysisContext,
    ) -> crate::core::analysis_context::Result<()> {
        ReadListener::do_after_all_analysed(&mut self.inner, context)
    }

    fn has_next(&mut self, _context: &AnalysisContext) -> bool {
        true
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
