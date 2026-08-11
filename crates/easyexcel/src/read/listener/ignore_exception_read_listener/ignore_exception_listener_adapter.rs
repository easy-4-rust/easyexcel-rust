// 忽略异常读取监听器适配器。
// 对应 Java：`com.alibaba.excel.read.listener.IgnoreExceptionReadListener`（适配器部分）。
// 从 `ignore_exception_read_listener.rs` 拆分而来，
// 遵循"一个 .rs 文件只对应一个 Java 对象"规范。

/// 将 `IgnoreExceptionReadListener` 的子接口默认语义接入真实 `ReadListener` vtable。
///
/// Java overrides `onException` to swallow the error and `hasNext` to
/// return `true`. This adapter ensures the same semantics when called
/// through `dyn ReadListener`.
pub struct IgnoreExceptionListenerAdapter<L> {
    inner: L,
}

impl<L> IgnoreExceptionListenerAdapter<L> {
    /// 创建忽略异常的读取监听器适配器。
    ///
    /// # 参数
    /// - `inner`: 实现了 `IgnoreExceptionReadListener` 的监听器。
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
