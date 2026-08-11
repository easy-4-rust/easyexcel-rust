// 忽略异常读取监听器适配器。
// 对应 Java：`com.alibaba.excel.event.AbstractIgnoreExceptionReadListener`（适配器部分）。
// 从 `abstract_ignore_exception_read_listener.rs` 拆分而来，
// 遵循"一个 .rs 文件只对应一个 Java 对象"规范。

/// 将 Java 抽象忽略异常监听器的默认方法接入 `ReadListener` 动态分派。
///
/// Java 抽象基类直接覆盖父接口默认方法；Rust 需要显式适配以保证经
/// `dyn ReadListener` 调用时不会重新落回 `Stop`。
pub struct AbstractIgnoreExceptionListenerAdapter<L> {
    inner: L,
}

impl<L> AbstractIgnoreExceptionListenerAdapter<L> {
    /// 创建适配器。
    ///
    /// # 参数
    /// - `inner`: 实现了 `AbstractIgnoreExceptionReadListener` 的监听器。
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

#[cfg(test)]
mod adapter_tests {
    use super::*;
    use std::cell::Cell;

    /// 测试用静默监听器。
    struct SilentTestListener {
        invoke_count: Cell<usize>,
        silent_exception_count: Cell<usize>,
        silent_extra_count: Cell<usize>,
        after_count: Cell<usize>,
    }

    impl SilentTestListener {
        fn new() -> Self {
            Self {
                invoke_count: Cell::new(0),
                silent_exception_count: Cell::new(0),
                silent_extra_count: Cell::new(0),
                after_count: Cell::new(0),
            }
        }
    }

    impl ReadListener<crate::CellValue> for SilentTestListener {
        fn invoke(
            &mut self,
            _data: crate::CellValue,
            _context: &AnalysisContext,
        ) -> crate::core::analysis_context::Result<()> {
            self.invoke_count.set(self.invoke_count.get() + 1);
            Ok(())
        }

        fn do_after_all_analysed(
            &mut self,
            _context: &AnalysisContext,
        ) -> crate::core::analysis_context::Result<()> {
            self.after_count.set(self.after_count.get() + 1);
            Ok(())
        }
    }

    impl crate::event::AbstractIgnoreExceptionReadListener<crate::CellValue> for SilentTestListener {
        fn on_exception_silent(
            &mut self,
            _error: &crate::core::excel_error::ExcelError,
            _context: &AnalysisContext,
        ) {
            self.silent_exception_count.set(self.silent_exception_count.get() + 1);
        }

        fn extra_silent(&mut self, _extra: &CellExtra, _context: &AnalysisContext) {
            self.silent_extra_count.set(self.silent_extra_count.get() + 1);
        }
    }

    #[test]
    fn new_creates_adapter() {
        let listener = SilentTestListener::new();
        let adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        assert_eq!(adapter.inner().invoke_count.get(), 0);
    }

    #[test]
    fn inner_returns_reference() {
        let listener = SilentTestListener::new();
        let adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let _inner: &SilentTestListener = adapter.inner();
    }

    #[test]
    fn inner_mut_returns_mutable_reference() {
        let listener = SilentTestListener::new();
        let mut adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let _inner: &mut SilentTestListener = adapter.inner_mut();
    }

    #[test]
    fn into_inner_consumes_adapter() {
        let listener = SilentTestListener::new();
        let adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let inner = adapter.into_inner();
        assert_eq!(inner.invoke_count.get(), 0);
    }

    #[test]
    fn on_exception_delegates_to_silent() {
        let listener = SilentTestListener::new();
        let mut adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let context = AnalysisContext::new("Sheet1", 0, 0);
        let error = crate::core::excel_error::ExcelError::Format("test".to_owned());
        let action = adapter.on_exception(&error, &context);
        // 忽略异常适配器应返回 Continue
        assert_eq!(action, crate::core::ErrorAction::Continue);
        assert_eq!(adapter.inner().silent_exception_count.get(), 1);
    }

    #[test]
    fn invoke_delegates_to_inner() {
        let listener = SilentTestListener::new();
        let mut adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let context = AnalysisContext::new("Sheet1", 0, 0);
        adapter.invoke(crate::CellValue::Int(1), &context).unwrap();
        assert_eq!(adapter.inner().invoke_count.get(), 1);
    }

    #[test]
    fn invoke_head_delegates_to_inner() {
        let listener = SilentTestListener::new();
        let mut adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let context = AnalysisContext::new("Sheet1", 0, 0);
        let mut head = HashMap::new();
        head.insert("Name".to_owned(), 0);
        adapter.invoke_head(&head, &context).unwrap();
    }

    #[test]
    fn extra_delegates_to_extra_silent() {
        let listener = SilentTestListener::new();
        let mut adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let context = AnalysisContext::new("Sheet1", 0, 0);
        let extra = CellExtra::new(
            crate::core::enum_cell_extra_type::CellExtraType::Merge,
            None, 0, 1, 0, 1,
        );
        adapter.extra(&extra, &context).unwrap();
        assert_eq!(adapter.inner().silent_extra_count.get(), 1);
    }

    #[test]
    fn do_after_all_analysed_delegates() {
        let listener = SilentTestListener::new();
        let mut adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let context = AnalysisContext::new("Sheet1", 0, 0);
        adapter.do_after_all_analysed(&context).unwrap();
        assert_eq!(adapter.inner().after_count.get(), 1);
    }

    #[test]
    fn has_next_always_returns_true() {
        let listener = SilentTestListener::new();
        let mut adapter = AbstractIgnoreExceptionListenerAdapter::new(listener);
        let context = AnalysisContext::new("Sheet1", 0, 0);
        assert!(adapter.has_next(&context));
    }
}
