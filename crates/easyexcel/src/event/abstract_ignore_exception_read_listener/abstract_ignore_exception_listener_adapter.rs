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
